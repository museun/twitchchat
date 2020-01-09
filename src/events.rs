/*!
Available event filters.

These can be used with [Dispatcher::subscribe][Subscribe] to get a [Stream] of [Messages] filtered to this event.

[Subscribe]: ./struct.Dispatcher.html#method.subscribe
[Stream]: https://docs.rs/futures/0.3.1/futures/stream/trait.Stream.html
[Messages]: ./messages/index.html
*/
use super::*;

make_event! {
    Cap             => messages::Cap
    ClearChat       => messages::ClearChat
    ClearMsg        => messages::ClearMsg
    GlobalUserState => messages::GlobalUserState
    IrcReady        => messages::IrcReady
    Join            => messages::Join
    Mode            => messages::Mode
    Notice          => messages::Notice
    Part            => messages::Part
    Ping            => messages::Ping
    Pong            => messages::Pong
    Privmsg         => messages::Privmsg
    Raw             => messages::Raw
    Ready           => messages::Ready
    Reconnect       => messages::Reconnect
    UserState       => messages::UserState
}
