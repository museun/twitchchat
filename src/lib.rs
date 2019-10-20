//! # twitchchat
//! This crate provides a way to interact with Twitch's chat
//!
//! Along with the messages/commands as Rust types, it provides methods for sending messags/commands.
//!
//! ## a simple example
//! ```no_run
//! use twitchchat::commands;
//! use twitchchat::*;
//!
//! // use an anonymous login (you should probably use your name and your chat oauth token)
//! let (nick, token) = twitchchat::ANONYMOUS_LOGIN;
//!
//! // connect with this nick, token
//! let mut client = twitchchat::connect_easy(nick, token)
//!     .unwrap() // this is an error if
//!               // the network connection can't be opened,
//!               // the nick/pass is invalid, etc
//!      // add some filters
//!     .filter::<commands::PrivMsg>() // filter to PrivMsg commands
//!     .filter::<commands::Join>();   // filter to Join commands
//!
//! // get a clonable, threadsafe writer
//! let writer = client.writer();
//!
//! // for each event from the client, blocking
//! // a client.nonblocking_iter() also exists
//! for event in client {
//!     match event {
//!         // when we're connected
//!         Event::IrcReady(..) => {
//!             // join a channel
//!             writer.join("museun").unwrap();
//!         }
//!         // when we get a priv msg
//!         Event::Message(Message::PrivMsg(msg)) => {
//!             // print out the sender : messsage
//!             println!("{}: {}", msg.user(), msg.message());
//!         }
//!         // when we get a join msg
//!         Event::Message(Message::Join(msg)) => {
//!             // print out the user and the channel that was joined
//!             println!("*** {} joined {}", msg.user(), msg.channel())
//!         }
//!         // when we get an error
//!         Event::Error(err) => {
//!             // print it out
//!             eprintln!("error: {}", err);
//!             // and exit the loop
//!             break;
//!         }
//!         // not used here
//!         Event::TwitchReady(..) => {
//!             // this only happens when you're using Capability::Tags, Capability::Commands and a non-anonymous login
//!         }
//!         // make the compiler happy
//!         _ => unreachable!(),
//!     }
//! }
//! ```
//! ## with custom capabilities
//! ```no_run
//! use twitchchat::commands;
//! use twitchchat::*;
//!
//! // use an anonymous login (you should probably use your name and your chat oauth token)
//! let (nick, token) = twitchchat::ANONYMOUS_LOGIN;
//! let config = UserConfig::builder()
//!     .token(token) // your oauth token
//!     .nick(nick) // your nickname
//!     .commands() // command capabilites (see: https://dev.twitch.tv/docs/irc/commands/ )
//!     .membership() // command capabilites (see: https://dev.twitch.tv/docs/irc/membership/ )
//!     .tags() // command capabilites (see: https://dev.twitch.tv/docs/irc/tags/ )
//!     .build() // verify the settings
//!     .unwrap();
//!
//! // connect with the config
//! let client = twitchchat::connect(&config)
//!     .unwrap()
//!     .filter::<commands::PrivMsg>();
//! let writer = client.writer();
//!
//! for event in client {
//!     match event {
//!         Event::IrcReady(..) => writer.join("museun").unwrap(),
//!         Event::Message(Message::PrivMsg(msg)) => {
//!             println!("{}: {}", msg.user(), msg.message());
//!         }
//!         Event::Error(..) => break,
//!         _ => continue,
//!     }
//! }
//! ```
//! ## by constructing the client manually with your own Read/Write types
//! ```no_run
//! use std::net::TcpStream;
//! use twitchchat::commands;
//! use twitchchat::*;
//!
//! // or anything that implements std::io::Read + Send + Sync and std::io::Write + Send + Sync
//! let (read, write) = TcpStream::connect(twitchchat::TWITCH_IRC_ADDRESS)
//!     .map(|w| (w.try_clone().unwrap(), w))
//!     .unwrap();
//!
//! // use an anonymous login (you should probably use your name and your chat oauth token)
//! let (nick, token) = twitchchat::ANONYMOUS_LOGIN;
//! let config = UserConfig::builder()
//!     .token(token) // your oauth token
//!     .nick(nick) // your nickname
//!     .commands() // command capabilites (see: https://dev.twitch.tv/docs/irc/commands/ )
//!     .membership() // command capabilites (see: https://dev.twitch.tv/docs/irc/membership/ )
//!     .tags() // command capabilites (see: https://dev.twitch.tv/docs/irc/tags/ )
//!     .build() // verify the settings
//!     .unwrap();
//!
//! let client = Client::register(config, read, write).unwrap();
//! let client = client.filter::<commands::PrivMsg>();
//! let writer = client.writer();
//!
//! for event in client {
//!     match event {
//!         Event::IrcReady(..) => writer.join("museun").unwrap(),
//!         Event::Message(Message::PrivMsg(msg)) => {
//!             println!("{}: {}", msg.user(), msg.message());
//!         }
//!         Event::Error(..) => break,
//!         _ => continue,
//!     }
//! }
//! ```
#![warn(
    missing_docs,
    unsafe_code,
    unused_lifetimes,
    unused_qualifications,
    unused_results
)]

/// IRC-related stuff (not really intended for use with real IRC networks)
pub mod irc;

mod tags;
/// IRCv3 Tags
pub use tags::Tags;

/// Types associated with twitch
mod twitch;
pub use twitch::*;

pub use crate::twitch::filter::Filter;

pub use self::twitch::UserConfig;

/// Message conversion types
pub mod conversion;
#[doc(inline)]
pub use conversion::ToMessage;

/// Simple function to connect to Twitch using a [`TcpStream (non-TLS)`](https://doc.rust-lang.org/std/net/struct.TcpStream.html)
///
/// This is a convenience function for doing the same as:
/// ```no_run
/// use std::net::TcpStream;
/// let conn = std::net::TcpStream::connect(twitchchat::TWITCH_IRC_ADDRESS).unwrap();
/// let (read, write) = (conn.try_clone().unwrap(), conn);
///
/// let client = twitchchat::Client::register(
///     twitchchat::UserConfig::with_caps()
///         .nick("your_name_here")
///         .token("your_password_here")
///         .build()
///         .unwrap(),
///     read,
///     write,
/// );
/// ```
pub fn connect<U>(config: U) -> Result<Client<std::net::TcpStream, std::net::TcpStream>, Error>
where
    U: std::borrow::Borrow<UserConfig>,
{
    let (read, write) = {
        let stream = std::net::TcpStream::connect(TWITCH_IRC_ADDRESS).map_err(Error::Connect)?;
        stream.set_nonblocking(true).map_err(Error::Connect)?;
        (stream.try_clone().unwrap(), stream)
    };
    Client::register(config, read, write)
}

/// The simpliest way to connect with this crate. It uses a [`TcpStream (non-TLS)`](https://doc.rust-lang.org/std/net/struct.TcpStream.html)
///
/// This is like [`connect`](./fn.connect.html) except it just takes in your `nick` and `token` and sets all of the capabilities.
pub fn connect_easy(
    nick: impl ToString,
    token: impl ToString,
) -> Result<Client<std::net::TcpStream, std::net::TcpStream>, Error> {
    let (read, write) = {
        let stream = std::net::TcpStream::connect(TWITCH_IRC_ADDRESS).map_err(Error::Connect)?;
        stream.set_nonblocking(true).map_err(Error::Connect)?;
        (stream.try_clone().unwrap(), stream)
    };
    Client::register(
        UserConfig::with_caps()
            .nick(nick)
            .token(token)
            .build()
            .ok_or_else(|| Error::InvalidRegistration)?,
        read,
        write,
    )
}

/// The Twitch IRC address for non-TLS connections
pub const TWITCH_IRC_ADDRESS: &str = "irc.chat.twitch.tv:6667";

/// The Twitch IRC address for TLS connections
pub const TWITCH_IRC_ADDRESS_TLS: &str = "irc.chat.twitch.tv:6697";

/// An anonymous login. You won't be able to send messages, but you can join channels and read messages
pub const ANONYMOUS_LOGIN: (&str, &str) = (
    twitch::userconfig::JUSTINFAN1234,
    twitch::userconfig::JUSTINFAN1234,
);
