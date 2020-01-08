# twitchchat

# NOTE this crate is undergoing massive changes.
**Please check out the async branch, or one of the v0.8.0 tags**



[![Build Status](https://circleci.com/gh/museun/twitchchat/tree/master.svg?style=shield)](https://circleci.com/gh/museun/twitchchat/cargo-readme/tree/master)

## twitchchat
This crate provides a way to interact with Twitch's chat

Along with the messages/commands as Rust types, it provides methods for sending messags/commands.

### a simple example
```rust
use twitchchat::commands;
use twitchchat::*;

// use an anonymous login (you should probably use your name and your chat oauth token)
let (nick, token) = twitchchat::ANONYMOUS_LOGIN;

// connect with this nick, token
let mut client = twitchchat::connect_easy(nick, token)
    .unwrap() // this is an error if
              // the network connection can't be opened,
              // the nick/pass is invalid, etc
     // add some filters
    .filter::<commands::PrivMsg>() // filter to PrivMsg commands
    .filter::<commands::Join>();   // filter to Join commands

// get a clonable, threadsafe writer
let writer = client.writer();

// for each event from the client, blocking
// a client.nonblocking_iter() also exists
for event in client {
    match event {
        // when we're connected
        Event::IrcReady(..) => {
            // join a channel
            writer.join("museun").unwrap();
        }
        // when we get a priv msg
        Event::Message(Message::PrivMsg(msg)) => {
            // print out the sender : messsage
            println!("{}: {}", msg.user(), msg.message());
        }
        // when we get a join msg
        Event::Message(Message::Join(msg)) => {
            // print out the user and the channel that was joined
            println!("*** {} joined {}", msg.user(), msg.channel())
        }
        // when we get an error
        Event::Error(err) => {
            // print it out
            eprintln!("error: {}", err);
            // and exit the loop
            break;
        }
        // not used here
        Event::TwitchReady(..) => {
            // this only happens when you're using Capability::Tags, Capability::Commands and a non-anonymous login
        }
        // make the compiler happy
        _ => unreachable!(),
    }
}
```
### with custom capabilities
```rust
use twitchchat::commands;
use twitchchat::*;

// use an anonymous login (you should probably use your name and your chat oauth token)
let (nick, token) = twitchchat::ANONYMOUS_LOGIN;
let config = UserConfig::builder()
    .token(token) // your oauth token
    .nick(nick) // your nickname
    .commands() // command capabilites (see: https://dev.twitch.tv/docs/irc/commands/ )
    .membership() // command capabilites (see: https://dev.twitch.tv/docs/irc/membership/ )
    .tags() // command capabilites (see: https://dev.twitch.tv/docs/irc/tags/ )
    .build() // verify the settings
    .unwrap();

// connect with the config
let client = twitchchat::connect(&config)
    .unwrap()
    .filter::<commands::PrivMsg>();
let writer = client.writer();

for event in client {
    match event {
        Event::IrcReady(..) => writer.join("museun").unwrap(),
        Event::Message(Message::PrivMsg(msg)) => {
            println!("{}: {}", msg.user(), msg.message());
        }
        Event::Error(..) => break,
        _ => continue,
    }
}
```
### by constructing the client manually with your own Read/Write types
```rust
use std::net::TcpStream;
use twitchchat::commands;
use twitchchat::*;

// or anything that implements std::io::Read + Send + Sync and std::io::Write + Send + Sync
let (read, write) = TcpStream::connect(twitchchat::TWITCH_IRC_ADDRESS)
    .map(|w| (w.try_clone().unwrap(), w))
    .unwrap();

// use an anonymous login (you should probably use your name and your chat oauth token)
let (nick, token) = twitchchat::ANONYMOUS_LOGIN;
let config = UserConfig::builder()
    .token(token) // your oauth token
    .nick(nick) // your nickname
    .commands() // command capabilites (see: https://dev.twitch.tv/docs/irc/commands/ )
    .membership() // command capabilites (see: https://dev.twitch.tv/docs/irc/membership/ )
    .tags() // command capabilites (see: https://dev.twitch.tv/docs/irc/tags/ )
    .build() // verify the settings
    .unwrap();

let client = Client::register(config, read, write).unwrap();
let client = client.filter::<commands::PrivMsg>();
let writer = client.writer();

for event in client {
    match event {
        Event::IrcReady(..) => writer.join("museun").unwrap(),
        Event::Message(Message::PrivMsg(msg)) => {
            println!("{}: {}", msg.user(), msg.message());
        }
        Event::Error(..) => break,
        _ => continue,
    }
}
```

License: 0BSD
