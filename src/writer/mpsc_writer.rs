use crate::encoder::Encodable;

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
    channel: crate::Sender<Box<[u8]>>,
}

impl std::fmt::Debug for MpscWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MpscWriter").finish()
    }
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
    pub const fn new(channel: crate::Sender<Box<[u8]>>) -> Self {
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

    fn split_buf(&mut self) -> Option<Box<[u8]>> {
        let end = match self.buf.iter().position(|&c| c == b'\n') {
            Some(p) if self.buf.get(p - 1) == Some(&b'\r') => p,
            _ => return None,
        };

        // include the \n
        let mut tail = self.buf.split_off(end + 1);
        std::mem::swap(&mut self.buf, &mut tail);
        Some(tail.into_boxed_slice())
    }

    fn inner_flush(&mut self) -> std::io::Result<()> {
        use crate::channel::TrySendError;

        let tail = match self.split_buf() {
            Some(tail) => tail,
            None => {
                log::warn!("cannot flush an incomplete buffer");
                return Ok(());
            }
        };

        match self.channel.try_send(tail) {
            Ok(..) => Ok(()),
            Err(TrySendError::Closed(..)) => Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "writer was closed",
            )),
            Err(TrySendError::Full(..)) => unreachable!(),
        }
    }
}

impl Write for MpscWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buf.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner_flush()
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
        Poll::Ready(this.inner_flush())
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.poll_flush(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mpsc_empty_flush() {
        let (tx, rx) = crate::channel::bounded(1);
        let mut m = MpscWriter::new(tx);
        assert!(m.flush().is_ok());
        assert!(rx.try_recv().is_none());

        let _ = m.write(b"asdf").unwrap();
        assert!(m.flush().is_ok());
        assert!(rx.try_recv().is_none());

        let _ = m.write(b"\r\n").unwrap();
        assert!(m.flush().is_ok());
        assert_eq!(&*rx.try_recv().unwrap(), b"asdf\r\n");

        assert!(m.flush().is_ok());
        assert!(rx.try_recv().is_none());
    }
}
