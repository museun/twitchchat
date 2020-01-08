use super::*;

pub async fn write_loop<W>(write: W, mut recv: Receiver) -> Result<Status, Error>
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

/// A writer that allows sending messages to the client
#[derive(Clone)]
pub struct Writer {
    buf: Vec<u8>,
    sender: Sender,
}

impl Writer {
    pub(super) fn new(sender: Sender) -> Self {
        Self {
            buf: vec![],
            sender,
        }
    }

    /// Send a raw message
    pub async fn raw(&mut self, data: &str) -> bool {
        let msg = crate::encode::raw(data);
        self.send_message(msg).await
    }

    /// Join a `channel`
    pub async fn join(&mut self, channel: &str) -> bool {
        let msg = crate::encode::join(channel);
        self.send_message(msg).await
    }

    /// Leave a `channel`
    pub async fn part(&mut self, channel: &str) -> bool {
        let msg = crate::encode::part(channel);
        self.send_message(msg).await
    }

    /// Send a message to a `target`
    pub async fn privmsg(&mut self, target: &str, data: &str) -> bool {
        let msg = crate::encode::privmsg(target, data);
        self.send_message(msg).await
    }

    /// Request a PONG response from the server
    pub async fn ping(&mut self, token: &str) -> bool {
        let msg = crate::encode::ping(token);
        self.send_message(msg).await
    }

    /// Reply to a PING request from the server
    pub async fn pong(&mut self, token: &str) -> bool {
        let msg = crate::encode::pong(token);
        self.send_message(msg).await
    }

    pub async fn ban<'a>(
        &'a mut self,
        username: &'a str,
        reason: impl Into<Option<&'a str>>,
    ) -> bool {
        let msg = crate::encode::ban(username, reason);
        self.send_message(msg).await
    }

    pub async fn clear(&mut self) -> bool {
        let msg = crate::encode::clear();
        self.send_message(msg).await
    }

    pub async fn color(&mut self, color: crate::color::Color) -> bool {
        let msg = crate::encode::color(color);
        self.send_message(msg).await
    }

    pub async fn command(&mut self, data: &str) -> bool {
        let msg = crate::encode::command(data);
        self.send_message(msg).await
    }

    pub async fn commercial(&mut self, length: impl Into<Option<usize>>) -> bool {
        let msg = crate::encode::commercial(length);
        self.send_message(msg).await
    }

    pub async fn disconnect(&mut self) -> bool {
        let msg = crate::encode::disconnect();
        self.send_message(msg).await
    }

    pub async fn emote_only(&mut self) -> bool {
        let msg = crate::encode::emote_only();
        self.send_message(msg).await
    }

    pub async fn emote_only_off(&mut self) -> bool {
        let msg = crate::encode::emote_only_off();
        self.send_message(msg).await
    }

    pub async fn followers(&mut self, duration: &str) -> bool {
        let msg = crate::encode::followers(duration);
        self.send_message(msg).await
    }

    pub async fn followers_off(&mut self) -> bool {
        let msg = crate::encode::followers_off();
        self.send_message(msg).await
    }

    pub async fn give_mod(&mut self, username: &str) -> bool {
        let msg = crate::encode::give_mod(username);
        self.send_message(msg).await
    }

    pub async fn help(&mut self) -> bool {
        let msg = crate::encode::help();
        self.send_message(msg).await
    }

    pub async fn host(&mut self, channel: &str) -> bool {
        let msg = crate::encode::host(channel);
        self.send_message(msg).await
    }

    pub async fn marker<'a>(&'a mut self, comment: impl Into<Option<&'a str>>) -> bool {
        let msg = crate::encode::marker(comment);
        self.send_message(msg).await
    }

    pub async fn me<'a>(&'a mut self, channel: &'a str, message: &'a str) -> bool {
        let msg = crate::encode::me(channel, message);
        self.send_message(msg).await
    }

    pub async fn mods(&mut self) -> bool {
        let msg = crate::encode::mods();
        self.send_message(msg).await
    }

    pub async fn r9k_beta(&mut self) -> bool {
        let msg = crate::encode::r9k_beta();
        self.send_message(msg).await
    }

    pub async fn r9k_beta_off(&mut self) -> bool {
        let msg = crate::encode::r9k_beta_off();
        self.send_message(msg).await
    }

    pub async fn raid(&mut self, channel: &str) -> bool {
        let msg = crate::encode::raid(channel);
        self.send_message(msg).await
    }

    pub async fn slow(&mut self, duration: impl Into<Option<usize>>) -> bool {
        let msg = crate::encode::slow(duration);
        self.send_message(msg).await
    }

    pub async fn slow_off(&mut self) -> bool {
        let msg = crate::encode::slow_off();
        self.send_message(msg).await
    }

    pub async fn subscribers(&mut self) -> bool {
        let msg = crate::encode::subscribers();
        self.send_message(msg).await
    }

    pub async fn subscribers_off(&mut self) -> bool {
        let msg = crate::encode::subscribers_off();
        self.send_message(msg).await
    }

    pub async fn timeout<'a>(
        &'a mut self,
        username: &'a str,
        duration: impl Into<Option<&'a str>>,
        message: impl Into<Option<&'a str>>,
    ) -> bool {
        let msg = crate::encode::timeout(username, duration, message);
        self.send_message(msg).await
    }

    pub async fn unban(&mut self, username: &str) -> bool {
        let msg = crate::encode::unban(username);
        self.send_message(msg).await
    }

    pub async fn unhost(&mut self) -> bool {
        let msg = crate::encode::unhost();
        self.send_message(msg).await
    }

    pub async fn unmod(&mut self, username: &str) -> bool {
        let msg = crate::encode::unmod(username);
        self.send_message(msg).await
    }

    pub async fn unraid(&mut self) -> bool {
        let msg = crate::encode::unraid();
        self.send_message(msg).await
    }

    pub async fn untimeout(&mut self, username: &str) -> bool {
        let msg = crate::encode::untimeout(username);
        self.send_message(msg).await
    }

    pub async fn unvip(&mut self, username: &str) -> bool {
        let msg = crate::encode::unvip(username);
        self.send_message(msg).await
    }

    pub async fn vip(&mut self, username: &str) -> bool {
        let msg = crate::encode::vip(username);
        self.send_message(msg).await
    }

    pub async fn vips(&mut self) -> bool {
        let msg = crate::encode::vips();
        self.send_message(msg).await
    }

    pub async fn whisper<'a>(&'a mut self, username: &'a str, message: &'a str) -> bool {
        let msg = crate::encode::whisper(username, message);
        self.send_message(msg).await
    }

    /// Send this encodable message
    pub async fn send_message<E: crate::Encodable>(&mut self, msg: E) -> bool {
        // TODO use BytesMut here
        crate::sync::encode(&msg, &mut self.buf).expect("encode");
        self.sender
            .send(std::mem::take(&mut self.buf))
            .await
            .is_ok()
    }
}
