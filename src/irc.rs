use std::convert::Infallible;

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
pub use error::Error;

mod parser;

pub fn parse(input: &str) -> impl Iterator<Item = Result<IrcMessage<'_>, Error>> + '_ {
    parser::IrcParserIter::new(input)
}

pub fn parse_one(input: &str) -> Result<(usize, IrcMessage<'_>), Error> {
    let pos = input
        .find("\r\n")
        .ok_or_else(|| Error::IncompleteMessage { pos: 0 })?;
    let msg = IrcMessage::parse(crate::Str::Borrowed(&input[..pos]))?;
    Ok((pos, msg))
}
