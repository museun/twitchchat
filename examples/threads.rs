fn main() {
    use std::sync::mpsc::{self, TryRecvError};
    use twitchchat::commands;
    use twitchchat::*;

    let (nick, token) = twitchchat::ANONYMOUS_LOGIN;
    let client = twitchchat::connect_easy(&nick, &token)
        .unwrap()
        .filter::<commands::PrivMsg>();

    let writer = client.writer();
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        for event in client {
            match event {
                Event::IrcReady(..) => writer.join("museun").unwrap(),
                Event::Message(Message::PrivMsg(msg)) => {
                    if tx.send(msg).is_err() {
                        break;
                    }
                }
                Event::Error(..) => break,
                _ => continue,
            }
        }
    });

    loop {
        match rx.try_recv() {
            Ok(msg) => println!("{}: {}", msg.user(), msg.message()),
            Err(TryRecvError::Disconnected) => break,
            Err(TryRecvError::Empty) => {}
        }
        println!("sleeping");
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
