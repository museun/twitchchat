mod from_irc_message;
pub use from_irc_message::FromIrcMessage;

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

    Ok((pos, IrcMessage::parse(crate::Str::Borrowed(&input[..pos]))?))
}
