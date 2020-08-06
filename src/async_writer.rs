use crate::{rate_limit::AsyncBlocker, AsyncEncoder, Encodable, RateLimit, Receiver, Sender};

use async_dup::Arc;
use futures_lite::AsyncWrite;

pub struct AsyncWriter<W> {
    inner: AsyncEncoder<async_dup::Arc<W>>,
    sender: Sender<()>,
    should_quit: Receiver<()>,

    rate_limit: Option<Arc<async_mutex::Mutex<RateLimit>>>,
    blocker: std::sync::Arc<dyn AsyncBlocker>,
}

impl<W> Clone for AsyncWriter<W>
where
    for<'a> &'a W: AsyncWrite + Unpin + Send + Sync,
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
    for<'a> &'a W: AsyncWrite + Unpin + Send + Sync,
{
    /// Create a new Writer
    pub(crate) fn new(
        inner: async_dup::Arc<W>,
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
        M: Encodable,
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
