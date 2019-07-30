use std::io::{BufRead, BufReader, Read, Write};

use super::{Error, Message};
use crate::irc::Message as IrcMessage;

/// ReadAdapter allows you to provide your own "reader" for the client
pub trait ReadAdapter {
    /// the Inner read type (can be a Unit)
    type Reader;

    /// Tries to read a message, otherwise returns a wrapped erro
    fn read_message(&mut self) -> Result<Message, Error>;

    /// Consume the adapter and returns the inner reader
    fn into_inner(self) -> Self::Reader;
}

/// WriteAdapter allows you to provide your own "writer" for the client
pub trait WriteAdapter {
    /// The inner write type
    type Writer;

    /// The error writing to this type can cause
    type Error;

    /// Tries to write a byte array (a utf-8 encoded string ending with \r\n) to the writer
    fn write_line(&mut self, line: &[u8]) -> Result<(), Self::Error>;

    /// Consume the adapter and returns the inner writer
    fn into_inner(self) -> Self::Writer;
}

/// Default synchronous Reader that uses an std::io::Read implementation
pub struct SyncReadAdapter<R> {
    reader: BufReader<R>,
}

impl<R: Read> SyncReadAdapter<R> {
    /// Create a new SyncReadAdapter from an std::io::Read
    pub fn new(read: R) -> Self {
        Self {
            reader: BufReader::new(read),
        }
    }
}

impl<R: Read> ReadAdapter for SyncReadAdapter<R> {
    type Reader = R;

    fn read_message(&mut self) -> Result<Message, Error> {
        let mut buf = String::new();
        let len = self
            .reader
            .read_line(&mut buf)
            .map_err(|_| Error::CannotRead)?;

        // 0 == EOF
        if len == 0 {
            // TODO: this should be a different error ("disconnected?")
            return Err(Error::CannotRead);
        }

        let buf = buf.trim_end();
        if buf.is_empty() {
            // TODO: this technically isn't an error (empty lines are allowed by the spec)
            return Err(Error::CannotRead);
        }
        log::trace!("<- {}", buf);

        log::trace!("trying to parse message");
        let msg = IrcMessage::parse(&buf).ok_or_else(
            || Error::InvalidMessage(buf.to_string()), //
        )?;
        log::trace!("parsed message: {:?}", msg);

        match &msg {
            IrcMessage::Unknown {
                prefix,
                head,
                args,
                tail,
                ..
            } => {
                use crate::irc::Prefix::*;
                if let (Some(Server { host }), Some(data)) = (prefix, tail) {
                    if head == "NOTICE"
                    && host == "tmi.twitch.tv"
                    && data == "Improperly formatted auth"
                    // excellent
                    && args.get(0).map(|k| k.as_str()) == Some("*")
                    {
                        log::trace!("got a registration error");
                        return Err(Error::InvalidRegistration);
                    }
                }
                Ok(Message::parse(msg))
            }
            _ => Ok(Message::Irc(Box::new(msg))),
        }
    }

    fn into_inner(self) -> Self::Reader {
        self.reader.into_inner()
    }
}

/// Default synchronous Writer that uses an std::io::Write implementation
pub struct SyncWriteAdapter<W> {
    writer: W,
}

impl<W: Write> SyncWriteAdapter<W> {
    /// Create a new SyncWriteAdapter from an std::io::Write
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl<W: Write> WriteAdapter for SyncWriteAdapter<W> {
    type Writer = W;
    type Error = Error;

    fn write_line(&mut self, line: &[u8]) -> Result<(), Self::Error> {
        self.writer
            .write_all(line)
            .and_then(|_| self.writer.write_all(b"\r\n"))
            .and_then(|_| self.writer.flush())
            .map_err(Error::Write)
    }

    fn into_inner(self) -> Self::Writer {
        self.writer
    }
}

/// Create a sync adapater from `std::io::{Read, Write}`
pub fn sync_adapters<R, W>(read: R, write: W) -> (SyncReadAdapter<R>, SyncWriteAdapter<W>)
where
    R: Read,
    W: Write,
{
    (SyncReadAdapter::new(read), SyncWriteAdapter::new(write))
}
