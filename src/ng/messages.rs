use super::{IrcMessage, Prefix, PrefixIndex, Str, StrIndex, TagIndices, Tags};
use std::convert::Infallible;

#[allow(unused_macros)]
macro_rules! raw {
    () => {
        /// Get the raw message
        pub fn raw(&self) -> &str {
            &*self.raw
        }
    };
}

#[allow(unused_macros)]
macro_rules! str_field {
    ($name:ident) => {
        pub fn $name(&self) -> &str {
            &self.raw[self.$name]
        }
    };
}

#[allow(unused_macros)]
macro_rules! opt_str_field {
    ($name:ident) => {
        pub fn $name(&self) -> Option<&str> {
            self.$name.map(|index| &self.raw[index])
        }
    };
}

#[allow(unused_macros)]
macro_rules! tags {
    () => {
        /// Get a view of parsable tags
        pub fn tags(&self) -> Tags<'_> {
            Tags {
                data: &self.raw,
                indices: &self.tags,
            }
        }
    };
}

#[derive(Debug)]
pub enum InvalidMessage {
    InvalidCommand { expected: String, got: String },
    ExpectedNick,

    ExpectedArg { pos: usize },
    ExpectedData,
}

impl std::fmt::Display for InvalidMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCommand { expected, got } => {
                write!(f, "invalid command. expected '{}' got '{}'", expected, got)
            }
            Self::ExpectedNick => write!(f, "expected a nickname attached to that message"),
            Self::ExpectedArg { pos } => write!(f, "expected arg at position: {}", pos),
            Self::ExpectedData => write!(f, "expected a data segment in the message"),
        }
    }
}

impl std::error::Error for InvalidMessage {}

pub trait FromIrcMessage<'a> {
    type Error;

    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl<'a> FromIrcMessage<'a> for IrcMessage<'a> {
    type Error = Infallible;

    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error> {
        Ok(msg)
    }
}

trait Validator {
    fn parse_tags(&self) -> TagIndices;
    fn expect_command(&self, cmd: &str) -> Result<(), InvalidMessage>;
    fn expect_nick(&self) -> Result<StrIndex, InvalidMessage>;
    fn expect_arg(&self, nth: usize) -> Result<&str, InvalidMessage>;
    fn expect_arg_index(&self, nth: usize) -> Result<StrIndex, InvalidMessage>;
    fn expect_data(&self) -> Result<&str, InvalidMessage>;
    fn expect_data_index(&self) -> Result<StrIndex, InvalidMessage>;
}

impl<'a> Validator for IrcMessage<'a> {
    fn parse_tags(&self) -> TagIndices {
        self.tags
            .map(|index| TagIndices::parse(&self.raw[index]))
            .unwrap_or_default()
    }

    fn expect_command(&self, cmd: &str) -> Result<(), InvalidMessage> {
        if self.get_command() != cmd {
            return Err(InvalidMessage::InvalidCommand {
                expected: cmd.to_string(),
                got: self.get_command().to_string(),
            });
        }
        Ok(())
    }

    fn expect_nick(&self) -> Result<StrIndex, InvalidMessage> {
        self.prefix
            .and_then(|p| p.nick_index())
            .ok_or_else(|| InvalidMessage::ExpectedNick)
    }

    fn expect_arg(&self, nth: usize) -> Result<&str, InvalidMessage> {
        self.nth_arg(nth)
            .ok_or_else(|| InvalidMessage::ExpectedArg { pos: nth })
    }

    fn expect_arg_index(&self, nth: usize) -> Result<StrIndex, InvalidMessage> {
        self.nth_arg_index(nth)
            .ok_or_else(|| InvalidMessage::ExpectedArg { pos: nth })
    }

    fn expect_data(&self) -> Result<&str, InvalidMessage> {
        self.expect_data_index().map(|index| &self.raw[index])
    }

    fn expect_data_index(&self) -> Result<StrIndex, InvalidMessage> {
        self.data.ok_or_else(|| InvalidMessage::ExpectedData)
    }
}

mod all_commands;
pub use all_commands::AllCommands;

mod irc_ready;
pub use irc_ready::IrcReady;

mod ready;
pub use ready::Ready;

mod cap;
pub use cap::Cap;

mod clear_chat;
pub use clear_chat::ClearChat;

mod clear_msg;
pub use clear_msg::ClearMsg;

// mod global_user_state;
// pub use global_user_state::GlobalUserState;

mod host_target;
pub use host_target::HostTarget;

mod join;
pub use join::Join;

mod notice;
pub use notice::Notice;

mod part;
pub use part::Part;

mod ping;
pub use ping::Ping;

mod pong;
pub use pong::Pong;

// mod privmsg;
// pub use privmsg::Privmsg;

// mod reconnect;
// pub use reconnect::Reconnect;

// mod room_state;
// pub use room_state::RoomState;

// mod user_notice;
// pub use user_notice::UserNotice;

// mod user_state;
// pub use user_state::UserState;

// mod whisper;
// pub use whisper::Whisper;
