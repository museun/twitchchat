use crate::{rate_limit::AsyncBlocker, AsyncEncoder, Encodable, RateLimit, Receiver, Sender};

use futures_lite::AsyncWrite;
use std::{
    io::{self},
    sync::Arc,
};

/// An asynchronous writer that has optional rate limiting.
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

    pub(crate) fn reconfigure<R, B>(&self, rate_limit: R, blocker: B) -> Self
    where
        W: Clone,
        R: Into<Option<RateLimit>>,
        B: AsyncBlocker,
    {
        Self::new(
            self.inner.writer.clone(),
            self.sender.clone(),
            self.should_quit.clone(),
            rate_limit.into(),
            Arc::new(blocker),
        )
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
            if let Some(rate) = &mut self.rate_limit {
                let fut = rate.take_async(&*self.blocker);
                futures_lite::pin!(fut);
                fut.await;
            }
        }

        Ok(())
    }
}
