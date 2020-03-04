use super::*;
use crate::encode::AsyncEncoder;

use std::pin::Pin;
use std::task::{Context, Poll};

pub(super) async fn write_loop<W>(
    write: W,
    mut rate: RateLimit,
    mut recv: Rx,
) -> Result<Status, Error>
where
    W: AsyncWrite + Send + Sync + Unpin + 'static,
{
    let mut writer = tokio::io::BufWriter::new(write);
    while let Some(data) = recv.next().await {
        let _ = rate.take().await;
        log::trace!("> {}", std::str::from_utf8(&data).unwrap().escape_debug());
        writer.write_all(&data).await?;
        writer.flush().await?
    }
    Ok(Status::Eof)
}

/// A writer that allows sending messages to the client
pub type Writer = AsyncEncoder<DisjointWriter>;

pub struct DisjointWriter {
    buffer: Vec<u8>,
    sender: Tx,
}

impl std::fmt::Debug for DisjointWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DisjointWriter").finish()
    }
}

impl Clone for DisjointWriter {
    fn clone(&self) -> Self {
        Self {
            buffer: vec![],
            sender: self.sender.clone(),
        }
    }
}

impl DisjointWriter {
    pub(super) fn new(sender: Tx) -> Self {
        Self {
            buffer: vec![],
            sender,
        }
    }
}

impl AsyncWrite for DisjointWriter {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        self.get_mut().buffer.extend_from_slice(buf);
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        use std::io::{Error, ErrorKind};
        use tokio::sync::mpsc::error;

        let this = self.get_mut();
        let data = std::mem::take(&mut this.buffer);

        match this.sender.try_send(data) {
            Ok(..) => Poll::Ready(Ok(())),
            Err(error::TrySendError::Closed(..)) => {
                let err = Err(Error::new(ErrorKind::Other, "client disconnected"));
                Poll::Ready(err)
            }
            _ => unreachable!(),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}
