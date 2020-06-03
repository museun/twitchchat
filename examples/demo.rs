/* in your Cargo.toml
[dependencies]
# this crate
twitchchat = "0.10"

# and now for tokio
# macros allows you to use `#[tokio::main]` and `tokio::pin!` and `tokio::select!`
# rt-multi gives you a multi-threaded runtime.
tokio = { version = "0.2", features = ["rt-multi", "macros"] }
*/

// or you can use `futures::stream::StreamExt`
use tokio::stream::StreamExt as _;

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

#[tokio::main]
async fn main() {
    // make a dispatcher (this is how you 'subscribe' to events)
    // this is clonable, so you can send it to other tasks/threasd
    let dispatcher = twitchchat::Dispatcher::new();

    // subscribe to a Privmsg event stream
    // whenever the client reads a PRIVMSG, it'll produce an item in this stream
    // you can subscribe multiple times to the same event
    let mut privmsg = dispatcher.subscribe::<twitchchat::events::Privmsg>();

    // spawn a task to consume the stream
    tokio::task::spawn(async move {
        while let Some(msg) = privmsg.next().await {
            eprintln!("[{}] {}: {}", msg.channel, msg.name, msg.data);
        }
    });

    // these can fail if the event wasn't registered. but the included events are always registered
    // for join (when a user joins a channel)
    let mut join = dispatcher.subscribe::<twitchchat::events::Join>();

    // for part (when a user leaves a channel)
    let mut part = dispatcher.subscribe::<twitchchat::events::Part>();

    // there is also an `All` event which is an enum of all possible events
    // and a `Raw` event which is the raw IRC message

    // make a new runner. this gives you the runner and a control type
    // the control type allows you to stop the runner, and gives you access to an async. encoder (writer)
    //
    // you have to give it your dispatcher (which is clonable)
    let (mut runner, mut control) = twitchchat::Runner::new(dispatcher.clone());

    // make a connector, this acts a factory incase you want to reconnect easily.
    // connect via TCP with TLS with this nick and password
    let connector = twitchchat::Connector::new(|| async move {
        let (nick, pass) = (get_nick(), get_pass());
        twitchchat::native_tls::connect_easy(&nick, &pass).await
    });

    // spawn the run off in another task so we don't block the current one.
    // you could just await on the future at the end of whatever block, but this is easier for this demonstration
    let handle = tokio::task::spawn(async move {
        // we have to use an async block to for a move.
        //
        // this takes the connector
        runner.run_to_completion(connector).await
    });

    // another privmsg so we can act like a bot
    let mut privmsg = dispatcher.subscribe::<twitchchat::events::Privmsg>();

    // we can block on the dispatcher for a specific event
    // if we call wait_for again for this event, it'll return the previous one
    eprintln!("waiting for irc ready");
    let ready = dispatcher
        .wait_for::<twitchchat::events::IrcReady>()
        .await
        .unwrap();
    eprintln!("our nickname: {}", ready.nickname);

    // we can clone the writer and send it places
    let mut writer = control.writer().clone();

    let channel = get_channel();
    // because we waited for IrcReady, we can confidently join channels
    writer.join(channel).await.unwrap();

    // a fancy main loop without using tasks
    loop {
        tokio::select! {
            Some(join_msg) = join.next() => {
                eprintln!("{} joined {}", join_msg.name, join_msg.channel);
            }

            Some(part_msg) = part.next() => {
                eprintln!("{} left {}", part_msg.name, part_msg.channel);
            }

            Some(msg) = privmsg.next() => {
                match msg.data.split(' ').next() {
                    Some("!hello") => {
                        let response = format!("hello {}!", msg.name);
                        if let Err(_err) = control.writer().privmsg(&msg.channel, &response).await {
                            // we cannot write, so we should bail
                            break;
                        }
                    }
                    Some("!quit") => {
                        // this causes the runner to shutdown
                        control.stop();
                    }
                    _ => {}
                }
            }

            // when the 3 streams in this select are done this'll get hit
            else => { break }
        }
    }

    // you can clear subscriptions with
    // dispatcher.clear_subscriptions::<event::Join>()
    // or all subscriptions
    // dispatcher.clear_subscriptions_all()

    // you can get the number of active subscriptions with
    // dispatcher.count_subscribers::<event::Join>()
    // or all subscriptions
    // dispatcher.count_subscribers_all()

    // await for the client to be done
    // unwrap the JoinHandle
    match handle.await.unwrap() {
        Ok(twitchchat::Status::Eof) => {
            eprintln!("done!");
        }
        Ok(twitchchat::Status::Canceled) => {
            eprintln!("client was stopped by user");
        }
        Ok(twitchchat::Status::Timeout) => {
            eprintln!("client's connection timed out");
        }
        Err(err) => {
            eprintln!("error: {}", err);
        }
    }

    // *note*: you should wait for all of your tasks to join before exiting
    // but we detached them to make this shorter

    // another way would be to clear all subscriptions
    // clearing the subscriptions would close each event stream
    dispatcher.clear_subscriptions_all();
}
