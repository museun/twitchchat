use std::net::TcpStream;
use twitchchat::{commands::PrivMsg, Client, UserConfig};

fn main() -> Result<(), Box<std::error::Error>> {
    // connect to twitch via a tcp stream, creating a read/write pair
    let (read, write) = {
        let stream = TcpStream::connect(twitchchat::TWITCH_IRC_ADDRESS)?;
        (stream.try_clone()?, stream)
    };

    // create a config
    let conf = user_config();

    // create a client from the read/write pair
    let mut client = Client::new(read, write);

    // register with the server, using the config
    client.register(conf)?;

    // wait until the server tells us who we are
    let local = client.wait_for_ready()?;
    let mention = format!("@{}", local.display_name.unwrap());

    // use a message filter. you can store the `Token` this returns
    // and remove this filter later on with the `Client::off` method
    client.on(move |msg: PrivMsg| {
        println!("{}: {}", msg.irc_name(), msg.message());
    });

    // multiple filters for the same type of message is allowed
    // clone so we can move it into the closure
    let mut clone = client.clone();
    client.on(move |msg: PrivMsg| {
        if msg.message().contains(&mention) {
            clone.send(msg.channel, "VoHiYo").unwrap();
        }
    });

    // join a channel
    client.join("museun")?;

    // send a message to the channel
    client.send("museun", "HeyGuys")?;

    // run until an error
    client.run()?;

    // (would have to box the error up after turning it into a trait object
    //  so lets just ? then Ok)
    Ok(())
}

fn user_config() -> UserConfig {
    let (nick, pass) = (var("MY_TWITCH_NICK"), var("MY_TWITCH_PASS"));
    let config = UserConfig::builder().nick(nick).token(pass);
    config.build().unwrap()
}

fn var(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("please set the env var `{}`", key))
}
