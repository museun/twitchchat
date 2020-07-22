use super::IrcMessage;

macro_rules! tags {
    () => {
        pub fn tags(&self) -> &Tags<'_> {
            &self.tags
        }
    };
}

macro_rules! into_inner {
    () => {
        pub fn into_inner(self) -> IrcMessage<'a> {
            self.msg
        }
    };
}
macro_rules! reborrow {
    ($ty:ident { $($field:tt),* }) => {
        impl<'a> Reborrow<'a> for $ty<'a> {
            fn reborrow<'b: 'a>(this: &'b $ty<'a>) -> $ty<'b> {
                $ty { $( $field: Reborrow::reborrow(&this.$field), )* }
            }
        }
    };
}

#[derive(Debug)]
pub enum ParseError {
    InvalidCommand { expected: String, got: String },
    ExpectedNick,
    ExpectedArg { pos: usize },
    ExpectedData,
}

pub trait FromIrcMessage<'a> {
    type Error;
    fn from_irc(msg: &IrcMessage<'a>) -> Result<Self, Self::Error>
    where
        Self: Sized + 'a;
}

impl<'a> FromIrcMessage<'a> for IrcMessage<'a> {
    type Error = ();
    fn from_irc(msg: &IrcMessage<'a>) -> Result<IrcMessage<'a>, Self::Error>
    where
        Self: Sized + 'a,
    {
        // TODO use the new MaybeOwned idea
        Ok(msg.clone())
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
pub use whisper::Whisper;
