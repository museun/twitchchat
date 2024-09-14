//! IRC message parsing.
//!
//! Twitch's chat is based on IRC, but not strictly conformant to the RFC-1459 and RFC-2812 specs.
//!
//! This provides a _Twitch-flavored_ IRC parser and types.
//!
//! # Parsing
//! You can use this crate to parse Twitch (IRC) messages from a `&str`.
//!
//! This will just borrow from the input `&str`.
//!
//! With the parsed type, you can further refine it into specific Twitch-oriented messages.
//!
//! ```
//! use twitchchat::{IrcMessage, MessageError};
//! // a raw message from the server
//! let input = "@key1=val;key2=true :user!user@user PRIVMSG #some_channel :\x01ACTION hello world\x01\r\n";
//!
//! type MsgResult<'a> = Result<IrcMessage<'a>, MessageError>;
//! // this'll return an iterator over any messages in the `input` string.
//! let mut messages: Vec<MsgResult<'_>> = twitchchat::irc::parse(input).collect();
//! // we should have only gotten 1 message
//! assert_eq!(messages.len(), 1);
//! // and unwrap whether it was an invalid message or not
//! messages.pop().unwrap();
//! ```
use crate::MaybeOwned;

/// A trait to convert an `IrcMessage` into `Self`.
pub trait FromIrcMessage<'a>: Sized {
    /// An error returned if this message could not be parsed.
    type Error;
    /// This method consumes an `IrcMessage` and tries to produce an instance of `Self`
    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error>;

    /// Consumes self returning the raw `MaybeOwned<'a>`
    fn into_inner(self) -> MaybeOwned<'a>;
}

/// A trait to convert a `Self` into an `IrcMessage`
pub trait IntoIrcMessage<'a>: Sized + 'a
where
    Self: FromIrcMessage<'a>,
{
    /// Consumes self producing an `IrcMessage`
    fn into_irc(self) -> IrcMessage<'a>;
}

impl<'a, T: 'a> IntoIrcMessage<'a> for T
where
    T: FromIrcMessage<'a>,
{
    fn into_irc(self) -> IrcMessage<'a> {
        IrcMessage::parse(self.into_inner()).expect("identity conversion")
    }
}

mod message;
pub use message::IrcMessage;

mod prefix;
pub use prefix::{Prefix, PrefixIndex};

pub(crate) mod tags;
pub use tags::{Tags, TagsIter};

mod tag_indices;
pub use tag_indices::TagIndices;

mod error;
pub use error::MessageError;

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
pub fn parse_one(input: &str) -> Result<(usize, IrcMessage<'_>), MessageError> {
    const CRLF: &str = "\r\n";

    let pos = input
        .find(CRLF)
        .ok_or(MessageError::IncompleteMessage { pos: 0 })?
        + CRLF.len();

    let next = &input[..pos];
    let done = next.len() == input.len();

    let msg = IrcMessage::parse(MaybeOwned::Borrowed(next))?;
    Ok((if done { 0 } else { pos }, msg))
}

// TODO add a test for parse_one. it was wrong
