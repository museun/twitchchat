/* in your Cargo.toml
[dependencies]
# this crate
twitchchat = "0.10"

# and now for tokio
# macros allows you to use `#[tokio::main]` and `tokio::pin!` and `tokio::select!`
# rt-multi gives you a multi-threaded runtime.
tokio = { version = "0.2", features = ["rt-multi", "macros"] }
*/

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
    // make a new dispatcher
    let dispatcher = twitchchat::Dispatcher::new();

    // give dispatcher to the Runner
    let (mut runner, mut control) = twitchchat::Runner::new(dispatcher.clone());

    // create a connector, this can be used to reconnect.
    let connector = twitchchat::Connector::new(|| async move {
        let (nick, pass) = (get_nick(), get_pass());
        twitchchat::native_tls::connect_easy(&nick, &pass).await
    });

    // this runs the client in a background task, giving a future you wait on
    //
    // you should call run before you 'block'
    let done = tokio::task::spawn(async move {
        // we have to wrap in this in an async block to force the move
        runner.run_to_completion(connector).await
    });

    // subscribe an Irc Ready event
    // GlobalUserState can also be used to 'wait' for ready
    // and then 'block' until we've received an IrcReady event
    let _ready = dispatcher
        .wait_for::<twitchchat::events::IrcReady>()
        .await
        .unwrap();

    // its safe to join channels after this point
    //
    // so lets get the channel from the environment
    let channel = get_channel();
    // join a channel
    control.writer().join(channel).await.unwrap();

    use twitchchat::Status;
    // wait for the run 'task' to resolve
    // this unwrap is incase the task panicked.
    match done.await.unwrap() {
        // if we received an EOF we're done
        Ok(Status::Eof) => eprintln!("done!"),
        // if the user canceled the connection
        Ok(Status::Canceled) => eprintln!("client was stopped by user"),
        // if the connection timed out
        Ok(Status::Timeout) => eprintln!("client connection timed out"),
        // if we received an error
        Err(err) => eprintln!("error: {}", err),
    }
}
