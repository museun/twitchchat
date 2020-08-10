use crate::{rate_limit::AsyncBlocker, AsyncEncoder, Encodable, RateLimit, Receiver, Sender};

use async_dup::Arc;
use futures_lite::AsyncWrite;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

pub struct MpscWriter {
    buf: Vec<u8>,
    channel: crate::Sender<Vec<u8>>,
}

impl MpscWriter {
    pub fn new(channel: crate::Sender<Vec<u8>>) -> Self {
        Self {
            buf: Vec::new(),
            channel,
        }
    }
}

impl AsyncWrite for MpscWriter {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let mut this = self.as_mut();
        this.buf.extend_from_slice(buf);
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let mut this = self.as_mut();
        let data = std::mem::take(&mut this.buf);
        match this.channel.try_send(data) {
            Ok(()) => Poll::Ready(Ok(())),
            // this should never happen, but place the 'data' back into self and have it try again
            Err(crate::channel::TrySendError::Full(data)) => {
                this.buf = data;
                Poll::Pending
            }
            Err(crate::channel::TrySendError::Closed(..)) => {
                let kind = std::io::ErrorKind::UnexpectedEof;
                let err = std::io::Error::new(kind, "writer was closed");
                Poll::Ready(Err(err))
            }
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        self.poll_flush(cx)
    }
}

pub struct AsyncWriter<W> {
    inner: AsyncEncoder<W>,
    sender: Sender<()>,
    should_quit: Receiver<()>,

    rate_limit: Option<Arc<async_mutex::Mutex<RateLimit>>>,
    blocker: std::sync::Arc<dyn AsyncBlocker>,
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
    /// Create a new Writer
    pub(crate) fn new(
        inner: W,
        sender: Sender<()>,
        should_quit: Receiver<()>,
        rate_limit: impl Into<Option<RateLimit>>,
        blocker: impl AsyncBlocker,
    ) -> Self {
        Self {
            inner: AsyncEncoder::new(inner),
            sender,
            rate_limit: rate_limit.into().map(async_mutex::Mutex::new).map(Arc::new),
            blocker: std::sync::Arc::new(blocker),
            should_quit,
        }
    }

    /// Consume the writer, sending a quit message.
    ///
    /// This will cause the main loop to exit. This blocks until the quit signal has been received.
    pub async fn quit(mut self) -> std::io::Result<()> {
        self.encode("QUIT\r\n").await?;
        let _ = self.should_quit.recv().await;
        log::info!("got shutdown signal");
        Ok(())
    }

    /// Encode this `Encodable` message to the writer.
    ///
    /// This flushes the data before returning
    pub async fn encode<M>(&mut self, msg: M) -> std::io::Result<()>
    where
        M: Encodable + Send + Sync,
    {
        self.inner.encode(msg).await?;
        let _ = self.sender.send(()).await;

        if let Some(rate) = &self.rate_limit {
            let mut lock = rate.lock().await;
            let fut = lock.take_async(&*self.blocker);
            futures_lite::pin!(fut);
            fut.await;
        }
        Ok(())
    }
}
