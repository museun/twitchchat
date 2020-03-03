use crate::color::Color;
use tokio::io::{AsyncWrite, AsyncWriteExt};

type Result = std::io::Result<()>;

struct ByteWriter<'a, W: AsyncWrite + Unpin> {
    inner: &'a mut W,
}

impl<'a, W: AsyncWrite + Unpin> ByteWriter<'a, W> {
    fn new(inner: &'a mut W) -> Self {
        Self { inner }
    }

    async fn write(&mut self, data: impl AsRef<[u8]>) -> Result {
        self.inner.write_all(data.as_ref()).await
    }

    async fn end(self) -> Result {
        self.inner.write_all(b"\r\n").await?;
        self.inner.flush().await
    }
}

/// An async encoder for messages
pub struct AsyncEncoder<W> {
    pub(crate) writer: W,
}

impl<W: AsyncWrite> AsyncEncoder<W> {
    /// Gets the inner writer out
    pub fn into_inner(self) -> W {
        self.writer
    }

    /// Get a mutable borrow of the inner writer
    pub fn inner(&mut self) -> &mut W {
        &mut self.writer
    }

    /// Make a new encoder from this writer
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

macro_rules! write {
    (cmd $w:expr, $($e:expr),* $(,)?) => {{
        write!($w, "PRIVMSG jtv :", $($e),*)
    }};
    ($w:expr, $($e:expr),* $(,)?) => {{
        let mut w = ByteWriter::new($w);
        $(w.write($e).await?;)*
        w.end().await
    }};
}

impl<W: AsyncWrite + Unpin> AsyncEncoder<W> {
    /// Permanently prevent a user from chatting. Reason is optional and will be
    /// shown to the target user and other moderators.
    ///
    /// Use [unban] to remove a ban.
    ///
    /// [unban]: ./struct.Encoder.html#method.unban
    pub async fn ban<'a>(&mut self, username: &str, reason: impl Into<Option<&'a str>>) -> Result {
        match reason.into() {
            Some(reason) => write!(cmd &mut self.writer, "/ban ", username, " ", reason),
            None => write!(cmd &mut self.writer, "/ban ", username),
        }
    }

    /// Clear chat history for all users in this room.
    pub async fn clear(&mut self) -> Result {
        self.command("/clear").await
    }

    /// Change your username color.
    pub async fn color(&mut self, color: Color) -> Result {
        write!(cmd &mut self.writer, "/color ", color.to_string())
    }

    /// Sends the command: data (e.g. /color #FFFFFF)
    pub async fn command(&mut self, data: &str) -> Result {
        write!(cmd &mut self.writer, data)
    }

    /// Triggers a commercial.
    ///
    /// Length (optional) must be a positive number of seconds.
    pub async fn commercial(&mut self, length: impl Into<Option<usize>>) -> Result {
        match length.into() {
            // TODO fast usize to string without an allocation
            Some(length) => write!(cmd &mut self.writer, "/commercial ", length.to_string()),
            None => self.command("/commercial").await,
        }
    }

    /// Reconnects to chat.
    pub async fn disconnect(&mut self) -> Result {
        self.command("/disconnect").await
    }

    /// Enables emote-only mode (only emoticons may be used in chat).
    ///
    /// Use [emote_only_off] to disable.
    ///
    /// [emote_only_off]: ./struct.Encoder.html#method.emote_only_off
    pub async fn emote_only(&mut self) -> Result {
        self.command("/emoteonly").await
    }

    /// Disables emote-only mode.
    pub async fn emote_only_off(&mut self) -> Result {
        self.command("/emoteonlyoff").await
    }

    // TODO use `time` here
    /// Enables followers-only mode (only users who have followed for `duration` may chat).
    ///
    /// Examples: `"30m"`, `"1 week"`, `"5 days 12 hours"`.
    ///
    /// Must be less than 3 months.
    pub async fn followers(&mut self, duration: &str) -> Result {
        write!(cmd &mut self.writer, "/followers ", duration)
    }

    /// Disables followers-only mode.
    pub async fn followers_off(&mut self) -> Result {
        self.command("/followersoff").await
    }

    /// Grant moderator status to a user.
    ///
    /// Use [mods] to list the moderators of this channel.
    ///
    /// [mods]: ./struct.Encoder.html#method.mods
    pub async fn give_mod(&mut self, username: &str) -> Result {
        write!(cmd &mut self.writer, "/mod ", username)
    }

    /// Lists the commands available to you in this room.
    pub async fn help(&mut self) -> Result {
        self.command("/help").await
    }

    /// Host another channel.
    ///
    /// Use [unhost] to unset host mode.
    ///
    /// [unhost]: ./struct.Encoder.html#method.unhost
    pub async fn host(&mut self, channel: &str) -> Result {
        write!(cmd &mut self.writer, "/host ", channel)
    }

    /// Join a channel
    pub async fn join(&mut self, channel: &str) -> Result {
        write!(&mut self.writer, "JOIN ", channel)
    }

    // TODO limit this to 140
    // TODO get rid of the Into<Option<&'a str>> -- why?
    /// Adds a stream marker (with an optional comment, **max 140** characters) at the current timestamp.
    ///
    /// You can use markers in the Highlighter for easier editing.
    pub async fn marker<'a>(&mut self, comment: impl Into<Option<&'a str>>) -> Result {
        match comment.into() {
            Some(comment) => write!(cmd &mut self.writer, "/marker ", comment ),
            None => self.command("/marker").await,
        }
    }

    /// Sends an "emote" message in the third person to the channel
    pub async fn me(&mut self, channel: &str, message: &str) -> Result {
        write!(&mut self.writer, "PRIVMSG ", channel, " :", "/me ", message)
    }

    /// Lists the moderators of this channel.
    pub async fn mods(&mut self) -> Result {
        self.command("/mods").await
    }

    /// Leave a channel
    pub async fn part(&mut self, channel: &str) -> Result {
        write!(&mut self.writer, "PART ", channel)
    }

    /// Request a heartbeat with the provided token
    pub async fn ping(&mut self, token: &str) -> Result {
        write!(&mut self.writer, "PING ", token)
    }

    /// Response to a heartbeat with the provided token
    pub async fn pong(&mut self, token: &str) -> Result {
        write!(&mut self.writer, "PONG :", token)
    }

    /// Send data to a target
    pub async fn privmsg(&mut self, target: &str, data: &str) -> Result {
        write!(&mut self.writer, "PRIVMSG ", target, " :", data)
    }

    /// Enables r9k mode.
    ///
    /// Use [r9k_beta_off] to disable.
    ///
    /// [r9k_beta_off]: ./struct.Encoder.html#method.r9k_beta_off
    pub async fn r9k_beta(&mut self) -> Result {
        self.command("/r9kbeta").await
    }

    /// Disables r9k mode.
    pub async fn r9k_beta_off(&mut self) -> Result {
        self.command("/r9kbetaoff").await
    }

    /// Raid another channel.
    ///
    /// Use [unraid] to cancel the Raid.
    ///
    /// [unraid]: ./struct.Encoder.html#method.unraid
    pub async fn raid(&mut self, channel: &str) -> Result {
        write!(cmd &mut self.writer, "/raid ", channel)
    }

    /// Send a raw IRC-style message
    pub async fn raw(&mut self, raw: impl AsRef<[u8]>) -> Result {
        write!(&mut self.writer, raw)
    }

    // TODO use `time` here
    /// Enables slow mode (limit how often users may send messages).
    ///
    /// Duration (optional, **default=120**) must be a positive number of seconds.
    ///
    /// Use [slow_off] to disable.
    ///
    /// [slow_off]: ./struct.Encoder.html#method.slow_off
    pub async fn slow(&mut self, duration: impl Into<Option<usize>>) -> Result {
        // TODO fast non-allocating usize to &[u8]
        write!(cmd &mut self.writer, "/slow ", duration.into().unwrap_or_else(|| 120).to_string())
    }

    /// Disables slow mode.
    pub async fn slow_off(&mut self) -> Result {
        self.command("/slowoff").await
    }

    /// Enables subscribers-only mode (only subscribers may chat in this channel).
    ///
    /// Use [subscribers_off] to disable.
    ///
    /// [subscribers_off]: ./struct.Encoder.html#methodruct.html#method.subscribers_off
    pub async fn subscribers(&mut self) -> Result {
        self.command("/subscribers").await
    }

    /// Disables subscribers-only mode.
    pub async fn subscribers_off(&mut self) -> Result {
        self.command("/subscribersoff").await
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
    pub async fn timeout<'a, 'b>(
        &mut self,
        username: &str,
        duration: impl Into<Option<&'a str>>,
        message: impl Into<Option<&'b str>>,
    ) -> Result {
        match (duration.into(), message.into()) {
            (Some(dur), Some(reason)) => {
                write!(cmd &mut self.writer, "/timeout ", username, " ", dur, " ", reason)
            }
            (None, Some(reason)) => {
                write!(cmd &mut self.writer, "/timeout ", username, " ", reason)
            }
            (Some(dur), None) => write!(cmd &mut self.writer, "/timeout ", username, " ", dur),
            (None, None) => write!(cmd &mut self.writer, "/timeout ", username),
        }
    }

    /// Removes a ban on a user.
    pub async fn unban(&mut self, username: &str) -> Result {
        write!(cmd &mut self.writer, "/unban ", username)
    }

    /// Stop hosting another channel.
    pub async fn unhost(&mut self) -> Result {
        self.command("/unhost").await
    }

    /// Revoke moderator status from a user.
    ///
    /// Use [mods] to list the moderators of this channel.
    ///
    /// [mods]: ./struct.Encoder.html#methodruct.html#method.mods
    pub async fn unmod(&mut self, username: &str) -> Result {
        write!(cmd &mut self.writer, "/unmod ", username)
    }

    /// Cancel the Raid.
    pub async fn unraid(&mut self) -> Result {
        self.command("/unraid").await
    }

    /// Removes a timeout on a user.
    pub async fn untimeout(&mut self, username: &str) -> Result {
        write!(cmd &mut self.writer, "/untimeout ", username)
    }

    /// Revoke VIP status from a user.
    ///
    /// Use [vips] to list the VIPs of this channel.
    ///
    /// [vips]: ./struct.Encoder.html#methodruct.html#method.vips
    pub async fn unvip(&mut self, username: &str) -> Result {
        write!(cmd &mut self.writer, "/unvip ", username)
    }

    /// Grant VIP status to a user.
    ///
    /// Use [vips] to list the VIPs of this channel.
    ///
    /// [vips]: ./struct.Encoder.html#methodruct.html#method.vips
    pub async fn vip(&mut self, username: &str) -> Result {
        write!(cmd &mut self.writer, "/vip ", username)
    }

    /// Lists the VIPs of this channel.
    pub async fn vips(&mut self) -> Result {
        self.command("/vips").await
    }

    /// Whispers the message to the username.
    pub async fn whisper(&mut self, username: &str, message: &str) -> Result {
        write!(cmd &mut self.writer, "/w ", username, " ", message)
    }
}

impl<W: AsyncWrite + Default> Default for AsyncEncoder<W> {
    fn default() -> Self {
        Self {
            writer: Default::default(),
        }
    }
}

impl<W> std::fmt::Debug for AsyncEncoder<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncEncoder").finish()
    }
}

impl<W: AsyncWrite + Clone> Clone for AsyncEncoder<W> {
    fn clone(&self) -> Self {
        Self {
            writer: self.writer.clone(),
        }
    }
}
