// NOTE: this demo requires `--feature smol`.

// `futures_lite` or `futures` would work. you'd just need the `StreamExt` trait to iterate over `EventStream`
use futures_lite::*;

use twitchchat::{commands, connector, messages, runner::Status};

fn expect_env_var(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("please set `{}`", key))
}

fn get_user_config() -> twitchchat::UserConfig {
    let name = expect_env_var("TWITCH_NAME");
    let token = expect_env_var("TWITCH_TOKEN");

    // you need a `UserConfig` to connect to Twitch
    twitchchat::UserConfig::builder()
        // the name of the associated twitch account
        .name(name)
        // and the provided OAuth token
        .token(token)
        // and enable all of the advanced message signaling from Twitch
        .enable_all_capabilities()
        .build()
        .unwrap()
}

fn channels_to_join() -> Vec<String> {
    expect_env_var("TWITCH_CHANNEL")
        .split(',')
        .map(ToString::to_string)
        .collect()
}

fn main() {
    let fut = async move {
        // create a user configuration
        let user_config = get_user_config();
        // get some channels to join from the environment
        let channels = channels_to_join();

        // create a connector using Smol, this connects to Twitch.
        // you can provide a different address with `custom`
        let connector = connector::smol::Connector::twitch();

        // create a new Dispatcher. this allows you to 'subscribe' to specific events
        let dispatcher = twitchchat::AsyncDispatcher::new();
        // lets subscribe to privmsgs (what users send to channels)
        let mut privmsg = dispatcher.subscribe::<messages::Privmsg>().await;
        // and the raw irc message
        let mut all = dispatcher.subscribe::<messages::IrcMessage>().await;

        // 'subscribe' returns a stream, so we'll spawn a task and loop over it until its done producing messages.
        // the event stream will 'close' when you the main loop exists or call reset() on the dispatcher
        smol::Task::spawn(async move {
            while let Some(_msg) = all.next().await {
                // do something with msg. we'll ignore it so the output isn't
                // spammed
            }
        })
        // and detach it so its runs in the background
        .detach();

        // do the same for the privmsg stream
        smol::Task::spawn(async move {
            while let Some(msg) = privmsg.next().await {
                println!("[{}] {}: {}", msg.channel(), msg.name(), msg.data())
            }
        })
        .detach();

        // create a new runner. this is a provided async 'main loop'
        let mut runner = twitchchat::AsyncRunner::new(dispatcher, user_config, connector);
        // which'll let you get a writer out.
        let mut writer = runner.writer();

        smol::Task::spawn({
            let mut writer = writer.clone();
            let channels = channels.clone();
            async move {
                println!("in 10 seconds we'll exit");
                smol::Timer::new(std::time::Duration::from_secs(10)).await;

                // send one final message to all channels
                for channel in channels {
                    let cmd = commands::privmsg(&channel, "goodbye, world");
                    writer.encode(cmd).await.unwrap();
                }

                // and tell it to shutdown
                writer.quit().await.unwrap();
            }
        })
        .detach();

        // before you run the main loop, you can 'block' for a specific message to come in
        println!("waiting for irc ready");
        runner.wait_for_ready::<messages::IrcReady>().await.unwrap();
        println!("we're ready, joining");

        for channel in channels {
            println!("joining: {}", channel);
            // the writer lets you encode types to it. these types are generally 0 cost.
            writer.encode(commands::join(&channel)).await.unwrap();
        }

        // and block the future until the loop exists
        println!("running main loop");
        let res = runner.run_to_completion().await;
        println!("after main loop");
        res
    };

    // run our main future on the smol runtime
    match smol::run(fut) {
        // this happens when your connection times out
        Ok(Status::TimedOut) => {
            println!("the connection timed out");
        }

        // this happens you use use the Runner::quit_signal() or use the 'quit' command
        Ok(Status::Cancelled) => {
            println!("the user requested the connection to close");
        }

        // this is the normal exit. Twitch closed your connection.
        // you can trigger this by sending a `quit command`
        Ok(Status::Eof) => {
            println!("the connection was closed");
        }

        // this happens when the crate ran into an unrecovable error (i/o, parsing, etc)
        Err(err) => {
            eprintln!("ran into an error: {}", err);
            std::process::exit(1)
        }

        // This is reserved for future use, you should ignore this
        _ => {}
    }
}
