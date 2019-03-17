//! # A simple "connection-less" twitch chat crate
//! This crate simply read lines from an [`std::io::Read`](https://doc.rust-lang.org/std/io/trait.Read.html) and produces data
//! types for the corresponding messages, and takes an [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html) which
//! can produce messages that twitch understands.
//!
//! # 'Theory' of operation
//! First, by creating a [`Client`](./struct.Client.html) from a
//! Read/Write pair (such as a cloned TcpStream) then calling
//! [`Client::register`](./struct.Client.html#method.register) with a
//! filled-out [`UserConfig`](./userconfig/struct.UserConfig.html) will connect you to
//! Twitch. Once connected, you can
//! [`Client::wait_for_ready`](./struct.Client.html#method.wait_for_ready)
//!  and the client will read **(blocking)** until Twitch sends a
//! [`GlobalUserState`](./twitch/commands/struct.GlobalUserState.html) message,
//! which it'll fill out a [`LocalUser`](./twitch/struct.LocalUser.html) with
//! various information.
//!
//! Once connected, you can
//! - use [`Client::join`](./struct.Client.html#method.join) to join a
//!   channel.
//! - use [`Client::on`](./struct.Client.html#method.on) to set up a
//!   message filter.
//! - use [`Client::read_message`](./struct.Client.html#method.read_message)
//!   to read a message (and pump the filters).
//! - or do various [*other things*](./struct.Client.html#method.host)
//!
//! # Message filters, and why blocking in them is a bad idea
//! The client provides a very simplistic callback registration system
//!
//! To use it, you simply just `register` a closure with the client via its
//! [`Client::on`](./struct.Client.html#method.on) method. It uses the
//! type of the closures argument, one of
//! [*these*](./twitch/commands/index.html#structs) to create a filter
//!
//! When [`Client::read_message`](./struct.Client.html#method.read_message)
//! is called, it'll check these filters and send a clone of the requested message
//! to the callback. Because it does this on same thread as the
//! [`Client::read_message`](./struct.Client.html#method.read_message)
//! call, you can lock up the system by simplying diverging.
//!
//! The client is thread safe, and clonable so one could call
//! [`Client::read_message`](./struct.Client.html#method.read_message)
//! with ones own sychronization scheme to allow for a simplistic thread pool,
//! but its best just to send the message off to a channel elsewhere
//!
//! # A simple example
//! ```no_run
//! use std::net::TcpStream;
//! use twitchchat::{commands::PrivMsg, Capability, Client};
//! use twitchchat::{TWITCH_IRC_ADDRESS, UserConfig};
//! # fn main() {
//! // create a simple TcpStream
//! let read = TcpStream::connect(TWITCH_IRC_ADDRESS).expect("to connect");
//! let write = read
//!     .try_clone()
//!     .expect("must be able to clone the tcpstream");
//!
//! // your password and your nickname
//! // the twitch oauth token must be prefixed with `oauth:your_token_here`
//! let (pass, nick) = (std::env::var("MY_TWITCH_OAUTH_TOKEN").unwrap(), "my_name");
//! let config = UserConfig::builder()
//!                 .token(pass)
//!                 .nick(nick)
//!                 .membership()    // this enables the membership CAP
//!                 .commands()      // this enables the commands CAP
//!                 .tags()          // this enables the tags CAP
//!                 .build()
//!                 .unwrap();
//!
//! // client takes a std::io::Read and an std::io::Write
//! let mut client = Client::new(read, write);
//!
//! // register with the user configuration
//! client.register(config).unwrap();
//!
//! // wait for everything to be ready (blocks)
//! let user = client.wait_for_ready().unwrap();
//! println!(
//!     "connected with {} (id: {}). our color is: {}",
//!     user.display_name.unwrap(),
//!     user.user_id,
//!     user.color.unwrap_or_default()
//! );
//!
//! // when we receive a commands::PrivMsg print out who sent it, and the message
//! // this can be done at any time, but its best to do it early
//! client.on(|msg: PrivMsg| {
//!     // this prints out name: msg
//!     let name = msg.display_name().unwrap_or_else(|| msg.irc_name());
//!     println!("{}: {}", name, msg.message())
//! });
//!
//! // join a channel
//! client.join("museun").unwrap();
//!
//! // sends a message to the channel
//! client.send("museun", "VoHiYo").unwrap();
//!
//! // blocks the thread, but any callbacks set in the .on handlers will get their messages
//! client.run();
//! # }
//! ```
//!
//! # TestStream
//! [`TestStream`](./struct.TestStream.html) is a simple TcpStream-like mock.
//!
//! It lets you inject/read its internal buffers, allowing you to easily write
//! unit tests for the [`Client`](./struct.Client.html)
//!
//! # UserConfig
//! [`UserConfig`](./struct.UserConfig.html) is required to [`Client::register`](./struct.Client.html#method.register)
//! (e.g. complete the connection) with Twitch
//!
//! ```no_run
//! use twitchchat::UserConfig;
//! let my_token = std::env::var("MY_TWITCH_OAUTH_TOKEN").unwrap();
//! let my_name = "my_name_123";
//! let config = UserConfig::builder()
//!     .nick(my_name)   // sets you nick
//!     .token(my_token) // sets you password (e.g. oauth token. must start with `oauth:`)
//!     // capabilities these are disabled by default. so using these "toggles" the flag (e.g. flips a boolean)
//!     .membership()    // this enables the membership CAP
//!     .commands()      // this enables the commands CAP
//!     .tags()          // this enables the tags CAP
//!     .build()         // create the config
//!     .unwrap();       // returns an Option, None if nick/token aren't semi-valid
//! ```
//!
//! # The irc module
//! The [`irc`](./irc/index.html) module contains a **very** simplistic
//! representation of the IRC protocol.
//!
#![warn(missing_docs)]
#![deny(unsafe_code)]
#![deny(unused_lifetimes)]
#![deny(unused_qualifications)]
#![deny(unused_results)]

/// IRC-related stuff
pub mod irc;

/// Types associated with twitch
mod twitch;
pub use twitch::*;

pub use self::twitch::UserConfig;

mod tee;
mod teststream;

mod ratelimit;

/// Helpers for writing tests
pub mod helpers {
    pub use super::ratelimit::RateLimit;
    pub use super::tee::{TeeReader, TeeWriter};
    pub use super::teststream::TestStream;
}

#[allow(dead_code)]
pub(crate) const VERSION_STR: &str =
    concat!(env!("CARGO_PKG_NAME"), ":", env!("CARGO_PKG_VERSION"));

/// The Twitch IRC address for non-TLS connections
pub const TWITCH_IRC_ADDRESS: &str = "irc.chat.twitch.tv:6667";
/// The Twitch IRC address for TLS connections
pub const TWITCH_IRC_ADDRESS_TLS: &str = "irc.chat.twitch.tv:6697";
