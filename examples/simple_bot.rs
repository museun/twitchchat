/* in your Cargo.toml
[dependencies]
# this crate
twitchchat = "0.11"

# and now for tokio
# macros allows you to use `#[tokio::main]` and `tokio::pin!` and `tokio::select!`
# rt-threaded gives you a multi-threaded runtime.
tokio = { version = "0.2", features = ["rt-threaded", "macros"] }
*/

use tokio::stream::StreamExt as _;
use twitchchat::{events, messages, Control, Dispatcher, IntoChannel, Runner, Writer};

// your twitch name. it should be associated with your oauth token
fn get_nick() -> String {
    std::env::var("TWITCH_NICK").unwrap()
}

// your oauth token
fn get_pass() -> String {
    std::env::var("TWITCH_PASS").unwrap()
}

// a channel to join
fn get_channel() -> String {
    std::env::var("TWITCH_CHANNEL").unwrap()
}

struct Bot {
    // you can store the writer (and clone it)
    writer: Writer,
    // and you can store the Control (and clone it)
    control: Control,
    start: std::time::Instant,
}

impl Bot {
    async fn run(mut self, dispatcher: Dispatcher, channel: impl IntoChannel) {
        // subscribe to the events we're interested in
        let mut events = dispatcher.subscribe::<events::Privmsg>();

        // and wait for a specific event (blocks the current task)
        let ready = dispatcher.wait_for::<events::IrcReady>().await.unwrap();
        eprintln!("connected! our name is: {}", ready.nickname);

        // and then join a channel
        eprintln!("joining our channel");
        self.writer.join(channel).await.unwrap();

        // and then our 'main loop'
        while let Some(msg) = events.next().await {
            if !self.handle(&*msg).await {
                return;
            }
        }
    }

    async fn handle(&mut self, msg: &messages::Privmsg<'_>) -> bool {
        // protip: we can reborrow/deref the `data` field (a `Cow`) to get a `&str`
        match &*msg.data {
            "!hello" => {
                let resp = format!("hello {}!", msg.name);
                self.writer.privmsg(&msg.channel, &resp).await.unwrap();
            }
            "!uptime" => {
                let dur = std::time::Instant::now() - self.start;
                let resp = format!("I've been running for.. {:.2?}.", dur);
                self.writer.privmsg(&msg.channel, &resp).await.unwrap();
            }
            "!quit" => {
                // this'll stop the runner (causing its future to return Ok(Status::Canceled))
                self.control.stop();
                return false; // to stop the 'Bot'
            }
            _ => {}
        };
        true // to keep the 'Bot' running
    }
}

#[tokio::main]
async fn main() {
    let dispatcher = Dispatcher::new();
    let (mut runner, mut control) = Runner::new(dispatcher.clone());

    // get the channel from the env
    let channel = get_channel();

    // make a bot and get a future to its main loop
    let bot = Bot {
        // just to show you can store it
        writer: control.writer().clone(),
        // but you probably want to store the control instead
        control,
        start: std::time::Instant::now(),
    }
    .run(dispatcher, channel);

    // create a connector, this can be used to reconnect.
    let connector = twitchchat::Connector::new(|| async move {
        let (nick, pass) = (get_nick(), get_pass());
        // connect to twitch
        twitchchat::native_tls::connect_easy(&nick, &pass).await
    });

    // and run the dispatcher/writer loop
    //
    // this uses a retry strategy that'll reconnect with the connect.
    // using the `run_with_retry` method will consume the 'Status' types
    let done = runner.run_with_retry(connector, twitchchat::RetryStrategy::on_timeout);

    // and select over our two futures
    tokio::select! {
        // wait for the bot to complete
        _ = bot => { eprintln!("done running the bot") }
        // or wait for the runner to complete
        status = done => {
            match status {
                Ok(()) => { eprintln!("we're done") }
                Err(err) => { eprintln!("error running: {}", err) }
            }
        }
    }
}
