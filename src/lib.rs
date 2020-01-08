#![cfg_attr(debug_assertions, allow(dead_code,))]
/*!
This crate provides a way to interace with [Twitch]'s chat.

Along with the messages as Rust types, it provides methods for sending messages.

# Simple example
```rust,ignore
#[tokio::main]
async fn main() {
    use futures::prelude::*;

    let (nick, pass) = (
        // twitch name
        std::env::var("TWITCH_NICK").unwrap(),
        // oauth token for twitch name
        std::env::var("TWITCH_PASS").unwrap(),
    );

    // putting this in the env so people don't join my channel when running this
    let channel = std::env::var("TWITCH_CHANNEL").unwrap();

    // connect via (tls or normal, 'Secure' determines that) tcp with this nick and password
    let (read, write) = twitchchat::connect_easy(&nick, &pass, twitchchat::Secure::Nope)
        .await
        .unwrap();

    // make a client. the client is clonable
    let client = twitchchat::Client::new();

    // get a future that resolves when the client is done reading, fails to read/write or is stopped
    let done = client.run(read, write);

    // get an event dispatcher
    let mut dispatcher = client.dispatcher().await;

    // subscribe to an event stream

    // for privmsg (what users send to channels)
    let mut privmsg = dispatcher.subscribe::<twitchchat::events::Privmsg>();
    // spawn a task to consume the stream
    tokio::task::spawn(async move {
        while let Some(msg) = privmsg.next().await {
            eprintln!("[{}] {}: {}", msg.channel, msg.user, msg.data);
        }
    });

    // for join (when a user joins a channel)
    let mut join = dispatcher.subscribe::<twitchchat::events::Join>();
    tokio::task::spawn(async move {
        while let Some(msg) = join.next().await {
            // we've joined a channel
            if msg.user == nick {
                eprintln!("you joined {}", msg.channel);
                break; // returning/dropping the stream un-subscribes it
            }
        }
    });

    // for privmsg again
    let mut bot = dispatcher.subscribe::<twitchchat::events::Privmsg>();
    // we can move the client to another task by cloning it
    let bot_client = client.clone();
    tokio::task::spawn(async move {
        let mut writer = bot_client.writer();
        while let Some(msg) = bot.next().await {
            match msg.data.split(" ").next() {
                Some("!quit") => {
                    // causes the client to shutdown
                    bot_client.stop().await.unwrap();
                }
                Some("!hello") => {
                    let response = format!("hello {}!", msg.user);
                    // send a message in response
                    let still_connected = writer.privmsg(&msg.channel, &response).await;
                    if !still_connected {
                        break;
                    }
                }
                _ => {}
            }
        }
    });

    // dispatcher has an RAII guard, so keep it scoped
    // dropping it here so everything can proceed while keeping example brief
    drop(dispatcher);

    // get a clonable writer from the client
    // join a channel, methods on writer return false if the client is disconnected
    if !client.writer().join(&channel).await {
        panic!("not connected!?")
    }

    // you can clear subscriptions with
    // client.dispatcher().await.clear_subscriptions::<event::Join>()
    // or all subscriptions
    // client.dispatcher().await.clear_subscriptions_all()

    // you can get the number of active subscriptions with
    // client.dispatcher().await.count_subscribers::<event::Join>()
    // or all subscriptions
    // client.dispatcher().await.count_subscribers_all()

    // await for the client to be done
    match done.await {
        Ok(twitchchat::client::Status::Eof) => {
            eprintln!("done!");
        }
        Ok(twitchchat::client::Status::Canceled) => {
            eprintln!("client was stopped by user");
        }
        Err(err) => {
            eprintln!("error: {}", err);
        }
    }

    // note you should wait for all of your tasks to join before exiting
    // but we detached them to make this shorter

    // another way would be to clear all subscriptions
    // clearing the subscriptions would close each event stream
    client.dispatcher().await.clear_subscriptions_all();
}
```

[Twitch]: https://www.twitch.tv
*/

#[macro_use]
mod macros;

cfg_async! {
    pub mod client;
    #[doc(inline)]
    pub use client::Client;
}

/// Decode messages from a `&str`
pub mod decode;
#[doc(inline)]
pub use decode::{decode, decode_many};

/// Encode messages to a writer
pub mod encode;
#[doc(inline)]
pub use encode::Encodable;

cfg_async! {
    #[doc(inline)]
    pub use encode::encode;
}

/// Common Twitch types
pub mod twitch;

#[doc(inline)]
pub use twitch::*;

cfg_async! {
    pub mod events;
}

pub mod messages;

/// The Twitch IRC address for non-TLS connections
pub const TWITCH_IRC_ADDRESS: &str = "irc.chat.twitch.tv:6667";

/// The Twitch IRC address for TLS connections
pub const TWITCH_IRC_ADDRESS_TLS: &str = "irc.chat.twitch.tv:6697";

/// The Twitch WebSocket address for non-TLS connections
pub const TWITCH_WS_ADDRESS: &str = "ws://irc-ws.chat.twitch.tv:80";

/// The Twitch WebSocket address for TLS connections
pub const TWITCH_WS_ADDRESS_TLS: &str = "wss://irc-ws.chat.twitch.tv:443";

cfg_async! {
    use tokio::io::{AsyncRead, AsyncWrite};
}

/// Connection type
///
/// Defaults to `Nope`
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Secure {
    /// Use TLS
    UseTls,
    /// Don't use TLS
    Nope,
}

impl Default for Secure {
    fn default() -> Self {
        Self::Nope
    }
}

impl Secure {
    /// Gets the requested (IRC) address
    pub fn get_address(&self) -> &'static str {
        match self {
            Secure::UseTls => TWITCH_IRC_ADDRESS_TLS,
            Secure::Nope => TWITCH_IRC_ADDRESS,
        }
    }
}

cfg_async! {
    /// Write the provided `UserConfig` to the ***async*** writer
    pub async fn register<W: ?Sized>(
        user_config: &UserConfig,
        writer: &mut W,
    ) -> std::io::Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        encode(user_config, writer).await
    }
}

cfg_async! {
    /// Opens an ***async*** TCP connection using the provided `UserConfig` and `Secure` setting
    ///
    /// If `secure` is `None`, it'll use the normal TCP socket
    pub async fn connect(
        user_config: &UserConfig,
        secure: impl Into<Option<Secure>>,
    ) -> std::io::Result<(impl AsyncRead, impl AsyncWrite)> {
        let addr = secure.into().unwrap_or_default().get_address();
        let mut stream = tokio::net::TcpStream::connect(addr).await?;
        register(user_config, &mut stream).await?;
        Ok(tokio::io::split(stream))
    }

    /// Opens an ***async*** TCP connection using the provided `name`, `token and `Secure` setting
    ///
    /// If `secure` is `None`, it'll use the normal TCP socket
    ///
    /// This enables all of the [Capabilities]
    ///
    /// [Capabilities]: ./enum.Capability.html
    pub async fn connect_easy(
        name: &str,
        token: &str,
        secure: impl Into<Option<Secure>>,
    ) -> std::io::Result<(impl AsyncRead, impl AsyncWrite)> {
        let config = simple_user_config(name, token).unwrap();
        connect(&config, secure).await
    }
}

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

/**
An anonymous login.

You won't be able to send messages, but you can join channels and read messages

# usage
```rust
# use twitchchat::{ANONYMOUS_LOGIN,UserConfig};
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

mod internal;
pub(crate) use internal::{IntoOwned, StringMarker};

/// Synchronous methods
pub mod sync;
