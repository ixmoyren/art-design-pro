pub mod accept_encoding;
pub mod content_encoding;
pub mod etag;
pub mod if_none_match;
#[macro_use]
mod util;

pub use util::encoding::*;
pub use util::quality::{IntoQuality, QualityValue};
