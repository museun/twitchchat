use crate::channel::Sender;
use crate::encoder::AsyncEncoder;
use crate::Encodable;

use futures_lite::AsyncWrite;
use io::Write;
use std::io::{self};

/// An asynchronous writer.
#[derive(Clone)]
pub struct AsyncWriter<W> {
    inner: AsyncEncoder<W>,
    activity_tx: Sender<()>,
}

impl<W> std::fmt::Debug for AsyncWriter<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncWriter").finish()
    }
}

impl<W> AsyncWriter<W>
where
    W: Write + Send + Sync,
{
    /// If the wrapped writer is synchronous, you can use this method to encode the message to it.
    pub fn encode_sync<M>(&mut self, msg: M) -> io::Result<()>
    where
        M: Encodable + Send + Sync,
    {
        self.inner.encode_sync(msg)
    }
}

impl<W> Write for AsyncWriter<W>
where
    W: Write + Send + Sync,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<W> AsyncWriter<W>
where
    W: AsyncWrite + Unpin + Send + Sync,
{
    pub(crate) fn new(inner: W, activity_tx: Sender<()>) -> Self {
        Self {
            inner: AsyncEncoder::new(inner),
            activity_tx,
        }
    }

    /// Encode this [Encodable] message to the writer.
    pub async fn encode<M>(&mut self, msg: M) -> io::Result<()>
    where
        M: Encodable + Send + Sync,
    {
        self.inner.encode(msg).await?;
        if self.activity_tx.send(()).await.is_err() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Runner has closed its receiver",
            ));
        }
        Ok(())
    }

    /// Encode a slice of [Encodable] messages to the writer.
    pub async fn encode_many<'a, I, M>(&mut self, msgs: I) -> io::Result<()>
    where
        I: IntoIterator<Item = &'a M> + Send + Sync + 'a,
        I::IntoIter: Send + Sync,
        M: Encodable + Send + Sync + 'a,
    {
        for msg in msgs {
            self.encode(msg).await?;
        }
        Ok(())
    }
}
