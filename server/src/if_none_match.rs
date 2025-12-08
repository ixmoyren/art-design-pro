use crate::derive_header;
use crate::etag::ETag;
use crate::util::entity::EntityTagRange;
use http::HeaderValue;

/// `If-None-Match` header, defined in
/// [RFC7232](https://tools.ietf.org/html/rfc7232#section-3.2)
///
/// The `If-None-Match` header field makes the request method conditional
/// on a recipient cache or origin server either not having any current
/// representation of the target resource, when the field-value is "*",
/// or having a selected representation with an entity-tag that does not
/// match any of those listed in the field-value.
///
/// A recipient MUST use the weak comparison function when comparing
/// entity-tags for If-None-Match (Section 2.3.2), since weak entity-tags
/// can be used for cache validation even if there have been changes to
/// the representation data.
///
/// # ABNF
///
/// ```text
/// If-None-Match = "*" / 1#entity-tag
/// ```
///
/// # Example values
///
/// * `"xyzzy"`
/// * `W/"xyzzy"`
/// * `"xyzzy", "r2d2xxxx", "c3piozzzz"`
/// * `W/"xyzzy", W/"r2d2xxxx", W/"c3piozzzz"`
/// * `*`
///
/// # Examples
///
/// ```
/// use headers::IfNoneMatch;
///
/// let if_none_match = IfNoneMatch::any();
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct IfNoneMatch(EntityTagRange);

derive_header! {
    IfNoneMatch(_),
    name: IF_NONE_MATCH
}

impl IfNoneMatch {
    /// Create a new `If-None-Match: *` header.
    pub fn any() -> IfNoneMatch {
        IfNoneMatch(EntityTagRange::Any)
    }

    /// Checks whether the ETag passes this precondition.
    pub fn precondition_passes(&self, etag: &ETag) -> bool {
        !self.0.matches_weak(&etag.0)
    }
}

impl From<ETag> for IfNoneMatch {
    fn from(etag: ETag) -> IfNoneMatch {
        IfNoneMatch(EntityTagRange::Tags(HeaderValue::from(etag.0).into()))
    }
}
