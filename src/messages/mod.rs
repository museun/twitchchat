/*!
Messages sent by the server

This can be obtained by [subscribing] to an [Event] on a [Dispatcher]

Or by using [Parse] on an [Message]

[subscribing]: ../client/struct.Dispatcher.html#method.subscribe
[Event]: ../events/index.html
[Dispatcher]: ../client/struct.Dispatcher.html
[Parse]: ../trait.Parse.html
[Message]: ../decode/struct.Message.html
*/

use crate::decode::Message;
use crate::Tags;
use crate::{Conversion, Parse, StringMarker};

mod error;
pub use error::InvalidMessage;

mod expect;
use expect::Expect as _;

pub use parse::*;
mod parse;

/// A raw IRC message
pub type Raw<T> = Message<T>;

/// Acknowledgement (or not) on a CAPS request
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cap<T = String>
where
    T: StringMarker,
{
    /// The capability name
    pub capability: T,
    /// Whether it was acknowledged
    pub acknowledged: bool,
}

/// When a user's message(s) have been purged.
///
/// Typically after a user is banned from chat or timed out
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClearChat<T = String>
where
    T: StringMarker,
{
    /// Tags attached to the message
    pub tags: Tags<T>,
    /// The channel this event happened on
    pub channel: T,
    /// The user, if any, that was being purged
    pub user: Option<T>,
}

/// When a single message has been removed from a channel.
///
/// This is triggered via /delete on IRC.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClearMsg<T = String>
where
    T: StringMarker,
{
    /// Tags attached to the message
    pub tags: Tags<T>,
    /// The channel this event happened on
    pub channel: T,
    /// The message that was deleted
    pub message: Option<T>,
}

/// Sent on successful login, if TAGs caps have been sent beforehand
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GlobalUserState<T = String>
where
    T: StringMarker,
{
    /// Your user-id
    pub user_id: T,
    /// Your display name, if set   
    pub display_name: Option<T>,
    /// Your color, if set. Defaults to `white`
    pub color: crate::color::Color,
    /// Your available emote sets, always contains atleast '0'
    pub emote_sets: Vec<T>,
    /// Any badges you have
    pub badges: Vec<crate::Badge<T>>,
}

/// Event kind for determine when a Host event beings or end
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HostTargetKind<T = String>
where
    T: StringMarker,
{
    /// The host event started
    Start {
        /// Target channel that is being hosted
        target: T,
    },
    /// The host event ended
    End,
}

/// When a channel starts to host another channel
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HostTarget<T = String>
where
    T: StringMarker,
{
    /// Source channel (the one doing the hosting).
    pub source: T,
    /// How many viewers are going along
    pub viewers: Option<usize>,
    /// What kind of event this was. e.g. `Start` or `End`
    pub kind: HostTargetKind<T>,
}

/// Happens when the IRC connection has been succesfully established
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrcReady<T = String>
where
    T: StringMarker,
{
    /// The name the server will refer to you as
    pub nickname: T,
}

/// User join message
///
/// The happens when a user (yourself included) joins a channel
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Join<T = String>
where
    T: StringMarker,
{
    /// Name of the user that joined the channel
    pub user: T,
    /// Channel which they joined
    pub channel: T,
}

/// Status of gaining or losing moderator (operator) status
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ModeStatus {
    /// Moderator status gained
    Gained,
    /// Moderator status lost
    Lost,
}

/// When a user gains or loses moderator (operator) status in a channel.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mode<T = String>
where
    T: StringMarker,
{
    /// The channel this event happened on
    pub channel: T,
    /// The status. gained, or lost
    pub status: ModeStatus,
    /// The user this applies too
    pub user: T,
}

/// The kind of the Names event
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NamesKind<T = String>
where
    T: StringMarker,
{
    /// Names begins, this'll continue until `End` is recieved
    Start {
        /// A list of user names
        users: Vec<T>,
    },
    /// Names end, this'll mark the end of the event
    End,
}

/// The names event
///
/// This'll will list people on a channel for your user
///
/// The `kind` field lets you determine if its still 'happening'
///
/// Your should keep a list of the names from the `Start` variant
///
/// And once you receive an End you'll have the complete lost
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Names<T = String>
where
    T: StringMarker,
{
    /// Your username
    pub user: T,
    /// The channel this event is happening for
    pub channel: T,
    /// The state of the event
    pub kind: NamesKind<T>,
}

/// General notices from the server.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Notice<T = String>
where
    T: StringMarker,
{
    /// The tags attached to this message
    pub tags: Tags<T>,
    /// The channel this event happened on
    pub channel: T,
    /// The message from the server
    pub message: T,
}

/// User leave message
///
/// The happens when a user (yourself included) leaves a channel
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Part<T = String>
where
    T: StringMarker,
{
    /// Name of the user that left the channel
    pub user: T,
    /// Channel which they left
    pub channel: T,
}

/// A ping request from the server
///
/// This is sent periodically, and handled by the `Client` internally
///
/// But you can use them however you want
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ping<T = String>
where
    T: StringMarker,
{
    /// Token associated with the PING event
    pub token: T,
}

/// A pong response sent from the server
///
/// This should be a response to sending a PING to the server
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pong<T = String>
where
    T: StringMarker,
{
    /// Token associated with the PONG event
    pub token: T,
}

/// Message sent by a user
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Privmsg<T = String>
where
    T: StringMarker,
{
    /// User who sent this messages
    pub user: T,
    /// Channel this message was sent on
    pub channel: T,
    /// Data that the user provided
    pub data: T,
    /// Tags attached to the message
    pub tags: Tags<T>,
}

/// Happens when the Twitch connection has been succesfully established
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ready<T = String>
where
    T: StringMarker,
{
    /// The name Twitch will refer to you as
    pub username: T,
}

/// Signals that you should reconnect and rejoin channels after a restart.
///
/// Twitch IRC processes occasionally need to be restarted. When this happens,
/// clients that have requested the IRC v3 twitch.tv/commands capability are
/// issued a RECONNECT. After a short time, the connection is closed. In this
/// case, reconnect and rejoin channels that were on the connection, as you
/// would normally.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Reconnect {}

/// Identifies the channel's chat settings (e.g., slow mode duration).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RoomState<T = String>
where
    T: StringMarker,
{
    /// Tags attached to this message
    pub tags: Tags<T>,
    /// Channel this event is happening on
    pub channel: T,
}

/// Announces Twitch-specific events to the channel (e.g., a user's subscription notification).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UserNotice<T = String>
where
    T: StringMarker,
{
    /// Tags attached to this message
    pub tags: Tags<T>,
    /// Channel this event is happening on
    pub channel: T,
    /// Optional message attached to the event
    pub message: Option<T>,
}

/// Identifies a user's chat settings or properties (e.g., chat color)..
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UserState<T = String>
where
    T: StringMarker,
{
    /// Tags attached to this message
    pub tags: Tags<T>,
    /// Channel this event happened on
    pub channel: T,
}

/// This is a collection of all possible message types
///
/// Subscribing to [events::All][all] will produce this, so you can have a single stream for multiple messages.
///
/// [all]: ../events/struct.All.html
#[allow(missing_docs)]
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AllCommands<T = String>
where
    T: StringMarker,
{
    Unknown(Raw<T>),
    Cap(Cap<T>),
    ClearChat(ClearChat<T>),
    ClearMsg(ClearMsg<T>),
    GlobalUserState(GlobalUserState<T>),
    HostTarget(HostTarget<T>),
    IrcReady(IrcReady<T>),
    Join(Join<T>),
    Mode(Mode<T>),
    Names(Names<T>),
    Notice(Notice<T>),
    Part(Part<T>),
    Ping(Ping<T>),
    Pong(Pong<T>),
    Privmsg(Privmsg<T>),
    Ready(Ready<T>),
    Reconnect(Reconnect),
    RoomState(RoomState<T>),
    UserNotice(UserNotice<T>),
    UserState(UserState<T>),
}

impl<T> From<Raw<T>> for AllCommands<T>
where
    T: StringMarker,
{
    fn from(msg: Raw<T>) -> Self {
        Self::Unknown(msg)
    }
}

impl<T> From<Cap<T>> for AllCommands<T>
where
    T: StringMarker,
{
    fn from(msg: Cap<T>) -> Self {
        Self::Cap(msg)
    }
}

impl<T> From<ClearChat<T>> for AllCommands<T>
where
    T: StringMarker,
{
    fn from(msg: ClearChat<T>) -> Self {
        Self::ClearChat(msg)
    }
}

impl<T> From<ClearMsg<T>> for AllCommands<T>
where
    T: StringMarker,
{
    fn from(msg: ClearMsg<T>) -> Self {
        Self::ClearMsg(msg)
    }
}

impl<T> From<GlobalUserState<T>> for AllCommands<T>
where
    T: StringMarker,
{
    fn from(msg: GlobalUserState<T>) -> Self {
        Self::GlobalUserState(msg)
    }
}

impl<T> From<HostTarget<T>> for AllCommands<T>
where
    T: StringMarker,
{
    fn from(msg: HostTarget<T>) -> Self {
        Self::HostTarget(msg)
    }
}

impl<T> From<IrcReady<T>> for AllCommands<T>
where
    T: StringMarker,
{
    fn from(msg: IrcReady<T>) -> Self {
        Self::IrcReady(msg)
    }
}

impl<T> From<Join<T>> for AllCommands<T>
where
    T: StringMarker,
{
    fn from(msg: Join<T>) -> Self {
        Self::Join(msg)
    }
}

impl<T> From<Mode<T>> for AllCommands<T>
where
    T: StringMarker,
{
    fn from(msg: Mode<T>) -> Self {
        Self::Mode(msg)
    }
}

impl<T> From<Names<T>> for AllCommands<T>
where
    T: StringMarker,
{
    fn from(msg: Names<T>) -> Self {
        Self::Names(msg)
    }
}

impl<T> From<Notice<T>> for AllCommands<T>
where
    T: StringMarker,
{
    fn from(msg: Notice<T>) -> Self {
        Self::Notice(msg)
    }
}

impl<T> From<Part<T>> for AllCommands<T>
where
    T: StringMarker,
{
    fn from(msg: Part<T>) -> Self {
        Self::Part(msg)
    }
}

impl<T> From<Ping<T>> for AllCommands<T>
where
    T: StringMarker,
{
    fn from(msg: Ping<T>) -> Self {
        Self::Ping(msg)
    }
}

impl<T> From<Pong<T>> for AllCommands<T>
where
    T: StringMarker,
{
    fn from(msg: Pong<T>) -> Self {
        Self::Pong(msg)
    }
}

impl<T> From<Privmsg<T>> for AllCommands<T>
where
    T: StringMarker,
{
    fn from(msg: Privmsg<T>) -> Self {
        Self::Privmsg(msg)
    }
}

impl<T> From<Ready<T>> for AllCommands<T>
where
    T: StringMarker,
{
    fn from(msg: Ready<T>) -> Self {
        Self::Ready(msg)
    }
}

impl<T> From<Reconnect> for AllCommands<T>
where
    T: StringMarker,
{
    fn from(msg: Reconnect) -> Self {
        Self::Reconnect(msg)
    }
}

impl<T> From<RoomState<T>> for AllCommands<T>
where
    T: StringMarker,
{
    fn from(msg: RoomState<T>) -> Self {
        Self::RoomState(msg)
    }
}

impl<T> From<UserNotice<T>> for AllCommands<T>
where
    T: StringMarker,
{
    fn from(msg: UserNotice<T>) -> Self {
        Self::UserNotice(msg)
    }
}

impl<T> From<UserState<T>> for AllCommands<T>
where
    T: StringMarker,
{
    fn from(msg: UserState<T>) -> Self {
        Self::UserState(msg)
    }
}

#[cfg(test)]
mod tests;
