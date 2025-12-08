use crate::util::entity::EntityTag;
use crate::{derive_header, error_type};
use std::str::FromStr;

/// `ETag` header, defined in [RFC7232](https://datatracker.ietf.org/doc/html/rfc7232#section-2.3)
///
/// The `ETag` header field in a response provides the current entity-tag
/// for the selected representation, as determined at the conclusion of
/// handling the request.  An entity-tag is an opaque validator for
/// differentiating between multiple representations of the same
/// resource, regardless of whether those multiple representations are
/// due to resource state changes over time, content negotiation
/// resulting in multiple representations being valid at the same time,
/// or both.  An entity-tag consists of an opaque quoted string, possibly
/// prefixed by a weakness indicator.
///
/// # ABNF
///
/// ```text
/// ETag       = entity-tag
/// ```
///
/// # Example values
///
/// * `"xyzzy"`
/// * `W/"xyzzy"`
/// * `""`
///
/// # Examples
///
/// ```
/// let etag = "\"xyzzy\"".parse::<headers::ETag>().unwrap();
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ETag(pub(super) EntityTag);

derive_header! {
    ETag(_),
    name: ETAG
}

impl ETag {
    #[cfg(test)]
    pub(crate) fn from_static(src: &'static str) -> ETag {
        ETag(EntityTag::from_static(src))
    }
}

error_type!(InvalidETag);

impl FromStr for ETag {
    type Err = InvalidETag;
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let val = src.parse().map_err(|_| InvalidETag { _inner: () })?;

        EntityTag::from_owned(val)
            .map(ETag)
            .ok_or(InvalidETag { _inner: () })
    }
}
