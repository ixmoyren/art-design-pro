use crate::util::encoding::Encoding;
use crate::util::flat_csv::FlatCsv;
use crate::util::quality::QualityValue;
use axum::http;
use headers_core::Error;
use http::HeaderValue;

/// `Accept-Encoding` header, defined in
/// [RFC7231](https://datatracker.ietf.org/doc/html/rfc7231#section-5.3.4)
///
/// The `Accept-Encoding` header field can be used by user agents to
/// indicate what response content-codings are
/// acceptable in the response.  An  `identity` token is used as a synonym
/// for "no encoding" in order to communicate when no encoding is
/// preferred.
///
/// # ABNF
///
/// ```text
/// Accept-Encoding  = #( codings [ weight ] )
/// codings          = content-coding / "identity" / "*"
/// ```
///
/// # Example values
/// * `compress, gzip`
/// * ``
/// * `*`
/// * `compress;q=0.5, gzip;q=1`
/// * `gzip;q=1.0, identity; q=0.5, *;q=0`
#[derive(Clone, Eq, PartialEq)]
pub struct AcceptEncoding(FlatCsv);

impl headers_core::Header for AcceptEncoding {
    fn name() -> &'static ::http::header::HeaderName {
        &::http::header::ACCEPT_ENCODING
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
            .map(AcceptEncoding::from)
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend(std::iter::once((&self.0).into()))
    }
}

impl AcceptEncoding {
    pub fn iter(&self) -> impl Iterator<Item = QualityValue<Encoding>> + '_ {
        self.0.iter().flat_map(|s| s.parse().ok())
    }

    pub fn choose(&self) -> Encoding {
        let mut quality_values = self.iter().collect::<Vec<_>>();
        quality_values.sort_by_key(|q| std::cmp::Reverse(q.quality()));
        if let Some(encoding) = quality_values.first() {
            encoding.value().clone()
        } else {
            Encoding::Identity
        }
    }

    pub fn choose_by(&self, accept_encoding: &AcceptEncoding) -> Encoding {
        let mut choose_values = accept_encoding.iter().collect::<Vec<_>>();
        choose_values.sort_by_key(|q| std::cmp::Reverse(q.quality()));
        let mut quality_values = self.iter().collect::<Vec<_>>();
        quality_values.sort_by_key(|q| std::cmp::Reverse(q.quality()));
        for v in choose_values {
            if let Some(v) = quality_values.iter().find(|q| (*q).value() == v.value()) {
                return v.value().clone();
            }
        }
        Encoding::Identity
    }
}

impl From<HeaderValue> for AcceptEncoding {
    fn from(value: HeaderValue) -> Self {
        Self(value.into())
    }
}

impl FromIterator<QualityValue<Encoding>> for AcceptEncoding {
    fn from_iter<T: IntoIterator<Item = QualityValue<Encoding>>>(iter: T) -> Self {
        let quality_values = iter
            .into_iter()
            .map(|quality_value| {
                quality_value
                    .to_string()
                    .parse::<HeaderValue>()
                    .expect("QualityValue is a valid HeaderValue")
            })
            .collect();
        AcceptEncoding(quality_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::encoding::Encoding::{Ext, Gzip, Identity, Zstd};
    use crate::util::quality::{IntoQuality, QualityValue};
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
    fn iter() {
        let allowed =
            test_decode::<AcceptEncoding>(&["gzip;q=1.0, identity; q=0.5, *;q=0"]).unwrap();

        let as_vec = allowed.iter().collect::<Vec<_>>();
        assert_eq!(as_vec.len(), 3);
        assert_eq!(as_vec[0], QualityValue::new(Gzip, 1000_u16.into_quality()));
        assert_eq!(
            as_vec[1],
            QualityValue::new(Identity, 500_u16.into_quality())
        );
        assert_eq!(
            as_vec[2],
            QualityValue::new(Ext("*".to_owned()), 0_u16.into_quality())
        );
    }

    #[test]
    fn from_iter() {
        let gzip = QualityValue::new(Gzip, 1000_u16.into_quality());
        let identity = QualityValue::new(Identity, 500_u16.into_quality());
        let accept: AcceptEncoding = vec![gzip, identity].into_iter().collect();

        let headers = test_encode(accept);
        assert_eq!(headers["accept-encoding"], "gzip, identity; q=0.5");
    }

    #[test]
    fn from_choose() {
        let gzip = QualityValue::new(Gzip, 400_u16.into_quality());
        let identity = QualityValue::new(Identity, 500_u16.into_quality());
        let zstd = QualityValue::new(Zstd, 900_u16.into_quality());
        let accept: AcceptEncoding = vec![gzip, identity, zstd].into_iter().collect();

        let encoding = accept.choose();
        assert_eq!(encoding, Zstd);
    }

    #[test]
    fn from_choose_by() {
        let gzip = QualityValue::new(Gzip, 400_u16.into_quality());
        let identity = QualityValue::new(Identity, 500_u16.into_quality());
        let zstd = QualityValue::new(Zstd, 900_u16.into_quality());
        let accept: AcceptEncoding = vec![gzip, identity, zstd].into_iter().collect();

        let support_accept_encoding: AcceptEncoding = vec![
            QualityValue::new(Identity, 500_u16.into_quality()),
            QualityValue::new(Zstd, 900_u16.into_quality()),
        ]
        .into_iter()
        .collect();

        let encoding = accept.choose_by(&support_accept_encoding);
        assert_eq!(encoding, Zstd);
        let support_accept_encoding: AcceptEncoding = vec![
            QualityValue::new(Identity, 500_u16.into_quality()),
            QualityValue::new(Zstd, 500_u16.into_quality()),
        ]
        .into_iter()
        .collect();
        let encoding = accept.choose_by(&support_accept_encoding);
        assert_eq!(encoding, Identity);
    }

    #[test]
    fn test_etag() {
        let str = "\"2021bf398cf8cd5ba2b698fef775e783e074c85c8bab6ecb0bfe1beeedb7de51\"";
        let result = str.parse::<headers::ETag>();
        println!("{result:?}");
    }
}
