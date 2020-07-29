mod message;
pub use message::IrcMessage;

mod prefix;
pub use prefix::{Prefix, PrefixIndex};

mod tags;
pub use tags::{TagIndices, Tags};

mod parser;

pub fn parse(input: &str) -> impl Iterator<Item = Result<IrcMessage<'_>, Error>> + '_ {
    parser::IrcParserIter::new(input)
}

pub fn parse_one(input: &str) -> Result<IrcMessage<'_>, Error> {
    match parse(input).next() {
        Some(res) => res,
        // this error is atleast at the end
        None => Err(Error::IncompleteMessage { pos: input.len() }),
    }
}

#[derive(Debug)]
pub enum Error {
    // TODO make this less bad
    IncompleteMessage { pos: usize },
}
