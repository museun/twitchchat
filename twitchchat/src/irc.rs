//! IRC message parsing.
//!
//! Twitch's chat is based on IRC, but not strictly conformant to the RFC-1459 and RFC-2812 specs.
//!
//! This provides a _Twitch-flavored_ IRC parser and types.
use std::convert::Infallible;

/// A trait to convert an `IrcMessage` into `Self`.
pub trait FromIrcMessage<'a>: Sized {
    type Error;
    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error>;
}

impl<'a> FromIrcMessage<'a> for IrcMessage<'a> {
    type Error = Infallible;
    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error> {
        Ok(msg)
    }
}

mod message;
pub use message::IrcMessage;

mod prefix;
pub use prefix::{Prefix, PrefixIndex};

mod tags;
pub use tags::Tags;

mod tag_indices;
pub use tag_indices::TagIndices;

mod error;
pub use error::InvalidMessage;

mod parser;
pub use parser::IrcParserIter;

/// Parses a string and returns an iterator over the `IrcMessages` in it.
///
/// This borrows from the input string.
pub fn parse(input: &str) -> IrcParserIter<'_> {
    IrcParserIter::new(input)
}

/// Attempts to parse one message.
///
/// This returns the index of the /next/ message (e.g, 0 for a single message) and the parsed message
pub fn parse_one(input: &str) -> Result<(usize, IrcMessage<'_>), InvalidMessage> {
    const CRLF: &str = "\r\n";

    let pos = input
        .find(CRLF)
        .ok_or_else(|| InvalidMessage::IncompleteMessage { pos: 0 })?;

    let msg = IrcMessage::parse(crate::Str::Borrowed(&input[..pos + CRLF.len()]))?;
    Ok((pos, msg))
}
