#[doc(inline)]
pub use crate::irc::tags::{escape_str, unescape_str};

mod tags_builder;
pub use tags_builder::{BuilderError, TagsBuilder, UserTags};
