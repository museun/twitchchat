/*!
Available event filters.

These can be used with [Dispatcher::subscribe][Subscribe] to get a [Stream] of [Messages] filtered to this event.

[Subscribe]: ./struct.Dispatcher.html#method.subscribe
[Stream]: https://docs.rs/futures/0.3.1/futures/stream/trait.Stream.html
[Messages]: ./messages/index.html
*/
use super::*;

make_event!(Join => messages::Join);
make_event!(Part => messages::Part);
make_event!(Privmsg => messages::Privmsg);
make_event!(Raw => messages::Raw);
make_event!(Ping => messages::Ping);
make_event!(Pong => messages::Pong);
