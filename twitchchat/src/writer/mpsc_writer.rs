use crate::Encodable;

use futures_lite::AsyncWrite;
use std::{
    io::{self, Write},
    pin::Pin,
    task::{Context, Poll},
};

/// A mpsc-based writer.
///
/// This can be used both a `std::io::Write` instance and a `AsyncWrite` instance.
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
    /// Create a new Writer with this Sender
    pub const fn new(channel: crate::Sender<Vec<u8>>) -> Self {
        Self {
            buf: Vec::new(),
            channel,
        }
    }

    /// Encode this message to the inner channel
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
