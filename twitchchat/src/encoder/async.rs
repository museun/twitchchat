use std::{
    io::{Result as IoResult, Write},
    pin::Pin,
    task::{Context, Poll},
};

use futures_lite::{AsyncWrite, AsyncWriteExt};

use crate::Encodable;

/// An asynchronous encoder.
pub struct AsyncEncoder<W> {
    pub(crate) writer: W,
    pos: usize,
    data: Vec<u8>,
}

impl<W> std::fmt::Debug for AsyncEncoder<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncEncoder").finish()
    }
}

impl<W> Clone for AsyncEncoder<W>
where
    W: Clone,
{
    fn clone(&self) -> Self {
        Self {
            writer: self.writer.clone(),
            pos: 0,
            data: vec![],
        }
    }
}

impl<W> Write for AsyncEncoder<W>
where
    W: Write + Send,
{
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> IoResult<()> {
        self.writer.flush()
    }
}

impl<W> AsyncEncoder<W>
where
    W: Write + Send,
{
    /// If the wrapped writer is synchronous, you can use this method to encode the message to it.
    pub fn encode_sync<M>(&mut self, msg: M) -> IoResult<()>
    where
        M: Encodable + Send,
    {
        msg.encode(&mut self.data)?;
        let data = &self.data[self.pos..];

        self.writer.write_all(data)?;
        self.writer.flush()?;

        self.data.clear();
        self.pos = 0;
        Ok(())
    }
}

impl<W> AsyncEncoder<W>
where
    W: AsyncWrite + Send + Unpin,
{
    /// Create a new Encoder over this [`futures::AsyncWrite`] instance
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            pos: 0,
            data: Vec::with_capacity(1024),
        }
    }

    /// Get the inner [`futures::AsyncWrite`] instance out
    ///
    /// This writes and flushes any buffered data before it consumes self.
    pub async fn into_inner(mut self) -> IoResult<W> {
        if self.data.is_empty() {
            return Ok(self.writer);
        }

        let data = std::mem::take(&mut self.data);
        self.writer.write_all(&data).await?;
        self.writer.flush().await?;
        Ok(self.writer)
    }

    /// Encode this [`Encodable`] message to the writer.
    ///
    /// This flushes the data before returning
    pub async fn encode<M>(&mut self, msg: M) -> IoResult<()>
    where
        M: Encodable + Send,
        W: Unpin,
    {
        msg.encode(&mut self.data)?;
        let data = &self.data[self.pos..];

        self.writer.write_all(data).await?;
        self.writer.flush().await?;

        self.data.clear();
        self.pos = 0;
        Ok(())
    }

    // TODO make this stateful

    /// Join a `channel`
    pub async fn join(&mut self, channel: &str) -> IoResult<()> {
        self.encode(crate::commands::join(channel)).await
    }

    /// Leave a `channel`
    pub async fn part(&mut self, channel: &str) -> IoResult<()> {
        self.encode(crate::commands::part(channel)).await
    }

    /// Send a message to a channel
    pub async fn privmsg(&mut self, channel: &str, data: &str) -> IoResult<()> {
        self.encode(crate::commands::privmsg(channel, data)).await
    }
}

impl<W> AsyncWrite for AsyncEncoder<W>
where
    W: AsyncWrite + Unpin + Send,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        ctx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<IoResult<usize>> {
        let mut this = self.as_mut();
        let writer = &mut this.writer;
        futures_lite::pin!(writer);
        writer.poll_write(ctx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<IoResult<()>> {
        let mut this = self.as_mut();
        let writer = &mut this.writer;
        futures_lite::pin!(writer);
        writer.poll_flush(ctx)
    }

    fn poll_close(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<IoResult<()>> {
        let mut this = self.as_mut();
        let writer = &mut this.writer;
        futures_lite::pin!(writer);
        writer.poll_close(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::join;

    #[test]
    fn encoder_async() {
        let fut = async move {
            let mut output = vec![];
            {
                let mut encoder = AsyncEncoder::new(&mut output);

                encoder.encode(join("#museun")).await.unwrap();
                encoder.encode(join("#shaken_bot")).await.unwrap();
            }

            let s = std::str::from_utf8(&output).unwrap();
            assert_eq!(s, "JOIN #museun\r\nJOIN #shaken_bot\r\n");
        };
        futures_lite::future::block_on(fut);
    }
}
