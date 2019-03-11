#![allow(dead_code)]
/// IRC-related stuff
pub mod irc;

/// Types associated with twitch
pub mod twitch;

#[cfg(feature = "teststream")]
mod teststream;

#[cfg(feature = "teststream")]
pub use teststream::TestStream;
