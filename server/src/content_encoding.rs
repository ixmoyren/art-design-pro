use crate::util::encoding::Encoding;
use crate::util::flat_csv::FlatCsv;
use headers_core::Error;
use http::HeaderValue;

#[derive(Debug, PartialEq)]
pub struct ContentEncoding(FlatCsv);

impl headers_core::Header for ContentEncoding {
    fn name() -> &'static ::http::header::HeaderName {
        &::http::header::CONTENT_ENCODING
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        values
            .next()
            .cloned()
            .ok_or_else(Error::invalid)
            .map(ContentEncoding::from)
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend(std::iter::once((&self.0).into()))
    }
}

impl From<Encoding> for ContentEncoding {
    fn from(value: Encoding) -> Self {
        let value = value.to_string();
        ContentEncoding(HeaderValue::try_from(value).unwrap().into())
    }
}

impl From<HeaderValue> for ContentEncoding {
    fn from(value: HeaderValue) -> Self {
        Self(value.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::content_encoding::ContentEncoding;
    use crate::util::encoding::Encoding::Gzip;
    use headers::HeaderMapExt;

    fn test_decode<T: headers_core::Header>(values: &[&str]) -> Option<T> {
        let mut map = ::http::HeaderMap::new();
        for val in values {
            map.append(T::name(), val.parse().unwrap());
        }
        map.typed_get()
    }

    fn test_encode<T: headers_core::Header>(header: T) -> ::http::HeaderMap {
        let mut map = ::http::HeaderMap::new();
        map.typed_insert(header);
        map
    }

    #[test]
    fn decode() {
        let content_encoding = test_decode::<ContentEncoding>(&["gzip"]).unwrap();

        assert_eq!(content_encoding, ContentEncoding::from(Gzip));
    }

    #[test]
    fn encode() {
        let content_encoding: ContentEncoding = ContentEncoding::from(Gzip);

        let headers = test_encode(content_encoding);
        assert_eq!(headers["content-encoding"], "gzip");
    }
}
