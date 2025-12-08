use axum::Router;
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::routing::get;
use axum_extra::TypedHeader;
use axum_extra::headers::IfNoneMatch;
use bytes::Bytes;
use dist::Dist;
use embed_it::Entry;
use headers::HeaderMapExt;
use http::StatusCode;
use server::accept_encoding::AcceptEncoding;
use server::content_encoding::ContentEncoding;
use server::{Encoding, IntoQuality, QualityValue};

#[tokio::main]
async fn main() {
    let router = app();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind to 0.0.0:8080");
    println!("Server on {}", listener.local_addr().unwrap());
    axum::serve(listener, router)
        .await
        .expect("Failed to start server");
}

fn app() -> Router {
    Router::new()
        .route("/", get(root_handle))
        .route("/{path}", get(handle))
}

async fn root_handle(
    if_none_match: Option<TypedHeader<IfNoneMatch>>,
    accept_encoding: Option<TypedHeader<AcceptEncoding>>,
) -> impl IntoResponse {
    static_handle("index.html".to_owned(), if_none_match, accept_encoding)
}

async fn handle(
    path: Option<Path<String>>,
    if_none_match: Option<TypedHeader<IfNoneMatch>>,
    accept_encoding: Option<TypedHeader<AcceptEncoding>>,
) -> impl IntoResponse {
    // 从 url 中提取要下载的静态文件路径，如果没有传入，默认返回 index.html
    let path = if let Some(Path(path)) = path
        && !path.is_empty()
    {
        path
    } else {
        "index.html".to_owned()
    };
    static_handle(path, if_none_match, accept_encoding)
}

fn static_handle(
    path: String,
    if_none_match: Option<TypedHeader<IfNoneMatch>>,
    accept_encoding: Option<TypedHeader<AcceptEncoding>>,
) -> impl IntoResponse {
    let mut base_header = headers::HeaderMap::new();
    // 从静态资源中查找要下载的静态文件路径
    let Some(entry) = Dist.get(path.as_str()) else {
        return (base_header, StatusCode::NOT_FOUND).into_response();
    };
    let file = match entry {
        Entry::Dir(dir) => {
            // 查找目录下是否有 index.html，如果有，就返回 imdex.html
            let path = format!("{}/index.html", dir.path().name());

            let Some(entry) = Dist.get(path.as_str()) else {
                // 如果没有直接返回 403 Forbidden
                return (base_header, StatusCode::NOT_FOUND).into_response();
            };
            let Some(file) = entry.file() else {
                // 不允许将 index.html 作为目录
                return (base_header, StatusCode::INTERNAL_SERVER_ERROR).into_response();
            };
            file
        }
        Entry::File(file) => *file,
    };
    let Ok(etag) = file.etag().value.as_str().parse::<headers::ETag>() else {
        return (base_header, StatusCode::INTERNAL_SERVER_ERROR).into_response();
    };
    if let Some(TypedHeader(if_none_match)) = if_none_match
        && if_none_match.precondition_passes(&etag)
    {
        return (base_header, StatusCode::NOT_MODIFIED).into_response();
    }
    // 保存 etag
    base_header.typed_insert(etag);
    // 服务器支持 zstd 和 brotli 两种压缩算法，需要根据客户端提供的 Accept-Encoding 来决定使用哪种压缩算法
    // 如果客户端没有上传 Accept-Encoding 那么服务器返回原始未压缩的内容，并且响应头设置 Content-Encoding 为 identity
    // 如果客户端提供的 Accept-Encoding，但是服务器不支持这些压缩算法，那么服务器返回原始未压缩的内容，并且响应头设置 Content-Encoding 为 identity
    // 如果客户端提供的 Accept-Encoding 中有多个，并且其中有服务器支持的算法，那么选择权重设置最高的那个，如果权重都一样，选择第一个
    let supported_accept_encoding: AcceptEncoding = [
        QualityValue::new(Encoding::Zstd, 1000_u16.into_quality()),
        QualityValue::new(Encoding::Brotli, 800_u16.into_quality()),
    ]
    .into_iter()
    .collect();
    let content = if let Some(TypedHeader(accept_encoding)) = accept_encoding {
        let encoding = accept_encoding.choose_by(&supported_accept_encoding);
        match encoding {
            Encoding::Brotli => {
                base_header.typed_insert(ContentEncoding::from(Encoding::Brotli));
                file.brotli_content()
            }
            Encoding::Zstd => {
                base_header.typed_insert(ContentEncoding::from(Encoding::Zstd));
                file.zstd_content()
            }
            _ => {
                base_header.typed_insert(ContentEncoding::from(Encoding::Identity));
                file.content()
            }
        }
    } else {
        base_header.typed_insert(ContentEncoding::from(Encoding::Identity));
        file.content()
    };
    base_header.typed_insert(supported_accept_encoding);
    (base_header, Bytes::from_static(content)).into_response()
}
