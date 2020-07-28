use super::{AsOwned, IrcMessage, Reborrow, Str, Tags};

#[allow(unused_macros)]
macro_rules! tags {
    () => {
        pub fn tags(&self) -> &Tags<'_> {
            &self.tags
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

pub trait FromIrcMessage<'a> {
    type Error;

    fn from_irc(msg: &'a IrcMessage<'a>) -> Result<Self, Self::Error>
    where
        Self: Sized + 'a;
}

impl<'a> FromIrcMessage<'a> for IrcMessage<'a> {
    type Error = Infallible;

    fn from_irc(msg: &'a IrcMessage<'a>) -> Result<Self, Self::Error> {
        Ok(msg.clone())
    }
}

trait Validator<'b: 'a, 'a> {
    fn parse_tags(&'b self) -> Tags<'b>;
    fn expect_command(&'b self, cmd: &str) -> Result<(), InvalidMessage>;
    fn expect_nick(&'b self) -> Result<Str<'b>, InvalidMessage>;
    fn expect_arg(&'b self, nth: usize) -> Result<Str<'b>, InvalidMessage>;
    fn expect_data(&'b self) -> Result<Str<'b>, InvalidMessage>;
}

impl<'b: 'a, 'a> Validator<'b, 'a> for IrcMessage<'a> {
    fn parse_tags(&'b self) -> Tags<'b> {
        self.tags.unwrap_or_default()
    }

    fn expect_command(&'b self, cmd: &str) -> Result<(), InvalidMessage> {
        if self.command != cmd {
            return Err(InvalidMessage::InvalidCommand {
                expected: cmd.to_string(),
                got: self.command.to_string(),
            });
        }

        Ok(())
    }

    fn expect_nick(&'b self) -> Result<Str<'b>, InvalidMessage> {
        self.prefix
            .as_ref()
            .and_then(|p| p.get_nick())
            .ok_or_else(|| InvalidMessage::ExpectedNick)
    }

    fn expect_arg(&'b self, nth: usize) -> Result<Str<'b>, InvalidMessage> {
        self.nth_arg(nth)
            .ok_or_else(|| InvalidMessage::ExpectedArg { pos: nth })
    }

    fn expect_data(&'b self) -> Result<Str<'b>, InvalidMessage> {
        self.get_data().ok_or_else(|| InvalidMessage::ExpectedData)
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

mod global_user_state;
pub use global_user_state::GlobalUserState;

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

mod privmsg;
pub use privmsg::Privmsg;

mod reconnect;
pub use reconnect::Reconnect;

mod room_state;
pub use room_state::RoomState;

mod user_notice;
pub use user_notice::UserNotice;

mod user_state;
pub use user_state::UserState;

mod whisper;
use std::convert::Infallible;
pub use whisper::Whisper;
