use std::fmt::Display;

use super::{Channel, Color, Error};
use crossbeam_channel as channel;

/// A thread-safe, clonable writer for the Twitch client
#[derive(Clone)]
pub struct Writer(channel::Sender<String>);

impl Writer {
    pub(crate) fn new() -> (Self, channel::Receiver<String>) {
        let (tx, rx) = channel::unbounded();
        (Self(tx), rx)
    }

    pub(crate) fn write_line<S: Display>(&self, data: S) -> Result<(), Error> {
        match self.0.try_send(format!("{}\r\n", data)) {
            Ok(..) => Ok(()),
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
    pub fn host<C>(&self, channel: C) -> Result<(), Error>
    where
        C: Into<Channel>,
    {
        let channel = Channel::validate(channel)?;
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
    pub fn raid<C>(&self, channel: C) -> Result<(), Error>
    where
        C: Into<Channel>,
    {
        let channel = Channel::validate(channel)?;
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
    pub fn ban<S>(&self, username: S, reason: Option<&str>) -> Result<(), Error>
    where
        S: Display,
    {
        match reason {
            Some(reason) => self.command(format!("/ban {} {}", username, reason)),
            None => self.command(format!("/ban {}", username)),
        }
    }

    /// Removes a ban on a user.
    pub fn unban<S>(&self, username: S) -> Result<(), Error>
    where
        S: Display,
    {
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
    pub fn followers<S>(&self, duration: S) -> Result<(), Error>
    where
        S: Display,
    {
        // TODO use https://docs.rs/chrono/0.4.6/chrono/#duration
        // and verify its < 3 months
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
    pub fn op<S>(&self, username: S) -> Result<(), Error>
    where
        S: Display,
    {
        self.command(&format!("/mod {}", username))
    }

    /// Revoke moderator status from a user.
    ///
    /// Use [`Writer::mods`](./struct.Writer.html#method.mods) to list the moderators of this channel.
    pub fn unmod<S>(&self, username: S) -> Result<(), Error>
    where
        S: Display,
    {
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
            None => self.command("/slow"),
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
    pub fn timeout<S>(
        &self,
        username: S,
        duration: Option<&str>,
        reason: Option<&str>,
    ) -> Result<(), Error>
    where
        S: Display,
    {
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
    pub fn untimeout<S>(&self, username: S) -> Result<(), Error>
    where
        S: Display,
    {
        self.command(format!("/untimeout {}", username))
    }

    /// Grant VIP status to a user.
    ///
    /// Use [`Writer::vips`](./struct.Writer.html#method.vips) to list the VIPs of this channel.
    pub fn vip<S>(&self, username: S) -> Result<(), Error>
    where
        S: Display,
    {
        self.command(format!("/vip {}", username))
    }

    /// Revoke VIP status from a user.
    ///
    /// Use [`Writer::vips`](./struct.Writer.html#method.vips) to list the VIPs of this channel.
    pub fn unvip<S>(&self, username: S) -> Result<(), Error>
    where
        S: Display,
    {
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
    /// ```no_run
    /// # use twitchchat::{helpers::TestStream, *};
    /// # let mut stream = TestStream::new();
    /// # let (r, w) = sync_adapters(stream.clone(), stream.clone());
    /// # let mut client = Client::new(r, w);
    /// let w = client.writer();
    /// w.join("museun").unwrap();
    /// w.join("#museun").unwrap();
    /// w.join("Museun").unwrap();
    /// w.join("#MUSEUN").unwrap();
    /// ```    
    pub fn join<C>(&self, channel: C) -> Result<(), Error>
    where
        C: Into<Channel>,
    {
        let channel = Channel::validate(channel)?;
        self.raw(format!("JOIN {}", *channel))
    }

    /// Parts a `channel`
    ///
    /// This ensures the channel name is lowercased and begins with a '#'.
    ///
    /// The following are equivilant
    /// ```no_run
    /// # use twitchchat::{helpers::TestStream, *};
    /// # let mut stream = TestStream::new();
    /// # let (r, w) = sync_adapters(stream.clone(), stream.clone());
    /// # let mut client = Client::new(r, w);
    /// let w = client.writer();
    /// w.part("museun").unwrap();
    /// w.part("#museun").unwrap();
    /// w.part("Museun").unwrap();
    /// w.part("#MUSEUN").unwrap();
    /// ```    
    pub fn part<C>(&self, channel: C) -> Result<(), Error>
    where
        C: Into<Channel>,
    {
        let channel = Channel::validate(channel)?;
        self.raw(format!("PART {}", *channel))
    }

    /// Sends an "emote" `message` in the third person to the `channel`
    ///
    /// This ensures the channel name is lowercased and begins with a '#'.
    pub fn me<C, S>(&self, channel: C, message: S) -> Result<(), Error>
    where
        C: Into<Channel>,
        S: Display,
    {
        let channel = Channel::validate(channel)?;
        self.send(channel, format!("/me {}", message))
    }

    /// Sends the `message` to the `channel`
    ///
    /// This ensures the channel name is lowercased and begins with a '#'.
    ///
    /// Same as [`send`](./struct.Client.html#method.send)
    pub fn privmsg<C, S>(&self, channel: C, message: S) -> Result<(), Error>
    where
        C: Into<Channel>,
        S: Display,
    {
        let channel = Channel::validate(channel)?;
        self.raw(format!("PRIVMSG {} :{}", *channel, message))
    }

    /// Sends the `message` to the `channel`
    ///
    /// This ensures the channel name is lowercased and begins with a '#'.
    ///
    /// Same as [`privmsg`](./struct.Client.html#method.privmsg)
    pub fn send<C, S>(&self, channel: C, message: S) -> Result<(), Error>
    where
        C: Into<Channel>,
        S: Display,
    {
        self.privmsg(channel, message)
    }

    /// Sends the command: `data` (e.g. `/color #FFFFFF`)
    pub fn command<S>(&self, data: S) -> Result<(), Error>
    where
        S: Display,
    {
        self.raw(format!("PRIVMSG jtv :{}", data))
    }

    /// Sends a raw line (appends the required `\r\n`)
    pub fn raw<S>(&self, data: S) -> Result<(), Error>
    where
        S: Display,
    {
        self.write_line(data)
    }
}
