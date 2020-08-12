//! A set of utilities for running a main loop
//!
//! This includes an asynchronous main loop called `AsyncRunner`

use crate::{
    messages::{GlobalUserState, IrcReady, Ready},
    FromIrcMessage, IrcMessage,
};

mod error;
pub use error::Error;

mod reset;
pub use reset::ResetConfig;

mod retry;
pub use retry::RetryStrategy;

mod status;
pub use status::Status;

mod async_runner;
pub use async_runner::AsyncRunner;

mod wait_for;
use wait_for::WaitFor;

/// Trait for determining if a message if one that you can wait for.
pub trait ReadyMessage<'a>: FromIrcMessage<'a> {
    // TODO this should return which caps
    /// Does this message require CAPs to be sent?
    fn requires_caps() -> bool {
        false
    }
    /// The command name of the message.
    fn command() -> &'static str;
}

impl<'a> ReadyMessage<'a> for IrcReady<'a> {
    fn command() -> &'static str {
        IrcMessage::IRC_READY
    }
}

impl<'a> ReadyMessage<'a> for Ready<'a> {
    fn command() -> &'static str {
        IrcMessage::READY
    }
}

impl<'a> ReadyMessage<'a> for GlobalUserState<'a> {
    fn command() -> &'static str {
        IrcMessage::GLOBAL_USER_STATE
    }
}
