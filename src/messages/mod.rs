/*!
Messages sent by the server

This can be obtained by [subscribing] to an [Event] on a [Dispatcher]

Or by using [TryFrom] on an [Message]

[subscribing]: ../struct.Dispatcher.html#method.subscribe
[Event]: ../events/index.html
[Dispatcher]: ../struct.Dispatcher.html
[TryFrom]: https://doc.rust-lang.org/std/convert/trait.TryFrom.html
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

pub type Raw<T> = crate::decode::Message<T>;

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalUserState<T = String>
where
    T: StringMarker,
{
    pub user_id: T,
    pub display_name: Option<T>,
    pub color: crate::color::Color,
    pub emote_sets: Vec<T>,
    pub badges: Vec<crate::Badge<T>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HostTargetKind<T = String>
where
    T: StringMarker,
{
    Start { target: T },
    End,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HostTarget<T = String>
where
    T: StringMarker,
{
    pub source: T,
    pub viewers: Option<usize>,
    pub kind: HostTargetKind<T>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cap<T = String>
where
    T: StringMarker,
{
    pub capability: T,
    pub acknowledged: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClearChat<T = String>
where
    T: StringMarker,
{
    pub tags: Tags<T>,
    pub channel: T,
    pub user: Option<T>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClearMsg<T = String>
where
    T: StringMarker,
{
    pub tags: Tags<T>,
    pub channel: T,
    pub message: Option<T>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrcReady<T = String>
where
    T: StringMarker,
{
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

// as_owned!(for ModeStatus);

#[derive(Debug, Clone, PartialEq)]
pub struct Mode<T = String>
where
    T: StringMarker,
{
    pub channel: T,
    pub status: ModeStatus,
    pub user: T,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Notice<T = String>
where
    T: StringMarker,
{
    pub tags: Tags<T>,
    pub channel: T,
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

#[derive(Debug, Clone, PartialEq)]
pub struct Ping<T = String>
where
    T: StringMarker,
{
    /// Token associated with the PING event
    pub token: T,
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct Ready<T = String>
where
    T: StringMarker,
{
    pub username: T,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reconnect {}

#[derive(Debug, Clone, PartialEq)]
pub struct UserState<T = String>
where
    T: StringMarker,
{
    pub tags: Tags<T>,
    pub channel: T,
}

#[cfg(test)]
mod tests;
