use super::{IrcMessage, Str, StrIndex};

// TODO get rid of these macros
macro_rules! raw {
    () => {
        /// Get the raw message
        pub fn raw(&self) -> &str {
            &*self.raw
        }

        /// Get the raw `Str<'a>` out
        pub fn into_inner(self) -> Str<'t> {
            self.raw
        }
    };
}

macro_rules! str_field {
    ($name:ident) => {
        pub fn $name(&self) -> &str {
            &self.raw[self.$name]
        }
    };
}

macro_rules! opt_str_field {
    ($name:ident) => {
        pub fn $name(&self) -> Option<&str> {
            self.$name.map(|index| &self.raw[index])
        }
    };
}

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

// mod privmsg;
// pub use privmsg::Privmsg;

mod reconnect;
pub use reconnect::Reconnect;

mod room_state;
pub use room_state::RoomState;

// mod user_notice;
// pub use user_notice::UserNotice;

mod user_state;
pub use user_state::UserState;

mod whisper;
pub use whisper::Whisper;
