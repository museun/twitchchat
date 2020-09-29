// NOTE: this demo requires `--feature async-io`.
use twitchchat::{commands, connector, runner::AsyncRunner, UserConfig};

// this is a helper module to reduce code deduplication
mod include;
use crate::include::{channels_to_join, get_user_config, main_loop};

async fn connect(user_config: &UserConfig, channels: &[String]) -> anyhow::Result<AsyncRunner> {
    // create a connector using ``async_io``, this connects to Twitch.
    // you can provide a different address with `custom`
    // this can fail if DNS resolution cannot happen
    let connector = connector::async_io::Connector::twitch()?;

    println!("we're connecting!");
    // create a new runner. this is a provided async 'main loop'
    // this method will block until you're ready
    let mut runner = AsyncRunner::connect(connector, user_config).await?;
    println!("..and we're connected");

    // and the identity Twitch gave you
    println!("our identity: {:#?}", runner.identity);

    for channel in channels {
        // the runner itself has 'blocking' join/part to ensure you join/leave a channel.
        // these two methods return whether the connection was closed early.
        // we'll ignore it for this demo
        println!("attempting to join '{}'", channel);
        let _ = runner.join(channel).await?;
        println!("joined '{}'!", channel);
    }

    Ok(runner)
}

fn main() -> anyhow::Result<()> {
    // create a user configuration
    let user_config = get_user_config()?;
    // get some channels to join from the environment
    let channels = channels_to_join()?;

    // any executor would work, we'll use async_executor so can spawn tasks
    let executor = async_executor::Executor::new();
    futures_lite::future::block_on(executor.run(async {
        // connect and join the provided channels
        let runner = connect(&user_config, &channels).await?;

        // you can get a handle to shutdown the runner
        let quit_handle = runner.quit_handle();

        // you can get a clonable writer
        let mut writer = runner.writer();

        // spawn something off in the background that'll exit in 10 seconds
        executor
            .spawn({
                let mut writer = writer.clone();
                let channels = channels.clone();
                async move {
                    println!("in 10 seconds we'll exit");
                    async_io::Timer::after(std::time::Duration::from_secs(10)).await;

                    // send one final message to all channels
                    for channel in channels {
                        let cmd = commands::privmsg(&channel, "goodbye, world");
                        writer.encode(cmd).await.unwrap();
                    }

                    println!("sending quit signal");
                    quit_handle.notify().await;
                }
            })
            .detach();

        // you can encode all sorts of 'commands'
        for channel in &channels {
            writer
                .encode(commands::privmsg(channel, "hello world!"))
                .await?;
        }

        println!("starting main loop");
        // your 'main loop'. you'll just call next_message() until you're done
        main_loop(runner).await
    }))
}
