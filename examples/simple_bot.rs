#![cfg_attr(debug_assertions, allow(dead_code,))]
// TODO correct this
// note this uses `smol`. you can use `tokio` or `async_std` or `async_io` if you prefer.

use anyhow::Context as _;
use std::collections::HashMap;

use twitchchat::PrivmsgExt as _;
use twitchchat::{
    commands::{privmsg, reply},
    messages::{AllCommands, Privmsg},
    runner::{AsyncRunner, Status},
    writer::{AsyncWriter, MpscWriter},
    UserConfig,
};

fn main() -> anyhow::Result<()> {
    let user_config = get_user_config()?;
    let channels = channels_to_join()?;

    let start = std::time::Instant::now();

    let mut bot = Bot::default()
        .with_command("!hello", |msg: &Privmsg<'_>, resp: &mut Writer| {
            let output = format!("hello {}!", msg.name());
            msg.reply(resp, &output).unwrap();
        })
        .with_command("!uptime", move |msg: &Privmsg<'_>, resp: &mut Writer| {
            let output = format!("its been running for {:.2?}", start.elapsed());
            msg.say(resp, &output).unwrap();
        });

    smol::run(async move { bot.run(&user_config, &channels).await })
}

// to make things easier to type
type Writer = AsyncWriter<MpscWriter>;

trait Command: Send + Sync {
    fn handle(&mut self, msg: &Privmsg<'_>, response: &mut Writer);
}

impl<F> Command for F
where
    F: Fn(&Privmsg<'_>, &mut Writer),
    F: Send + Sync,
{
    fn handle(&mut self, msg: &Privmsg<'_>, response: &mut Writer) {
        (self)(msg, response)
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
        let connector = twitchchat::connector::smol::Connector::twitch();

        let mut runner = AsyncRunner::connect(connector, user_config).await?;
        println!("connecting, we are: {}", runner.identity.username());

        for channel in channels {
            println!("joining: {}", channel);
            runner.join(channel).await?;
        }

        // if you store this somewhere, you can quit the bot gracefully
        // let quit = runner.quit_handle();

        println!("starting main loop");
        self.main_loop(&mut runner).await
    }

    // the main loop of the bot
    async fn main_loop(&mut self, runner: &mut AsyncRunner) -> anyhow::Result<()> {
        let mut writer = runner.writer();
        loop {
            match runner.next_message().await? {
                Status::Message(AllCommands::Privmsg(pm)) => {
                    if let Some(cmd) = Self::parse_command(pm.data()) {
                        if let Some(command) = self.commands.get_mut(cmd) {
                            println!("dispatching to: {}", cmd.escape_debug());
                            command.handle(&pm, &mut writer);
                        }
                    }
                }
                Status::Quit | Status::Eof => break,
                Status::Message(msg) => continue,
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

// some helpers for the demo
fn get_env_var(key: &str) -> anyhow::Result<String> {
    std::env::var(key).with_context(|| format!("please set `{}`", key))
}

fn channels_to_join() -> anyhow::Result<Vec<String>> {
    let channels = get_env_var("TWITCH_CHANNEL")?
        .split(',')
        .map(ToString::to_string)
        .collect();
    Ok(channels)
}

fn get_user_config() -> anyhow::Result<twitchchat::UserConfig> {
    let name = get_env_var("TWITCH_NAME")?;
    let token = get_env_var("TWITCH_TOKEN")?;

    // you need a `UserConfig` to connect to Twitch
    let config = UserConfig::builder()
        // the name of the associated twitch account
        .name(name)
        // and the provided OAuth token
        .token(token)
        // and enable all of the advanced message signaling from Twitch
        .enable_all_capabilities()
        .build()?;

    Ok(config)
}
