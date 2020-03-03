/* in your Cargo.toml
[dependencies]
twitchchat = "0.8"                               # this crate
tokio = { version = "0.2", features = ["full"] } # you need tokio to run it
*/

use twitchchat::{events, Client};

// so .next() can be used on the EventStream
// futures::stream::StreamExt will also work
use tokio::stream::StreamExt as _;

#[tokio::main]
async fn main() {
    let (nick, pass) = twitchchat::ANONYMOUS_LOGIN;
    let stream = twitchchat::connect_easy_tls(&nick, &pass).await.unwrap();
    // split the stream | TODO decide on R+W or R,W
    let (read, write) = tokio::io::split(stream);

    let client = Client::new();

    // client is clonable and can be sent across tasks
    let bot = client.clone();
    tokio::task::spawn(async move {
        // subscribe to 'PRIVMSG' events, this is a `Stream`
        let mut privmsgs = bot.dispatcher().await.subscribe::<events::Privmsg>();
        // 'msg' is a twitchchat::messages::Privmsg<'static> here
        while let Some(msg) = privmsgs.next().await {
            eprintln!("[{}] {}: {}", msg.channel, msg.name, msg.data);
        }
    });

    // run the client
    let done = client.run(read, write);

    // 'block' until we're connected
    let ready = client.wait_for_irc_ready().await.unwrap();
    eprintln!("your irc name: {}", ready.nickname);

    // the writer is also clonable
    client.writer().join("#museun").await.unwrap();

    // this resolves when the client disconnects
    // or is forced to stop with Client::stop
    use twitchchat::client::Status;
    match done.await {
        // client was disconnected by the server
        Ok(Status::Eof) => {}
        // client was canceled by the user (`stop`)
        Ok(Status::Canceled) => {}
        // an error was received when trying to read or write
        Err(err) => eprintln!("error!: {}", err),
    };
}
