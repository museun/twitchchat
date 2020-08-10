use crate::{rate_limit::AsyncBlocker, AsyncEncoder, Encodable, RateLimit, Receiver, Sender};

use futures_lite::AsyncWrite;
use std::{
    io::{self, Write},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

pub struct MpscWriter {
    buf: Vec<u8>,
    channel: crate::Sender<Vec<u8>>,
}

impl Clone for MpscWriter {
    fn clone(&self) -> MpscWriter {
        Self {
            buf: Vec::new(),
            channel: self.channel.clone(),
        }
    }
}

impl MpscWriter {
    pub const fn new(channel: crate::Sender<Vec<u8>>) -> Self {
        Self {
            buf: Vec::new(),
            channel,
        }
    }

    pub fn encode<M>(&mut self, msg: M) -> io::Result<()>
    where
        M: Encodable + Send,
    {
        msg.encode(&mut self.buf)?;
        self.flush()
    }
}

impl Write for MpscWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buf.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let buf = std::mem::take(&mut self.buf);
        match self.channel.try_send(buf) {
            Ok(..) => Ok(()),
            Err(crate::channel::TrySendError::Closed(..)) => Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "writer was closed",
            )),
            // this should never happen, but place the 'data' back into self and
            // have it try again
            Err(crate::channel::TrySendError::Full(data)) => {
                self.buf = data;
                Ok(())
            }
        }
    }
}

impl AsyncWrite for MpscWriter {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let mut this = self.as_mut();
        this.buf.extend_from_slice(buf);
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let mut this = self.as_mut();
        let data = std::mem::take(&mut this.buf);
        match this.channel.try_send(data) {
            Ok(()) => Poll::Ready(Ok(())),
            // this should never happen, but place the 'data' back into self and
            // have it try again
            Err(crate::channel::TrySendError::Full(data)) => {
                this.buf = data;
                Poll::Pending
            }
            Err(crate::channel::TrySendError::Closed(..)) => {
                let kind = io::ErrorKind::UnexpectedEof;
                let err = io::Error::new(kind, "writer was closed");
                Poll::Ready(Err(err))
            }
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.poll_flush(cx)
    }
}

pub struct AsyncWriter<W> {
    inner: AsyncEncoder<W>,
    sender: Sender<()>,
    should_quit: Receiver<()>,

    rate_limit: Option<RateLimit>,
    blocker: Arc<dyn AsyncBlocker>,
}

impl<W> Clone for AsyncWriter<W>
where
    W: AsyncWrite + Unpin + Send + Sync + Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            sender: self.sender.clone(),
            rate_limit: self.rate_limit.clone(),
            blocker: self.blocker.clone(),
            should_quit: self.should_quit.clone(),
        }
    }
}

impl<W> AsyncWriter<W>
where
    W: AsyncWrite + Unpin + Send + Sync,
{
    pub(crate) fn new<R, B>(
        inner: W,
        sender: Sender<()>,
        should_quit: Receiver<()>,
        rate_limit: R,
        blocker: B,
    ) -> Self
    where
        R: Into<Option<RateLimit>>,
        B: AsyncBlocker,
    {
        Self {
            inner: AsyncEncoder::new(inner),
            sender,
            rate_limit: rate_limit.into(),
            blocker: Arc::new(blocker),
            should_quit,
        }
    }

    /// Clone this writer with a new rate limiter
    pub fn clone_with_new_rate_limit<R>(&self, rate_limit: R) -> Self
    where
        W: Clone,
        R: Into<Option<RateLimit>> + Send + Sync,
    {
        Self {
            rate_limit: rate_limit.into(),
            ..self.clone()
        }
    }

    /// Overwrites the rate limiter for this writer and any future writer cloned from this.
    pub fn set_rate_limit<R>(&mut self, rate_limit: R)
    where
        R: Into<Option<RateLimit>> + Send + Sync,
    {
        let rate_limit = rate_limit.into();
        self.rate_limit = rate_limit;
    }

    /// Consume the writer, sending a quit message.
    ///
    /// This will cause the main loop to exit. This blocks until the quit signal has been received.
    pub async fn quit(mut self) -> io::Result<()> {
        self.encode("QUIT\r\n").await?;
        let _ = self.should_quit.recv().await;
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

        if let Some(rate) = &mut self.rate_limit {
            let fut = rate.take_async(&*self.blocker);
            futures_lite::pin!(fut);
            fut.await;
        }
        Ok(())
    }
}
