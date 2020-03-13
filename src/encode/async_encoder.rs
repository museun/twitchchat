use super::conv_channel;
use crate::{color::Color, IntoChannel, RateLimit};

use std::sync::Arc;

use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::sync::Mutex;

type Result = std::result::Result<(), crate::Error>;

// TODO the old version had a 'SafeEncode' method
// which cleared the Vec<u8> on error
// is that needed? and should it be baked in here?
// we'd have to rework that macro or something
// to fake specialization on AsyncEncoder<Vec<u8>>

// macro_rules! write {
//     (cmd $($e:expr),* $(,)?) => {{
//         write!(&mut writer, "PRIVMSG jtv :", $($e),*)
//     }};
//     ($($e:expr),* $(,)?) => {{
//         $(writer.append($e).await?;)*
//          writer.end().await.map_err(crate::Error::from)
//     }};
// }

struct ByteWriter<'a, W: AsyncWrite + Unpin> {
    inner: &'a mut W,
}

impl<'a, W: AsyncWrite + Unpin> ByteWriter<'a, W> {
    fn new(inner: &'a mut W) -> Self {
        Self { inner }
    }

    async fn command(self, parts: &[&str]) -> Result {
        self.inner.write_all(b"PRIVMSG jtv :").await?;
        self.parts(parts).await
    }

    async fn parts(self, parts: &[&str]) -> Result {
        for part in parts {
            self.inner.write_all(part.as_bytes()).await?
        }
        self.end().await
    }

    async fn write_bytes(self, data: impl AsRef<[u8]>) -> Result {
        self.inner.write_all(data.as_ref()).await?;
        self.end().await
    }

    async fn end(self) -> Result {
        self.inner.write_all(b"\r\n").await?;
        self.inner.flush().await?;
        Ok(())
    }
}

async fn try_rate_limit(limit: Option<Arc<Mutex<RateLimit>>>) {
    if let Some(limit) = limit {
        limit.lock().await.take().await;
    }
}

/// An async encoder for messages
pub struct AsyncEncoder<W> {
    pub(crate) writer: W,
    pub(crate) rate_limit: Option<Arc<Mutex<RateLimit>>>,
}

impl<W: AsyncWrite> AsyncEncoder<W> {
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
        Self {
            writer,
            rate_limit: None,
        }
    }

    /// Compose this with a rate limiter -- intended for internal use only
    pub(crate) fn with_rate_limiter(self, rate_limit: Arc<Mutex<RateLimit>>) -> Self {
        Self {
            rate_limit: Some(rate_limit),
            ..self
        }
    }
}

impl<W: AsyncWrite + Unpin> AsyncEncoder<W> {
    /// Permanently prevent a user from chatting. Reason is optional and will be
    /// shown to the target user and other moderators.
    ///
    /// Use [unban] to remove a ban.
    ///
    /// [unban]: ./struct.Encoder.html#method.unban
    pub async fn ban<'a>(&mut self, username: &str, reason: impl Into<Option<&'a str>>) -> Result {
        try_rate_limit(self.rate_limit.as_ref().map(Arc::clone)).await;
        let writer = ByteWriter::new(&mut self.writer);
        match reason.into() {
            Some(reason) => writer.command(&["/ban", " ", username, " ", reason]).await,
            None => writer.command(&["/ban", " ", username]).await,
        }
    }

    /// Clear chat history for all users in this room.
    pub async fn clear(&mut self) -> Result {
        self.command("/clear").await
    }

    /// Change your username color.
    pub async fn color(&mut self, color: Color) -> Result {
        try_rate_limit(self.rate_limit.as_ref().map(Arc::clone)).await;
        let writer = ByteWriter::new(&mut self.writer);
        writer.command(&["/color", " ", &color.to_string()]).await
    }

    /// Sends the command: data (e.g. /color #FFFFFF)
    pub async fn command(&mut self, data: &str) -> Result {
        try_rate_limit(self.rate_limit.as_ref().map(Arc::clone)).await;
        let writer = ByteWriter::new(&mut self.writer);
        writer.command(&[data]).await
    }

    /// Triggers a commercial.
    ///
    /// Length (optional) must be a positive number of seconds.
    pub async fn commercial(&mut self, length: impl Into<Option<usize>>) -> Result {
        try_rate_limit(self.rate_limit.as_ref().map(Arc::clone)).await;
        let writer = ByteWriter::new(&mut self.writer);
        match length.into() {
            // TODO fast usize to string without an allocation
            Some(length) => {
                writer
                    .command(&["/commercial", " ", &length.to_string()])
                    .await
            }
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
        try_rate_limit(self.rate_limit.as_ref().map(Arc::clone)).await;
        let writer = ByteWriter::new(&mut self.writer);
        writer.command(&["/followers", " ", duration]).await
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
        try_rate_limit(self.rate_limit.as_ref().map(Arc::clone)).await;
        let writer = ByteWriter::new(&mut self.writer);
        writer.command(&["/mod", " ", username]).await
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
    pub async fn host(&mut self, channel: impl IntoChannel) -> Result {
        try_rate_limit(self.rate_limit.as_ref().map(Arc::clone)).await;
        let writer = ByteWriter::new(&mut self.writer);
        let channel = conv_channel(channel)?;
        writer.command(&["/host", " ", &channel]).await
    }

    /// Join a channel
    pub async fn join(&mut self, channel: impl IntoChannel) -> Result {
        try_rate_limit(self.rate_limit.as_ref().map(Arc::clone)).await;
        let writer = ByteWriter::new(&mut self.writer);
        let channel = conv_channel(channel)?;
        writer.parts(&["JOIN", " ", &channel]).await
    }

    /// Adds a stream marker (with an optional comment, **max 140** characters) at the current timestamp.
    ///
    /// You can use markers in the Highlighter for easier editing.
    ///
    /// If the string exceeds 140 characters then it will be truncated
    pub async fn marker<'a>(&mut self, comment: impl Into<Option<&'a str>>) -> Result {
        try_rate_limit(self.rate_limit.as_ref().map(Arc::clone)).await;
        let writer = ByteWriter::new(&mut self.writer);
        match comment.into() {
            None => self.command("/marker").await,
            Some(marker) if marker.len() <= 140 => writer.command(&["/marker", " ", marker]).await,
            Some(marker) => writer.command(&["/marker", " ", &marker[..140]]).await,
        }
    }

    /// Sends an "emote" message in the third person to the channel
    pub async fn me(&mut self, channel: impl IntoChannel, message: &str) -> Result {
        try_rate_limit(self.rate_limit.as_ref().map(Arc::clone)).await;
        let writer = ByteWriter::new(&mut self.writer);
        let channel = conv_channel(channel)?;
        writer
            .parts(&["PRIVMSG", " ", &channel, " :", "/me", " ", message])
            .await
    }

    /// Lists the moderators of this channel.
    pub async fn mods(&mut self) -> Result {
        self.command("/mods").await
    }

    /// Leave a channel
    pub async fn part(&mut self, channel: impl IntoChannel) -> Result {
        try_rate_limit(self.rate_limit.as_ref().map(Arc::clone)).await;
        let writer = ByteWriter::new(&mut self.writer);
        let channel = conv_channel(channel)?;
        writer.parts(&["PART", " ", &channel]).await
    }

    /// Request a heartbeat with the provided token
    pub async fn ping(&mut self, token: &str) -> Result {
        try_rate_limit(self.rate_limit.as_ref().map(Arc::clone)).await;
        let writer = ByteWriter::new(&mut self.writer);
        writer.parts(&["PING", " ", token]).await
    }

    /// Response to a heartbeat with the provided token
    pub async fn pong(&mut self, token: &str) -> Result {
        try_rate_limit(self.rate_limit.as_ref().map(Arc::clone)).await;
        let writer = ByteWriter::new(&mut self.writer);
        writer.parts(&["PONG", " :", token]).await
    }

    /// Send data to a target
    pub async fn privmsg(&mut self, target: &str, data: &str) -> Result {
        try_rate_limit(self.rate_limit.as_ref().map(Arc::clone)).await;
        let writer = ByteWriter::new(&mut self.writer);
        writer.parts(&["PRIVMSG", " ", target, " :", data]).await
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
    pub async fn raid(&mut self, channel: impl IntoChannel) -> Result {
        try_rate_limit(self.rate_limit.as_ref().map(Arc::clone)).await;
        let writer = ByteWriter::new(&mut self.writer);
        let channel = conv_channel(channel)?;
        writer.command(&["/raid", " ", &channel]).await
    }

    /// Send a raw IRC-style message
    pub async fn raw(&mut self, raw: impl AsRef<[u8]>) -> Result {
        try_rate_limit(self.rate_limit.as_ref().map(Arc::clone)).await;
        let writer = ByteWriter::new(&mut self.writer);
        writer.write_bytes(raw).await
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
        try_rate_limit(self.rate_limit.as_ref().map(Arc::clone)).await;
        let writer = ByteWriter::new(&mut self.writer);
        // TODO fast non-allocating usize to &[u8]
        writer
            .command(&[
                "/slow",
                " ",
                &duration.into().unwrap_or_else(|| 120).to_string(),
            ])
            .await
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
        try_rate_limit(self.rate_limit.as_ref().map(Arc::clone)).await;
        let writer = ByteWriter::new(&mut self.writer);
        match (duration.into(), message.into()) {
            (Some(dur), Some(reason)) => {
                writer
                    .command(&["/timeout", " ", username, " ", dur, " ", reason])
                    .await //
            }
            (None, Some(reason)) => {
                writer
                    .command(&["/timeout", " ", username, " ", reason])
                    .await //
            }
            (Some(dur), None) => {
                writer.command(&["/timeout", " ", username, " ", dur]).await //
            }
            (None, None) => {
                writer.command(&["/timeout", " ", username]).await //
            }
        }
    }

    /// Removes a ban on a user.
    pub async fn unban(&mut self, username: &str) -> Result {
        try_rate_limit(self.rate_limit.as_ref().map(Arc::clone)).await;
        let writer = ByteWriter::new(&mut self.writer);
        writer.command(&["/unban", " ", username]).await
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
        try_rate_limit(self.rate_limit.as_ref().map(Arc::clone)).await;
        let writer = ByteWriter::new(&mut self.writer);
        writer.command(&["/unmod", " ", username]).await
    }

    /// Cancel the Raid.
    pub async fn unraid(&mut self) -> Result {
        self.command("/unraid").await
    }

    /// Removes a timeout on a user.
    pub async fn untimeout(&mut self, username: &str) -> Result {
        try_rate_limit(self.rate_limit.as_ref().map(Arc::clone)).await;
        let writer = ByteWriter::new(&mut self.writer);
        writer.command(&["/untimeout", " ", username]).await
    }

    /// Revoke VIP status from a user.
    ///
    /// Use [vips] to list the VIPs of this channel.
    ///
    /// [vips]: ./struct.Encoder.html#methodruct.html#method.vips
    pub async fn unvip(&mut self, username: &str) -> Result {
        try_rate_limit(self.rate_limit.as_ref().map(Arc::clone)).await;
        let writer = ByteWriter::new(&mut self.writer);
        writer.command(&["/unvip", " ", username]).await
    }

    /// Grant VIP status to a user.
    ///
    /// Use [vips] to list the VIPs of this channel.
    ///
    /// [vips]: ./struct.Encoder.html#methodruct.html#method.vips
    pub async fn vip(&mut self, username: &str) -> Result {
        try_rate_limit(self.rate_limit.as_ref().map(Arc::clone)).await;
        let writer = ByteWriter::new(&mut self.writer);
        writer.command(&["/vip", " ", username]).await
    }

    /// Lists the VIPs of this channel.
    pub async fn vips(&mut self) -> Result {
        self.command("/vips").await
    }

    /// Whispers the message to the username.
    pub async fn whisper(&mut self, username: &str, message: &str) -> Result {
        try_rate_limit(self.rate_limit.as_ref().map(Arc::clone)).await;
        let writer = ByteWriter::new(&mut self.writer);
        writer.command(&["/w", " ", username, " ", message]).await
    }
}

impl<W: AsyncWrite + Default> Default for AsyncEncoder<W> {
    fn default() -> Self {
        Self {
            writer: Default::default(),
            rate_limit: None,
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
            rate_limit: self.rate_limit.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn rate_limit() {
        use futures::prelude::*;

        tokio::time::pause();

        let rate_limit = Default::default();
        let mut writers = std::iter::repeat(())
            .take(10)
            .map(|_| AsyncEncoder::new(vec![]))
            .map(|e| e.with_rate_limiter(Arc::clone(&rate_limit)))
            .collect::<Vec<_>>();

        // `20` per `30` seconds
        for i in 0..10 {
            for (j, writer) in writers.iter_mut().enumerate().map(|(j, w)| (j + 1, w)) {
                let fut = writer.ping("asdf").now_or_never();
                if i * 10 + j > 20 {
                    assert!(fut.is_none(), "{},{}", i, j);
                    continue;
                }

                assert!(fut.is_some(), "{},{}", i, j);
                assert!(fut.unwrap().is_ok(), "{},{}", i, j);
            }
        }

        for writer in &mut writers {
            assert!(writer.pong("asdf").now_or_never().is_none())
        }

        tokio::time::advance(std::time::Duration::from_secs(31)).await;
        for writer in &mut writers {
            assert!(writer.pong("asdf").now_or_never().is_some())
        }
    }
}
