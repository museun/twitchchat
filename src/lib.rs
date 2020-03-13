#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
/*!
This crate provides a way to interface with [Twitch]'s chat.

Along with the messages as Rust types, it provides methods for sending messages.

# Event handling.
You'll create a dispatcher which'll let you subscribe and unsubscribe for specific events.

The dispatcher is clonable and the `Runner` requires a clone of it to filter the events for you.

### Example
```rust
# use twitchchat::{Dispatcher, Runner, RateLimit, events, messages::AllCommands};
# use tokio::spawn;
# use futures::stream::StreamExt as _;
# let conn = tokio_test::io::Builder::new().read(b"PING :123456789\r\n").write(b"PONG :123456789\r\n").build();
# let fut = async move {
let dispatcher = Dispatcher::new();
let mut all = dispatcher.subscribe::<events::All>();

// give our a clone of our dispatcher to the runner
let (runner, control) = Runner::new(dispatcher.clone(), RateLimit::default());

// loop over our stream
spawn(async move {
    while let Some(msg) = all.next().await {
        // its an Arc, so you can just deref/reborrow it
        match &*msg {
            AllCommands::Unknown(raw) => {},
            AllCommands::Ping(ping) => {},
            _ => { break; /* we can end the task by returning from it */ }
        }
    }

    // when the task ends/drops, the EventStream will unsubscribe itself
});

// consume the runner, driving any streams we subscribe to
// (and any writes from the 'Control')
runner.run(conn).await.unwrap();
# };
# tokio::runtime::Runtime::new().unwrap().block_on(fut);
```

### Using the dispatcher
You can [subscribe] to any number of [events][events].

```rust
# use twitchchat::{Dispatcher, events};
let dispatcher = Dispatcher::new();
// subscribe to a specific event
let join = dispatcher.subscribe::<events::Join>();
// you can do this as many times as you want
let join2 = dispatcher.subscribe::<events::Join>();
// there is a catch-all event type that is an enum of all possible events
let all = dispatcher.subscribe::<events::All>();
```

These events are [Streams][stream] which produce an one of the correlated [messages][messages].

```rust
# use twitchchat::{Dispatcher, events, messages};
# use tokio::stream::StreamExt as _;
# use futures::stream::StreamExt as _;
# async move {
let dispatcher = Dispatcher::new();

// subscribe to a specific event
let join = dispatcher.subscribe::<events::Join>();

// join is a Stream<Item = Arc<messages::Join<'static>>>
let fut = join.for_each(|item| async move {
    println!("{:?}", item)
});
# };
```

## Unsubscribing from events
Dropping this [EventStream] will unsubscribe from the events.

When the dispatcher is dropped, all of the streams will produce ***None***.

You can also clear subscriptions to cause `EventStreams` to produce `None` on their next resolvement

### Clearing subscriptions via the dispatcher
You can clear event subscriptions for [specific events][specific] or for [all events][all]

[Dispatcher]: ./struct.Dispatcher.html
[EventStream]: ./struct.EventStream.html
[subscribe]: ./struct.Dispatcher.html#method.subscribe
[messages]: ../messages/index.html
[events]: ../events/index.html
[specific]: ./struct.Dispatcher.html#method.clear_subscriptions
[all]: ./struct.Dispatcher.html#method.clear_subscriptions_all

[Stream]: https://docs.rs/tokio/0.2/tokio/stream/trait.Stream.html

# Demonstration
See `examples/demo.rs` for a demo of the api

---
Here's a quick link to the [Event Mapping](./struct.Dispatcher.html#a-table-of-mappings)

[Twitch]: https://www.twitch.tv
*/

#[cfg(all(doctest, feature = "async", feature = "tokio_native_tls"))]
doc_comment::doctest!("../README.md");

static_assertions::assert_cfg!(
    not(all(
        feature = "tokio_native_tls", //
        feature = "tokio_rustls",     //
    )),
    "only a single TLS library can be used."
);

#[macro_use]
#[doc(hidden)]
pub mod macros;

cfg_async! {
    mod runner;
    pub use runner::{
        writer::Writer,
        dispatcher::Dispatcher,
        stream::EventStream,
        runner::Runner,
        control::Control,
        status::Status
    };
}

cfg_async! {
    pub mod events;
}

cfg_async! {
    mod register;
    #[doc(inline)]
    pub use register::register;
}

cfg_async! {
    mod connect;
    #[doc(inline)]
    pub use connect::*;
}

/// Decode messages from a `&str`
pub mod decode;
#[doc(inline)]
pub use decode::{decode, decode_one};

/// Encode data to a `Writer`
pub mod encode;
#[doc(inline)]
pub use encode::Encoder;

/// Common Twitch types
pub mod twitch;

#[doc(inline)]
pub use twitch::*;

pub mod messages;

pub mod sync;

mod parse;
pub use parse::Parse;

mod as_owned;
#[doc(inline)]
pub use as_owned::AsOwned;

mod error;
#[doc(inline)]
pub use error::Error;

/// The Twitch IRC address for non-TLS connections
pub const TWITCH_IRC_ADDRESS: &str = "irc.chat.twitch.tv:6667";

/// The Twitch IRC address for TLS connections
pub const TWITCH_IRC_ADDRESS_TLS: &str = "irc.chat.twitch.tv:6697";

/// The Twitch WebSocket address for non-TLS connections
pub const TWITCH_WS_ADDRESS: &str = "ws://irc-ws.chat.twitch.tv:80";

/// The Twitch WebSocket address for TLS connections
pub const TWITCH_WS_ADDRESS_TLS: &str = "wss://irc-ws.chat.twitch.tv:443";

/**
An anonymous login.

You won't be able to send messages, but you can join channels and read messages

# usage
```rust
# use twitchchat::{ANONYMOUS_LOGIN, UserConfig};
let (nick, pass) = twitchchat::ANONYMOUS_LOGIN;
let _config = UserConfig::builder()
    .name(nick)
    .token(pass)
    .build()
    .unwrap();
```
*/
pub const ANONYMOUS_LOGIN: (&str, &str) = (JUSTINFAN1234, JUSTINFAN1234);
pub(crate) const JUSTINFAN1234: &str = "justinfan1234";

fn simple_user_config(name: &str, token: &str) -> Result<UserConfig, UserConfigError> {
    UserConfig::builder()
        .name(name)
        .token(token)
        .capabilities(&[
            Capability::Membership,
            Capability::Tags,
            Capability::Commands,
        ])
        .build()
}

// TODO see https://github.com/museun/twitchchat/issues/91
cfg_async! {
    #[doc(inline)]
    pub mod rate_limit;
    #[doc(inline)]
    pub use rate_limit::RateLimit;
}
