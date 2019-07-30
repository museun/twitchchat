//! # twitchchat
#![warn(
    missing_docs,
    unsafe_code,
    unused_lifetimes,
    unused_qualifications,
    unused_results
)]

/// IRC-related stuff (not really intended for use with real IRC networks)
pub mod irc;

mod tags;
/// IRCv3 Tags
pub use tags::Tags;

/// Types associated with twitch
mod twitch;
pub use twitch::*;

pub use self::twitch::UserConfig;

#[allow(dead_code)]
pub(crate) const VERSION_STR: &str =
    concat!(env!("CARGO_PKG_NAME"), ":", env!("CARGO_PKG_VERSION"));

/// The Twitch IRC address for non-TLS connections
pub const TWITCH_IRC_ADDRESS: &str = "irc.chat.twitch.tv:6667";

/// The Twitch IRC address for TLS connections
pub const TWITCH_IRC_ADDRESS_TLS: &str = "irc.chat.twitch.tv:6697";

// TODO remove this stuff
/// Message conversion types
pub mod conversion;
#[doc(inline)]
pub use conversion::ToMessage;
