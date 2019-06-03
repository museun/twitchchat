mod badge;
mod color;
mod emotes;

use std::sync::atomic::{AtomicUsize, Ordering};

use hashbrown::HashMap;

pub use self::badge::{Badge, BadgeInfo, BadgeKind};
pub use self::emotes::Emotes;

pub use self::color::{twitch_colors as colors, Color, TwitchColor, RGB};

/// An assortment of Twitch commands
pub mod commands;

mod capability;
pub use self::capability::Capability;

mod error;
pub use self::error::Error;

mod client;
pub use self::client::Client;

mod adapter;
pub use self::adapter::{ReadAdapter, ReadError, SyncReadAdapter};

mod writer;
pub use self::writer::Writer;

mod extension;
pub use self::extension::WriterExt;

/// Twitch channel types
mod channel;
pub use self::channel::Channel;

#[doc(hidden)]
pub mod userconfig;
pub use self::userconfig::UserConfig;
pub use self::userconfig::UserConfigBuilder;

/// Information gathered during the [`GLOBALUSERSTATE`](./commands/struct.GlobalUserState.html) event
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LocalUser {
    /// Your user id
    pub user_id: u64,
    /// Your display name, if set
    pub display_name: Option<String>,
    /// Your color, if set
    pub color: Option<Color>,
    /// Your badges
    pub badges: Vec<Badge>,
    /// Your list of emote sets
    pub emote_sets: Vec<u64>,
    /// The capabilities the server acknowledged
    pub caps: Vec<Capability>,
}

/// Messages created by the [`Client`](./struct.Client.html).
///
/// Wraps [`commands`](./commands/index.html)
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Message {
    /// An irc Message
    Irc(crate::irc::types::Message),
    /// Join a channel.
    Join(commands::Join),
    /// Depart from a channel.
    Part(commands::Part),
    /// Send a message to a channel.
    PrivMsg(commands::PrivMsg),
    /// Gain/lose moderator (operator) status in a channel.
    Mode(commands::Mode),
    /// List current chatters in a channel. (begin)
    NamesStart(commands::NamesStart),
    /// List current chatters in a channel. (end)
    NamesEnd(commands::NamesEnd),
    /// Purge a user's typically after a user is banned from chat or timed out.
    ClearChat(commands::ClearChat),
    /// Single message removal on a channel. This is triggered via /delete
    /// <target-msg-id> on IRC.
    ClearMsg(commands::ClearMsg),
    /// Channel starts host mode.
    HostTargetStart(commands::HostTargetStart),
    /// Channel stops host mode.
    HostTargetEnd(commands::HostTargetEnd),
    /// General notices from the server.
    Notice(commands::Notice),
    /// Rejoin channels after a restart.
    Reconnect(commands::Reconnect),
    /// Identifies the channel's chat settings (e.g., slow mode duration).
    RoomState(commands::RoomState),
    /// Announces Twitch-specific events to the channel (e.g., a user's
    /// subscription notification).
    UserNotice(commands::UserNotice),
    /// Identifies a user's chat settings or properties (e.g., chat color)..
    UserState(commands::UserState),
    /// On successful login.
    GlobalUserState(commands::GlobalUserState),
    // Reserve the right to add more fields to this enum
    #[doc(hidden)]
    __Nonexhaustive,
}

impl Message {
    /// Converts a message into the internal message type, then into the Twitch 'command'
    pub fn parse(msg: impl crate::ToMessage) -> Self {
        // TODO be smarter about this
        use crate::irc::types::{Message as IrcMessage, Prefix};
        let msg = IrcMessage::Unknown {
            prefix: msg.prefix().map(|nick| Prefix::User {
                nick: nick.to_string(),
                user: nick.to_string(),
                host: nick.to_string(),
            }),
            tags: match msg.tags() {
                Some(crate::TagType::Raw(raw)) => crate::Tags::parse(raw),
                Some(crate::TagType::List(list)) => crate::Tags(
                    list.clone()
                        .into_iter()
                        .collect::<HashMap<String, String>>(),
                ),
                Some(crate::TagType::Map(map)) => crate::Tags(map.clone()),
                None => crate::Tags::default(),
            },
            head: msg.command().map(ToString::to_string).unwrap_or_default(),
            args: match msg.args() {
                Some(crate::ArgsType::Raw(raw)) => {
                    raw.split(' ').map(ToString::to_string).collect()
                }
                Some(crate::ArgsType::List(list)) => list.clone(),
                None => vec![],
            },
            tail: msg.data().map(ToString::to_string),
        };

        commands::parse(&msg).unwrap_or_else(|| Message::Irc(msg))
    }
}

pub(crate) mod filter;

mod handler;
pub use handler::Handler;

/// Token allows you to keep track of things
///
/// Keep this around if you want to remove the thing.
///
/// This is used in both the simple filters ([`Client::on`](./struct.Client.html#method.on) and [`Client::off`](./struct.Client.html#method.off))
///
/// and in the more flexible handler system ([`Client::handler`](./struct.Client.html#method.handler), [`Client::remove_handler`](./struct.Client.html#method.remove_handler))
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Token(pub(super) usize);

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Default)]
pub(crate) struct TokenGen(AtomicUsize);

impl TokenGen {
    pub fn next(&mut self) -> Token {
        Token(self.0.fetch_add(1, Ordering::Relaxed))
    }
}
