#[tokio::main]
async fn main() {
    use futures::prelude::*;

    let (nick, pass) = (
        // twitch name
        std::env::var("TWITCH_NICK").unwrap(),
        // oauth token for twitch name
        std::env::var("TWITCH_PASS").unwrap(),
    );

    // putting this in the env so people don't join my channel when running this
    let channel = std::env::var("TWITCH_CHANNEL").unwrap();

    // connect via tls over tcp with this nick and password
    let (read, write) = twitchchat::connect_easy(&nick, &pass, twitchchat::Secure::Nope)
        .await
        .unwrap();

    // make a client. the client is clonable
    let client = twitchchat::Client::new();

    // get a future that resolves when the client is done reading, fails to read/write or is stopped
    let done = client.run(read, write);

    // get an event dispatcher
    let mut dispatcher = client.dispatcher().await;

    // subscribe to an event stream

    // for privmsg (what users send to channels)
    let mut privmsg = dispatcher.subscribe::<twitchchat::events::Privmsg>();
    // spawn a task to consume the stream
    tokio::task::spawn(async move {
        while let Some(msg) = privmsg.next().await {
            eprintln!("[{}] {}: {}", msg.channel, msg.user, msg.data);
        }
    });

    // for join (when a user joins a channel)
    let mut join = dispatcher.subscribe::<twitchchat::events::Join>();
    tokio::task::spawn(async move {
        while let Some(msg) = join.next().await {
            // we've joined a channel
            if msg.user == nick {
                eprintln!("you joined {}", msg.channel);
                break; // returning/dropping the stream un-subscribes it
            }
        }
    });

    // for privmsg again
    let mut bot = dispatcher.subscribe::<twitchchat::events::Privmsg>();
    // we can move the client to another task by cloning it
    let bot_client = client.clone();
    tokio::task::spawn(async move {
        let mut writer = bot_client.writer();
        while let Some(msg) = bot.next().await {
            match msg.data.split(" ").next() {
                Some("!quit") => {
                    // causes the client to shutdown
                    bot_client.stop().await.unwrap();
                }
                Some("!hello") => {
                    let response = format!("hello {}!", msg.user);
                    // send a message in response
                    let still_connected = writer.privmsg(&msg.channel, &response).await;
                    if !still_connected {
                        break;
                    }
                }
                _ => {}
            }
        }
    });

    // dispatcher has an RAII guard, so keep it scoped
    // dropping it here so everything can proceed while keeping example brief
    drop(dispatcher);

    // get a clonable writer from the client
    // join a channel, methods on writer return false if the client is connected
    if !client.writer().join(&channel).await {
        panic!("not connected!?")
    }

    // you can clear subscriptions with
    // client.dispatcher().await.clear_subscriptions::<event::Join>()
    // or all subscriptions
    // client.dispatcher().await.clear_subscriptions_all()

    // you can get the number of active subscriptions with
    // client.dispatcher().await.count_subscribers::<event::Join>()
    // or all subscriptions
    // client.dispatcher().await.count_subscribers_all()

    // await for the client to be done
    match done.await {
        Ok(twitchchat::client::Status::Eof) => {
            eprintln!("done!");
        }
        Ok(twitchchat::client::Status::Canceled) => {
            eprintln!("client was stopped by user");
        }
        Err(err) => {
            eprintln!("error: {}", err);
        }
    }

    // note you should wait for all of your tasks to join before exiting
    // but we detached them to make this shorter

    // another way would be to clear all subscriptions
    // clearing the subscriptions would close each event stream
    client.dispatcher().await.count_subscribers_all();
}
