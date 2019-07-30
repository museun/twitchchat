fn main() {
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
}
