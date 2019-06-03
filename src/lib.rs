/*!
# A simple "connection-less" twitch chat crate
This crate produces types for twitch messages and gives you methods for sending messages.

# 'Theory' of operation
First, by creating a [`Client`](./struct.Client.html) from a [`ReadAdapter`](./trait.ReadAdapter.html) and a
[`WriteAdapter`](./trait.WriteAdapter.html). This can be simple be done by using [`twitchchat::sync_adapters`](./fn.sync_adapters.html)
on an [`std::io::TcpStream`](https://doc.rust-lang.org/std/net/struct.TcpStream.html).

Then, calling [`Client::register`](./struct.Client.html#method.register) with a
filled-out [`UserConfig`](./userconfig/struct.UserConfig.html) will connect you to Twitch.

Once connected, you can [`Client::wait_for_ready`](./struct.Client.html#method.wait_for_ready) and the client will read **(blocking)** until Twitch sends a
[`GlobalUserState`](./commands/struct.GlobalUserState.html) message, which it'll fill out a [`LocalUser`](./struct.LocalUser.html) with various information.

Once connected, you can
- use [`Writer::join`](./struct.Writer.html#method.join) to join a
channel.
- use [`Client::on`](./struct.Client.html#method.on) to set up a
message filter.
- use [`Client::read_message`](./struct.Client.html#method.read_message)
to read a message (and pump the filters).
- or do various [*other things*](./struct.Writer.html#method.host)

# Thread-safe writing:
Use [`Client::writer`](./struct.Client.html#method.writer) to get a clonable, thread-safe [`Writer`](./struct.Writer.html) that is used to writer messages to the `Client`.

# Message filters, and why blocking in them is a bad idea
The client provides a very simplistic callback registration system

To use it, you simply just `register` a closure with the client via its
[`Client::on`](./struct.Client.html#method.on) method. It uses the type of the closures argument, one of [*these*](./commands/index.html#structs) to create a filter

It also gives you a clone of the [`Writer`](./struct.Writer.html) so you don't need to move one into the closure.

When [`Client::read_message`](./struct.Client.html#method.read_message) is called, it'll check these filters and send a clone of the requested message to the callback. Because it does this on same thread as the [`Client::read_message`](./struct.Client.html#method.read_message) call, you can lock up the system by simplying diverging.

The client is thread safe, and clonable so one could call [`Client::read_message`](./struct.Client.html#method.read_message) with ones own sychronization scheme to allow for a simplistic thread pool, but its best just to send the message off to a channel elsewhere


# A simple example
```no_run
use std::net::TcpStream;
use twitchchat::{commands::PrivMsg, Capability};
use twitchchat::{Client, Writer, SyncReadAdapter, SyncWriteAdapter};
use twitchchat::{TWITCH_IRC_ADDRESS, UserConfig};
# fn main() {
// create a simple TcpStream
let read = TcpStream::connect(TWITCH_IRC_ADDRESS).expect("connect");
let write = read
    .try_clone()
    .expect("must be able to clone the tcpstream");

// your password and your nickname
// the twitch oauth token must be prefixed with `oauth:your_token_here`
let (pass, nick) = (std::env::var("MY_TWITCH_OAUTH_TOKEN").unwrap(), "my_name");
let config = UserConfig::with_caps() // if you need or only want specific capabilities, use .builder(), or toggle them after this call
                .token(pass)
                .nick(nick)
                .build()
                .unwrap();

// shorthand for SyncReadAdapter::new(read) and SyncWriteAdapter::new(write)
// which impl the Adapters for std::io::Read and std::io::Write
let (read, write) = twitchchat::sync_adapters(read, write);

// client takes a ReadAdapter and an std::io::Write
let mut client = Client::new(read, write);

// register with the user configuration
client.register(config).unwrap();

// wait for everything to be ready (blocks)
let user = client.wait_for_ready().unwrap();
println!(
    "connected with {} (id: {}). our color is: {}",
    user.display_name.unwrap(),
    user.user_id,
    user.color.unwrap_or_default()
);

// when we receive a commands::PrivMsg print out who sent it, and the message
// this can be done at any time, but its best to do it early
client.on(|msg: PrivMsg, _: Writer| {
    // this prints out name: msg
    let name = msg.display_name().unwrap_or_else(|| msg.user());
    println!("{}: {}", name, msg.message())
});

let w = client.writer();
// join a channel
w.join("museun").unwrap();

// sends a message to the channel
w.send("museun", "VoHiYo").unwrap();

// blocks the thread, but any callbacks set in the .on handlers will get their messages
client.run();
# }
```

# TestStream
[`TestStream`](./helpers/struct.TestStream.html) is a simple TcpStream-like mock.

It lets you inject/read its internal buffers, allowing you to easily write
unit tests for the [`Client`](./struct.Client.html)

# UserConfig
[`UserConfig`](./struct.UserConfig.html) is required to [`Client::register`](./struct.Client.html#method.register)
(e.g. complete the connection) with Twitch

```no_run
use twitchchat::UserConfig;
let my_token = std::env::var("MY_TWITCH_OAUTH_TOKEN").unwrap();
let my_name = "my_name_123";
let config = UserConfig::builder()
    .nick(my_name)   // sets you nick
    .token(my_token) // sets you password (e.g. oauth token. must start with `oauth:`)
    // capabilities these are disabled by default. so using these "toggles" the flag (e.g. flips a boolean)
    .membership()    // this enables the membership CAP
    .commands()      // this enables the commands CAP
    .tags()          // this enables the tags CAP
    .build()         // create the config
    .unwrap();       // returns an Option, None if nick/token aren't semi-valid
```

# The irc module
The [`irc`](./irc/index.html) module contains a **very** simplistic representation of the IRC protocol.
*/
#![warn(missing_docs)]
#![deny(unsafe_code)]
#![deny(unused_lifetimes)]
#![deny(unused_qualifications)]
#![deny(unused_results)]

/// IRC-related stuff
pub mod irc;

mod tags;
/// IRCv3 Tags
pub use tags::Tags;

/// Types associated with twitch
mod twitch;
pub use twitch::*;

pub use self::twitch::UserConfig;

mod teststream;

/// Helpers for writing tests
pub mod helpers {
    pub use super::teststream::TestStream;
    pub use ratelimit::RateLimit;
    pub use tee::{TeeReader, TeeWriter};
}

#[allow(dead_code)]
pub(crate) const VERSION_STR: &str =
    concat!(env!("CARGO_PKG_NAME"), ":", env!("CARGO_PKG_VERSION"));

/// The Twitch IRC address for non-TLS connections
pub const TWITCH_IRC_ADDRESS: &str = "irc.chat.twitch.tv:6667";
/// The Twitch IRC address for TLS connections
pub const TWITCH_IRC_ADDRESS_TLS: &str = "irc.chat.twitch.tv:6697";

/// Convert an IRC-like message type into something that the Twitch commands can be parsed from
///
/// Refer to this form when implementing this trait:
///
/// raw string form: `@tags :prefix command args :data\r\n`
///
/// Example:
/** ```
struct MyPrivMsg {
    tags: hashbrown::HashMap<String, String>,
    sender: String,
    channel: String,
    data: String,
}
impl MyPrivMsg {
    pub fn new<S: ToString>(channel: S, sender: S, data: S, tags: &[(S, S)]) -> Self {
        Self {
            tags: tags
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            channel: channel.to_string(),
            sender: sender.to_string(),
            data: data.to_string(),
        }
    }
}

impl twitchchat::ToMessage for MyPrivMsg {
    fn tags(&self) -> Option<twitchchat::TagType<'_>> {
        Some(twitchchat::TagType::Map(&self.tags))
    }
    fn prefix(&self) -> Option<&str> {
        Some(self.sender.as_str())
    }
    fn command(&self) -> Option<&str> {
        Some("PRIVMSG")
    }
    fn args(&self) -> Option<twitchchat::ArgsType<'_>> {
        Some(twitchchat::ArgsType::Raw(self.channel.as_str()))
    }
    fn data(&self) -> Option<&str> {
        Some(self.data.as_str())
    }
}

let msg = MyPrivMsg::new(
    "test_channel",
    "museun",
    "hello world",
    &[("color", "#FF4500"), ("display-name", "Museun")],
);
let twitch_msg = twitchchat::Message::parse(msg);
let pm = match twitch_msg {
    twitchchat::Message::PrivMsg(pm) => pm,
    _ => unreachable!(),
};
assert_eq!(pm.user(), "museun");
assert_eq!(pm.channel(), "#test_channel");
assert_eq!(pm.message(), "hello world");
assert_eq!(pm.color().unwrap().kind, twitchchat::TwitchColor::OrangeRed);
```
**/

pub trait ToMessage {
    /// Get the tags portion of the IRC message
    fn tags(&self) -> Option<TagType<'_>>;
    /// Get the prefix portion of the IRC message
    fn prefix(&self) -> Option<&str>;
    /// Get the command portion of the IRC message
    fn command(&self) -> Option<&str>;
    /// Get the args portion of the IRC message
    fn args(&self) -> Option<ArgsType<'_>>;
    /// Get the data portion of the IRC message
    fn data(&self) -> Option<&str>;
}

use hashbrown::HashMap;

/// A representation of IRCv3 tags, a raw string or a Vec of Key-Vals
pub enum TagType<'a> {
    /// Raw string
    Raw(&'a str),
    /// List of Key -> Values (owned)
    List(&'a Vec<(String, String)>),
    /// Map of Key -> Values (owned)
    Map(&'a HashMap<String, String>),
}

/// A representation of the args list portion of the IRC message
pub enum ArgsType<'a> {
    /// A raw string
    Raw(&'a str),
    /// A list of parts parsed from the whitespace-separated raw string
    List(&'a Vec<String>),
}

use crate::irc::types::Message as IrcMessage;
impl ToMessage for IrcMessage {
    fn tags(&self) -> Option<TagType<'_>> {
        match self {
            IrcMessage::Unknown { tags, .. } => Some(TagType::Map(&tags.0)),
            _ => None,
        }
    }
    fn prefix(&self) -> Option<&str> {
        match self {
            IrcMessage::Unknown {
                prefix: Some(crate::irc::types::Prefix::User { nick, .. }),
                ..
            } => Some(&nick),
            _ => None,
        }
    }
    fn command(&self) -> Option<&str> {
        match self {
            IrcMessage::Unknown { head, .. } => Some(&head),
            _ => None,
        }
    }
    fn args(&self) -> Option<ArgsType<'_>> {
        match self {
            IrcMessage::Unknown { args, .. } => Some(ArgsType::List(&args)),
            _ => None,
        }
    }
    fn data(&self) -> Option<&str> {
        match self {
            IrcMessage::Unknown { tail, .. } => tail.as_ref().map(String::as_str),
            _ => None,
        }
    }
}
