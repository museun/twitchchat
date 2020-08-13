use crate::{util::NotifyHandle, AsyncEncoder, Encodable, Sender};

use futures_lite::AsyncWrite;
use std::io::{self};

/// An asynchronous writer.
#[derive(Clone)]
pub struct AsyncWriter<W> {
    inner: AsyncEncoder<W>,
    sender: Sender<()>,
    quit: NotifyHandle,
}

impl<W> std::fmt::Debug for AsyncWriter<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncWriter").finish()
    }
}

impl<W> AsyncWriter<W>
where
    W: AsyncWrite + Unpin + Send + Sync,
{
    pub(crate) fn new(inner: W, sender: Sender<()>, quit: NotifyHandle) -> Self {
        Self {
            inner: AsyncEncoder::new(inner),
            sender,
            quit,
        }
    }

    /// Consume the writer, sending a quit message.
    ///
    /// This will cause the main loop to exit. This blocks until the quit signal has been received.
    pub async fn quit(self) -> io::Result<()> {
        let mut this = self;
        this.encode("QUIT\r\n").await?;
        let _ = this.quit.notify().await;
        log::info!("got shutdown signal");
        Ok(())
    }

    /// Encode this `Encodable` message to the writer.
    ///
    /// This flushes the data before returning
    pub async fn encode<M>(&mut self, msg: M) -> io::Result<()>
    where
        M: Encodable + Send + Sync,
    {
        self.inner.encode(msg).await?;
        let _ = self.sender.send(()).await;
        Ok(())
    }

    /// Encode a slice of `Encodable` messages to the writer.
    ///
    /// This flushes the data before returning
    pub async fn encode_many<M>(&mut self, msgs: &[M]) -> io::Result<()>
    where
        M: Encodable + Send + Sync,
    {
        for msg in msgs {
            self.inner.encode(msg).await?;
            let _ = self.sender.send(()).await;
        }

        Ok(())
    }
}
