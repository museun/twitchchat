//! Twitch messages that can be parsed from `IrcMessage`, or subscribed to from the `Dispatcher`
//!
macro_rules! serde_struct {
    (@one $($x:tt)*) => { () };
    (@len $($e:expr),*) => { <[()]>::len(&[$(serde_struct!(@one $e)),*]); };

    ($ty:ident { $($field:ident),* $(,)? }) => {
        #[cfg(feature = "serde")]
        impl<'a> ::serde::Serialize for $ty<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                use ::serde::ser::SerializeMap as _;
                let len = serde_struct!(@len $($field),*);
                let mut s = serializer.serialize_map(Some(len))?;
                $( s.serialize_entry(stringify!($field), &self.$field())?; )*
                s.end()
            }
        }

        serde_struct!{ @de $ty }
    };

    (@de $ty:ident) => {
        #[cfg(feature = "serde")]
        impl<'de, 'a> ::serde::Deserialize<'de> for $ty<'a> {
            fn deserialize<D>(deserializer: D) -> Result<$ty<'a>, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                deserializer.deserialize_map($crate::serde::RawVisitor::default())
            }
        }
    };
}

macro_rules! impl_custom_debug {
    ($ty:ident { $($field:ident),* $(,)? }) => {
        impl<'a> std::fmt::Debug for $ty<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($ty))
                    $( .field(stringify!($field), &self.$field()) )*
                .finish()
            }
        }
    };
}

// TODO get rid of these macros
macro_rules! raw {
    () => {
        /// Get the raw message
        pub fn raw(&self) -> &str {
            &*self.raw
        }
    };
}

macro_rules! into_inner_raw {
    () => {
        /// Consumes the message, returning the raw [`Str<'_>`](./enum.Str.html)
        fn into_inner(self) -> Str<'a> {
            self.raw
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

macro_rules! str_field {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        pub fn $name(&self) -> &str {
            &self.raw[self.$name]
        }
    };
    ($name:ident) => {
        pub fn $name(&self) -> &str {
            &self.raw[self.$name]
        }
    };
}

macro_rules! opt_str_field {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        pub fn $name(&self) -> Option<&str> {
            self.$name.map(|index| &self.raw[index])
        }
    };

    ($name:ident) => {
        pub fn $name(&self) -> Option<&str> {
            self.$name.map(|index| &self.raw[index])
        }
    };
}

mod commands;
pub use commands::Commands;

mod irc_ready;
pub use irc_ready::IrcReady;

mod ready;
pub use ready::Ready;

mod cap;
pub use cap::{Cap, Capability};

mod clear_chat;
pub use clear_chat::ClearChat;

mod clear_msg;
pub use clear_msg::ClearMsg;

mod global_user_state;
pub use global_user_state::GlobalUserState;

mod host_target;
pub use host_target::{HostTarget, HostTargetKind};

mod join;
pub use join::Join;

mod notice;
pub use notice::{MessageId, Notice};

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
pub use room_state::{FollowersOnly, RoomState};

mod user_notice;
pub use user_notice::{NoticeType, SubPlan, UserNotice};

mod user_state;
pub use user_state::UserState;

mod whisper;
pub use whisper::Whisper;

pub use crate::irc::IrcMessage;
