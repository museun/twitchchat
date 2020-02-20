/* in your Cargo.toml
[dependencies]
twitchchat = "0.8.3"                             # this crate
tokio = { version = "0.2", features = ["full"] } # you need tokio to run it
*/

use tokio::stream::StreamExt as _; // for .next()

#[tokio::main]
async fn main() {
    let (nick, pass, channel) = (
        std::env::var("TWITCH_NICK").unwrap(),
        std::env::var("TWITCH_PASS").unwrap(),
        std::env::var("TWITCH_CHANNEL").unwrap(),
    );

    let tls = twitchchat::Secure::UseTls;
    let (read, write) = twitchchat::connect_easy(&nick, &pass, tls).await.unwrap();

    let client = twitchchat::Client::new();
    // this runs the client in a background task, giving a future you wait on
    // you should call run before you 'block'
    let done = client.run(read, write);

    // subscribe an Irc Ready event
    // GlobalUserState can also be used to 'wait' for ready
    let mut ready = client
        .dispatcher()
        .await
        .subscribe::<twitchchat::events::IrcReady>();

    // 'block' until we've received an IrcReady event
    let _ready = ready.next().await.unwrap();
    // its safe to join channels after this point

    // join a channel
    client.writer().join(channel).await.unwrap();

    use twitchchat::client::Status;
    match done.await {
        Ok(Status::Eof) => eprintln!("done!"),
        Ok(Status::Canceled) => eprintln!("client was stopped by user"),
        Err(err) => eprintln!("error: {}", err),
    }
}
