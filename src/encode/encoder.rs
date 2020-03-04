use super::conv_channel;
use crate::color::Color;
use crate::IntoChannel;

use std::io::Write;

type Result = std::result::Result<(), crate::Error>;

/// An encoder for messages
pub struct Encoder<W> {
    pub(crate) writer: W,
}

impl<W: Write> Encoder<W> {
    /// Gets the inner writer out
    pub fn into_inner(self) -> W {
        self.writer
    }

    /// Get a mutable borrow of the inner writer
    pub fn inner_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    /// Get a borrow of the inner writer
    pub fn inner(&self) -> &W {
        &self.writer
    }

    /// Make a new encoder from this writer
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

// TODO get rid of these formats!() by using an internal macro that uses format_args!()
impl<W: Write> Encoder<W> {
    /// Permanently prevent a user from chatting. Reason is optional and will be shown to the target user and other moderators.
    ///
    /// Use [unban] to remove a ban.
    ///
    /// [unban]: ./struct.Encoder.html#method.unban
    pub fn ban<'a>(&mut self, username: &str, reason: impl Into<Option<&'a str>>) -> Result {
        let data = match reason.into() {
            Some(reason) => format!("/ban {} {}", username, &reason),
            None => format!("/ban {}", username),
        };
        self.command(&data)
    }

    /// Clear chat history for all users in this room.
    pub fn clear(&mut self) -> Result {
        self.command("/clear")
    }

    /// Change your username color.
    pub fn color(&mut self, color: Color) -> Result {
        self.command(&format!("/color {}", color))
    }

    /// Sends the command: data (e.g. /color #FFFFFF)
    pub fn command(&mut self, data: &str) -> Result {
        self.writer
            .write_fmt(format_args!("PRIVMSG jtv :{}\r\n", data))
            .map_err(Into::into)
    }

    /// Triggers a commercial.
    ///
    /// Length (optional) must be a positive number of seconds.
    pub fn commercial(&mut self, length: impl Into<Option<usize>>) -> Result {
        match length.into() {
            Some(length) => self.command(&format!("/commercial {}", length)),
            None => self.command("/commercial"),
        }
    }

    /// Reconnects to chat.
    pub fn disconnect(&mut self) -> Result {
        self.command("/disconnect")
    }

    /// Enables emote-only mode (only emoticons may be used in chat).
    ///
    /// Use [emote_only_off] to disable.
    ///
    /// [emote_only_off]: ./struct.Encoder.html#method.emote_only_off
    pub fn emote_only(&mut self) -> Result {
        self.command("/emoteonly")
    }

    /// Disables emote-only mode.
    pub fn emote_only_off(&mut self) -> Result {
        self.command("/emoteonlyoff")
    }

    // TODO use `time` here
    /// Enables followers-only mode (only users who have followed for `duration` may chat).
    ///
    /// Examples: `"30m"`, `"1 week"`, `"5 days 12 hours"`.
    ///
    /// Must be less than 3 months.
    pub fn followers(&mut self, duration: &str) -> Result {
        self.command(&format!("/followers {}", duration))
    }

    /// Disables followers-only mode.
    pub fn followers_off(&mut self) -> Result {
        self.command("/followersoff")
    }

    /// Grant moderator status to a user.
    ///
    /// Use [mods] to list the moderators of this channel.
    ///
    /// [mods]: ./struct.Encoder.html#method.mods
    pub fn give_mod(&mut self, username: &str) -> Result {
        self.command(&format!("/mod {}", username,))
    }

    /// Lists the commands available to you in this room.
    pub fn help(&mut self) -> Result {
        self.command("/help")
    }

    /// Host another channel.
    ///
    /// Use [unhost] to unset host mode.
    ///
    /// [unhost]: ./struct.Encoder.html#method.unhost
    pub fn host(&mut self, channel: impl IntoChannel) -> Result {
        let channel = conv_channel(channel)?;
        self.command(&format!("/host {}", channel))
    }

    /// Join a channel
    pub fn join(&mut self, channel: impl IntoChannel) -> Result {
        let channel = conv_channel(channel)?;
        self.writer
            .write_fmt(format_args!("JOIN {}\r\n", channel))?;
        Ok(())
    }

    // TODO limit this to 140
    // TODO get rid of the Into<Option<&'a str>>
    /// Adds a stream marker (with an optional comment, **max 140** characters) at the current timestamp.
    ///
    /// You can use markers in the Highlighter for easier editing.
    pub fn marker<'a>(&mut self, comment: impl Into<Option<&'a str>>) -> Result {
        match comment.into() {
            Some(comment) => self.command(&format!("/marker {}", comment)),
            None => self.command("/marker"),
        }
    }

    /// Sends an "emote" message in the third person to the channel
    pub fn me(&mut self, channel: impl IntoChannel, message: &str) -> Result {
        let channel = conv_channel(channel)?;
        self.privmsg(&channel, &format!("/me {}", message))
    }

    /// Lists the moderators of this channel.
    pub fn mods(&mut self) -> Result {
        self.command("/mods")
    }

    /// Leave a channel
    pub fn part(&mut self, channel: impl IntoChannel) -> Result {
        let channel = conv_channel(channel)?;
        self.writer
            .write_fmt(format_args!("PART {}\r\n", channel))?;
        Ok(())
    }

    /// Request a heartbeat with the provided token
    pub fn ping(&mut self, token: &str) -> Result {
        self.writer.write_fmt(format_args!("PING {}\r\n", token))?;
        Ok(())
    }

    /// Response to a heartbeat with the provided token
    pub fn pong(&mut self, token: &str) -> Result {
        self.writer.write_fmt(format_args!("PONG :{}\r\n", token))?;
        Ok(())
    }

    /// Send data to a target
    pub fn privmsg(&mut self, target: &str, data: &str) -> Result {
        self.writer
            .write_fmt(format_args!("PRIVMSG {} :{}\r\n", target, data))?;
        Ok(())
    }

    /// Enables r9k mode.
    ///
    /// Use [r9k_beta_off] to disable.
    ///
    /// [r9k_beta_off]: ./struct.Encoder.html#method.r9k_beta_off
    pub fn r9k_beta(&mut self) -> Result {
        self.command("/r9kbeta")
    }

    /// Disables r9k mode.
    pub fn r9k_beta_off(&mut self) -> Result {
        self.command("/r9kbetaoff")
    }

    /// Raid another channel.
    ///
    /// Use [unraid] to cancel the Raid.
    ///
    /// [unraid]: ./struct.Encoder.html#method.unraid
    pub fn raid(&mut self, channel: impl IntoChannel) -> Result {
        let channel = conv_channel(channel)?;
        self.command(&format!("/raid {}", channel))
    }

    /// Send a raw IRC-style message
    pub fn raw(&mut self, raw: impl AsRef<[u8]>) -> Result {
        self.writer.write_all(raw.as_ref())?;
        self.writer.write_all(b"\r\n")?;
        Ok(())
    }

    // TODO use `time` here
    /// Enables slow mode (limit how often users may send messages).
    ///
    /// Duration (optional, **default=120**) must be a positive number of seconds.
    ///
    /// Use [slow_off] to disable.
    ///
    /// [slow_off]: ./struct.Encoder.html#method.slow_off
    pub fn slow(&mut self, duration: impl Into<Option<usize>>) -> Result {
        self.command(&format!("/slow {}", duration.into().unwrap_or_else(|| 120)))
    }

    /// Disables slow mode.
    pub fn slow_off(&mut self) -> Result {
        self.command("/slowoff")
    }

    /// Enables subscribers-only mode (only subscribers may chat in this channel).
    ///
    /// Use [subscribers_off] to disable.
    ///
    /// [subscribers_off]: ./struct.Encoder.html#methodruct.html#method.subscribers_off
    pub fn subscribers(&mut self) -> Result {
        self.command("/subscribers")
    }

    /// Disables subscribers-only mode.
    pub fn subscribers_off(&mut self) -> Result {
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
    /// Use [untimeout] to remove a timeout.
    ///
    /// [untimeout]: ./struct.Encoder.html#methodruct.html#method.untimeout
    pub fn timeout<'a, 'b>(
        &mut self,
        username: &str,
        duration: impl Into<Option<&'a str>>,
        message: impl Into<Option<&'b str>>,
    ) -> Result {
        let data = match (duration.into(), message.into()) {
            (Some(dur), Some(reason)) => format!("/timeout {} {} {}", username, dur, reason),
            (None, Some(reason)) => format!("/timeout {} {}", username, reason),
            (Some(dur), None) => format!("/timeout {} {}", username, dur),
            (None, None) => format!("/timeout {}", username),
        };
        self.command(&data)
    }

    /// Removes a ban on a user.
    pub fn unban(&mut self, username: &str) -> Result {
        self.command(&format!("/unban {}", username))
    }

    /// Stop hosting another channel.
    pub fn unhost(&mut self) -> Result {
        self.command("/unhost")
    }

    /// Revoke moderator status from a user.
    ///
    /// Use [mods] to list the moderators of this channel.
    ///
    /// [mods]: ./struct.Encoder.html#methodruct.html#method.mods
    pub fn unmod(&mut self, username: &str) -> Result {
        self.command(&format!("/unmod {}", username))
    }

    /// Cancel the Raid.
    pub fn unraid(&mut self) -> Result {
        self.command("/unraid")
    }

    /// Removes a timeout on a user.
    pub fn untimeout(&mut self, username: &str) -> Result {
        self.command(&format!("/untimeout {}", username))
    }

    /// Revoke VIP status from a user.
    ///
    /// Use [vips] to list the VIPs of this channel.
    ///
    /// [vips]: ./struct.Encoder.html#methodruct.html#method.vips
    pub fn unvip(&mut self, username: &str) -> Result {
        self.command(&format!("/unvip {}", username))
    }

    /// Grant VIP status to a user.
    ///
    /// Use [vips] to list the VIPs of this channel.
    ///
    /// [vips]: ./struct.Encoder.html#methodruct.html#method.vips
    pub fn vip(&mut self, username: &str) -> Result {
        self.command(&format!("/vip {}", username))
    }

    /// Lists the VIPs of this channel.

    pub fn vips(&mut self) -> Result {
        self.command("/vips")
    }

    /// Whispers the message to the username.
    pub fn whisper(&mut self, username: &str, message: &str) -> Result {
        self.command(&format!("/w {} {}", username, message))
    }
}

impl<W: Write + Default> Default for Encoder<W> {
    fn default() -> Self {
        Self {
            writer: Default::default(),
        }
    }
}

impl<W> std::fmt::Debug for Encoder<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Encoder").finish()
    }
}

impl<W: Write + Clone> Clone for Encoder<W> {
    fn clone(&self) -> Self {
        Self {
            writer: self.writer.clone(),
        }
    }
}
