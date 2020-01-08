/*!
Available event filters.

These can be used with [Dispatcher::subscribe][Subscribe] to get a [Stream] of [Messages] filtered to this event.

[Subscribe]: ./struct.Dispatcher.html#method.subscribe
[Stream]: https://docs.rs/futures/0.3.1/futures/stream/trait.Stream.html
[Messages]: ./messages/index.html
*/
use super::*;

make_event!(Join            => messages::Join);
make_event!(Part            => messages::Part);
make_event!(Privmsg         => messages::Privmsg);
make_event!(Raw             => messages::Raw);
make_event!(Ping            => messages::Ping);
make_event!(Pong            => messages::Pong);
make_event!(IrcReady        => messages::IrcReady);
make_event!(Ready           => messages::Ready);
make_event!(Cap             => messages::Cap);
make_event!(GlobalUserState => messages::GlobalUserState);
make_event!(Notice          => messages::Notice);
make_event!(ClearChat       => messages::ClearChat);
make_event!(ClearMsg        => messages::ClearMsg);
make_event!(Reconnect       => messages::Reconnect);
make_event!(UserState       => messages::UserState);
make_event!(Mode            => messages::Mode);
