use super::DecodeError;

use crate::{
    irc::IrcMessage,
    wait_for::{wait_inner, Event, State},
    IntoOwned as _,
};

use std::{
    collections::VecDeque,
    io::{BufRead, BufReader, Read},
};

/// A decoder over [`std::io::Read`] that produces [`IrcMessage`]s
///
/// This will return an [`DecodeError::Eof`] when reading manually.
///
/// When reading it as an iterator, `Eof` will signal the end of the iterator (e.g. `None`)
pub struct Decoder<R> {
    // TODO don't use this. it'll be bad with a non-blocking stream
    reader: BufReader<R>,
    buf: Vec<u8>,
    back_queue: VecDeque<IrcMessage<'static>>,
}

impl<R> Decoder<R>
where
    R: Read,
{
    /// Create a new Decoder from this [`std::io::Read`] instance
    pub fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
            buf: Vec::with_capacity(1024),
            back_queue: VecDeque::new(),
        }
    }

    /// Read the next message.
    ///
    /// This returns a borrowed [`IrcMessage`] which is valid until the next Decoder call is made.
    ///
    /// If you just want an owned one, use the [`Decoder`] as an iterator. e.g. dec.next().
    pub fn read_message(&mut self) -> Result<IrcMessage<'_>, DecodeError> {
        if let Some(msg) = self.back_queue.pop_front() {
            return Ok(msg);
        }

        self.buf.clear();
        let n = self
            .reader
            .read_until(b'\n', &mut self.buf)
            .map_err(DecodeError::Io)?;
        if n == 0 {
            return Err(DecodeError::Eof);
        }

        let str = std::str::from_utf8(&self.buf[..n]).map_err(DecodeError::InvalidUtf8)?;

        // this should only ever parse 1 message
        crate::irc::parse_one(str)
            .map_err(DecodeError::ParseError)
            .map(|(_, msg)| msg)
    }

    // TODO this should respond to PINGs
    pub fn wait_for(
        &mut self,
        event: Event,
    ) -> Result<(IrcMessage<'static>, Vec<IrcMessage<'static>>), DecodeError> {
        let mut missed = vec![];
        loop {
            match wait_inner(self.read_message(), event)? {
                State::Done(msg) => break Ok((msg.into_owned(), missed)),
                State::Requeue(msg) => missed.push(msg.into_owned()),
                // TODO this should actually do parking not yielding
                State::Yield => std::thread::yield_now(),
            }
        }
    }

    /// Returns an iterator over messages.
    ///
    /// This will produce `Result`s of `IrcMessages` until an EOF is received
    pub fn iter(&mut self) -> &mut Self {
        self
    }

    /// Consume the decoder returning the inner Reader
    pub fn into_inner(self) -> R {
        self.reader.into_inner()
    }
}

/// This will produce `Result<IrcMessage<'static>, DecodeError>` until an `Eof` is received
impl<R: Read> Iterator for Decoder<R> {
    type Item = Result<IrcMessage<'static>, DecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            break match self.read_message() {
                Ok(msg) => Some(Ok(msg.into_owned())),
                Err(DecodeError::Eof) => None,

                // block until we get a message
                Err(DecodeError::Io(err)) if crate::util::is_blocking_error(&err) => continue,

                Err(err) => Some(Err(err)),
            };
        }
    }
}

impl<R> Extend<IrcMessage<'static>> for Decoder<R> {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = IrcMessage<'static>>,
    {
        self.back_queue.extend(iter)
    }
}

impl<R> std::fmt::Debug for Decoder<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Decoder").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_sync() {
        let data = b"hello\r\nworld\r\ntesting this\r\nand another thing\r\n".to_vec();
        let mut reader = std::io::Cursor::new(data);

        // reading from the iterator won't produce the EOF
        let v = Decoder::new(&mut reader)
            .iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        // no EOF
        assert_eq!(v.len(), 4);

        reader.set_position(0);
        // manually reading should produce an EOF
        let mut dec = Decoder::new(reader);
        for _ in 0..4 {
            dec.read_message().unwrap();
        }
        assert!(matches!(dec.read_message().unwrap_err(), DecodeError::Eof))
    }
}
