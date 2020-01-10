use crate::Tags;
type Result<T> = std::result::Result<T, ParseError>;

/// Tries to decode one message, returning the amount of remaining data in the input
pub fn decode(input: &str) -> Result<(usize, Message<&'_ str>)> {
    let pos = input
        .find("\r\n")
        .ok_or_else(|| ParseError::IncompleteMessage { pos: 0 })?;
    let next = if pos + 2 == input.len() { 0 } else { pos + 2 };
    Message::parse(&input[..pos + 2]).map(|msg| (next, msg))
}

/// Tries to decode potentially many messages from this input string
pub fn decode_many(input: &str) -> impl Iterator<Item = Result<Message<&'_ str>>> + '_ {
    ParseIter::new(input)
}

mod parser;
use parser::*;

mod message;
pub use message::*;

mod prefix;
pub use prefix::*;

mod error;
pub use error::*;

#[cfg(test)]
mod tests;
