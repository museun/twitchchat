use super::*;
use crate::encode::AsyncEncoder;

use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::AsyncWrite;

/// A writer that allows sending messages to the client
pub type Writer = AsyncEncoder<MpscWriter>;

/// A tokio mpsc based writer
pub struct MpscWriter {
    buffer: Vec<u8>,
    sender: Tx,
}

impl std::fmt::Debug for MpscWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MpscWriter").finish()
    }
}

impl Clone for MpscWriter {
    fn clone(&self) -> Self {
        Self {
            buffer: Vec::new(),
            sender: self.sender.clone(),
        }
    }
}

impl MpscWriter {
    /// Create a new MpscServer from this channel's sender
    pub const fn new(sender: Tx) -> Self {
        Self {
            buffer: Vec::new(),
            sender,
        }
    }
}

impl AsyncWrite for MpscWriter {
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
