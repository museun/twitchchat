use crate::{color::Color, IntoChannel};

use std::io::Write;

type Result = std::result::Result<(), crate::Error>;

struct ByteWriter<'a, W: Write> {
    inner: &'a mut W,
}

impl<'a, W: Write> ByteWriter<'a, W> {
    fn new(inner: &'a mut W) -> Self {
        Self { inner }
    }

    fn jtv_command(self, parts: &[&dyn AsRef<str>]) -> Result {
        self.inner.write_all(b"PRIVMSG jtv :")?;
        self.parts(parts)
    }

    fn command(self, channel: &crate::Channel, parts: &[&dyn AsRef<str>]) -> Result {
        self.inner.write_all(b"PRIVMSG ")?;
        self.inner.write_all(channel.as_ref().as_bytes())?;
        self.inner.write_all(b" :")?;
        self.parts(parts)
    }

    fn parts(self, parts: &[&dyn AsRef<str>]) -> Result {
        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                self.inner.write_all(b" ")?;
            }
            self.inner.write_all(part.as_ref().as_bytes())?;
        }
        self.end()
    }

    fn parts_term(self, parts: &[&dyn AsRef<str>]) -> Result {
        for part in parts.iter() {
            self.inner.write_all(part.as_ref().as_bytes())?;
        }
        self.end()
    }

    fn write_bytes(self, data: impl AsRef<[u8]>) -> Result {
        self.inner.write_all(data.as_ref())?;
        self.end()
    }

    fn end(self) -> Result {
        self.inner.write_all(b"\r\n")?;
        self.inner.flush()?;
        Ok(())
    }
}

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

// NOTE: This is literally copy/pasted from the async_encoder, with async/await removed
impl<W: Write> Encoder<W> {
    /// Permanently prevent a user from chatting. Reason is optional and will be
    /// shown to the target user and other moderators.
    ///
    /// Use [unban] to remove a ban.
    ///
    /// [unban]: ./struct.Encoder.html#method.unban
    pub fn ban<'a>(
        &mut self,
        channel: impl IntoChannel,
        username: &str,
        reason: impl Into<Option<&'a str>>,
    ) -> Result {
        let channel = channel.into_channel()?;
        let writer = ByteWriter::new(&mut self.writer);
        match reason.into() {
            Some(reason) => writer.command(&channel, &[&"/ban", &username, &reason]),
            None => writer.command(&channel, &[&"/ban", &username]),
        }
    }

    /// Clear chat history for all users in this room.
    pub fn clear(&mut self, channel: impl IntoChannel) -> Result {
        let channel = channel.into_channel()?;
        ByteWriter::new(&mut self.writer).command(&channel, &[&"/clear"])
    }

    /// Change your username color.
    pub fn color(&mut self, color: Color) -> Result {
        ByteWriter::new(&mut self.writer).jtv_command(&[&"/color", &color.to_string()])
    }

    /// Sends the command: data (e.g. /color #FFFFFF)
    pub fn command(&mut self, channel: impl IntoChannel, data: &str) -> Result {
        let channel = channel.into_channel()?;
        ByteWriter::new(&mut self.writer).command(&channel, &[&data])
    }

    /// Sends the command: data to the 'jtv' channel (e.g. /color #FFFFFF)
    pub fn jtv_command(&mut self, data: &str) -> Result {
        ByteWriter::new(&mut self.writer).jtv_command(&[&data])
    }

    /// Triggers a commercial.
    ///
    /// Length (optional) must be a positive number of seconds.
    pub fn commercial(
        &mut self,
        channel: impl IntoChannel,
        length: impl Into<Option<usize>>,
    ) -> Result {
        let channel = channel.into_channel()?;
        let writer = ByteWriter::new(&mut self.writer);
        match length.into() {
            Some(length) => writer.command(&channel, &[&"/commercial", &length.to_string()]),
            None => writer.command(&channel, &[&"/commercial"]),
        }
    }

    /// Reconnects to chat.
    pub fn disconnect(&mut self) -> Result {
        ByteWriter::new(&mut self.writer).jtv_command(&[&"/disconnect"])
    }

    /// Enables emote-only mode (only emoticons may be used in chat).
    ///
    /// Use [emote_only_off] to disable.
    ///
    /// [emote_only_off]: ./struct.Encoder.html#method.emote_only_off
    pub fn emote_only(&mut self, channel: impl IntoChannel) -> Result {
        let channel = channel.into_channel()?;
        ByteWriter::new(&mut self.writer).command(&channel, &[&"/emoteonly"])
    }

    /// Disables emote-only mode.
    pub fn emote_only_off(&mut self, channel: impl IntoChannel) -> Result {
        let channel = channel.into_channel()?;
        ByteWriter::new(&mut self.writer).command(&channel, &[&"/emoteonlyoff"])
    }

    // TODO use `time` here
    /// Enables followers-only mode (only users who have followed for `duration` may chat).
    ///
    /// Examples: `"30m"`, `"1 week"`, `"5 days 12 hours"`.
    ///
    /// Must be less than 3 months.
    pub fn followers(&mut self, channel: impl IntoChannel, duration: &str) -> Result {
        let channel = channel.into_channel()?;
        ByteWriter::new(&mut self.writer).command(&channel, &[&"/followers", &duration])
    }

    /// Disables followers-only mode.
    pub fn followers_off(&mut self, channel: impl IntoChannel) -> Result {
        let channel = channel.into_channel()?;
        ByteWriter::new(&mut self.writer).command(&channel, &[&"/followersoff"])
    }

    /// Grant moderator status to a user.
    ///
    /// Use [mods] to list the moderators of this channel.
    ///
    /// [mods]: ./struct.Encoder.html#method.mods
    pub fn give_mod(&mut self, channel: impl IntoChannel, username: &str) -> Result {
        let channel = channel.into_channel()?;
        ByteWriter::new(&mut self.writer).command(&channel, &[&"/mod", &username])
    }

    /// Lists the commands available to you in this room.
    pub fn help(&mut self, channel: impl IntoChannel) -> Result {
        let channel = channel.into_channel()?;
        ByteWriter::new(&mut self.writer).command(&channel, &[&"/help"])
    }

    /// Host another channel.
    ///
    /// Use [unhost] to unset host mode.
    ///
    /// [unhost]: ./struct.Encoder.html#method.unhost
    pub fn host(&mut self, source: impl IntoChannel, target: impl IntoChannel) -> Result {
        let source = source.into_channel()?;
        let target = target.into_channel()?;
        ByteWriter::new(&mut self.writer).command(&source, &[&"/host", &target])
    }

    /// Join a channel
    pub fn join(&mut self, channel: impl IntoChannel) -> Result {
        let channel = channel.into_channel()?;
        ByteWriter::new(&mut self.writer).parts(&[&"JOIN", &channel])
    }

    /// Adds a stream marker (with an optional comment, **max 140** characters) at the current timestamp.
    ///
    /// You can use markers in the Highlighter for easier editing.
    ///
    /// If the string exceeds 140 characters then it will be truncated
    pub fn marker<'a>(
        &mut self,
        channel: impl IntoChannel,
        comment: impl Into<Option<&'a str>>,
    ) -> Result {
        let channel = channel.into_channel()?;
        let writer = ByteWriter::new(&mut self.writer);
        match comment.into() {
            None => {
                writer.command(&channel, &[&"/marker"]) //
            }
            Some(marker) if marker.len() <= 140 => writer.command(&channel, &[&"/marker", &marker]),
            Some(marker) => writer.command(&channel, &[&"/marker", &&marker[..140]]),
        }
    }

    /// Sends an "emote" message in the third person to the channel
    pub fn me(&mut self, channel: impl IntoChannel, message: impl AsRef<str>) -> Result {
        let channel = channel.into_channel()?;
        ByteWriter::new(&mut self.writer).command(&channel, &[&"/me", &message])
    }

    /// Lists the moderators of this channel.
    pub fn mods(&mut self, channel: impl IntoChannel) -> Result {
        let channel = channel.into_channel()?;
        ByteWriter::new(&mut self.writer).command(&channel, &[&"/mods"])
    }

    /// Leave a channel
    pub fn part(&mut self, channel: impl IntoChannel) -> Result {
        let channel = channel.into_channel()?;
        ByteWriter::new(&mut self.writer).parts(&[&"PART", &channel])
    }

    /// Request a heartbeat with the provided token
    pub fn ping(&mut self, token: impl AsRef<str>) -> Result {
        ByteWriter::new(&mut self.writer).parts(&[&"PING", &token])
    }

    /// Response to a heartbeat with the provided token
    pub fn pong(&mut self, token: impl AsRef<str>) -> Result {
        ByteWriter::new(&mut self.writer).parts_term(&[&"PONG", &" :", &token])
    }

    /// Send data to a channel
    pub fn privmsg(&mut self, channel: impl IntoChannel, data: impl AsRef<str>) -> Result {
        let channel = channel.into_channel()?;
        ByteWriter::new(&mut self.writer).parts_term(&[&"PRIVMSG ", &channel, &" :", &data])
    }

    /// Enables r9k mode.
    ///
    /// Use [r9k_beta_off] to disable.
    ///
    /// [r9k_beta_off]: ./struct.Encoder.html#method.r9k_beta_off
    pub fn r9k_beta(&mut self, channel: impl IntoChannel) -> Result {
        let channel = channel.into_channel()?;
        ByteWriter::new(&mut self.writer).command(&channel, &[&"/r9kbeta"])
    }

    /// Disables r9k mode.
    pub fn r9k_beta_off(&mut self, channel: impl IntoChannel) -> Result {
        let channel = channel.into_channel()?;
        ByteWriter::new(&mut self.writer).command(&channel, &[&"/r9kbetaoff"])
    }

    /// Raid another channel.
    ///
    /// Use [unraid] to cancel the Raid.
    ///
    /// [unraid]: ./struct.Encoder.html#method.unraid
    pub fn raid(&mut self, source: impl IntoChannel, target: impl IntoChannel) -> Result {
        let source = source.into_channel()?;
        let target = target.into_channel()?;
        ByteWriter::new(&mut self.writer).command(&source, &[&"/raid", &target])
    }

    /// Send a raw IRC-style message
    pub fn raw(&mut self, raw: impl AsRef<[u8]>) -> Result {
        ByteWriter::new(&mut self.writer).write_bytes(raw)
    }

    // TODO use `time` here
    /// Enables slow mode (limit how often users may send messages).
    ///
    /// Duration (optional, **default=120**) must be a positive number of seconds.
    ///
    /// Use [slow_off] to disable.
    ///
    /// [slow_off]: ./struct.Encoder.html#method.slow_off
    pub fn slow(
        &mut self,
        channel: impl IntoChannel,
        duration: impl Into<Option<usize>>,
    ) -> Result {
        let channel = channel.into_channel()?;
        let dur = duration.into().unwrap_or_else(|| 120).to_string();
        ByteWriter::new(&mut self.writer).command(&channel, &[&"/slow", &dur])
    }

    /// Disables slow mode.
    pub fn slow_off(&mut self, channel: impl IntoChannel) -> Result {
        let channel = channel.into_channel()?;
        ByteWriter::new(&mut self.writer).command(&channel, &[&"/slowoff"])
    }

    /// Enables subscribers-only mode (only subscribers may chat in this channel).
    ///
    /// Use [subscribers_off] to disable.
    ///
    /// [subscribers_off]: ./struct.Encoder.html#methodruct.html#method.subscribers_off
    pub fn subscribers(&mut self, channel: impl IntoChannel) -> Result {
        let channel = channel.into_channel()?;
        ByteWriter::new(&mut self.writer).command(&channel, &[&"/subscribers"])
    }

    /// Disables subscribers-only mode.
    pub fn subscribers_off(&mut self, channel: impl IntoChannel) -> Result {
        let channel = channel.into_channel()?;
        ByteWriter::new(&mut self.writer).command(&channel, &[&"/subscribersoff"])
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
        channel: impl IntoChannel,
        username: &str,
        duration: impl Into<Option<&'a str>>,
        message: impl Into<Option<&'b str>>,
    ) -> Result {
        let channel = channel.into_channel()?;
        let writer = ByteWriter::new(&mut self.writer);
        match (duration.into(), message.into()) {
            (Some(dur), Some(reason)) => {
                writer.command(&channel, &[&"/timeout", &username, &dur, &reason])
            }
            (None, Some(reason)) => writer.command(&channel, &[&"/timeout", &username, &reason]),
            (Some(dur), None) => writer.command(&channel, &[&"/timeout", &username, &dur]),
            (None, None) => {
                writer //
                    .command(&channel, &[&"/timeout", &username])
            }
        }
    }

    /// Removes a ban on a user.
    pub fn unban(&mut self, channel: impl IntoChannel, username: impl AsRef<str>) -> Result {
        let channel = channel.into_channel()?;
        ByteWriter::new(&mut self.writer).command(&channel, &[&"/unban", &username])
    }

    /// Stop hosting another channel.
    pub fn unhost(&mut self, channel: impl IntoChannel) -> Result {
        let channel = channel.into_channel()?;
        ByteWriter::new(&mut self.writer).command(&channel, &[&"/unhost"])
    }

    /// Revoke moderator status from a user.
    ///
    /// Use [mods] to list the moderators of this channel.
    ///
    /// [mods]: ./struct.Encoder.html#methodruct.html#method.mods
    pub fn unmod(&mut self, channel: impl IntoChannel, username: &str) -> Result {
        let channel = channel.into_channel()?;
        ByteWriter::new(&mut self.writer).command(&channel, &[&"/unmod", &username])
    }

    /// Cancel the Raid.
    pub fn unraid(&mut self, channel: impl IntoChannel) -> Result {
        let channel = channel.into_channel()?;
        ByteWriter::new(&mut self.writer).command(&channel, &[&"/unraid"])
    }

    /// Removes a timeout on a user.
    pub fn untimeout(&mut self, channel: impl IntoChannel, username: &str) -> Result {
        let channel = channel.into_channel()?;
        ByteWriter::new(&mut self.writer).command(&channel, &[&"/untimeout", &username])
    }

    /// Revoke VIP status from a user.
    ///
    /// Use [vips] to list the VIPs of this channel.
    ///
    /// [vips]: ./struct.Encoder.html#methodruct.html#method.vips
    pub fn unvip(&mut self, channel: impl IntoChannel, username: &str) -> Result {
        let channel = channel.into_channel()?;
        ByteWriter::new(&mut self.writer).command(&channel, &[&"/unvip", &username])
    }

    /// Grant VIP status to a user.
    ///
    /// Use [vips] to list the VIPs of this channel.
    ///
    /// [vips]: ./struct.Encoder.html#methodruct.html#method.vips
    pub fn vip(&mut self, channel: impl IntoChannel, username: &str) -> Result {
        let channel = channel.into_channel()?;
        ByteWriter::new(&mut self.writer).command(&channel, &[&"/vip", &username])
    }

    /// Lists the VIPs of this channel.
    pub fn vips(&mut self, channel: impl IntoChannel) -> Result {
        let channel = channel.into_channel()?;
        ByteWriter::new(&mut self.writer).command(&channel, &[&"/vips"])
    }

    /// Whispers the message to the username.
    pub fn whisper(&mut self, username: impl AsRef<str>, message: impl AsRef<str>) -> Result {
        ByteWriter::new(&mut self.writer).jtv_command(&[&"/w", &username, &message])
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
