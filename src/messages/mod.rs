/*!
Messages sent by the server

This can be obtained by [subscribing] to an [Event] on a [Dispatcher]

Or by using [Parse] on an [Message]

[subscribing]: ../struct.Dispatcher.html#method.subscribe
[Event]: ../events/index.html
[Dispatcher]: ../struct.Dispatcher.html
[Parse]: ../../trait.Parse.html
[Message]: ../../decode/struct.Message.html
*/

use crate::decode::Message;
use crate::Conversion as _;
use crate::StringMarker;
use crate::Tags;

mod error;
pub use error::InvalidMessage;

mod expect;
use expect::Expect as _;

pub use parse::*;
mod parse;

/// A raw IRC message
pub type Raw<T> = Message<T>;

/// The kind of the Names event
#[derive(Debug, Clone, PartialEq)]
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

/// Identifies the channel's chat settings (e.g., slow mode duration).
#[derive(Debug, Clone, PartialEq)]
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

/// Sent on successful login, if TAGs caps have been sent beforehand
#[derive(Debug, Clone, PartialEq)]
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

/// Acknowledgement (or not) on a CAPS request
#[derive(Debug, Clone, PartialEq)]
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

/// Happens when the IRC connection has been succesfully established
#[derive(Debug, Clone, PartialEq)]
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
pub enum ModeStatus {
    /// Moderator status gained
    Gained,
    /// Moderator status lost
    Lost,
}

/// When a user gains or loses moderator (operator) status in a channel.
#[derive(Debug, Clone, PartialEq)]
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

/// General notices from the server.
#[derive(Debug, Clone, PartialEq)]
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
pub struct Pong<T = String>
where
    T: StringMarker,
{
    /// Token associated with the PONG event
    pub token: T,
}

/// Message sent by a user
#[derive(Debug, Clone, PartialEq)]
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
pub struct Reconnect {}

/// Identifies a user's chat settings or properties (e.g., chat color)..
#[derive(Debug, Clone, PartialEq)]
pub struct UserState<T = String>
where
    T: StringMarker,
{
    /// Tags attached to this message
    pub tags: Tags<T>,
    /// Channel this event happened on
    pub channel: T,
}

#[cfg(test)]
mod tests;
