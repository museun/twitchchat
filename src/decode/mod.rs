use crate::Tags;
type Result<T> = std::result::Result<T, ParseError>;

/**
 Tries to decode one message, returning the amount of remaining data in the input

# Example

## A single message
```rust
# use twitchchat::*;
# use std::borrow::Cow;

let input = ":test!test@test JOIN #museun\r\n";
let (pos, message) = decode_one(&input).unwrap();
assert_eq!(pos, 0); // no more messages were found

let expected = messages::Raw {
    raw: Cow::Borrowed(":test!test@test JOIN #museun\r\n"),
    tags: Tags::default(),
    prefix: Some(decode::Prefix::User { nick: Cow::Borrowed("test") }),
    command: Cow::Borrowed("JOIN"),
    args: Cow::Borrowed("#museun"),
    data: None,
};
assert_eq!(message, expected);
```

# Multiple messages
```rust
# use twitchchat::*;
# use std::borrow::Cow;

let input = ":test!test@test JOIN #museun\r\n:test!test@test JOIN #shaken_bot\r\n";
let (pos, message) = decode_one(&input).unwrap();
assert_eq!(pos, 30); // another message probably starts at offset '30'

let expected = messages::Raw {
    raw: Cow::Borrowed(":test!test@test JOIN #museun\r\n"),
    tags: Tags::default(),
    prefix: Some(decode::Prefix::User { nick: Cow::Borrowed("test") }),
    command: Cow::Borrowed("JOIN"),
    args: Cow::Borrowed("#museun"),
    data: None,
};
assert_eq!(message, expected);

// continue from where it left off
let (pos, message) = decode_one(&input[pos..]).unwrap();
assert_eq!(pos, 0); // no more messages were found

let expected = messages::Raw {
    raw: Cow::Borrowed(":test!test@test JOIN #shaken_bot\r\n"),
    tags: Tags::default(),
    prefix: Some(decode::Prefix::User { nick: Cow::Borrowed("test") }),
    command: Cow::Borrowed("JOIN"),
    args: Cow::Borrowed("#shaken_bot"),
    data: None,
};
assert_eq!(message, expected);
```
*/
pub fn decode_one<'t>(input: &'t str) -> Result<(usize, Message<'t>)> {
    let pos = input
        .find("\r\n")
        .ok_or_else(|| ParseError::IncompleteMessage { pos: 0 })?;
    let next = if pos + 2 == input.len() { 0 } else { pos + 2 };
    Message::parse(&input[..pos + 2]).map(|msg| (next, msg))
}

/**
Tries to decode potentially many messages from this input string

# Example
```rust
# use twitchchat::*;
# use std::borrow::Cow;

let input = ":test!test@test JOIN #museun\r\n:test!test@test JOIN #shaken_bot\r\n";

let expected = &[
    messages::Raw {
        raw: Cow::Borrowed(":test!test@test JOIN #museun\r\n"),
        tags: Tags::default(),
        prefix: Some(decode::Prefix::User { nick: Cow::Borrowed("test") }),
        command: Cow::Borrowed("JOIN"),
        args: Cow::Borrowed("#museun"),
        data: None,
    },
    messages::Raw {
        raw: Cow::Borrowed(":test!test@test JOIN #shaken_bot\r\n"),
        tags: Tags::default(),
        prefix: Some(decode::Prefix::User { nick: Cow::Borrowed("test") }),
        command: Cow::Borrowed("JOIN"),
        args: Cow::Borrowed("#shaken_bot"),
        data: None,
    },
];

for (message, expected) in decode(&input).zip(expected.iter()) {
    let msg = message.expect("valid message");
    assert_eq!(msg, *expected);
}
```
*/
pub const fn decode(input: &str) -> DecodeIter<'_> {
    DecodeIter(ParseIter::new(input))
}

/// An iterator over decoded messages
pub struct DecodeIter<'a>(ParseIter<'a>);

impl<'a> Iterator for DecodeIter<'a> {
    type Item = Result<Message<'a>>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a> std::iter::FusedIterator for DecodeIter<'a> {}

impl<'a> std::fmt::Debug for DecodeIter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebugIter").finish()
    }
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
