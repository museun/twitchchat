use super::{commands, Message};

macro_rules! filter_this {
    ($($t:ident),+ $(,)?) => {
        $(impl MessageFilter for commands::$t {
            fn to_filter() -> Filter {
                Filter::$t
            }
        }

        impl From<Message> for commands::$t {
            fn from(msg: Message) -> Self {
                match msg {
                    Message::$t(d @ commands::$t { .. }) => d,
                    _ => unreachable!(),
                }
            }
        })*
    };
}

filter_this!(
    Join,            //
    Part,            //
    PrivMsg,         //
    Mode,            //
    NamesStart,      //
    NamesEnd,        //
    ClearChat,       //
    ClearMsg,        //
    HostTargetStart, //
    HostTargetEnd,   //
    Notice,          //
    Reconnect,       //
    RoomState,       //
    UserNotice,      //
    UserState,       //
    GlobalUserState, //
);

// special cast the Boxed IRC mesage
impl MessageFilter for crate::irc::Message {
    fn to_filter() -> Filter {
        Filter::Irc
    }
}

impl From<Message> for crate::irc::Message {
    fn from(msg: Message) -> Self {
        match msg {
            Message::Irc(msg) => *msg,
            _ => unreachable!(),
        }
    }
}

/// A filter that can be applied to [`Client::filter`](./struct.Client.html#method.filter)
#[derive(Copy, Clone, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub enum Filter {
    /// An irc Message
    Irc,
    /// Join a channel.
    Join,
    /// Depart from a channel.
    Part,
    /// Send a message to a channel.
    PrivMsg,
    /// Gain/lose moderator (operator) status in a channel.
    Mode,
    /// List current chatters in a channel. (begin)
    NamesStart,
    /// List current chatters in a channel. (end)
    NamesEnd,
    /// Purge a user's typically after a user is banned from chat or timed out.
    ClearChat,
    /// Single message removal on a channel. This is triggered via /delete <target-msg-id> on IRC.
    ClearMsg,
    /// Channel starts host mode.
    HostTargetStart,
    /// Channel stops host mode.
    HostTargetEnd,
    /// General notices from the server.
    Notice,
    /// Rejoin channels after a restart.
    Reconnect,
    /// Identifies the channel's chat settings (e.g., slow mode duration).
    RoomState,
    /// Announces Twitch-specific events to the channel (e.g., a user's subscription notification).
    UserNotice,
    /// Identifies a user's chat settings or properties (e.g., chat color)..
    UserState,
    /// On successful login.
    GlobalUserState,
    // Reserve the right to add more fields to this enum
    #[doc(hidden)]
    __Nonexhaustive,
}

pub trait MessageFilter {
    fn to_filter() -> Filter;
}

impl Message {
    pub(crate) fn what_filter(&self) -> Filter {
        use Filter::*;
        match self {
            Message::Join { .. } => Join,
            Message::Part { .. } => Part,
            Message::PrivMsg { .. } => PrivMsg,
            Message::Mode { .. } => Mode,
            Message::NamesStart { .. } => NamesStart,
            Message::NamesEnd { .. } => NamesEnd,
            Message::ClearChat { .. } => ClearChat,
            Message::ClearMsg { .. } => ClearMsg,
            Message::HostTargetStart { .. } => HostTargetStart,
            Message::HostTargetEnd { .. } => HostTargetEnd,
            Message::Notice { .. } => Notice,
            Message::Reconnect { .. } => Reconnect,
            Message::RoomState { .. } => RoomState,
            Message::UserNotice { .. } => UserNotice,
            Message::UserState { .. } => UserState,
            Message::GlobalUserState { .. } => GlobalUserState,
            Message::Irc { .. } => Irc,
            Message::__Nonexhaustive => unreachable!(),
        }
    }
}
