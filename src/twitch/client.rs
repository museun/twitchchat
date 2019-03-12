use hashbrown::HashMap;
use parking_lot::{Mutex, RwLock};

use std::io::{BufRead, BufReader, Read, Write};
use std::sync::Arc;

use super::{commands, Capability, Color, Error, LocalUser, Message};
use crate::irc::types::Message as IrcMessage;
use crate::UserConfig;

type FilterMap = HashMap<super::dumb::Filter, Box<Fn(Message) + Send + Sync>>;

/// Client is the IRC client for interacting with Twitch's chat.
// TODO write usage
#[derive(Clone)]
pub struct Client<R, W> {
    read: Arc<Mutex<BufReader<R>>>,
    write: Arc<Mutex<W>>,
    filters: Arc<RwLock<FilterMap>>,
}

impl<R, W> Client<R, W>
where
    R: Read,
    W: Write,
{
    /// Create a new Client from a [Read](std::io::Read), [Write](std::io::Write) pair
    pub fn new(read: R, write: W) -> Self {
        Self {
            read: Arc::new(Mutex::new(BufReader::new(read))),
            write: Arc::new(Mutex::new(write)),
            filters: Arc::new(RwLock::new(FilterMap::new())),
        }
    }

    /// Runs, consuming all messages. (pumping them through .on() filters)
    pub fn run(mut self) -> Result<(), Error> {
        loop {
            let _ = self.read_message()?;
        }
    }

    pub fn register(&mut self, config: UserConfig) -> Result<(), Error> {
        for cap in config.caps.into_iter().filter_map(Capability::get_command) {
            self.write_line(cap)?;
        }

        self.write_line(&format!("PASS {}", config.token))?;
        self.write_line(&format!("NICK {}", config.nick))
    }

    /// Waits for the `GLOBALUSERSTATE` before continuing, discarding any messages received
    ///
    /// Returns some useful information about your user
    pub fn wait_for_ready(&mut self) -> Result<LocalUser, Error> {
        loop {
            match self.read_message()? {
                Message::GlobalUserState(state) => {
                    return Ok(LocalUser {
                        user_id: state.user_id(),
                        display_name: state.display_name().map(ToString::to_string),
                        color: state.color(),
                        badges: state.badges(),
                        emote_sets: state.emote_sets(),
                    });
                }
                _ => continue,
            }
        }
    }

    /// Reads a message
    ///
    /// This can be an `IRC Message`, or a parsed `Twitch Command`
    /// Will automatically handle some ~tedious~ messages, like the /heartbeat/
    pub fn read_message(&mut self) -> Result<Message, Error> {
        // TODO provide an internal buffer to prevent this dumb allocation
        // using https://docs.rs/bytes/0.4.11/bytes/
        let mut buf = String::new();
        {
            let mut read = self.read.lock();
            read.read_line(&mut buf).unwrap(); //.map_err(Error::Read)?;
        }
        let buf = buf.trim_end();

        let msg = IrcMessage::parse(&buf) //
            .ok_or_else(|| Error::InvalidMessage(buf.to_string()))?;

        // handle PINGs automatically
        if let IrcMessage::Ping { token } = &msg {
            self.write_line(&format!("PONG :{}n", token))?;
        }

        // sanity check, doing it here instead of after its been re-parsed to fail early
        if let IrcMessage::Unknown {
            prefix,
            head,
            args,
            tail,
            ..
        } = &msg
        {
            if let (Some(crate::irc::types::Prefix::Server { host }), Some(data)) = (prefix, tail) {
                if head == "NOTICE"
                    && host == "tmi.twitch.tv"
                    && data == "Improperly formatted auth"
                    // excellent
                    && args.get(0) == Some(&"*".into())
                {
                    return Err(Error::InvalidRegistration);
                }
            }
        }

        let msg = commands::parse(&msg).unwrap_or_else(|| Message::Irc(msg));
        {
            let filter_map = &*self.filters.read();
            let key = msg.what_filter();
            if let Some(filter) = filter_map.get(&key) {
                (filter)(msg.clone()) // when in doubt
            }
        }

        Ok(msg)
    }
}

impl<R, W> Client<R, W> {
    /// When a message, matching the type of the closure, is received run this function with it.
    pub fn on<F, T>(&mut self, f: F) -> bool
    where
        F: Fn(T) + 'static + Send + Sync, // hmm
        T: From<Message>,
        T: super::dumb::MessageFilter,
    {
        let filter = T::to_filter();

        self.filters
            .write()
            .insert(filter, Box::new(move |msg| f(msg.into())))
            .is_none()
    }
}

impl<R, W> Client<R, W>
where
    W: Write,
{
    pub(crate) fn write_line(&mut self, data: &str) -> Result<(), Error> {
        let mut write = self.write.lock();
        write
            .write_all(data.as_bytes())
            .and_then(|_| write.write_all(b"\r\n"))
            .and_then(|_| write.flush())
            .map_err(Error::Write)
    }
}

// TODO decide on AsRef or just &str
impl<R, W> Client<R, W>
where
    W: Write,
{
    // TODO: https://dev.twitch.tv/docs/irc/guide/#scopes-for-irc-commands

    // /host,        /unhost 	       channel_editor
    // /marker	                       channel_editor
    // /raid,        /unraid 	       channel_editor
    // /color	                       chat:edit
    // /disconnect	                   chat:edit
    // /help	                       chat:edit
    // /me	                           chat:edit
    // /mods	                       chat:edit
    // /vips	                       chat:edit
    // /commercial	                   channel_commercial
    // /ban,         /unban            channel:moderate
    // /clear	                       channel:moderate
    // /delete	                       channel:moderate
    // /emoteonly,   /emoteonlyoff     channel:moderate
    // /followers,   /followersoff 	   channel:moderate
    // /mod,         /unmod 	       channel:moderate
    // /r9kbeta,     /r9kbetaoff 	   channel:moderate
    // /slow,        /slowoff 	       channel:moderate
    // /subscribers, /subscribersoff   channel:moderate
    // /timeout,     /untimeout 	   channel:moderate
    // /vip,         /unvip	           channel:moderate
    // /w	                           whispers:edit

    // TODO make this into a rust-doc format (e.g. contextual)
    // Usage: "/host <channel>" - Host another channel. Use "/unhost" to unset
    // host mode.
    pub fn host(&mut self, channel: &str) -> Result<(), Error> {
        self.command(&format!("/host {}", channel))
    }

    // Usage: "/unhost" - Stop hosting another channel.
    pub fn unhost(&mut self) -> Result<(), Error> {
        self.command("/unhost")
    }

    // Usage: "/marker" - Adds a stream marker (with an optional comment, max
    // 140 characters) at the current timestamp. You can use markers in the
    // Highlighter for easier editing.
    pub fn marker(&mut self, comment: Option<&str>) -> Result<(), Error> {
        match comment {
            Some(comment) => {
                // TODO use https://github.com/unicode-rs/unicode-width
                let cmd = if comment.len() <= 140 {
                    format!("/marker {}", comment)
                } else {
                    let comment = comment.chars().take(140).collect::<String>();
                    format!("/marker {}", comment)
                };
                self.command(&cmd)
            }
            _ => self.command("/marker"),
        }
    }

    // Usage: "/raid <channel>" - Raid another channel. Use "/unraid" to cancel
    // the Raid.
    pub fn raid(&mut self, channel: &str) -> Result<(), Error> {
        self.command(&format!("/raid {}", channel))
    }

    // Usage: "/unraid" - Cancel the Raid.
    pub fn unraid(&mut self) -> Result<(), Error> {
        self.command("/unraid")
    }

    // Usage: "/color <color>" - Change your username color. Color must be in
    // hex (#000000) or one of the following: Blue, BlueViolet, CadetBlue,
    // Chocolate, Coral, DodgerBlue, Firebrick, GoldenRod, Green, HotPink,
    // OrangeRed, Red, SeaGreen, SpringGreen, YellowGreen.
    pub fn color<C: Into<Color>>(&mut self, color: C) -> Result<(), Error> {
        self.command(&format!("/color {}", color.into()))
    }

    // Usage: "/disconnect" - Reconnects to chat.
    pub fn disconnect(&mut self) -> Result<(), Error> {
        self.command("/disconnect")
    }

    // Usage: "/help" - Lists the commands available to you in this room.
    pub fn help(&mut self) -> Result<(), Error> {
        self.command("/help")
    }

    // Usage: "/me <message>" - Send an "emote" message in the third person.
    pub fn me(&mut self, msg: &str) -> Result<(), Error> {
        self.command(&format!("/me {}", msg))
    }

    // Usage: "/mods" - Lists the moderators of this channel.
    pub fn mods(&mut self) -> Result<(), Error> {
        self.command("/mods")
    }

    // Usage: "/vips" - Lists the VIPs of this channel.
    pub fn vips(&mut self) -> Result<(), Error> {
        self.command("/vips")
    }

    // Usage: "/commercial [length]" - Triggers a commercial. Length (optional)
    // must be a positive number of seconds.
    pub fn commercial(&mut self, length: Option<usize>) -> Result<(), Error> {
        match length {
            Some(n) => self.command(&format!("/commercial {}", n)),
            None => self.command("/commercial"),
        }
    }

    // Usage: "/ban <username> [reason]" - Permanently prevent a user from
    // chatting. Reason is optional and will be shown to the target user and
    // other moderators. Use "unban" to remove a ban.
    pub fn ban(&mut self, username: &str, reason: Option<&str>) -> Result<(), Error> {
        match reason {
            Some(reason) => self.command(&format!("/ban {} {}", username, reason)),
            None => self.command(&format!("/ban {}", username)),
        }
    }

    // Usage: "/unban <username>" - Removes a ban on a user.
    pub fn unban(&mut self, username: &str) -> Result<(), Error> {
        self.command(&format!("/unban {}", username))
    }

    // Usage: "/clear" - Clear chat history for all users in this room.
    pub fn clear(&mut self) -> Result<(), Error> {
        self.command("/clear")
    }

    // ???
    // pub fn delete(&mut self) -> Result<(), Error> {
    //     unimplemented!()
    // }

    // Usage: "/emoteonly" - Enables emote-only mode (only emoticons may be used
    // in chat). Use "emoteonlyoff" to disable.
    pub fn emoteonly(&mut self) -> Result<(), Error> {
        self.command("/emoteonly")
    }

    // Usage: "/emoteonlyoff" - Disables emote-only mode.
    pub fn emoteonlyoff(&mut self) -> Result<(), Error> {
        self.command("/emoteonlyoff")
    }

    // Usage: "/followers [duration]" - Enables followers-only mode (only users
    // who have followed for 'duration' may chat). Examples: "30m", "1 week", "5
    // days 12 hours". Must be less than 3 months.
    pub fn followers(&mut self, duration: &str) -> Result<(), Error> {
        // TODO use https://docs.rs/chrono/0.4.6/chrono/#duration
        // and verify its < 3 months
        self.command(&format!("/followers {}", duration))
    }

    // Usage: "/followersoff - Disables followers-only mode.
    pub fn followersoff(&mut self) -> Result<(), Error> {
        self.command("/followersoff")
    }

    // Usage: "/mod <username>" - Grant moderator status to a user. Use "mods"
    // to list the moderators of this channel.
    // (NOTE: renamed to 'op' because r#mod is annoying to type)
    pub fn op(&mut self, username: &str) -> Result<(), Error> {
        self.command(&format!("/mod {}", username))
    }

    // Usage: "/unmod <username>" - Revoke moderator status from a user. Use
    // "mods" to list the moderators of this channel.
    pub fn unmod(&mut self, username: &str) -> Result<(), Error> {
        self.command(&format!("/unmod {}", username))
    }

    // Usage: "/r9kbeta" - Enables r9k mode. Use "r9kbetaoff" to disable.
    pub fn r9kbeta(&mut self) -> Result<(), Error> {
        self.command("/r9kbeta")
    }

    // Usage: "/r9kbetaoff" - Disables r9k mode.
    pub fn r9kbetaoff(&mut self) -> Result<(), Error> {
        self.command("/r9kbetaoff")
    }

    // Usage: "/slow [duration]" - Enables slow mode (limit how often users may
    // send messages). Duration (optional, default=120) must be a positive
    // number of seconds. Use "slowoff" to disable.
    pub fn slow(&mut self, duration: Option<usize>) -> Result<(), Error> {
        // TODO use https://docs.rs/chrono/0.4.6/chrono/#duration
        match duration {
            Some(dur) => self.command(&format!("/slow {}", dur)),
            None => self.command("/slow"),
        }
    }

    // Usage: "/slowoff" - Disables slow mode.
    pub fn slowoff(&mut self) -> Result<(), Error> {
        self.command("/slowoff")
    }

    // Usage: "/subscribers" - Enables subscribers-only mode (only subscribers
    // may chat in this channel). Use "subscribersoff" to disable.
    pub fn subscribers(&mut self) -> Result<(), Error> {
        self.command("/subscribers")
    }

    // Usage: "/subscribersoff" - Disables subscribers-only mode.
    pub fn subscribersoff(&mut self) -> Result<(), Error> {
        self.command("/subscribersoff")
    }

    // Usage: "/timeout <username> [duration][time unit] [reason]" - Temporarily
    // prevent a user from chatting. Duration (optional, default=10 minutes)
    // must be a positive integer; time unit (optional, default=s) must be one
    // of s, m, h, d, w; maximum duration is 2 weeks. Combinations like 1d2h are
    // also allowed. Reason is optional and will be shown to the target user and
    // other moderators. Use "untimeout" to remove a timeout.
    pub fn timeout(
        &mut self,
        username: &str,
        duration: Option<&str>,
        reason: Option<&str>,
    ) -> Result<(), Error> {
        // TODO use https://docs.rs/chrono/0.4.6/chrono/#duration
        // and verify the duration stuff
        let timeout = match (duration, reason) {
            (Some(dur), Some(reason)) => format!("/timeout {} {} {}", username, dur, reason),
            (None, Some(reason)) => format!("/timeout {} {}", username, reason),
            (Some(dur), None) => format!("/timeout {} {}", username, dur),
            (None, None) => format!("/timeout {}", username),
        };
        self.command(&timeout)
    }

    // Usage: "/untimeout <username>" - Removes a timeout on a user.
    pub fn untimeout(&mut self, username: &str) -> Result<(), Error> {
        self.command(&format!("/untimeout {}", username))
    }

    // Usage: "/vip <username>" - Grant VIP status to a user. Use "vips" to list
    // the VIPs of this channel.
    pub fn vip(&mut self, username: &str) -> Result<(), Error> {
        self.command(&format!("/vip {}", username))
    }

    // Usage: "/unvip <username>" - Revoke VIP status from a user. Use "vips" to
    // list the VIPs of this channel.
    pub fn unvip(&mut self, username: &str) -> Result<(), Error> {
        self.command(&format!("/unvip {}", username))
    }

    // Usage: "/w <username> <message>" - Whispers the message to the username.
    pub fn whisper(&mut self, username: &str, message: &str) -> Result<(), Error> {
        self.command(&format!("/w {} {}", username, message))
    }

    pub fn join(&mut self, channel: &str) -> Result<(), Error> {
        self.raw(&format!("JOIN {}", channel))
    }

    pub fn part(&mut self, channel: &str) -> Result<(), Error> {
        self.raw(&format!("PART {}", channel))
    }

    pub fn send(&mut self, channel: &str, msg: &str) -> Result<(), Error> {
        self.raw(&format!("PRIVMSG {} :{}", channel, msg))
    }

    /// Sends the command: `data` (e.g. `/color #FFFFFF`)
    pub fn command(&mut self, data: &str) -> Result<(), Error> {
        self.raw(&format!("PRIVMSG jtv :{}", data))
    }

    /// Sends a raw line (appends the required `\r\n`)
    pub fn raw(&mut self, data: &str) -> Result<(), Error> {
        self.write_line(data)
    }
}
