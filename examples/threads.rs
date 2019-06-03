use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use twitchchat::{Client, UserConfig};

use rand::prelude::*;

fn main() -> Result<(), Box<std::error::Error>> {
    // connect to twitch via a tcp stream, creating a read/write pair
    let (read, write) = {
        let stream = TcpStream::connect(twitchchat::TWITCH_IRC_ADDRESS)?;
        (stream.try_clone()?, stream)
    };

    // create the read and write adapters from the TcpStream
    let (read, write) = twitchchat::sync_adapters(read, write);

    // create a config
    let conf = user_config();

    // create a client from the read/write pair
    let mut client = Client::new(read, write);

    // register with the server, using the config
    client.register(conf)?;

    // wait until the server tells us who we are
    let _local = client.wait_for_ready()?;

    // get a thread-safe writer
    let w = client.writer();
    thread::spawn(move || {
        const EMOTES: [&str; 9] = [
            "Kappa",
            "SMOrc",
            "LUL",
            "SeemsGood",
            "HeyGuys",
            "PogChamp",
            "NotLikeThis",
            "WutFace",
            "ResidentSleeper",
        ];

        let mut rng = thread_rng();
        loop {
            // every 5 to 10 seconds
            thread::sleep(Duration::from_secs(rng.gen_range(5, 10)));
            // pick 3 random emotes
            let poop: Vec<_> = EMOTES.choose_multiple(&mut rng, 3).map(|s| *s).collect();
            // and send them
            if w.send("museun", poop.join(" ")).is_err() {
                return;
            };
        }
    });

    // join a channel
    client.writer().join("museun")?;

    // run until an error
    client.run()?;

    Ok(())
}

fn user_config() -> UserConfig {
    let (nick, pass) = (var("MY_TWITCH_NICK"), var("MY_TWITCH_PASS"));
    let config = UserConfig::builder()
        .nick(nick)
        .token(pass)
        .membership()
        .commands()
        .tags();;
    config.build().unwrap()
}

fn var(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("please set the env var `{}`", key))
}
