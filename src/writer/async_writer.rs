use crate::{AsyncEncoder, Encodable, Sender};

use futures_lite::AsyncWrite;
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
    W: AsyncWrite + Unpin + Send + Sync,
{
    pub(crate) fn new(inner: W, activity_tx: Sender<()>) -> Self {
        Self {
            inner: AsyncEncoder::new(inner),
            activity_tx,
        }
    }

    /// Encode this `Encodable` message to the writer.
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

    /// Encode a slice of `Encodable` messages to the writer.
    pub async fn encode_many<'a, I, M>(&mut self, msgs: I) -> io::Result<()>
    where
        I: IntoIterator<Item = &'a M> + Send + Sync + 'a,
        M: Encodable + Send + Sync + 'a,
    {
        for msg in msgs {
            self.encode(msg).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn is_that_right() {
        let (tx, rx) = crate::channel::bounded(10);

        let (_a, _b) = crate::channel::unbounded();
        let w = crate::writer::MpscWriter::new(tx);
        let mut w = AsyncWriter::new(w, _a);

        use crate::commands::*;
        let fut = async move {
            w.encode_many(&[raw("hello"), raw("world")]).await.unwrap();
            w.encode_many(vec![&raw("hello"), &raw("world")])
                .await
                .unwrap();
            // w.encode_many([&raw("hello"), &raw("world")]).await.unwrap();
        };

        futures_lite::future::block_on(fut);

        while let Some(t) = rx.try_recv() {
            eprintln!("{}", std::str::from_utf8(&*t).unwrap().escape_debug());
        }
    }
}
