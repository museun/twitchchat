//! Helpful testing utilities

#[doc(inline)]
pub use crate::irc::tags::{escape_str, unescape_str};

mod tags_builder;
pub use tags_builder::{BuilderError, TagsBuilder, UserTags};

#[allow(missing_docs)]
#[doc(hidden)]
#[cfg(feature = "sink_stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "sink_stream")))]
pub mod dummy;
