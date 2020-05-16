#![allow(missing_debug_implementations, missing_copy_implementations)]
/*!
Available event filters.

These can be used with [Dispatcher::subscribe][Subscribe] to get a [Stream] of [Messages] filtered to this event.

See the [table]

[Subscribe]: ../struct.Dispatcher.html#method.subscribe
[Stream]: https://docs.rs/tokio/0.2/tokio/stream/trait.Stream.html
[Messages]: ../messages/index.html
[table]: ../struct.Dispatcher.html#a-table-of-mappings
*/
use super::messages;

use crate::AsOwned;
use std::fmt::Debug;

/// A marker trait for Event subscription
#[doc(hidden)]
pub trait Event<'a>: private::EventSealed {
    /// Event message parsing
    type Parsed: crate::Parse<&'a crate::decode::Message<'a>> + AsOwned;
}

/// A trait to convert an Event::Parsed to a 'static type
#[doc(hidden)]
pub trait EventMapped<'a, T>: private::EventMappedSealed<T>
where
    T: Event<'a>,
{
    /// Event message mapping
    type Owned: Clone + Debug + Send + Sync + 'static;
    /// Converts this to the owned representation
    fn into_owned(data: T::Parsed) -> Self::Owned;
}

impl<'a, T> EventMapped<'a, T> for T
where
    T: Event<'a>,
    <T::Parsed as AsOwned>::Owned: Send + Sync + 'static,
    <T::Parsed as AsOwned>::Owned: Clone + Debug,
{
    type Owned = <T::Parsed as AsOwned>::Owned;
    fn into_owned(data: T::Parsed) -> Self::Owned {
        <T::Parsed as AsOwned>::as_owned(&data)
    }
}

mod private {
    use super::{Event, EventMapped};

    pub trait EventSealed {}
    impl<'a, T: Event<'a>> EventSealed for T {}

    pub trait EventMappedSealed<E> {}
    impl<'a, T: EventMapped<'a, E>, E: Event<'a>> EventMappedSealed<E> for T {}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn event_mapped() {
        fn e<'a, T>(msg: &'a crate::decode::Message<'a>) -> T::Owned
        where
            T: Event<'a> + 'static,
            T: EventMapped<'a, T>,
        {
            use crate::Parse as _;
            T::into_owned(T::Parsed::parse(msg).unwrap())
        }

        let msg = crate::decode("PING :1234567890\r\n")
            .next()
            .unwrap()
            .unwrap();

        let msg: crate::messages::Ping<'static> = e::<crate::events::Ping>(&msg);
        assert_eq!(msg.token, "1234567890")
    }
}

/// Used to get a [messages::Cap][Cap]
///
/// [Cap]: ../messages/struct.Cap.html
#[non_exhaustive]
pub struct Cap;

impl<'t> Event<'t> for Cap {
    type Parsed = messages::Cap<'t>;
}

/// Used to get a [messages::ClearChat][ClearChat]
///
/// [ClearChat]: ../messages/struct.ClearChat.html
#[non_exhaustive]
pub struct ClearChat;

impl<'t> Event<'t> for ClearChat {
    type Parsed = messages::ClearChat<'t>;
}

/// Used to get a [messages::ClearMsg][ClearMsg]
///
/// [ClearMsg]: ../messages/struct.ClearMsg.html
#[non_exhaustive]
pub struct ClearMsg;

impl<'t> Event<'t> for ClearMsg {
    type Parsed = messages::ClearMsg<'t>;
}

/// Used to get a [messages::GlobalUserState][GlobalUserState]
///
/// [GlobalUserState]: ../messages/struct.GlobalUserState.html
#[non_exhaustive]
pub struct GlobalUserState;

impl<'t> Event<'t> for GlobalUserState {
    type Parsed = messages::GlobalUserState<'t>;
}

/// Used to get a [messages::HostTarget][HostTarget]
///
/// [HostTarget]: ../messages/struct.HostTarget.html
#[non_exhaustive]
pub struct HostTarget;

impl<'t> Event<'t> for HostTarget {
    type Parsed = messages::HostTarget<'t>;
}

/// Used to get a [messages::IrcReady][IrcReady]
///
/// [IrcReady]: ../messages/struct.IrcReady.html
#[non_exhaustive]
pub struct IrcReady;

impl<'t> Event<'t> for IrcReady {
    type Parsed = messages::IrcReady<'t>;
}

/// Used to get a [messages::Join][Join]
///
/// [Join]: ../messages/struct.Join.html
#[non_exhaustive]
pub struct Join;

impl<'t> Event<'t> for Join {
    type Parsed = messages::Join<'t>;
}

/// Used to get a [messages::Mode][Mode]
///
/// [Mode]: ../messages/struct.Mode.html
#[non_exhaustive]
#[deprecated(
    since = "0.10.2",
    note = "Twitch has deprecated this event. see https://discuss.dev.twitch.tv/t/irc-update-removing-mode-and-names-capabilities/25568"
)]
pub struct Mode;

#[allow(deprecated)]
impl<'t> Event<'t> for Mode {
    type Parsed = messages::Mode<'t>;
}

/// Used to get a [messages::Names][Names]
///
/// [Names]: ../messages/struct.Names.html
#[non_exhaustive]
#[deprecated(
    since = "0.10.2",
    note = "Twitch has deprecated this event. see https://discuss.dev.twitch.tv/t/irc-update-removing-mode-and-names-capabilities/25568"
)]
pub struct Names;

#[allow(deprecated)]
impl<'t> Event<'t> for Names {
    type Parsed = messages::Names<'t>;
}

/// Used to get a [messages::Notice][Notice]
///
/// [Notice]: ../messages/struct.Notice.html
#[non_exhaustive]
pub struct Notice;

impl<'t> Event<'t> for Notice {
    type Parsed = messages::Notice<'t>;
}

/// Used to get a [messages::Part][Part]
///
/// [Part]: ../messages/struct.Part.html
#[non_exhaustive]
pub struct Part;

impl<'t> Event<'t> for Part {
    type Parsed = messages::Part<'t>;
}

/// Used to get a [messages::Ping][Ping]
///
/// [Ping]: ../messages/struct.Ping.html
#[non_exhaustive]
pub struct Ping;

impl<'t> Event<'t> for Ping {
    type Parsed = messages::Ping<'t>;
}

/// Used to get a [messages::Pong][Pong]
///
/// [Pong]: ../messages/struct.Pong.html
#[non_exhaustive]
pub struct Pong;

impl<'t> Event<'t> for Pong {
    type Parsed = messages::Pong<'t>;
}

/// Used to get a [messages::Privmsg][Privmsg]
///
/// [Privmsg]: ../messages/struct.Privmsg.html
#[non_exhaustive]
pub struct Privmsg;

impl<'t> Event<'t> for Privmsg {
    type Parsed = messages::Privmsg<'t>;
}

/// Used to get a [messages::Raw][Raw]
///
/// [Raw]: ../messages/type.Raw.html
#[non_exhaustive]
pub struct Raw;

impl<'t> Event<'t> for Raw {
    type Parsed = messages::Raw<'t>;
}

/// Used to get a [messages::Ready][Ready]
///
/// [Ready]: ../messages/struct.Ready.html
#[non_exhaustive]
pub struct Ready;

impl<'t> Event<'t> for Ready {
    type Parsed = messages::Ready<'t>;
}

/// Used to get a [messages::Reconnect][Reconnect]
///
/// [Reconnect]: ../messages/struct.Reconnect.html
#[non_exhaustive]
pub struct Reconnect;

impl<'t> Event<'t> for Reconnect {
    type Parsed = messages::Reconnect;
}

/// Used to get a [messages::RoomState][RoomState]
///
/// [RoomState]: ../messages/struct.RoomState.html
#[non_exhaustive]
pub struct RoomState;

impl<'t> Event<'t> for RoomState {
    type Parsed = messages::RoomState<'t>;
}

/// Used to get a [messages::UserNotice][UserNotice]
///
/// [UserNotice]: ../messages/struct.UserNotice.html
#[non_exhaustive]
pub struct UserNotice;

impl<'t> Event<'t> for UserNotice {
    type Parsed = messages::UserNotice<'t>;
}

/// Used to get a [messages::UserState][UserState]
///
/// [UserState]: ../messages/struct.UserState.html
#[non_exhaustive]
pub struct UserState;

impl<'t> Event<'t> for UserState {
    type Parsed = messages::UserState<'t>;
}

/// Used to get a [messages::AllCommands][AllCommands]
///
/// [AllCommands]: ../messages/enum.AllCommands.html
#[non_exhaustive]
pub struct All;

impl<'t> Event<'t> for All {
    type Parsed = messages::AllCommands<'t>;
}
