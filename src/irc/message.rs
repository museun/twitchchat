use crate::irc::types::{Prefix, Tags};
use log::*;

/// A simple IRC message
///
/// Twitch messages will be part of the Unknown variant.
#[derive(Debug, PartialEq, Clone)]
pub enum Message {
    /// Ping command. The client should respond to this with a `PONG :${token}\r\n` message
    ///
    /// `token` the token sent with the ping, expected to receive back on a `PONG`
    Ping {
        token: String,
    },

    Cap {
        acknowledge: bool,
        cap: String,
    },

    Connected {
        name: String,
    },

    Ready {
        name: String,
    },

    /// Unknown message.
    ///
    /// `head` will be the "COMMAND" part
    /// `args` will be the args up to the *colon*
    /// `tail` will be the data poriton after the *colon*
    Unknown {
        prefix: Option<Prefix>,
        tags: Tags,
        head: String,
        args: Vec<String>,
        tail: Option<String>,
    },
}

impl Message {
    pub(crate) fn parse(input: &str) -> Option<Self> {
        let input = input.trim(); // sanity check
        if input.is_empty() {
            return None;
        }

        trace!("parsing: {}", input);
        let (tags, input) = if input.starts_with('@') {
            let pos = input.find(' ')?;
            (Tags::parse(&input[..pos]), &input[pos + 1..])
        } else {
            (Tags::default(), input)
        };

        // :prefix command
        let (prefix, input) = if input.starts_with(':') {
            (Prefix::parse(&input), &input[input.find(' ')? + 1..])
        } else {
            (None, input)
        };

        let mut parts = Parts::new(input);
        let ty = match parts.head {
            "PING" => Message::Ping {
                token: parts.data()?,
            },
            "CAP" => Message::Cap {
                acknowledge: parts.args.first().map(|d| *d == "ACK").unwrap(),
                cap: parts.tail.map(str::to_string).unwrap(),
            },
            "001" => Message::Connected {
                name: parts.next()?,
            },
            "376" => Message::Ready {
                name: parts.next()?,
            },
            head => Message::Unknown {
                prefix,
                tags,
                head: head.to_string(),
                // reverse it because parts reverses it to act like a stack
                args: parts.args.into_iter().rev().map(str::to_string).collect(),
                tail: parts.tail.map(str::to_string),
            },
        };
        Some(ty)
    }
}

#[derive(Debug)]
struct Parts<'a> {
    head: &'a str,
    args: Vec<&'a str>,
    tail: Option<&'a str>,
}

impl<'a> Parts<'a> {
    fn new(input: &'a str) -> Self {
        let mut iter = input.split_terminator(" :");
        let index = input.find(" :");
        let (mut iter, tail) = (
            iter.next().map(|s| s.split_terminator(' ')).unwrap(), // TODO handle these potential panics
            index.map(|i| &input[i + 2..]).filter(|s| !s.is_empty()), // TODO handle these potential panics
        );
        let head = iter.next().unwrap(); // TODO handle these potential panics
        let args = iter.rev().collect();
        Self { head, args, tail }
    }

    fn next(&mut self) -> Option<String> {
        self.args.pop().map(str::to_string)
    }

    fn data(&self) -> Option<String> {
        self.tail.map(str::to_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_empty_data() {
        assert_eq!(Message::parse(""), None);
        assert_eq!(Message::parse("            "), None);
    }
}
