/*!
Messages sent by the server

This can be obtained by [subscribing] to an [Event] on a [Dispatcher]

Or by using [TryFrom] on an [Message]

[subscribing]: ../struct.Dispatcher.html#method.subscribe
[Event]: ../events/index.html
[Dispatcher]: ../struct.Dispatcher.html
[TryFrom]: https://doc.rust-lang.org/std/convert/trait.TryFrom.html
[Message]: ../../decode/struct.Message.html
*/

mod prelude {
    pub use crate::decode::{Message, Prefix};
    pub use crate::Tags;
    pub(crate) use crate::{IntoOwned, StringMarker};

    pub use std::any::{Any, TypeId};
    pub use std::collections::HashMap;
    pub use std::convert::TryFrom;
}
use prelude::*;

/// An error returned when trying to use [TryFrom] on a [Message] to a specific [message][msg]
///
/// [TryFrom]: https://doc.rust-lang.org/std/convert/trait.TryFrom.html
/// [Message]: ../decode/struct.Message.html
/// [msg]: ./messages/index.html
#[derive(Debug)]
#[non_exhaustive]
pub enum InvalidMessage {
    /// An invalid command was found for this message
    InvalidCommand {
        /// Expected this command
        expected: String,
        /// Got this command
        got: String,
    },
    /// Expected a nickname attached to this message
    ExpectedNick,
    /// Expected an argument at a position in this message
    ExpectedArg {
        /// Argument position
        pos: usize,
    },
    /// Expected this message to have data attached
    ExpectedData,
}

impl std::fmt::Display for InvalidMessage {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            _ => unimplemented!(),
        }
    }
}

impl std::error::Error for InvalidMessage {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub trait Expect {
    fn expect_command(&self, cmd: &str) -> Result<(), InvalidMessage>;
    fn expect_nick(&self) -> Result<&str, InvalidMessage>;
    fn expect_arg(&self, nth: usize) -> Result<&str, InvalidMessage>;
    fn expect_data(&self) -> Result<&str, InvalidMessage>;
}

impl<'a> Expect for crate::decode::Message<&'a str> {
    fn expect_command(&self, cmd: &str) -> Result<(), InvalidMessage> {
        if self.command != cmd {
            return Err(InvalidMessage::InvalidCommand {
                expected: cmd.to_string(),
                got: self.command.to_string(),
            });
        }
        Ok(())
    }

    fn expect_nick(&self) -> Result<&str, InvalidMessage> {
        self.prefix
            .as_ref()
            .and_then(|s| s.nick())
            .ok_or_else(|| InvalidMessage::ExpectedNick)
    }

    fn expect_arg(&self, nth: usize) -> Result<&str, InvalidMessage> {
        self.args
            .split_whitespace()
            .nth(nth)
            .ok_or_else(|| InvalidMessage::ExpectedArg { pos: nth })
    }

    fn expect_data(&self) -> Result<&str, InvalidMessage> {
        self.data.ok_or_else(|| InvalidMessage::ExpectedData)
    }
}

macro_rules! import_modules {
    ($($module:ident)*) => {
        $( mod $module; pub use $module::*; )*
    };
}

import_modules! {
    cap
    clear_chat
    clear_msg
    global_user_state
    irc_ready
    join
    notice
    part
    ping
    pong
    privmsg
    raw
    ready
    reconnect
}
