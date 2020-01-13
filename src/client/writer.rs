use super::*;
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

/// A writer that allows sending messages to the client
#[derive(Clone)]
pub struct Writer {
    buf: Vec<u8>,
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
            buf: vec![],
            sender,
        }
    }

    /// Send a raw message
    pub async fn raw(&mut self, data: &str) -> Result {
        let msg = crate::encode::raw(data);
        self.send_message(msg).await
    }

    /// Join a `channel`
    pub async fn join(&mut self, channel: impl IntoChannel) -> Result {
        let channel = channel.into_channel()?;
        let msg = crate::encode::join(&channel);
        self.send_message(msg).await
    }

    /// Leave a `channel`
    pub async fn part(&mut self, channel: impl IntoChannel) -> Result {
        let channel = channel.into_channel()?;
        let msg = crate::encode::part(&channel);
        self.send_message(msg).await
    }

    /// Send a message to a `target`
    pub async fn privmsg(&mut self, target: impl IntoChannel, data: &str) -> Result {
        let target = target.into_channel()?;
        let msg = crate::encode::privmsg(&target, data);
        self.send_message(msg).await
    }

    /// Request a PONG response from the server
    pub async fn ping(&mut self, token: &str) -> Result {
        let msg = crate::encode::ping(token);
        self.send_message(msg).await
    }

    /// Reply to a PING request from the server
    pub async fn pong(&mut self, token: &str) -> Result {
        let msg = crate::encode::pong(token);
        self.send_message(msg).await
    }

    /// Permanently prevent a user from chatting. Reason is optional and will be shown to the target user and other moderators.
    ///
    /// Use [unban] to remove a ban.
    ///
    /// [unban]: ./struct.Writer.html#method.unban
    pub async fn ban<'a>(
        &'a mut self,
        username: &'a str,
        reason: impl Into<Option<&'a str>>,
    ) -> Result {
        let msg = crate::encode::ban(username, reason);
        self.send_message(msg).await
    }

    /// Clear chat history for all users in this room.
    pub async fn clear(&mut self) -> Result {
        let msg = crate::encode::clear();
        self.send_message(msg).await
    }

    /// Change your username color.
    pub async fn color(&mut self, color: crate::color::Color) -> Result {
        let msg = crate::encode::color(color);
        self.send_message(msg).await
    }

    /// Sends the command: data (e.g. /color #FFFFFF)
    pub async fn command(&mut self, data: &str) -> Result {
        let msg = crate::encode::command(data);
        self.send_message(msg).await
    }

    /// Triggers a commercial.
    ///
    /// Length (optional) must be a positive number of seconds.
    pub async fn commercial(&mut self, length: impl Into<Option<usize>>) -> Result {
        let msg = crate::encode::commercial(length);
        self.send_message(msg).await
    }

    /// Reconnects to chat.
    pub async fn disconnect(&mut self) -> Result {
        let msg = crate::encode::disconnect();
        self.send_message(msg).await
    }

    /// Enables emote-only mode (only emoticons may be used in chat).
    ///
    /// Use [emote_only_off] to disable.
    ///
    /// [emote_only_off]: ./struct.Writer.html#method.emote_only_off
    pub async fn emote_only(&mut self) -> Result {
        let msg = crate::encode::emote_only();
        self.send_message(msg).await
    }

    /// Disables emote-only mode.
    pub async fn emote_only_off(&mut self) -> Result {
        let msg = crate::encode::emote_only_off();
        self.send_message(msg).await
    }

    /// Enables followers-only mode (only users who have followed for `duration` may chat).
    ///
    /// Examples: "30m", "1 week", "5 days 12 hours".
    ///
    /// Must be less than 3 months.
    pub async fn followers(&mut self, duration: &str) -> Result {
        let msg = crate::encode::followers(duration);
        self.send_message(msg).await
    }

    /// Disables followers-only mode.
    pub async fn followers_off(&mut self) -> Result {
        let msg = crate::encode::followers_off();
        self.send_message(msg).await
    }

    /// Grant moderator status to a user.
    ///
    /// Use [mods] to list the moderators of this channel.
    ///
    /// [mods]: ./struct.Writer.html#method.mods
    pub async fn give_mod(&mut self, username: &str) -> Result {
        let msg = crate::encode::give_mod(username);
        self.send_message(msg).await
    }

    /// Lists the commands available to you in this room.
    pub async fn help(&mut self) -> Result {
        let msg = crate::encode::help();
        self.send_message(msg).await
    }
    /// Host another channel.
    ///
    /// Use [unhost] to unset host mode.
    ///
    /// [unhost]: ./struct.Writer.html#method.unhost
    pub async fn host(&mut self, channel: impl IntoChannel) -> Result {
        let channel = channel.into_channel()?;
        let msg = crate::encode::host(&channel);
        self.send_message(msg).await
    }

    /// Adds a stream marker (with an optional comment, **max 140** characters) at the current timestamp.
    ///
    /// You can use markers in the Highlighter for easier editing.
    pub async fn marker<'a>(&'a mut self, comment: impl Into<Option<&'a str>>) -> Result {
        let msg = crate::encode::marker(comment);
        self.send_message(msg).await
    }

    /// Sends an "emote" message in the third person to the channel
    pub async fn me<'a>(&'a mut self, channel: &'a str, message: &'a str) -> Result {
        let msg = crate::encode::me(channel, message);
        self.send_message(msg).await
    }

    /// Lists the moderators of this channel.
    pub async fn mods(&mut self) -> Result {
        let msg = crate::encode::mods();
        self.send_message(msg).await
    }

    /// Enables r9k mode.
    ///
    /// Use [r9k_beta_off] to disable.
    ///
    /// [r9k_beta_off]: ./struct.Writer.html#method.r9k_beta_off
    pub async fn r9k_beta(&mut self) -> Result {
        let msg = crate::encode::r9k_beta();
        self.send_message(msg).await
    }

    /// Disables r9k mode.
    pub async fn r9k_beta_off(&mut self) -> Result {
        let msg = crate::encode::r9k_beta_off();
        self.send_message(msg).await
    }

    /// Raid another channel.
    ///
    /// Use [unraid] to cancel the Raid.
    ///
    /// [unraid]: ./struct.Writer.html#method.unraid
    pub async fn raid(&mut self, channel: impl IntoChannel) -> Result {
        let channel = channel.into_channel()?;
        let msg = crate::encode::raid(&channel);
        self.send_message(msg).await
    }

    /// Enables slow mode (limit how often users may send messages).
    ///
    /// Duration (optional, **default=120**) must be a positive number of seconds.
    ///
    /// Use [slow_off] to disable.
    ///
    /// [slow_off]: ./struct.Writer.html#method.slow_off
    pub async fn slow(&mut self, duration: impl Into<Option<usize>>) -> Result {
        let msg = crate::encode::slow(duration);
        self.send_message(msg).await
    }

    /// Disables slow mode.
    pub async fn slow_off(&mut self) -> Result {
        let msg = crate::encode::slow_off();
        self.send_message(msg).await
    }

    /// Enables subscribers-only mode (only subscribers may chat in this channel).
    ///
    /// Use [subscribers_off] to disable.
    ///
    /// [subscribers_off]: ./struct.Writer.html#method.subscribers_off
    pub async fn subscribers(&mut self) -> Result {
        let msg = crate::encode::subscribers();
        self.send_message(msg).await
    }

    /// Disables subscribers-only mode.
    pub async fn subscribers_off(&mut self) -> Result {
        let msg = crate::encode::subscribers_off();
        self.send_message(msg).await
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
    pub async fn timeout<'a>(
        &'a mut self,
        username: &'a str,
        duration: impl Into<Option<&'a str>>,
        message: impl Into<Option<&'a str>>,
    ) -> Result {
        let msg = crate::encode::timeout(username, duration, message);
        self.send_message(msg).await
    }

    /// Removes a ban on a user.
    pub async fn unban(&mut self, username: &str) -> Result {
        let msg = crate::encode::unban(username);
        self.send_message(msg).await
    }

    /// Stop hosting another channel.
    pub async fn unhost(&mut self) -> Result {
        let msg = crate::encode::unhost();
        self.send_message(msg).await
    }

    /// Revoke moderator status from a user.
    ///
    /// Use [mods] to list the moderators of this channel.
    ///
    /// [mods]: ./struct.Writer.html#method.mods
    pub async fn unmod(&mut self, username: &str) -> Result {
        let msg = crate::encode::unmod(username);
        self.send_message(msg).await
    }

    /// Cancel the Raid.
    pub async fn unraid(&mut self) -> Result {
        let msg = crate::encode::unraid();
        self.send_message(msg).await
    }

    /// Removes a timeout on a user.
    pub async fn untimeout(&mut self, username: &str) -> Result {
        let msg = crate::encode::untimeout(username);
        self.send_message(msg).await
    }

    /// Revoke VIP status from a user.
    ///
    /// Use [vips] to list the VIPs of this channel.
    ///
    /// [vips]: ./struct.Writer.html#method.vips
    pub async fn unvip(&mut self, username: &str) -> Result {
        let msg = crate::encode::unvip(username);
        self.send_message(msg).await
    }

    /// Grant VIP status to a user.
    ///
    /// Use [vips] to list the VIPs of this channel.
    ///
    /// [vips]: ./struct.Writer.html#method.vips
    pub async fn vip(&mut self, username: &str) -> Result {
        let msg = crate::encode::vip(username);
        self.send_message(msg).await
    }

    /// Lists the VIPs of this channel.
    pub async fn vips(&mut self) -> Result {
        let msg = crate::encode::vips();
        self.send_message(msg).await
    }

    /// Whispers the message to the username.
    pub async fn whisper<'a>(&'a mut self, username: &'a str, message: &'a str) -> Result {
        let msg = crate::encode::whisper(username, message);
        self.send_message(msg).await
    }

    /// Send this encodable message
    pub async fn send_message<E: crate::Encodable>(&mut self, msg: E) -> Result {
        // TODO use BytesMut here
        crate::sync::encode(&msg, &mut self.buf)?;
        self.sender
            .send(std::mem::take(&mut self.buf))
            .await
            .map_err(|_| Error::ClientDisconnect)
    }
}
