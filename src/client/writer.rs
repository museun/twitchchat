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
