mod str;
pub use self::str::*;

mod tags_builder;
pub use tags_builder::{BuilderError, TagsBuilder, UserTags};

#[cfg(feature = "testing")]
#[cfg_attr(docsrs, doc(cfg(feature = "testing")))]
mod conn;

#[cfg(feature = "testing")]
#[cfg_attr(docsrs, doc(cfg(feature = "testing")))]
pub use conn::{create_mock_connection, TestConn};
