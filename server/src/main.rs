use axum::Router;
use std::path::PathBuf;
use std::str::FromStr;
use tower_http::services::ServeDir;

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
    // 使用相对目录来访问 dist
    let static_dir = PathBuf::from_str("../dist").expect("Failed to get dist path");
    let static_files = ServeDir::new(&static_dir);
    let asset_dir = static_dir.join("assets");
    let asset_files = ServeDir::new(&asset_dir);

    Router::new()
        .route_service("/", static_files)
        .nest_service("/assets", asset_files)
}
