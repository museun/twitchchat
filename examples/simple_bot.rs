// note this uses `smol`. you can use `tokio` or `async_std` or `async_io` if you prefer.
use futures_lite::{AsyncRead, AsyncWrite, StreamExt as _};
use twitchchat::{
    messages::{Commands, Privmsg},
    runner::wait_until_ready,
    writer::MpscWriter,
    AsyncDecoder, AsyncEncoder, FromIrcMessage, PrivmsgExt as _, UserConfig,
};

// this is a helper module to reduce code deduplication
mod include;
use crate::include::{channels_to_join, get_user_config};

use std::collections::HashMap;

fn main() -> anyhow::Result<()> {
    // you'll need a user configuration
    let user_config = get_user_config()?;
    // and some channels to join
    let channels = channels_to_join()?;

    let start = std::time::Instant::now();

    let mut bot = Bot::default()
        .with_command("!hello", |args: Args| {
            let output = format!("hello {}!", args.msg.name());
            // We can 'reply' to this message using a writer + our output message
            args.writer.reply(args.msg, &output).unwrap();
        })
        .with_command("!uptime", move |args: Args| {
            let output = format!("its been running for {:.2?}", start.elapsed());
            // We can send a message back (without quoting the sender) using a writer + our output message
            args.writer.say(args.msg, &output).unwrap();
        })
        .with_command("!quit", move |args: Args| {
            // because we're using sync stuff, turn async into sync with smol!
            let writer = args.writer.clone();
            // (note there is an async version of this function, too)
            smol::block_on(async move { writer.shutdown_sync() });
        });

    // run the bot in the executor
    smol::block_on(async move { bot.run(&user_config, &channels).await })
}

struct Args<'a, 'b: 'a> {
    msg: &'a Privmsg<'b>,
    writer: &'a mut MpscWriter,
}

trait Command: Send + Sync {
    fn handle(&mut self, args: Args<'_, '_>);
}

impl<F> Command for F
where
    F: Fn(Args<'_, '_>),
    F: Send + Sync,
{
    fn handle(&mut self, args: Args<'_, '_>) {
        (self)(args)
    }
}

#[derive(Default)]
struct Bot {
    commands: HashMap<String, Box<dyn Command>>,
}

impl Bot {
    // add this command to the bot
    fn with_command(mut self, name: impl Into<String>, cmd: impl Command + 'static) -> Self {
        self.commands.insert(name.into(), Box::new(cmd));
        self
    }

    // run the bot until its done
    async fn run(&mut self, user_config: &UserConfig, channels: &[String]) -> anyhow::Result<()> {
        let mut stream = twitchchat_smol::connect_twitch().await?;

        let (identity, _missed_messages) = wait_until_ready(&mut stream, user_config).await?;
        println!("connecting, we are: {}", identity.username());

        let (decode, mut encode) = twitchchat::split::r#async::make_pair(stream);

        for channel in channels {
            println!("joining: {}", channel);
            encode.join(channel).await?
        }

        // if you store this somewhere, you can quit the bot gracefully
        // let quit = runner.quit_handle();

        println!("starting main loop");
        self.main_loop(decode, encode).await
    }

    // the main loop of the bot
    async fn main_loop<D, E>(
        &mut self,
        mut decoder: AsyncDecoder<D>,
        encoder: AsyncEncoder<E>,
    ) -> anyhow::Result<()>
    where
        D: AsyncRead + Send + Sync + Unpin + 'static,
        E: AsyncWrite + Send + Sync + Unpin + 'static,
    {
        // this is clonable, but we can just share it via &mut
        let mut writer = MpscWriter::from_async_encoder(encoder);

        while let Some(Ok(msg)) = decoder.next().await {
            let msg = Commands::from_irc(msg)
                .expect("this can only fail if you're parsing custom messages");

            // if we get a Privmsg (you'll get an Commands enum for all messages received)
            if let Commands::Privmsg(pm) = msg {
                // see if its a command and do stuff with it
                if let Some(cmd) = Self::parse_command(pm.data()) {
                    if let Some(command) = self.commands.get_mut(cmd) {
                        println!("dispatching to: {}", cmd.escape_debug());
                        let args = Args {
                            msg: &pm,
                            writer: &mut writer,
                        };
                        command.handle(args);
                    }
                }
            }
        }

        println!("end of main loop");
        Ok(())
    }

    fn parse_command(input: &str) -> Option<&str> {
        if !input.starts_with('!') {
            return None;
        }
        input.splitn(2, ' ').next()
    }
}
