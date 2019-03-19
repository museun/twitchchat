use std::io::Write;
use std::sync::Arc;

use super::{Channel, Error, MutexWrapper as Mutex, TwitchColor};

/// A thread-safe, clonable writer for the Twitch client
pub struct Writer<W>(pub Arc<Mutex<W>>);

impl<W: Write> Clone for Writer<W> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

// TODO decide on AsRef or just &str
#[allow(missing_docs)] // while we work things out
impl<W: Write> Writer<W> {
    pub(crate) fn write_line<S: AsRef<[u8]>>(&self, data: S) -> Result<(), Error> {
        let mut write = self.0.lock();
        write
            .write_all(data.as_ref())
            .and_then(|_| write.write_all(b"\r\n"))
            .and_then(|_| write.flush())
            .map_err(Error::Write)
    }

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
    pub fn host(&self, channel: &str) -> Result<(), Error> {
        self.command(&format!("/host {}", channel))
    }

    // Usage: "/unhost" - Stop hosting another channel.
    pub fn unhost(&self) -> Result<(), Error> {
        self.command("/unhost")
    }

    // Usage: "/marker" - Adds a stream marker (with an optional comment, max
    // 140 characters) at the current timestamp. You can use markers in the
    // Highlighter for easier editing.
    pub fn marker(&self, comment: Option<&str>) -> Result<(), Error> {
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
    pub fn raid(&self, channel: &str) -> Result<(), Error> {
        self.command(&format!("/raid {}", channel))
    }

    // Usage: "/unraid" - Cancel the Raid.
    pub fn unraid(&self) -> Result<(), Error> {
        self.command("/unraid")
    }

    // Usage: "/color <color>" - Change your username color. Color must be in
    // hex (#000000) or one of the following: Blue, BlueViolet, CadetBlue,
    // Chocolate, Coral, DodgerBlue, Firebrick, GoldenRod, Green, HotPink,
    // OrangeRed, Red, SeaGreen, SpringGreen, YellowGreen.
    pub fn color<C: Into<TwitchColor>>(&self, color: C) -> Result<(), Error> {
        self.command(&format!("/color {}", color.into()))
    }

    // Usage: "/disconnect" - Reconnects to chat.
    pub fn disconnect(&self) -> Result<(), Error> {
        self.command("/disconnect")
    }

    // Usage: "/help" - Lists the commands available to you in this room.
    pub fn help(&self) -> Result<(), Error> {
        self.command("/help")
    }

    // Usage: "/mods" - Lists the moderators of this channel.
    pub fn mods(&self) -> Result<(), Error> {
        self.command("/mods")
    }

    // Usage: "/vips" - Lists the VIPs of this channel.
    pub fn vips(&self) -> Result<(), Error> {
        self.command("/vips")
    }

    // Usage: "/commercial [length]" - Triggers a commercial. Length (optional)
    // must be a positive number of seconds.
    pub fn commercial(&self, length: Option<usize>) -> Result<(), Error> {
        match length {
            Some(n) => self.command(&format!("/commercial {}", n)),
            None => self.command("/commercial"),
        }
    }

    // Usage: "/ban <username> [reason]" - Permanently prevent a user from
    // chatting. Reason is optional and will be shown to the target user and
    // other moderators. Use "unban" to remove a ban.
    pub fn ban(&self, username: &str, reason: Option<&str>) -> Result<(), Error> {
        match reason {
            Some(reason) => self.command(&format!("/ban {} {}", username, reason)),
            None => self.command(&format!("/ban {}", username)),
        }
    }

    // Usage: "/unban <username>" - Removes a ban on a user.
    pub fn unban(&self, username: &str) -> Result<(), Error> {
        self.command(&format!("/unban {}", username))
    }

    // Usage: "/clear" - Clear chat history for all users in this room.
    pub fn clear(&self) -> Result<(), Error> {
        self.command("/clear")
    }

    // ???
    // pub fn delete(&self) -> Result<(), Error> {
    //     unimplemented!()
    // }

    // Usage: "/emoteonly" - Enables emote-only mode (only emoticons may be used
    // in chat). Use "emoteonlyoff" to disable.
    pub fn emoteonly(&self) -> Result<(), Error> {
        self.command("/emoteonly")
    }

    // Usage: "/emoteonlyoff" - Disables emote-only mode.
    pub fn emoteonlyoff(&self) -> Result<(), Error> {
        self.command("/emoteonlyoff")
    }

    // Usage: "/followers [duration]" - Enables followers-only mode (only users
    // who have followed for 'duration' may chat). Examples: "30m", "1 week", "5
    // days 12 hours". Must be less than 3 months.
    pub fn followers(&self, duration: &str) -> Result<(), Error> {
        // TODO use https://docs.rs/chrono/0.4.6/chrono/#duration
        // and verify its < 3 months
        self.command(&format!("/followers {}", duration))
    }

    // Usage: "/followersoff - Disables followers-only mode.
    pub fn followersoff(&self) -> Result<(), Error> {
        self.command("/followersoff")
    }

    // Usage: "/mod <username>" - Grant moderator status to a user. Use "mods"
    // to list the moderators of this channel.
    // (NOTE: renamed to 'op' because r#mod is annoying to type)
    pub fn op(&self, username: &str) -> Result<(), Error> {
        self.command(&format!("/mod {}", username))
    }

    // Usage: "/unmod <username>" - Revoke moderator status from a user. Use
    // "mods" to list the moderators of this channel.
    pub fn unmod(&self, username: &str) -> Result<(), Error> {
        self.command(&format!("/unmod {}", username))
    }

    // Usage: "/r9kbeta" - Enables r9k mode. Use "r9kbetaoff" to disable.
    pub fn r9kbeta(&self) -> Result<(), Error> {
        self.command("/r9kbeta")
    }

    // Usage: "/r9kbetaoff" - Disables r9k mode.
    pub fn r9kbetaoff(&self) -> Result<(), Error> {
        self.command("/r9kbetaoff")
    }

    // Usage: "/slow [duration]" - Enables slow mode (limit how often users may
    // send messages). Duration (optional, default=120) must be a positive
    // number of seconds. Use "slowoff" to disable.
    pub fn slow(&self, duration: Option<usize>) -> Result<(), Error> {
        // TODO use https://docs.rs/chrono/0.4.6/chrono/#duration
        match duration {
            Some(dur) => self.command(&format!("/slow {}", dur)),
            None => self.command("/slow"),
        }
    }

    // Usage: "/slowoff" - Disables slow mode.
    pub fn slowoff(&self) -> Result<(), Error> {
        self.command("/slowoff")
    }

    // Usage: "/subscribers" - Enables subscribers-only mode (only subscribers
    // may chat in this channel). Use "subscribersoff" to disable.
    pub fn subscribers(&self) -> Result<(), Error> {
        self.command("/subscribers")
    }

    // Usage: "/subscribersoff" - Disables subscribers-only mode.
    pub fn subscribersoff(&self) -> Result<(), Error> {
        self.command("/subscribersoff")
    }

    // Usage: "/timeout <username> [duration][time unit] [reason]" - Temporarily
    // prevent a user from chatting. Duration (optional, default=10 minutes)
    // must be a positive integer; time unit (optional, default=s) must be one
    // of s, m, h, d, w; maximum duration is 2 weeks. Combinations like 1d2h are
    // also allowed. Reason is optional and will be shown to the target user and
    // other moderators. Use "untimeout" to remove a timeout.
    pub fn timeout(
        &self,
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
    pub fn untimeout(&self, username: &str) -> Result<(), Error> {
        self.command(&format!("/untimeout {}", username))
    }

    // Usage: "/vip <username>" - Grant VIP status to a user. Use "vips" to list
    // the VIPs of this channel.
    pub fn vip(&self, username: &str) -> Result<(), Error> {
        self.command(&format!("/vip {}", username))
    }

    // Usage: "/unvip <username>" - Revoke VIP status from a user. Use "vips" to
    // list the VIPs of this channel.
    pub fn unvip(&self, username: &str) -> Result<(), Error> {
        self.command(&format!("/unvip {}", username))
    }

    // Usage: "/w <username> <message>" - Whispers the message to the username.
    pub fn whisper(&self, username: &str, message: &str) -> Result<(), Error> {
        self.command(&format!("/w {} {}", username, message))
    }

    /// Joins a `channel`
    ///
    /// This ensures the channel name is lowercased and begins with a '#'.
    ///
    /// The following are equivilant
    /// ```no_run
    /// # use twitchchat::{helpers::TestStream, Client};
    /// # let mut stream = TestStream::new();
    /// # let (r, w) = (stream.clone(), stream.clone());
    /// # let mut client = Client::new(r, w);
    /// let w = client.writer();
    /// w.join("museun").unwrap();
    /// w.join("#museun").unwrap();
    /// w.join("Museun").unwrap();
    /// w.join("#MUSEUN").unwrap();
    /// ```    
    pub fn join<C: Into<Channel>>(&self, channel: C) -> Result<(), Error> {
        let channel = Channel::validate(channel)?;
        self.raw(&format!("JOIN {}", *channel))
    }

    /// Parts a `channel`
    ///
    /// This ensures the channel name is lowercased and begins with a '#'.
    ///
    /// The following are equivilant
    /// ```no_run
    /// # use twitchchat::{helpers::TestStream, Client};
    /// # let mut stream = TestStream::new();
    /// # let (r, w) = (stream.clone(), stream.clone());
    /// # let mut client = Client::new(r, w);
    /// let w = client.writer();
    /// w.part("museun").unwrap();
    /// w.part("#museun").unwrap();
    /// w.part("Museun").unwrap();
    /// w.part("#MUSEUN").unwrap();
    /// ```    
    pub fn part<C: Into<Channel>>(&self, channel: C) -> Result<(), Error> {
        let channel = Channel::validate(channel)?;
        self.raw(&format!("PART {}", *channel))
    }

    /// Sends an "emote" `message` in the third person to the `channel`
    ///
    /// This ensures the channel name is lowercased and begins with a '#'.
    pub fn me<C, S>(&self, channel: C, message: S) -> Result<(), Error>
    where
        C: Into<Channel>,
        S: AsRef<str>,
    {
        self.send(channel, &format!("/me {}", message.as_ref()))
    }

    /// Sends the `message` to the `channel`
    ///
    /// This ensures the channel name is lowercased and begins with a '#'.
    ///
    /// Same as [`send`](./struct.Client.html#method.send)
    pub fn privmsg<C, S>(&self, channel: C, message: S) -> Result<(), Error>
    where
        C: Into<Channel>,
        S: AsRef<str>,
    {
        let channel = Channel::validate(channel)?;
        self.raw(&format!("PRIVMSG {} :{}", *channel, message.as_ref()))
    }

    /// Sends the `message` to the `channel`
    ///
    /// This ensures the channel name is lowercased and begins with a '#'.
    ///
    /// Same as [`privmsg`](./struct.Client.html#method.privmsg)
    pub fn send<C, S>(&self, channel: C, message: S) -> Result<(), Error>
    where
        C: Into<Channel>,
        S: AsRef<str>,
    {
        self.privmsg(channel, message)
    }

    /// Sends the command: `data` (e.g. `/color #FFFFFF`)
    pub fn command<S>(&self, data: S) -> Result<(), Error>
    where
        S: AsRef<str>,
    {
        self.raw(&format!("PRIVMSG jtv :{}", data.as_ref()))
    }

    /// Sends a raw line (appends the required `\r\n`)
    pub fn raw<S>(&self, data: S) -> Result<(), Error>
    where
        S: AsRef<str>,
    {
        self.write_line(data.as_ref())
    }
}
