use headers_core::Error;
use http::HeaderValue;

pub mod encoding;
pub mod entity;
pub mod flat_csv;
pub mod iter;
pub mod quality;

#[macro_export]
macro_rules! error_type {
    ($name:ident) => {
        #[doc(hidden)]
        pub struct $name {
            _inner: (),
        }

        impl ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                f.debug_struct(stringify!($name)).finish()
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                f.write_str(stringify!($name))
            }
        }

        impl ::std::error::Error for $name {}
    };
}

#[macro_export]
macro_rules! derive_header {
    ($type:ident(_), name: $name:ident) => {
        impl headers_core::Header for $type {
            fn name() -> &'static ::http::header::HeaderName {
                &::http::header::$name
            }

            fn decode<'i, I>(values: &mut I) -> Result<Self, headers_core::Error>
            where
                I: Iterator<Item = &'i ::http::header::HeaderValue>,
            {
                crate::util::TryFromValues::try_from_values(values).map($type)
            }

            fn encode<E: Extend<http::header::HeaderValue>>(&self, values: &mut E) {
                values.extend(::std::iter::once((&self.0).into()));
            }
        }
    };
}

/// A helper trait for use when deriving `Header`.
pub(crate) trait TryFromValues: Sized {
    /// Try to convert from the values into an instance of `Self`.
    fn try_from_values<'i, I>(values: &mut I) -> Result<Self, Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>;
}

impl TryFromValues for HeaderValue {
    fn try_from_values<'i, I>(values: &mut I) -> Result<Self, Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        values.next().cloned().ok_or_else(Error::invalid)
    }
}
