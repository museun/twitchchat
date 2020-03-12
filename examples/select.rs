/* in your Cargo.toml
[dependencies]
twitchchat = "0.9"                                         # this crate
tokio = { version = "0.2", features = ["full", "macros"] } # you need tokio to run it
*/

// or you can use futures::stream::StreamExt
use tokio::stream::StreamExt as _;

use twitchchat::{events, Control, Dispatcher, Runner, Status};

#[tokio::main]
async fn main() {
    let channels = &[std::env::var("TWITCH_CHANNEL").unwrap()];

    let dispatcher = Dispatcher::new();
    let (runner, control) = Runner::new(dispatcher.clone());
    let fut = run_loop(control.clone(), dispatcher, channels);

    let (nick, pass) = twitchchat::ANONYMOUS_LOGIN;
    let conn = twitchchat::connect_easy_tls(nick, pass).await.unwrap();

    tokio::select! {
        _ = fut => { control.stop() }
        status = runner.run(conn) => {
            match status {
                Ok(Status::Eof) => {}
                Ok(Status::Canceled) => {}
                Err(err) => panic!(err),
            }
        }
    }
}

async fn run_loop(mut control: Control, mut dispatcher: Dispatcher, channels: &[String]) {
    let mut join = dispatcher.subscribe::<events::Join>();
    let mut part = dispatcher.subscribe::<events::Part>();

    async fn wait_and_join(
        control: &mut Control,
        dispatcher: &mut Dispatcher,
        channels: &[String],
    ) {
        let ready = dispatcher.wait_for::<events::IrcReady>().await.unwrap();
        eprintln!("our name: {}", ready.nickname);

        let w = control.writer();
        for channel in channels {
            eprintln!("joining: {}", channel);
            let _ = w.join(channel).await;
            eprintln!("joined");
        }
        eprintln!("joined all channels")
    }

    wait_and_join(&mut control, &mut dispatcher, channels).await;

    loop {
        tokio::select! {
            Some(msg) = join.next() => {
                eprintln!("{} joined {}", msg.name, msg.channel);
            }
            Some(msg) = part.next() => {
                eprintln!("{} left {}", msg.name, msg.channel);
            }
            else => { break }
        }
    }
}
