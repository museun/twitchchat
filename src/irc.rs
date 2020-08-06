use crate::Str;

mod from_irc_message;
pub use from_irc_message::{FromIrcMessage, InvalidMessage};

mod message;
pub use message::IrcMessage;

mod prefix;
pub use prefix::{Prefix, PrefixIndex};

mod tags;
pub use tags::Tags;

mod tag_indices;
pub use tag_indices::TagIndices;

mod parser;

pub fn parse(input: &str) -> impl Iterator<Item = Result<IrcMessage<'_>, Error>> + '_ {
    parser::IrcParserIter::new(input)
}

pub fn parse_one(input: &str) -> Result<(usize, IrcMessage<'_>), Error> {
    let pos = input
        .find("\r\n")
        .ok_or_else(|| Error::IncompleteMessage { pos: 0 })?;

    Ok((pos, IrcMessage::parse(Str::Borrowed(&input[..pos]))?))
}

#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    // TODO make this less bad
    IncompleteMessage { pos: usize },
    EmptyMessage,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IncompleteMessage { pos } => {
                write!(f, "incomplete message starting at: {}", pos)
            }
            Error::EmptyMessage => write!(f, "no message could be parsed"),
        }
    }
}

impl std::error::Error for Error {}
