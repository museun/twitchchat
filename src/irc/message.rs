use crate::irc::types::{Prefix, Tags};
use log::*;

/// A simple IRC message
///
/// Twitch messages will be part of the Unknown variant.
#[derive(Debug, PartialEq, Clone)]
pub enum Message {
    /// Ping command. The client should respond to this with a `PONG :${token}\r\n` message        
    Ping {
        /// The token sent with the ping, expected to receive back on a `PONG`
        token: String,
    },

    /// Acknowledgement (or not) on a CAPS request
    // TODO https://ircv3.net/specs/core/capability-negotiation.html#the-cap-command
    // THIS: https://ircv3.net/specs/core/capability-negotiation.html#the-cap-nak-subcommand
    Cap {
        /// Whether it was acknowledged
        acknowledge: bool,
        /// Which CAP was enabled
        cap: String,
    },

    /// Happens when you've connected to the server. Corresponds to the `001` IRC message
    Connected {
        /// The name the server assigned you
        name: String,
    },

    /// Happens after the server sent you the MOTD. Corresponds to the `376` IRC message
    Ready {
        /// The name the server assigned you
        name: String,
    },

    /// Unknown message.
    Unknown {
        /// Optional prefix. The sender of the message
        prefix: Option<Prefix>,
        /// Any parsed tags
        tags: Tags,
        /// the `COMMAND` portion of the IRC message
        head: String,
        /// The argument list that follows the commands
        args: Vec<String>,
        /// Any trailing data (generally after the ':')
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
                acknowledge: parts
                    .args
                    .first()
                    .map(|d| *d == "ACK")
                    .unwrap_or_else(|| false),
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
