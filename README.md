# twitchchat
![Crates.io](https://img.shields.io/crates/l/twitchchat/0.1.7.svg?style=flat-square) 
[![doc.rs](https://docs.rs/twitchchat/badge.svg)](https://docs.rs/twitchchat/latest/twitchchat/)
[![Crates.io](https://img.shields.io/crates/v/twitchchat.svg)](https://crates.io/crates/twitchchat)
[![CircleCI](https://circleci.com/gh/museun/twitchchat.svg?style=svg)](https://circleci.com/gh/museun/twitchchat)
![AppVeyor](https://img.shields.io/appveyor/ci/museun/twitchchat.svg)


interface to the irc portion of twitch's chat

you provide the `std::io::Read` and the `std::io::Write` <br>
...and this provides all of the types for Twitch chat message.

see the [docs](https://docs.rs/twitchchat/latest/twitchchat) for more info

optional features: 

|feature | description | --- |
|--- | --- | --- |
| serde | enables serde derives | allowing you to turn stuff to json, and load it from json |
| hashbrown | enables hashbrown types | faster hashmaps, lower memory allocations |
| serde_hashbrown | enables serde and hashbrown+serde | basically serde+hashbrown. **use this if you want serde and hashbrown** |
| parking_lot | enables faster mutexes | --- |
| --- | --- |
| all | enables all of the above | --- |

they are disabled by default.


a demo of it:
```rust
fn main() {
    use std::net::TcpStream;
    use twitchchat::commands::PrivMsg;
    use twitchchat::{Client, UserConfig};

    // create a userconfig
    let userconfig = UserConfig::builder()
        .nick(env!("MY_TWITCH_NAME"))
        .token(env!("MY_TWITCH_PASS"))
        .build()
        .expect("semi-valid config");

    // connect to twitch
    let read = TcpStream::connect(twitchchat::TWITCH_IRC_ADDRESS).expect("connect");
    // clone the tcpstream
    let write = read.try_clone().expect("must be able to clone");

    // create a new client from the read, write pairs
    let mut client = Client::new(read, write);

    // clone the client so it can be sent across threads (and into move closures)
    let mut kappa = client.clone();

    // when we receive a PrivMsg run this function
    // tok allows us to remove this later, if we want
    let _tok = client.on(move |msg: PrivMsg| {
        // moves the 'kappa' client clone into this thread
        const KAPPA: usize = 25;
        // print out `user: message`
        println!("{}: {}", msg.display_name().unwrap(), msg.message());

        let kappas = msg
            .emotes()
            .iter()
            // filter Kappas
            .filter(|e| e.id == KAPPA)
            // count how many times it appears
            .map(|d| d.ranges.len())
            .sum::<usize>();

        // if someone sent more than 3 Kappas, send a Kappa back
        if kappas >= 3 {
            kappa.send(msg.channel, "Kappa").unwrap();
        }
    });

    // log if the broadcaster, a sub or a mod talks
    client.on(move |msg: PrivMsg| {
        use twitchchat::BadgeKind::{Broadcaster, Subscriber};

        let name = msg.display_name().unwrap_or_else(|| msg.irc_name());
        let badges = msg
            .badges()
            .iter()
            // filter to just the "BadgeKind"
            .map(|badge| badge.kind.clone())
            .collect::<Vec<_>>();

        match (
            badges.contains(&Broadcaster),
            badges.contains(&Subscriber),
            msg.moderator(), // or badges.contains(&Moderator)
        ) {
            (true, _, _) => println!("{} is the broadcaster", name),
            (_, true, _) => println!("{} is a subscriber", name),
            (_, _, true) => println!("{} is a mod", name),
            (_, _, _) => {
                // just a normal viewer
            }
        };
    });

    // 'register' (sends out creds.) with the server
    client.register(userconfig).expect("register with twitch");

    // blocks the thread until the server tells us who we were
    match client.wait_for_ready() {
        // and print it out
        Ok(user) => {
            // id: 23196011, name: Some("museun"), color: Some(OrangeRed)
            println!(
                "id: {}, name: {:?}, color: {:?}",
                user.user_id, user.display_name, user.color
            )
        }
        Err(twitchchat::Error::InvalidRegistration) => {
            eprintln!("invalid nick/pass");
            std::process::exit(1);
        }
        Err(err) => panic!(err),
    };

    // join a channel
    client.join("museun").unwrap();

    {
        // after 3 seconds, send a VoHiYo
        let mut client = client.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(3));
            client.send("museun", "VoHiYo").unwrap();
        });
    }

    // block this thread until the connection ends
    // this will call the filters when it receives the appropirate message
    if let Err(err) = client.run() {
        eprintln!("error while running: {}", err);
        std::process::exit(1);
    }
}
```
