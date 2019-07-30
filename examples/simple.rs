fn main() {
    // all of the messages twitch can send
    use twitchchat::commands;
    // glob all of the useful stuff
    use twitchchat::*;

    // use an anonymous login (you should probably use your name and your chat oauth token)
    let (nick, token) = twitchchat::ANONYMOUS_LOGIN;

    // connect with the the nick/token and all the caps enabled
    let client = twitchchat::connect_easy(&nick, &token)
        .unwrap()
        .filter::<commands::PrivMsg>();

    // get the writer (can be cloned and sent to other threads)
    let writer = client.writer();

    // for each event from the client
    for event in client {
        match event {
            // when we get an IrcReady Message (the `anonymous` login won't get the full-featured `TwitchReady` event)
            // join a channel using the writer
            Event::IrcReady(..) => writer.join("museun").unwrap(),
            // When we get the filtered message
            // print out name: their_message
            Event::Message(Message::PrivMsg(msg)) => {
                println!("{}: {}", msg.user(), msg.message());
            }
            // break on errors
            Event::Error(..) => break,
            // ignore anything else
            _ => continue,
        }
    }
}
