use super::*;

/// Messages created by the [`Client`](./struct.Client.html).
///
/// Wraps [`commands`](./commands/index.html)
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Message {
    /// An irc Message
    Irc(Box<crate::irc::Message>),
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
        use crate::conversion::{ArgsType, TagType};
        use crate::irc::{Message as IrcMessage, Prefix};

        let msg = IrcMessage::Unknown {
            prefix: msg.prefix().map(|nick| Prefix::User {
                nick: nick.to_string(),
                user: nick.to_string(),
                host: nick.to_string(),
            }),
            tags: match msg.tags() {
                Some(TagType::Raw(raw)) => crate::Tags::parse(raw),
                Some(TagType::List(list)) => crate::Tags(
                    list.clone()
                        .into_iter()
                        .collect::<HashMap<String, String>>(),
                ),
                Some(TagType::Map(map)) => crate::Tags(map.clone()),
                None => crate::Tags::default(),
            },
            head: msg.command().map(ToString::to_string).unwrap_or_default(),
            args: match msg.args() {
                Some(ArgsType::Raw(raw)) => raw.split(' ').map(ToString::to_string).collect(),
                Some(ArgsType::List(list)) => list.clone(),
                None => vec![],
            },
            tail: msg.data().map(ToString::to_string),
        };

        commands::parse(&msg).unwrap_or_else(|| Message::Irc(Box::new(msg)))
    }
}
