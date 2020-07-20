use crate::encode::AsyncEncoder;

use async_channel::TrySendError;
use futures_lite::AsyncWrite;
use std::pin::Pin;
use std::task::{Context, Poll};

type Tx<T = Vec<u8>> = async_channel::Sender<T>;

/// A writer that allows sending messages to the client
// TODO this was renamed
pub type AsyncWriter = AsyncEncoder<AsyncMpscWriter>;

/// A tokio mpsc based writer
pub struct AsyncMpscWriter {
    buffer: Vec<u8>,
    sender: Tx,
}

impl std::fmt::Debug for AsyncMpscWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncMpscWriter").finish()
    }
}

impl Clone for AsyncMpscWriter {
    fn clone(&self) -> Self {
        Self {
            buffer: Vec::new(),
            sender: self.sender.clone(),
        }
    }
}

impl AsyncMpscWriter {
    /// Create a new AsyncMpscWriter from this channel's sender
    pub const fn new(sender: Tx) -> Self {
        Self {
            buffer: Vec::new(),
            sender,
        }
    }
}

impl AsyncWrite for AsyncMpscWriter {
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

        let this = self.get_mut();
        let data = std::mem::take(&mut this.buffer);

        match this.sender.try_send(data) {
            Ok(..) => Poll::Ready(Ok(())),
            Err(TrySendError::Closed(..)) => {
                let err = Err(Error::new(ErrorKind::Other, "client disconnected"));
                Poll::Ready(err)
            }
            _ => unreachable!(),
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}
