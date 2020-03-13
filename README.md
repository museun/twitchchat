# twitchchat
[![Documentation][docs_badge]][docs]
[![Crates][crates_badge]][crates]
[![Actions][actions_badge]][actions]

This crate provides a way to interact with [Twitch]'s chat.

Along with parse messages as Rust types, it provides methods for sending messages.

## Demonstration
See [examples/demo.rs][demo] for a larger example

## Configuration features
Feature | Description
--- | ---
default | enables `async` and `tokio_native_tls` (the default)
async   | enables [`tokio`](https://crates.io/crates/tokio) support (and all of the _async_ methods/types)
tokio_native_tls | uses [`native_tls`](https://crates.io/crates/native-tls) _(OpenSSL, SChannel, SecureTransport)_ for TLS
tokio_rustls     | uses [`rusttls`](https://crates.io/crates/rustls) for TLS
serde | enables [`serde`](https://crates.io/crates/serde) Serialize/Deserialize on most of the types

***NOTE***: Only one _TLS_ feature can be enabled at once.


## Connecting to twitch
This crate allows you connect to Twitch with a TLS stream, or without one.

### With TLS
**NOTE** the async blocks are here so the doctests will work, you'll likely have it in a 'larger' async context

> Connect with a UserConfig:
```rust no_run
// make a user config, builder lets you configure it.
let user_config = twitchchat::UserConfig::builder().build().unwrap();
// the conn type is an tokio::io::{AsyncRead + AsyncWrite}
let conn = async { twitchchat::connect_tls(&user_config).await.unwrap() };
```
> Connect with default capabilities using just a username and oauth token:
```rust no_run
let my_oauth = std::env::var("MY_TWITCH_OAUTH").unwrap();
// the conn type is an tokio::io::{AsyncRead + AsyncWrite}
let conn = async { twitchchat::connect_easy_tls("my_name", &my_oauth).await.unwrap() };
```

### Without TLS (an unsecure plain-text connection)
> Connect with a UserConfig:
```rust no_run
// make a user config, builder lets you configure it.
let user_config = twitchchat::UserConfig::builder().build().unwrap();
// the conn type is an tokio::io::{AsyncRead+AsyncWrite}
let conn = async { twitchchat::connect(&user_config).await.unwrap() };

```
> Connect with default capabilities using just a username and oauth token:
```rust no_run
let my_oauth = std::env::var("MY_TWITCH_OAUTH").unwrap();
// the conn type is an tokio::io::{AsyncRead+AsyncWrite}
let conn = async { twitchchat::connect_easy("my_name", &my_oauth).await.unwrap() };
```

You can even connect with an ***anonymous*** users that _doesn't_ require an OAuth token.

This will let you join and read messages from channels, but not write them.
Also you'd be limited in what sort of metadata you can receive.

> With TLS
```rust no_run
let (nick, pass) = twitchchat::ANONYMOUS_LOGIN;
let conn = async { twitchchat::connect_easy_tls(nick, pass).await.unwrap() };
```

> Without TLS
```rust no_run
let (nick, pass) = twitchchat::ANONYMOUS_LOGIN;
let conn = async { twitchchat::connect_easy(nick, pass).await.unwrap() };
```

---
A ***synchronous*** 'connect' is provided for completionist sake.

This crate is intended to be used with async types, but the `Encoder` and `decode` methods will work without them.

Disabling `async` will only give you these types.

See [`twitchchat::sync`](https://docs.rs/twitchchat/latest/twitchchat/sync/index.html) for synchronous types.

```rust no_run
// make a user config, builder lets you configure it.
let user_config = twitchchat::UserConfig::builder().build().unwrap();
let conn: std::net::TcpStream = twitchchat::sync::connect(&user_config).unwrap();
```
Or
```rust no_run
let my_oauth = std::env::var("MY_TWITCH_OAUTH").unwrap();
let conn: std::net::TcpStream = twitchchat::sync::connect_easy("my_name", &my_oauth).unwrap();
```
---

## Parsing messages

Parsing is done with the `decode(&str)` or `decode_one(&str)` methods. Twitch (IRC) messages are delimited by `CRLF [0xD, 0xA]`

> Parsing potentially many messages.
```rust
let input = "@badge-info=subscriber/8;color=#59517B;tmi-sent-ts=1580932171144;user-type= :tmi.twitch.tv USERNOTICE #justinfan1234\r\n";

// decode takes in a string and returns an Iterator of Result<Message<'a>, Error>
// the flatten here just 'unwraps' the Results safely
for msg in twitchchat::decode(&input).flatten() {
    // msg is a twitchchat::decode::Message<'a>
    assert_eq!(msg.command, "USERNOTICE");
    assert_eq!(msg.args, "#justinfan1234");
    // helper method for splitting 'args' cheaply
    assert_eq!(msg.arg(0), Some("#justinfan1234"));

    use twitchchat::color::{Color, TwitchColor, RGB};
    assert_eq!(msg.tags.get_parsed::<_, Color>("color"), Some(Color {
        kind: TwitchColor::Turbo,
        rgb: RGB(89, 81, 123)
    }));
}
```

> Parsing a single message
```rust
let input =
    ":tmi.twitch.tv PING 1234567\r\n:museun!museun@museun.tmi.twitch.tv JOIN #museun\r\n";

// parse one message at a time
// this returns the index of the start of the possible next message
let (d, message) = twitchchat::decode_one(input).unwrap();
assert!(d > 0);
assert_eq!(message.command, "PING");

// use the new index
let (i, message) = twitchchat::decode_one(&input[d..]).unwrap();
assert_eq!(i, 0);
assert_eq!(message.command, "JOIN");
```

> Parsing a `decode::Message<'a>` into a _subtype_
```rust
// Enables parsing a decode::Message<'a> into some subtype (e.g. messages::Privmsg<'a>).
use twitchchat::Parse as _;
use twitchchat::{decode, messages};

let input = ":museun!museun@museun.tmi.twitch.tv JOIN #some_test_channel\r\n";
let msg: decode::Message<'_> = twitchchat::decode(input).next().unwrap().unwrap();
// this borrows from the 'decode::Message' so its super cheap.
let join: messages::Join<'_> = messages::Join::parse(&msg).unwrap();
assert_eq!(join.channel, "#some_test_channel");
assert_eq!(join.name, "museun");
```

> Parsing a `decode::Message<'a>` into an enum of _all_ possible messages
```rust
// Enables parsing a decode::Message<'a> into some subtype (e.g. messages::Privmsg<'a>).
use twitchchat::Parse as _;
use twitchchat::{decode, messages};

let input = ":museun!museun@museun.tmi.twitch.tv JOIN #some_test_channel\r\n";
let msg: decode::Message<'_> = twitchchat::decode(input).next().unwrap().unwrap();
// this borrows from the 'decode::Message' so its super cheap.
match messages::AllCommands::parse(&msg).unwrap() {
    // 'join' here would be `messages::Join<'a>`
    messages::AllCommands::Join(join) => {
        assert_eq!(join.channel, "#some_test_channel");
        assert_eq!(join.name, "museun");
    }
    _ => panic!("not a join message")
}
```

> Taking ownership of a parsed message
```rust
// Enables parsing a decode::Message<'a> into some subtype (e.g. messages::Privmsg<'a>).
use twitchchat::Parse as _;
// Enables converting a Message<'a> to a Message<'static> (or any of the subtypes).
use twitchchat::AsOwned as _;
use twitchchat::{decode, messages};
let input = ":museun!museun@museun.tmi.twitch.tv JOIN #some_test_channel\r\n";

let msg: decode::Message<'_> = twitchchat::decode(input).next().unwrap().unwrap();
// can be used to take ownership of a 'Message<'a>'
let owned: decode::Message<'static> = msg.as_owned();
assert_eq!(owned.command, "JOIN");

let join = messages::Join::parse(&msg).unwrap();
// or even a subtype
let join_owned: messages::Join<'static> = join.as_owned();

assert_eq!(join_owned.channel, "#some_test_channel");
assert_eq!(join_owned.name, "museun");
```

> Getting data out of the _tags_
```rust
use twitchchat::Parse as _;
use twitchchat::{decode, messages};

let input = "@badge-info=subscriber/8;color=#59517B;tmi-sent-ts=1580932171144;user-type= :tmi.twitch.tv USERNOTICE #justinfan1234\r\n";

let msg = decode(&input).next().unwrap().unwrap();
let user_notice = messages::UserNotice::parse(&msg).unwrap();

// the tags are parsed and are accessible as methods
// colors can be parsed into rgb/named types
assert_eq!(
    user_notice.color().unwrap(),
    "#59517B".parse::<twitchchat::color::Color>().unwrap()
);

// you can manually get tags from the message
let ts = user_notice.tags.get("tmi-sent-ts").unwrap();
assert_eq!(ts, "1580932171144");

// or as a type
let ts = user_notice
    .tags
    .get_parsed::<_, u64>("tmi-sent-ts")
    .unwrap();
assert_eq!(ts, 1580932171144);
```

## Event dispatching/streams
Along with connecting to twitch and parsing strings, this crate can do that for you and provide you typed asynchronous Streams for events you're interested in.

```rust no_run
use twitchchat::{messages, events};
// for working with streams, the futures::stream::StreamExt trait will also work.
use tokio::stream::StreamExt as _;

/// make a new event dispatcher
let dispatcher = twitchchat::Dispatcher::new();

// you can subscribe to an event
// this'll return a Stream which'll produce a `messages` type
// for example, events::Join will produce messages::Join<'static>
let mut joins = dispatcher.subscribe::<events::Join>();
// you can subscribe to the same event multiple times
let mut more_joins = dispatcher.subscribe::<events::Join>();
// you can subscribe to 'All' to get an enum of all possible events
let mut all = dispatcher.subscribe::<events::All>();
// and you can subscribe to 'Raw' to get the 'raw' decode::Message type
let mut raw = dispatcher.subscribe::<events::Raw>();

// dropping these streams will 'unsubscribe' them

let fut = async move {
    while let Some(msg) = joins.next().await {
        // msg is an Arc<messages::Join<'static>> here
        // so if you reborrow it, you can temporarily get rid of the arc;
        let msg: &messages::Join<'static> = &*msg;
    }
    // returning from this task will also unsubscribe this event stream
};

// lets be fancy and use a select over stream
let fut = async move {
    loop {
        tokio::select! {
            Some(join) = &mut more_joins.next() => {}
            Some(all) = &mut all.next() => {
                match &*all {
                    messages::AllCommands::Ping(ping) => {}
                    _ => {}
                }
            }
            Some(raw) = &mut raw.next() => {}
            else => { break }
        }
    }
};
```

> Finally, writing (encoding) messages.
```rust
// you probably want to keep a dispatcher around so you can read from the conn
let (_runner, mut control) = twitchchat::Runner::new(twitchchat::Dispatcher::new(), twitchchat::RateLimit::default());

// the control type is also clonable
let mut ctrl_clone = control.clone();

// the Control type has a way to get a &mut borrow to a writer
let writer = control.writer();

// async block is here for the test, you'll likely have a larger async context
async move {
    writer.privmsg("#museun", "hello world!").await.unwrap();
    ctrl_clone.writer().join("#museun").await.unwrap()
};

// you can also clone the writer and send across tasks/threads
let writer = control.writer();

let mut w1 = writer.clone();
let mut w2 = w1.clone();

async move {
    w1.join("foo").await.unwrap();
    w2.part("foo").await.unwrap();
};
```

## You can use the `AsyncEncoder` and `Encoder` types to wrap io types.
> Async
```rust
use std::io::Cursor;
use twitchchat::encode::AsyncEncoder;
// AsyncEncoder wraps a tokio::io::AsyncWrite type and provides a 'typed' way of writing messages.

async {
    // cursor implements AsyncWrite
    let cursor = Cursor::new(vec![]);
    let mut encoder = AsyncEncoder::new(cursor);
    encoder.privmsg("#museun", "hello world!").await.unwrap();
    // get a reference to the inner type
    { let cursor: &Cursor<Vec<u8>> = encoder.inner(); }
    // get a mutable reference to the inner type
    { let cursor: &mut Cursor<Vec<u8>> = encoder.inner_mut(); }
    // convert the encoder back into the wrapped type
    let cursor: Cursor<Vec<u8>> = encoder.into_inner();
};
```

> Sync
```rust
use std::io::Cursor;
// or get it from twitchchat:sync::Encoder;
use twitchchat::encode::Encoder;
// Encoder wraps a std::io::Write type and provides a 'typed' way of writing messages.

// cursor implements Write
let cursor = Cursor::new(vec![]);
let mut encoder = Encoder::new(cursor);
encoder.privmsg("#museun", "hello world!").unwrap();
// get a reference to the inner type
{ let cursor: &Cursor<Vec<u8>> = encoder.inner(); }
// get a mutable reference to the inner type
{ let cursor: &mut Cursor<Vec<u8>> = encoder.inner_mut(); }
// convert the encoder back into the wrapped type
let cursor: Cursor<Vec<u8>> = encoder.into_inner();

```

## Putting it together, a simple "bot"
```rust no_run
use tokio::stream::StreamExt as _;
use twitchchat::runner::Writer;
use twitchchat::{events, messages, Control, Dispatcher, IntoChannel, Runner, Status, RateLimit};

fn get_creds() -> (String, String, String) {
    fn get_it(name: &str) -> String {
        std::env::var(name).unwrap_or_else(|_| {
            eprintln!("env var `{}` is required", name);
            std::process::exit(1);
        })
    }

    (
        get_it("TWITCH_NICK"),
        get_it("TWITCH_PASS"),
        get_it("TWITCH_CHANNEL"),
    )
}

struct Bot {
    // you can store the writer (and clone it)
    writer: Writer,
    // and you can store/clone the Control
    control: Control,
    start: std::time::Instant,
}

impl Bot {
    async fn run(mut self, dispatcher: Dispatcher, channel: impl IntoChannel) {
        // subscribe to the events we're interested in
        let mut events = dispatcher.subscribe::<events::Privmsg>();

        // and wait for a specific event (blocks the current task)
        let ready = dispatcher.wait_for::<events::IrcReady>().await.unwrap();
        eprintln!("connected! our name is: {}", ready.nickname);

        // and then join a channel
        eprintln!("joining our channel");
        self.writer.join(channel).await.unwrap();

        // and then our 'main loop'
        while let Some(msg) = events.next().await {
            if !self.handle(&*msg).await {
                return;
            }
        }
    }

    async fn handle(&mut self, msg: &messages::Privmsg<'_>) -> bool {
        match &*msg.data {
            "!hello" => {
                let resp = format!("hello {}!", msg.name);
                self.writer.privmsg(&msg.channel, &resp).await.unwrap();
            }
            "!uptime" => {
                let dur = std::time::Instant::now() - self.start;
                let resp = format!("I've been running for.. {:.2?}.", dur);
                self.writer.privmsg(&msg.channel, &resp).await.unwrap();
            }
            "!quit" => {
                // this'll stop the runner (causing its future to return Ok(Status::Canceled))
                self.control.stop();
                return false; // to stop the 'Bot'
            }
            _ => {}
        };
        true // to keep the 'Bot' running
    }
}

#[tokio::main]
async fn main() {
    let (user, pass, channel) = get_creds();

    let dispatcher = Dispatcher::new();
    let (runner, mut control) = Runner::new(dispatcher.clone(), RateLimit::default());

    // make a bot and get a future to its main loop
    let bot = Bot {
        // just to show you can store it
        writer: control.writer().clone(),
        // but you probably want to store the control instead
        control,
        start: std::time::Instant::now(),
    }
    .run(dispatcher, channel);

    // connect to twitch
    let conn = twitchchat::connect_easy_tls(&user, &pass).await.unwrap();
    // and run the dispatcher/writer loop
    let done = runner.run(conn);

    // and select over our two futures
    tokio::select! {
        // wait for the bot to complete
        _ = bot => { eprintln!("done running the bot") }
        // or wait for the runner to complete
        status = done => {
            match status {
                Ok(Status::Canceled) => { eprintln!("runner was canceled") }
                Ok(Status::Eof) => { eprintln!("got an eof, exiting") }
                Err(err) => { eprintln!("error running: {}", err) }
            }
        }
    }
}
```

## License
`twitchchat` is primarily distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE][APACHE] and [LICENSE-MIT][MIT] for details.

[docs_badge]: https://docs.rs/twitchchat/badge.svg
[docs]: https://docs.rs/twitchchat
[crates_badge]: https://img.shields.io/crates/v/twitchchat.svg
[crates]: https://crates.io/crates/twitchchat
[actions_badge]: https://github.com/museun/twitchchat/workflows/Rust/badge.svg
[actions]: https://github.com/museun/twitchchat/actions

[demo]: ./examples/demo.rs

[APACHE]: ./LICENSE-APACHE
[MIT]: ./LICENSE-MIT
[Twitch]: https://dev.twitch.tv
