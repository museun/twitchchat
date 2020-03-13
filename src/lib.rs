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

## Runner
The runner is the main "event loop" of this crate.

It is created with a [Dispatcher][dispatcher]. It returns the new runner and the
[Control][control] type.

Once you're ready to start reading from the **Reader** and processing **Writes** you should call [Runner::run](#method.run).

# Returns
- A [`future`][future] which resolves to a [Status][status] once the Runner has finished.

Interacting with the `Runner` is done via the [Control][control] type.

# Example
```rust
# use tokio::spawn;
# tokio::runtime::Runtime::new().unwrap().block_on(async {
# let conn = tokio_test::io::Builder::new().wait(std::time::Duration::from_millis(10000)).build();
use twitchchat::{Dispatcher, Status, Runner, RateLimit};
// make a dispatcher
let dispatcher = Dispatcher::new();
// do stuff with the dispatcher (its clonable)
// ..

// create a new runner
let (runner, control) = Runner::new(dispatcher, RateLimit::default());

// spawn a task that kills the runner after some time
let ctl = control.clone();
spawn(async move {
    // pretend some time has passed
    ctl.stop()
});

// run, blocking the task.
// you can spawn this in a task and await on that join handle if you prefer
match runner.run(conn).await {
    // for the doc test
    Ok(Status::Canceled) => { assert!(true) }
    Ok(Status::Eof) => { panic!("eof") }
    Err(err) => { panic!("err") }
};
# });
```

[dispatcher]: ./struct.Dispatcher.html
[control]: ./struct.Control.html
[status]: ./enum.Status.html
[future]: https://doc.rust-lang.org/std/future/trait.Future.html

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
