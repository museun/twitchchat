use tokio::stream::StreamExt as _;
use twitchchat::{events, messages, Control, Dispatcher, IntoChannel, Runner, Status, Writer};

fn get_creds() -> (String, String, String) {
    fn get_it(name: &str) -> String {
        std::env::var(name).unwrap_or_else(|_| {
            eprintln!("env var `{}` is required", name);
            std::process::exit(1);
        })
    }

    (
        get_it("TWITCH_NICK"),
        get_it("TWITCH_PASS"),
        get_it("TWITCH_CHANNEL"),
    )
}

struct Bot {
    // you can store the writer (and clone it)
    writer: Writer,
    // and you can store/clone the Control
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
    let (user, pass, channel) = get_creds();

    let dispatcher = Dispatcher::new();
    let (runner, mut control) = Runner::new(dispatcher.clone(), twitchchat::RateLimit::default());

    // make a bot and get a future to its main loop
    let bot = Bot {
        // just to show you can store it
        writer: control.writer().clone(),
        // but you probably want to store the control instead
        control,
        start: std::time::Instant::now(),
    }
    .run(dispatcher, channel);

    // connect to twitch
    let conn = twitchchat::connect_easy_tls(&user, &pass).await.unwrap();
    // and run the dispatcher/writer loop
    let done = runner.run(conn);

    // and select over our two futures
    tokio::select! {
        // wait for the bot to complete
        _ = bot => { eprintln!("done running the bot") }
        // or wait for the runner to complete
        status = done => {
            match status {
                Ok(Status::Canceled) => { eprintln!("runner was canceled") }
                Ok(Status::Eof) => { eprintln!("got an eof, exiting") }
                Ok(Status::Timeout) => { eprintln!("client connection timed out") }
                Err(err) => { eprintln!("error running: {}", err) }
            }
        }
    }
}
