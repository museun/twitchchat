use std::io::Result as IoResult;

use futures::{Sink, SinkExt as _};

use crate::Encodable;

/// A `Encoder` wraps a [`futures::Sink`] and provides a way to use [`Encodable`] with it.
///
pub struct SinkEncoder<IO, M> {
    sink: IO,
    buf: Vec<u8>,
    _marker: std::marker::PhantomData<M>,
}

impl<IO, M> SinkEncoder<IO, M>
where
    IO: Sink<M> + Unpin,
    IO::Error: std::error::Error + Send + Sync + 'static,
    M: From<String>,
{
    /// Create a new `SinkEncoder` from an existing [`Sink`]
    pub fn new(sink: IO) -> Self {
        Self {
            sink,
            buf: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Encode this [`Encodable`] message to the writer.
    pub async fn encode(&mut self, msg: impl Encodable) -> IoResult<()> {
        use std::io::{Error, ErrorKind};
        fn read_str(d: &[u8]) -> IoResult<&str> {
            std::str::from_utf8(d).map_err(|err| Error::new(ErrorKind::InvalidData, err))
        }

        self.buf.clear();
        msg.encode(&mut self.buf)?;

        macro_rules! send_it {
            ($msg:expr) => {
                self.sink
                    .send($msg.to_string().into())
                    .await
                    .map_err(|err| Error::new(ErrorKind::Other, err))
            };
        }

        if !self.buf.ends_with(b"\n") {
            send_it!(read_str(&self.buf)?)?;
            return send_it!("\n");
        }

        let mut msg = &*self.buf;
        while let Some(p) = msg
            .iter()
            .position(|&c| c == b'\n')
            .filter(|&c| c < msg.len() && c != 0)
        {
            let (left, right) = msg.split_at(p + 1);
            msg = right;
            send_it!(read_str(left)?)?;
        }

        Ok(())
    }

    /// Join a `channel`
    pub async fn join(&mut self, channel: &str) -> IoResult<()> {
        self.encode(crate::commands::join(channel)).await
    }

    /// Leave a `channel`
    pub async fn part(&mut self, channel: &str) -> IoResult<()> {
        self.encode(crate::commands::part(channel)).await
    }

    /// Send a message to a channel
    pub async fn privmsg(&mut self, channel: &str, data: &str) -> IoResult<()> {
        self.encode(crate::commands::privmsg(channel, data)).await
    }
}

impl<IO, M> std::fmt::Debug for SinkEncoder<IO, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SinkEncoder").finish()
    }
}
