/*!
Available event filters.

These can be used with [Dispatcher::subscribe][Subscribe] to get a [Stream] of [Messages] filtered to this event.

See the [table]

[Subscribe]: ../client/struct.Dispatcher.html#method.subscribe
[Stream]: https://docs.rs/futures/0.3.1/futures/stream/trait.Stream.html
[Messages]: ../messages/index.html
[table]: ../client/struct.Dispatcher.html#a-table-of-mappings
*/
use super::*;

make_event! {
    Cap             => messages::Cap
    ClearChat       => messages::ClearChat
    ClearMsg        => messages::ClearMsg
    GlobalUserState => messages::GlobalUserState
    HostTarget      => messages::HostTarget
    IrcReady        => messages::IrcReady
    Join            => messages::Join
    Mode            => messages::Mode
    Names           => messages::Names
    Notice          => messages::Notice
    Part            => messages::Part
    Ping            => messages::Ping
    Pong            => messages::Pong
    Privmsg         => messages::Privmsg
    Raw             => messages::Raw<String>
    Ready           => messages::Ready
    Reconnect       => messages::Reconnect
    RoomState       => messages::RoomState
    UserState       => messages::UserState
}

/// Used to get a [messages::AllCommands][AllCommands]
///
/// [AllCommands]: ../messages/enum.AllCommands.html
#[non_exhaustive]
#[allow(missing_debug_implementations, missing_copy_implementations)]
pub struct All;

impl<'a> crate::client::Event<'a> for All {
    type Mapped = messages::AllCommands;
}
