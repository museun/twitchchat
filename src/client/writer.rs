use super::*;
use crate::encode::Encoder;
use crate::IntoChannel;

pub(super) async fn write_loop<W>(
    write: W,
    mut recv: Receiver,
) -> std::result::Result<Status, Error>
where
    W: AsyncWrite + Send + Sync + Unpin + 'static,
{
    let mut writer = tokio::io::BufWriter::new(write);
    while let Some(data) = recv.next().await {
        log::trace!("> {}", std::str::from_utf8(&data).unwrap().escape_debug());
        writer.write_all(&data).await?;
        writer.flush().await?
    }
    Ok(Status::Eof)
}

type Result = std::result::Result<(), Error>;

trait SafeEncode {
    fn clear_data(&mut self);

    fn try_write<F>(&mut self, mut func: F) -> std::io::Result<()>
    where
        F: FnMut(&mut Self) -> std::io::Result<()>,
    {
        match func(self) {
            Ok(res) => Ok(res),
            Err(err) => {
                self.clear_data();
                Err(err)
            }
        }
    }
}

impl SafeEncode for Encoder<Vec<u8>> {
    fn clear_data(&mut self) {
        self.writer.clear();
    }
}

/// A writer that allows sending messages to the client
#[derive(Clone)]
pub struct Writer {
    encoder: Encoder<Vec<u8>>,
    sender: Sender,
}

impl std::fmt::Debug for Writer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Writer").finish()
    }
}

impl Writer {
    pub(super) fn new(sender: Sender) -> Self {
        Self {
            encoder: Encoder::new(vec![]),
            sender,
        }
    }

    /// Send a raw message
    pub async fn raw(&mut self, data: &str) -> Result {
        self.encoder.try_write(|enc| enc.raw(data))?;
        self.flush_message().await
    }

    /// Join a `channel`
    pub async fn join(&mut self, channel: impl IntoChannel) -> Result {
        let channel = channel.into_channel()?;
        self.encoder.try_write(|enc| enc.join(&channel))?;
        self.flush_message().await
    }

    /// Leave a `channel`
    pub async fn part(&mut self, channel: impl IntoChannel) -> Result {
        let channel = channel.into_channel()?;
        self.encoder
            .try_write(|enc| enc.try_write(|enc| enc.part(&channel)))?;
        self.flush_message().await
    }

    /// Send a message to a `target`
    pub async fn privmsg(&mut self, target: impl IntoChannel, data: &str) -> Result {
        let target = target.into_channel()?;
        self.encoder.try_write(|enc| enc.privmsg(&target, data))?;
        self.flush_message().await
    }

    /// Request a PONG response from the server
    pub async fn ping(&mut self, token: &str) -> Result {
        self.encoder.try_write(|enc| enc.ping(token))?;
        self.flush_message().await
    }

    /// Reply to a PING request from the server
    pub async fn pong(&mut self, token: &str) -> Result {
        self.encoder.try_write(|enc| enc.pong(token))?;
        self.flush_message().await
    }

    /// Permanently prevent a user from chatting. Reason is optional and will be shown to the target user and other moderators.
    ///
    /// Use [unban] to remove a ban.
    ///
    /// [unban]: ./struct.Writer.html#method.unban
    pub async fn ban<'a>(&mut self, username: &str, reason: impl Into<Option<&'a str>>) -> Result {
        let reason = reason.into();
        self.encoder.try_write(|enc| enc.ban(username, reason))?;
        self.flush_message().await
    }

    /// Clear chat history for all users in this room.
    pub async fn clear(&mut self) -> Result {
        self.encoder.try_write(|enc| enc.clear())?;
        self.flush_message().await
    }

    /// Change your username color.
    pub async fn color(&mut self, color: crate::color::Color) -> Result {
        self.encoder.try_write(|enc| enc.color(color))?;
        self.flush_message().await
    }

    /// Sends the command: data (e.g. /color #FFFFFF)
    pub async fn command(&mut self, data: &str) -> Result {
        self.encoder.try_write(|enc| enc.command(data))?;
        self.flush_message().await
    }

    /// Triggers a commercial.
    ///
    /// Length (optional) must be a positive number of seconds.
    pub async fn commercial(&mut self, length: impl Into<Option<usize>>) -> Result {
        let length = length.into();
        self.encoder.try_write(|enc| enc.commercial(length))?;
        self.flush_message().await
    }

    /// Reconnects to chat.
    pub async fn disconnect(&mut self) -> Result {
        self.encoder.try_write(|enc| enc.disconnect())?;
        self.flush_message().await
    }

    /// Enables emote-only mode (only emoticons may be used in chat).
    ///
    /// Use [emote_only_off] to disable.
    ///
    /// [emote_only_off]: ./struct.Writer.html#method.emote_only_off
    pub async fn emote_only(&mut self) -> Result {
        self.encoder.try_write(|enc| enc.emote_only())?;
        self.flush_message().await
    }

    /// Disables emote-only mode.
    pub async fn emote_only_off(&mut self) -> Result {
        self.encoder.try_write(|enc| enc.emote_only_off())?;
        self.flush_message().await
    }

    /// Enables followers-only mode (only users who have followed for `duration` may chat).
    ///
    /// Examples: "30m", "1 week", "5 days 12 hours".
    ///
    /// Must be less than 3 months.
    pub async fn followers(&mut self, duration: &str) -> Result {
        self.encoder.try_write(|enc| enc.followers(duration))?;
        self.flush_message().await
    }

    /// Disables followers-only mode.
    pub async fn followers_off(&mut self) -> Result {
        self.encoder.try_write(|enc| enc.followers_off())?;
        self.flush_message().await
    }

    /// Grant moderator status to a user.
    ///
    /// Use [mods] to list the moderators of this channel.
    ///
    /// [mods]: ./struct.Writer.html#method.mods
    pub async fn give_mod(&mut self, username: &str) -> Result {
        self.encoder.try_write(|enc| enc.give_mod(username))?;
        self.flush_message().await
    }

    /// Lists the commands available to you in this room.
    pub async fn help(&mut self) -> Result {
        self.encoder.try_write(|enc| enc.help())?;
        self.flush_message().await
    }

    /// Host another channel.
    ///
    /// Use [unhost] to unset host mode.
    ///
    /// [unhost]: ./struct.Writer.html#method.unhost
    pub async fn host(&mut self, channel: impl IntoChannel) -> Result {
        let channel = channel.into_channel()?;
        self.encoder.try_write(|enc| enc.host(&channel))?;
        self.flush_message().await
    }

    /// Adds a stream marker (with an optional comment, **max 140** characters) at the current timestamp.
    ///
    /// You can use markers in the Highlighter for easier editing.
    pub async fn marker<'a>(&mut self, comment: impl Into<Option<&'a str>>) -> Result {
        let comment = comment.into();
        self.encoder.try_write(|enc| enc.marker(comment))?;
        self.flush_message().await
    }

    /// Sends an "emote" message in the third person to the channel
    pub async fn me(&mut self, channel: &str, message: &str) -> Result {
        self.encoder.try_write(|enc| enc.me(channel, message))?;
        self.flush_message().await
    }

    /// Lists the moderators of this channel.
    pub async fn mods(&mut self) -> Result {
        self.encoder.try_write(|enc| enc.mods())?;
        self.flush_message().await
    }

    /// Enables r9k mode.
    ///
    /// Use [r9k_beta_off] to disable.
    ///
    /// [r9k_beta_off]: ./struct.Writer.html#method.r9k_beta_off
    pub async fn r9k_beta(&mut self) -> Result {
        self.encoder.try_write(|enc| enc.r9k_beta())?;
        self.flush_message().await
    }

    /// Disables r9k mode.
    pub async fn r9k_beta_off(&mut self) -> Result {
        self.encoder.try_write(|enc| enc.r9k_beta_off())?;
        self.flush_message().await
    }

    /// Raid another channel.
    ///
    /// Use [unraid] to cancel the Raid.
    ///
    /// [unraid]: ./struct.Writer.html#method.unraid
    pub async fn raid(&mut self, channel: impl IntoChannel) -> Result {
        let channel = channel.into_channel()?;
        self.encoder.try_write(|enc| enc.raid(&channel))?;
        self.flush_message().await
    }

    /// Enables slow mode (limit how often users may send messages).
    ///
    /// Duration (optional, **default=120**) must be a positive number of seconds.
    ///
    /// Use [slow_off] to disable.
    ///
    /// [slow_off]: ./struct.Writer.html#method.slow_off
    pub async fn slow(&mut self, duration: impl Into<Option<usize>>) -> Result {
        let duration = duration.into();
        self.encoder.try_write(|enc| enc.slow(duration))?;
        self.flush_message().await
    }

    /// Disables slow mode.
    pub async fn slow_off(&mut self) -> Result {
        self.encoder.try_write(|enc| enc.slow_off())?;
        self.flush_message().await
    }

    /// Enables subscribers-only mode (only subscribers may chat in this channel).
    ///
    /// Use [subscribers_off] to disable.
    ///
    /// [subscribers_off]: ./struct.Writer.html#method.subscribers_off
    pub async fn subscribers(&mut self) -> Result {
        self.encoder.try_write(|enc| enc.subscribers())?;
        self.flush_message().await
    }

    /// Disables subscribers-only mode.
    pub async fn subscribers_off(&mut self) -> Result {
        self.encoder.try_write(|enc| enc.subscribers_off())?;
        self.flush_message().await
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
    /// [untimeout]: ./struct.Writer.html#method.untimeout
    pub async fn timeout<'a, 'b>(
        &mut self,
        username: &str,
        duration: impl Into<Option<&'a str>>,
        message: impl Into<Option<&'b str>>,
    ) -> Result {
        let duration = duration.into();
        let message = message.into();
        self.encoder
            .try_write(|enc| enc.timeout(username, duration, message))?;
        self.flush_message().await
    }

    /// Removes a ban on a user.
    pub async fn unban(&mut self, username: &str) -> Result {
        self.encoder.try_write(|enc| enc.unban(username))?;
        self.flush_message().await
    }

    /// Stop hosting another channel.
    pub async fn unhost(&mut self) -> Result {
        self.encoder.try_write(|enc| enc.unhost())?;
        self.flush_message().await
    }

    /// Revoke moderator status from a user.
    ///
    /// Use [mods] to list the moderators of this channel.
    ///
    /// [mods]: ./struct.Writer.html#method.mods
    pub async fn unmod(&mut self, username: &str) -> Result {
        self.encoder.try_write(|enc| enc.unmod(username))?;
        self.flush_message().await
    }

    /// Cancel the Raid.
    pub async fn unraid(&mut self) -> Result {
        self.encoder.try_write(|enc| enc.unraid())?;
        self.flush_message().await
    }

    /// Removes a timeout on a user.
    pub async fn untimeout(&mut self, username: &str) -> Result {
        self.encoder.try_write(|enc| enc.untimeout(username))?;
        self.flush_message().await
    }

    /// Revoke VIP status from a user.
    ///
    /// Use [vips] to list the VIPs of this channel.
    ///
    /// [vips]: ./struct.Writer.html#method.vips
    pub async fn unvip(&mut self, username: &str) -> Result {
        self.encoder.try_write(|enc| enc.unvip(username))?;
        self.flush_message().await
    }

    /// Grant VIP status to a user.
    ///
    /// Use [vips] to list the VIPs of this channel.
    ///
    /// [vips]: ./struct.Writer.html#method.vips
    pub async fn vip(&mut self, username: &str) -> Result {
        self.encoder.try_write(|enc| enc.vip(username))?;
        self.flush_message().await
    }

    /// Lists the VIPs of this channel.
    pub async fn vips(&mut self) -> Result {
        self.encoder.try_write(|enc| enc.vips())?;
        self.flush_message().await
    }

    /// Whispers the message to the username.
    pub async fn whisper(&mut self, username: &str, message: &str) -> Result {
        self.encoder
            .try_write(|enc| enc.whisper(username, message))?;
        self.flush_message().await
    }

    /// Send this encodable message
    async fn flush_message(&mut self) -> Result {
        let bytes = std::mem::take(&mut self.encoder).into_inner();
        self.sender
            .send(bytes)
            .await
            .map_err(|_| Error::ClientDisconnect)
    }
}
