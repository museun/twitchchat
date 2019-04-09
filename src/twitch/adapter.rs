use log::*;
use std::io::{BufRead, BufReader, Read, Write};

use super::{Capability, Error, Message, Writer};
use crate::irc::types::Message as IrcMessage;

/// An error returned by the ReadAdapter
#[derive(Debug)]
pub enum ReadError<E: std::fmt::Debug + std::fmt::Display + std::error::Error> {
    /// An invalid message was read, containing the bad message
    InvalidMessage(String),
    /// Capability required, a list of which ones are required
    CapabilityRequired(Vec<Capability>),
    /// An inner error
    Inner(E),
}

impl<E: std::fmt::Debug + std::fmt::Display + std::error::Error> std::fmt::Display
    for ReadError<E>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadError::InvalidMessage(raw) => {
                write!(f, "invalid message, from '{}' (trimmed)", raw.trim())
            }
            ReadError::CapabilityRequired(list) => {
                let caps = list
                    .iter()
                    .map(|f| format!("{:?}", f))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{} are required to do that", caps)
            }
            ReadError::Inner(err) => write!(f, "{}", err),
        }
    }
}

impl<E: std::fmt::Debug + std::fmt::Display + std::error::Error> std::error::Error
    for ReadError<E>
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // TODO this
        None
    }
}

impl From<Error> for ReadError<Error> {
    fn from(err: Error) -> Self {
        match err {
            Error::InvalidMessage(raw) => ReadError::InvalidMessage(raw),
            Error::CapabilityRequired(list) => ReadError::CapabilityRequired(list),
            e => ReadError::Inner(e),
        }
    }
}

/// ReadAdapter allows one to write different 'readers' for the twitch Client
pub trait ReadAdapter<W> {
    /// the Innr read type (can be a Unit)
    type Reader;
    /// An Error that can be returned when trying to read a message
    type Error: std::fmt::Debug + std::fmt::Display + std::error::Error;

    /// Give an instance of the Writer to the reader
    fn give_writer(&mut self, writer: Writer<W>);

    /// Tries to read a message, otherwise returns a wrapped erro
    fn read_message(&mut self) -> Result<Message, ReadError<Self::Error>>;

    /// Consume the adapter and returns the inner reader
    fn into_inner(self) -> Self::Reader;
}

/// Default Sync Reader that uses an std::io::Read implementation
pub struct SyncReadAdapter<R, W> {
    reader: BufReader<R>,
    writer: Option<Writer<W>>,
}

impl<R: Read, W> SyncReadAdapter<R, W> {
    /// Create a new SyncReadAdapter from an std::io::Read
    pub fn new(read: R) -> Self {
        Self {
            reader: BufReader::new(read),
            writer: None,
        }
    }
}

impl<R: Read, W: Write> ReadAdapter<W> for SyncReadAdapter<R, W> {
    type Reader = R;
    type Error = Error;

    fn give_writer(&mut self, writer: Writer<W>) {
        let _ = self.writer.replace(writer);
    }

    fn read_message(&mut self) -> Result<Message, ReadError<Self::Error>> {
        let mut buf = String::new();
        let len = self.reader.read_line(&mut buf).map_err(Error::Read)?;
        // 0 == EOF
        if len == 0 {
            return Err(Error::CannotRead.into());
        }

        let buf = buf.trim_end();
        if buf.is_empty() {
            return Err(Error::CannotRead.into());
        }

        trace!("<- {}", buf);

        trace!("trying to parse message");
        let msg = IrcMessage::parse(&buf) //
            .ok_or_else(|| Error::InvalidMessage(buf.to_string()))?;
        trace!("parsed message");

        // handle PINGs automatically
        if let IrcMessage::Ping { token } = &msg {
            self.writer
                .as_ref()
                .expect("writer must have been set")
                .write_line(&format!("PONG :{}", token))?;
        }

        // sanity check, doing it here instead of after its been re-parsed to fail early
        if let IrcMessage::Unknown {
            prefix,
            head,
            args,
            tail,
            ..
        } = &msg
        {
            if let (Some(crate::irc::types::Prefix::Server { host }), Some(data)) = (prefix, tail) {
                if head == "NOTICE"
                    && host == "tmi.twitch.tv"
                    && data == "Improperly formatted auth"
                    // excellent
                    && args.get(0) == Some(&"*".into())
                {
                    trace!("got a registartion error");
                    return Err(Error::InvalidRegistration.into());
                }
            }
        }

        Ok(Message::parse(msg))
    }

    fn into_inner(self) -> Self::Reader {
        self.reader.into_inner()
    }
}
