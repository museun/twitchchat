use std::fmt::Display;
use std::sync::Arc;

use std::time::Duration;

use super::{Color, Error, IntoChannel};
use crossbeam_channel as channel;
use rate_limit::SyncLimiter;

// TODO more accurate rate limit:
// 20 per 30 seconds	Users sending commands or messages to channels in which they do not have Moderator or Operator status
// 100 per 30 seconds	Users sending commands or messages to channels in which they have Moderator or Operator status

/// A thread-safe, clonable writer for the Twitch client
#[derive(Clone)]
pub struct Writer {
    tx: channel::Sender<String>,
    rate: Arc<SyncLimiter>,
}

impl Writer {
    pub(crate) fn new() -> (Self, channel::Receiver<String>) {
        let rate = Arc::new(SyncLimiter::full(50, Duration::from_secs(15)));
        let (tx, rx) = channel::bounded(32);
        (Self { tx, rate }, rx)
    }

    pub(crate) fn write_line(&self, data: impl Display) -> Result<(), Error> {
        match self.tx.try_send(format!("{}\r\n", data)) {
            Ok(..) => {
                let _ = self.rate.take();
                Ok(())
            }
            Err(channel::TrySendError::Disconnected(..)) => Err(Error::NotConnected),
            Err(channel::TrySendError::Full(..)) => {
                unreachable!("channel shouldn't be buffered, or remotely full")
            }
        }
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

    /// Host another channel.
    ///
    /// Use [`Writer::unhost`](./struct.Writer.html#method.unhost) to unset host mode.
    pub fn host(&self, channel: impl IntoChannel) -> Result<(), Error> {
        let channel = channel.into_channel()?;
        self.command(format!("/host {}", *channel))
    }

    /// Stop hosting another channel.
    pub fn unhost(&self) -> Result<(), Error> {
        self.command("/unhost")
    }

    /// Adds a stream marker (with an optional comment, max 140 characters) at the current timestamp.
    ///
    /// You can use markers in the Highlighter for easier editing.
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
                self.command(cmd)
            }
            _ => self.command("/marker"),
        }
    }

    /// Raid another channel.
    ///
    /// Use [`Writer::unraid`](./struct.Writer.html#method.unraid) to cancel the Raid.
    pub fn raid(&self, channel: impl IntoChannel) -> Result<(), Error> {
        let channel = channel.into_channel()?;
        self.command(format!("/raid {}", *channel))
    }

    /// Cancel the Raid.
    pub fn unraid(&self) -> Result<(), Error> {
        self.command("/unraid")
    }

    /// Change your username color.
    pub fn color(&self, color: Color) -> Result<(), Error> {
        self.command(format!("/color {}", color))
    }

    /// Reconnects to chat.
    pub fn disconnect(&self) -> Result<(), Error> {
        self.command("/disconnect")
    }

    /// Lists the commands available to you in this room.
    pub fn help(&self) -> Result<(), Error> {
        self.command("/help")
    }

    /// Lists the moderators of this channel.
    pub fn mods(&self) -> Result<(), Error> {
        self.command("/mods")
    }

    /// Lists the VIPs of this channel.
    pub fn vips(&self) -> Result<(), Error> {
        self.command("/vips")
    }

    /// Triggers a commercial.
    ///
    /// Length (optional) must be a positive number of seconds.
    pub fn commercial(&self, length: Option<usize>) -> Result<(), Error> {
        match length {
            Some(n) => self.command(format!("/commercial {}", n)),
            None => self.command("/commercial"),
        }
    }

    /// Permanently prevent a user from chatting.
    /// Reason is optional and will be shown to the target user and other moderators.
    ///
    /// Use [`Writer::unban`](./struct.Writer.html#method.unban) to remove a ban.
    pub fn ban(&self, username: impl Display, reason: Option<&str>) -> Result<(), Error> {
        match reason {
            Some(reason) => self.command(format!("/ban {} {}", username, reason)),
            None => self.command(format!("/ban {}", username)),
        }
    }

    /// Removes a ban on a user.
    pub fn unban(&self, username: impl Display) -> Result<(), Error> {
        self.command(format!("/unban {}", username))
    }

    /// Clear chat history for all users in this room.
    pub fn clear(&self) -> Result<(), Error> {
        self.command("/clear")
    }

    // ???
    // pub fn delete(&self) -> Result<(), Error> {
    //     unimplemented!()
    // }

    /// Enables emote-only mode (only emoticons may be used in chat).
    ///
    /// Use [`Writer::emoteonlyoff`](./struct.Writer.html#method.emoteonlyoff) to disable.
    pub fn emoteonly(&self) -> Result<(), Error> {
        self.command("/emoteonly")
    }

    /// Disables emote-only mode.
    pub fn emoteonlyoff(&self) -> Result<(), Error> {
        self.command("/emoteonlyoff")
    }

    /// Enables followers-only mode (only users who have followed for 'duration' may chat).
    ///
    /// Examples: "30m", "1 week", "5 days 12 hours".
    ///
    /// Must be less than 3 months.
    pub fn followers(&self, duration: impl Display) -> Result<(), Error> {
        // TODO parse the `duration` and verify its < 3 months
        // chrono doesn't have this parsing, but it'd not be difficult to write one
        // an extra error variant would have to be added for "invalid timespec" (timestamp?)
        self.command(&format!("/followers {}", duration))
    }

    /// Disables followers-only mode.
    pub fn followersoff(&self) -> Result<(), Error> {
        self.command("/followersoff")
    }

    /// Grant moderator status to a user.
    ///
    /// Use [`Writer::mods`](./struct.Writer.html#method.mods) to list the moderators of this channel.
    ///
    /// (**NOTE**: renamed to `op` because r#mod is annoying to type)
    pub fn op(&self, username: impl Display) -> Result<(), Error> {
        self.command(&format!("/mod {}", username))
    }

    /// Grant moderator status to a user. (See [`Writer::op`](./struct.Writer.html#method.op) for an alternative name)
    ///
    /// Use [`Writer::mods`](./struct.Writer.html#method.mods) to list the moderators of this channel.    
    pub fn r#mod(&self, username: impl Display) -> Result<(), Error> {
        self.command(&format!("/mod {}", username))
    }

    /// Revoke moderator status from a user.
    ///
    /// Use [`Writer::mods`](./struct.Writer.html#method.mods) to list the moderators of this channel.
    pub fn unmod(&self, username: impl Display) -> Result<(), Error> {
        self.command(&format!("/unmod {}", username))
    }

    /// Enables r9k mode.
    ///
    /// Use [`Writer::r9kbetaoff`](./struct.Writer.html#method.r9kbetaoff) to disable.
    pub fn r9kbeta(&self) -> Result<(), Error> {
        self.command("/r9kbeta")
    }

    /// Disables r9k mode.
    pub fn r9kbetaoff(&self) -> Result<(), Error> {
        self.command("/r9kbetaoff")
    }

    /// Enables slow mode (limit how often users may send messages).
    ///
    /// Duration (optional, default=120) must be a positive number of seconds.
    ///
    /// Use [`Writer::slowoff`](./struct.Writer.html#method.slowoff) to disable.
    pub fn slow(&self, duration: Option<usize>) -> Result<(), Error> {
        // TODO use https://docs.rs/chrono/0.4.6/chrono/#duration
        match duration {
            Some(dur) => self.command(format!("/slow {}", dur)),
            None => self.command("/slow 120"),
        }
    }

    /// Disables slow mode.
    pub fn slowoff(&self) -> Result<(), Error> {
        self.command("/slowoff")
    }

    /// Enables subscribers-only mode (only subscribers may chat in this channel).
    ///
    /// Use [`Writer::subscribersoff`](./struct.Writer.html#method.subscribersoff) to disable.
    pub fn subscribers(&self) -> Result<(), Error> {
        self.command("/subscribers")
    }

    /// Disables subscribers-only mode.
    pub fn subscribersoff(&self) -> Result<(), Error> {
        self.command("/subscribersoff")
    }

    /// Temporarily prevent a user from chatting.
    ///
    /// * duration (*optional*, default=`10 minutes`) must be a positive integer.
    /// * time unit (*optional*, default=`s`) must be one of
    ///   * s
    ///   * m
    ///   * h
    ///   * d
    ///   * w
    /// * maximum duration is `2 weeks`.
    ///
    /// Combinations like `1d2h` are also allowed.
    ///
    /// Reason is optional and will be shown to the target user and other moderators.
    ///
    /// Use [`Writer::untimeout`](./struct.Writer.html#method.untimeout) to remove a timeout.
    pub fn timeout(
        &self,
        username: impl Display,
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
        self.command(timeout)
    }

    /// Removes a timeout on a user.
    pub fn untimeout(&self, username: impl Display) -> Result<(), Error> {
        self.command(format!("/untimeout {}", username))
    }

    /// Grant VIP status to a user.
    ///
    /// Use [`Writer::vips`](./struct.Writer.html#method.vips) to list the VIPs of this channel.
    pub fn vip(&self, username: impl Display) -> Result<(), Error> {
        self.command(format!("/vip {}", username))
    }

    /// Revoke VIP status from a user.
    ///
    /// Use [`Writer::vips`](./struct.Writer.html#method.vips) to list the VIPs of this channel.
    pub fn unvip(&self, username: impl Display) -> Result<(), Error> {
        self.command(format!("/unvip {}", username))
    }

    /// Whispers the message to the username.
    pub fn whisper(&self, username: impl Display, message: impl Display) -> Result<(), Error> {
        self.command(format!("/w {} {}", username, message))
    }

    /// Joins a `channel`
    ///
    /// This ensures the channel name is lowercased and begins with a '#'.
    ///
    /// The following are equivilant
    /// ```ignore
    /// let w = client.writer();
    /// w.join("museun").unwrap();
    /// w.join("#museun").unwrap();
    /// w.join("Museun").unwrap();
    /// w.join("#MUSEUN").unwrap();
    /// ```    
    pub fn join(&self, channel: impl IntoChannel) -> Result<(), Error> {
        let channel = channel.into_channel()?;
        self.raw(format!("JOIN {}", *channel))
    }

    /// Join many channels (from an iterator)
    pub fn join_many<'a, I>(&self, channels: I) -> Result<(), Error>
    where
        I: IntoIterator + 'a,
        I::Item: AsRef<str> + 'a,
    {
        let mut buf = String::with_capacity(512);

        for channel in channels {
            let channel = channel.as_ref();
            let len = if channel.starts_with('#') {
                channel.len()
            } else {
                channel.len() + 1
            };

            if buf.len() + len + 1 > 510 {
                self.write_line(&buf)?;
                buf.clear();
            }

            if buf.is_empty() {
                buf.push_str("JOIN ");
            } else {
                buf.push(',');
            }

            if !channel.starts_with('#') {
                buf.push_str(&["#", channel].concat());
            } else {
                buf.push_str(&channel);
            }
        }

        if !buf.is_empty() {
            self.write_line(&buf)?;
        }

        Ok(())
    }

    /// Parts a `channel`
    ///
    /// This ensures the channel name is lowercased and begins with a '#'.
    ///
    /// The following are equivilant
    /// ```ignore
    /// let w = client.writer();
    /// w.part("museun").unwrap();
    /// w.part("#museun").unwrap();
    /// w.part("Museun").unwrap();
    /// w.part("#MUSEUN").unwrap();
    /// ```    
    pub fn part(&self, channel: impl IntoChannel) -> Result<(), Error> {
        let channel = channel.into_channel()?;
        self.raw(format!("PART {}", *channel))
    }

    /// Sends an "emote" `message` in the third person to the `channel`
    ///
    /// This ensures the channel name is lowercased and begins with a '#'.
    pub fn me(&self, channel: impl IntoChannel, message: impl Display) -> Result<(), Error> {
        self.send(channel, format!("/me {}", message))
    }

    /// Sends the `message` to the `channel`
    ///
    /// This ensures the channel name is lowercased and begins with a '#'.
    ///
    /// Same as [`send`](./struct.Client.html#method.send)
    pub fn privmsg(&self, channel: impl IntoChannel, message: impl Display) -> Result<(), Error> {
        let channel = channel.into_channel()?;
        self.raw(format!("PRIVMSG {} :{}", *channel, message))
    }

    /// Sends the `message` to the `channel`
    ///
    /// This ensures the channel name is lowercased and begins with a '#'.
    ///
    /// Same as [`privmsg`](./struct.Client.html#method.privmsg)
    pub fn send(&self, channel: impl IntoChannel, message: impl Display) -> Result<(), Error> {
        self.privmsg(channel, message)
    }

    /// Sends the command: `data` (e.g. `/color #FFFFFF`)
    pub fn command(&self, data: impl Display) -> Result<(), Error> {
        self.raw(format!("PRIVMSG jtv :{}", data))
    }

    /// Sends a raw line (appends the required `\r\n`)
    pub fn raw(&self, data: impl Display) -> Result<(), Error> {
        self.write_line(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_writer(
        f: impl FnOnce(&Writer) -> Result<(), Error>,
        expected: impl PartialEq<String> + std::fmt::Debug,
    ) {
        let (w, rx) = Writer::new();
        f(&w).unwrap();
        assert_eq!(expected, rx.recv().unwrap())
    }

    struct TestDisplay<'a>(&'a dyn Display);

    impl<'a> Display for TestDisplay<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    #[test]
    fn host() {
        let expected = "PRIVMSG jtv :/host #museun\r\n";
        test_writer(|w| w.host("museun"), expected);
        test_writer(|w| w.host("#museun"), expected);
        test_writer(|w| w.host(TestDisplay(&"museun").to_string()), expected);
    }

    #[test]
    fn unhost() {
        let expected = "PRIVMSG jtv :/unhost\r\n";
        test_writer(|w| w.unhost(), expected);
    }

    #[test]
    fn marker() {
        test_writer(|w| w.marker(None), "PRIVMSG jtv :/marker\r\n");
        let short = "a".repeat(140);
        test_writer(
            |w| w.marker(Some(&short)),
            format!("PRIVMSG jtv :/marker {}\r\n", short),
        );

        let long = "a".repeat(141);
        test_writer(
            |w| w.marker(Some(&long)),
            format!("PRIVMSG jtv :/marker {}\r\n", &long[..140]),
        );
    }

    #[test]
    fn raid() {
        let expected = "PRIVMSG jtv :/raid #museun\r\n";
        test_writer(|w| w.raid("museun"), expected);
        test_writer(|w| w.raid("#museun"), expected);
        test_writer(|w| w.raid(TestDisplay(&"museun").to_string()), expected);
    }

    #[test]
    fn unraid() {
        let expected = "PRIVMSG jtv :/unraid\r\n";
        test_writer(|w| w.unraid(), expected);
    }

    #[test]
    fn color() {
        use std::str::FromStr as _;
        let color = Color::from_str("blue").unwrap();
        let expected = format!("PRIVMSG jtv :/color {}\r\n", &color);
        test_writer(|w| w.color(color), expected);
    }

    #[test]
    fn disconnect() {
        let expected = "PRIVMSG jtv :/disconnect\r\n";
        test_writer(|w| w.disconnect(), expected);
    }

    #[test]
    fn help() {
        let expected = "PRIVMSG jtv :/help\r\n";
        test_writer(|w| w.help(), expected);
    }

    #[test]
    fn mods() {
        let expected = "PRIVMSG jtv :/mods\r\n";
        test_writer(|w| w.mods(), expected);
    }

    #[test]
    fn vips() {
        let expected = "PRIVMSG jtv :/vips\r\n";
        test_writer(|w| w.vips(), expected);
    }

    #[test]
    fn commercial() {
        let expected = "PRIVMSG jtv :/commercial\r\n";
        test_writer(|w| w.commercial(None), expected);

        let expected = "PRIVMSG jtv :/commercial 42\r\n";
        test_writer(|w| w.commercial(Some(42)), expected);
    }

    #[test]
    fn ban() {
        let expected = "PRIVMSG jtv :/ban test_user\r\n";
        test_writer(|w| w.ban("test_user", None), expected);

        let expected = "PRIVMSG jtv :/ban test_user spamming\r\n";
        test_writer(|w| w.ban("test_user", Some(&"spamming")), expected);

        let expected = "PRIVMSG jtv :/ban test_user\r\n";
        test_writer(|w| w.ban(TestDisplay(&"test_user"), None), expected);

        let expected = "PRIVMSG jtv :/ban test_user spamming\r\n";
        test_writer(
            |w| w.ban(TestDisplay(&"test_user"), Some(&"spamming")),
            expected,
        );
    }

    #[test]
    fn unban() {
        let expected = "PRIVMSG jtv :/unban test_user\r\n";
        test_writer(|w| w.unban("test_user"), expected);

        let expected = "PRIVMSG jtv :/unban test_user\r\n";
        test_writer(|w| w.unban(TestDisplay(&"test_user")), expected);
    }

    #[test]
    fn clear() {
        let expected = "PRIVMSG jtv :/clear\r\n";
        test_writer(|w| w.clear(), expected);
    }

    #[test]
    fn emoteonly() {
        let expected = "PRIVMSG jtv :/emoteonly\r\n";
        test_writer(|w| w.emoteonly(), expected);
    }

    #[test]
    fn emoteonlyoff() {
        let expected = "PRIVMSG jtv :/emoteonlyoff\r\n";
        test_writer(|w| w.emoteonlyoff(), expected);
    }

    #[test]
    fn followers() {
        let expected = "PRIVMSG jtv :/followers 1 week\r\n";
        test_writer(|w| w.followers("1 week"), expected);

        let expected = "PRIVMSG jtv :/followers 1 week\r\n";
        test_writer(|w| w.followers(TestDisplay(&"1 week")), expected);
    }

    #[test]
    fn followersoff() {
        let expected = "PRIVMSG jtv :/followersoff\r\n";
        test_writer(|w| w.followersoff(), expected);
    }

    #[test]
    fn op() {
        let expected = "PRIVMSG jtv :/mod museun\r\n";
        test_writer(|w| w.op("museun"), expected);
        test_writer(|w| w.op(TestDisplay(&"museun")), expected);
        test_writer(|w| w.r#mod("museun"), expected);
        test_writer(|w| w.r#mod(TestDisplay(&"museun")), expected);
    }

    #[test]
    fn unmod() {
        let expected = "PRIVMSG jtv :/unmod museun\r\n";
        test_writer(|w| w.unmod("museun"), expected);
        test_writer(|w| w.unmod(TestDisplay(&"museun")), expected);
    }

    #[test]
    fn r9kbeta() {
        let expected = "PRIVMSG jtv :/r9kbeta\r\n";
        test_writer(|w| w.r9kbeta(), expected);
    }

    #[test]
    fn r9kbetaoff() {
        let expected = "PRIVMSG jtv :/r9kbetaoff\r\n";
        test_writer(|w| w.r9kbetaoff(), expected);
    }

    #[test]
    fn slow() {
        let expected = "PRIVMSG jtv :/slow 120\r\n";
        test_writer(|w| w.slow(None), expected);
        let expected = "PRIVMSG jtv :/slow 240\r\n";
        test_writer(|w| w.slow(Some(240)), expected);
    }

    #[test]
    fn slowoff() {
        let expected = "PRIVMSG jtv :/slowoff\r\n";
        test_writer(|w| w.slowoff(), expected);
    }

    #[test]
    fn subscribers() {
        let expected = "PRIVMSG jtv :/subscribers\r\n";
        test_writer(|w| w.subscribers(), expected);
    }

    #[test]
    fn subscribersoff() {
        let expected = "PRIVMSG jtv :/subscribersoff\r\n";
        test_writer(|w| w.subscribersoff(), expected);
    }

    #[test]
    fn timeout() {
        let expected = "PRIVMSG jtv :/timeout museun\r\n";
        test_writer(|w| w.timeout("museun", None, None), expected);
        test_writer(|w| w.timeout(TestDisplay(&"museun"), None, None), expected);

        let expected = "PRIVMSG jtv :/timeout museun 1d2h\r\n";
        test_writer(|w| w.timeout("museun", Some("1d2h"), None), expected);
        test_writer(
            |w| w.timeout(TestDisplay(&"museun"), Some("1d2h"), None),
            expected,
        );

        let expected = "PRIVMSG jtv :/timeout museun spamming\r\n";
        test_writer(|w| w.timeout("museun", None, Some("spamming")), expected);
        test_writer(
            |w| w.timeout(TestDisplay(&"museun"), None, Some("spamming")),
            expected,
        );

        let expected = "PRIVMSG jtv :/timeout museun 1d2h spamming\r\n";
        test_writer(
            |w| w.timeout("museun", Some("1d2h"), Some("spamming")),
            expected,
        );
        test_writer(
            |w| w.timeout(TestDisplay(&"museun"), Some("1d2h"), Some("spamming")),
            expected,
        );
    }

    #[test]
    fn untimeout() {
        let expected = "PRIVMSG jtv :/untimeout museun\r\n";
        test_writer(|w| w.untimeout("museun"), expected);
        test_writer(|w| w.untimeout(TestDisplay(&"museun")), expected);
    }

    #[test]
    fn vip() {
        let expected = "PRIVMSG jtv :/vip museun\r\n";
        test_writer(|w| w.vip("museun"), expected);
        test_writer(|w| w.vip(TestDisplay(&"museun")), expected);
    }

    #[test]
    fn unvip() {
        let expected = "PRIVMSG jtv :/unvip museun\r\n";
        test_writer(|w| w.unvip("museun"), expected);
        test_writer(|w| w.unvip(TestDisplay(&"museun")), expected);
    }

    #[test]
    fn whisper() {
        let expected = "PRIVMSG jtv :/w museun foobar\r\n";
        test_writer(|w| w.whisper("museun", "foobar"), expected);
        test_writer(|w| w.whisper(TestDisplay(&"museun"), "foobar"), expected);
        test_writer(|w| w.whisper("museun", TestDisplay(&"foobar")), expected);
    }

    #[test]
    fn join() {
        let expected = "JOIN #museun\r\n";
        test_writer(|w| w.join("museun"), expected);
        test_writer(|w| w.join(TestDisplay(&"museun").to_string()), expected);
    }

    #[test]
    fn part() {
        let expected = "PART #museun\r\n";
        test_writer(|w| w.part("museun"), expected);
        test_writer(|w| w.part(TestDisplay(&"museun").to_string()), expected);
    }

    #[test]
    fn me() {
        let expected = "PRIVMSG #museun :/me testing\r\n";
        test_writer(|w| w.me("museun", "testing"), expected);
        test_writer(|w| w.me("#museun", "testing"), expected);
        test_writer(
            |w| w.me(TestDisplay(&"museun").to_string(), "testing"),
            expected,
        );
        test_writer(|w| w.me("#museun", TestDisplay(&"testing")), expected);
    }

    #[test]
    fn privmsg() {
        let expected = "PRIVMSG #museun :testing\r\n";
        test_writer(|w| w.privmsg("museun", "testing"), expected);
        test_writer(|w| w.privmsg("#museun", "testing"), expected);
        test_writer(
            |w| w.privmsg(TestDisplay(&"museun").to_string(), "testing"),
            expected,
        );
        test_writer(|w| w.privmsg("#museun", TestDisplay(&"testing")), expected);
    }

    #[test]
    fn send() {
        let expected = "PRIVMSG #museun :testing\r\n";
        test_writer(|w| w.send("museun", "testing"), expected);
        test_writer(|w| w.send("#museun", "testing"), expected);
        test_writer(
            |w| w.send(TestDisplay(&"museun").to_string(), "testing"),
            expected,
        );
        test_writer(|w| w.send("#museun", TestDisplay(&"testing")), expected);
    }

    #[test]
    fn command() {
        let expected = "PRIVMSG jtv :testing\r\n";
        test_writer(|w| w.command("testing"), expected);
        test_writer(|w| w.command(TestDisplay(&"testing")), expected);
    }

    #[test]
    fn raw() {
        let expected = "PRIVMSG museun :this is a test\r\n";
        test_writer(|w| w.raw(&expected[..expected.len() - 2]), expected);
    }

    // fn make_channel_list() -> impl Iterator<Item = String> {
    //     use rand::prelude::*;
    //     std::iter::from_fn(move || {
    //         let mut rng = thread_rng();
    //         let range = rng.gen_range(5, 30);
    //         Some(
    //             rng.sample_iter(&rand::distributions::Alphanumeric)
    //                 .take(range)
    //                 .collect(),
    //         )
    //     })
    // }

    // TODO join many test
}
