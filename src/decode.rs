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

/// An error occured while parsing a line.
#[derive(Debug)]
pub enum ParseError {
    /// An empty line was found
    EmptyLine,

    /// An incomplete message was parsed
    IncompleteMessage {
        // Position of the start of this invalid message
        pos: usize,
    },

    /// An empty message was parsed
    EmptyMessage,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::EmptyLine => f.write_str("an empty line was attempted to be decoded"),
            ParseError::IncompleteMessage { pos } => {
                write!(f, "an incomplete message was parsed. at {}", pos)
            }
            ParseError::EmptyMessage => f.write_str("an empty message was parsed"),
        }
    }
}

impl std::error::Error for ParseError {}

/// An IRC-like message
#[derive(Debug, Clone)]
pub struct Message<T>
where
    T: crate::StringMarker,
{
    /// The raw string
    pub raw: T,
    /// Any targets found in the message
    pub tags: Tags<T>,
    /// The prefix of the message
    pub prefix: Option<Prefix<T>>,
    /// The command of the message
    pub command: T,
    /// Arguments to the command
    pub args: T,
    /// Any data provided
    pub data: Option<T>,
}

impl<'a> Message<&'a str> {
    fn parse(input: &'a str) -> Result<Self> {
        let raw = input;
        if !input.ends_with("\r\n") {
            return Err(ParseError::IncompleteMessage { pos: 0 });
        }

        let input = &input.trim_start_matches(' ')[..input.len() - 2];
        if input.is_empty() {
            return Err(ParseError::EmptyMessage);
        }

        let mut parser = Parser::new(input);
        Ok(Self {
            raw,
            tags: parser.tags(),
            prefix: parser.prefix(),
            command: parser.command(),
            args: parser.args(),
            data: parser.data(),
        })
    }

    pub fn arg(&self, nth: usize) -> Option<&str> {
        self.args.split_whitespace().nth(nth)
    }

    pub fn into_owned(&self) -> Message<String> {
        Message {
            raw: self.raw.to_string(),
            tags: Tags(
                self.tags
                    .clone()
                    .into_inner()
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            ),
            prefix: self.prefix.map(|s| s.into_owned()),
            command: self.command.to_string(),
            args: self.args.to_string(),
            data: self.data.map(ToString::to_string),
        }
    }
}

/// Prefix is the sender of a message
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Prefix<T>
where
    T: crate::StringMarker,
{
    /// A user sent this message
    User {
        /// Name of the user
        nick: T,
    },
    /// The server sent this message
    Server {
        /// Name of the server
        host: T,
    },
}

impl<'a> Prefix<&'a str> {
    fn parse(input: &'a str) -> Option<Self> {
        if !input.starts_with(':') {
            return None;
        }

        let input = input[1..input.find(' ').unwrap_or_else(|| input.len())].trim();
        let prefix = match input.find('!') {
            Some(pos) => Prefix::User {
                nick: &input[..pos],
            },
            None => Prefix::Server { host: input },
        };
        prefix.into()
    }

    pub fn nick(&self) -> Option<&str> {
        match self {
            Prefix::User { nick } => Some(nick),
            _ => None,
        }
    }

    pub fn host(&self) -> Option<&str> {
        match self {
            Prefix::Server { host } => Some(host),
            _ => None,
        }
    }

    pub fn into_owned(&self) -> Prefix<String> {
        match self {
            Prefix::User { nick } => Prefix::User {
                nick: (*nick).to_string(),
            },
            Prefix::Server { host } => Prefix::Server {
                host: (*host).to_string(),
            },
        }
    }
}

// @tags :prefix cmd args :data\r\n
struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    // '@tags '
    fn tags(&mut self) -> Tags<&'a str> {
        let input = &self.input[self.pos..];
        if input.starts_with('@') {
            if let Some(pos) = input.find(' ') {
                self.pos += pos + 1;
                return Tags::parse(&input[..pos]).unwrap_or_default();
            }
        }
        Tags::default()
    }

    // ':prefix '
    fn prefix(&mut self) -> Option<Prefix<&'a str>> {
        let input = &self.input[self.pos..];
        if !input.starts_with(':') {
            return None;
        }
        let pos = input.find(' ')?;
        self.pos += pos + 1;
        Prefix::parse(&input[..pos])
    }

    // 'cmd '
    fn command(&mut self) -> &'a str {
        let input = &self.input[self.pos..];
        let pos = input.find(' ').unwrap_or_else(|| input.len());
        self.pos += pos + 1;
        &input[..pos]
    }

    // 'args '
    fn args(&mut self) -> &'a str {
        if self.pos > self.input.len() {
            return "";
        }

        let input = &self.input[self.pos..];
        let pos = input.find(':').unwrap_or_else(|| input.len());
        self.pos += pos + 1;
        &input[..pos].trim()
    }

    // ':data'
    fn data(&mut self) -> Option<&'a str> {
        self.input.get(self.pos..).filter(|s| !s.is_empty())
    }
}

struct ParseIter<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> ParseIter<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }
}

impl<'a> Iterator for ParseIter<'a> {
    type Item = Result<Message<&'a str>>;
    fn next(&mut self) -> Option<Self::Item> {
        const CRLF: &str = "\r\n";
        if self.pos == self.input.len() {
            return None;
        }

        let index = match self.input[self.pos..].find(CRLF) {
            Some(index) => index + CRLF.len() + self.pos,
            None => {
                let err = Err(ParseError::IncompleteMessage { pos: self.pos });
                self.pos = self.input.len(); // so we can bail
                return err.into();
            }
        };

        let pos = std::mem::replace(&mut self.pos, index);
        Message::parse(&self.input[pos..index]).into()
    }
}

impl<'a> std::iter::FusedIterator for ParseIter<'a> {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_user_prefix() {
        let prefix = Prefix::parse(":museun!museun@museun.tmi.twitch.tv").unwrap();
        assert_eq!(
            prefix,
            Prefix::User {
                nick: "museun".into(),
            },
        )
    }

    #[test]
    fn parse_server_prefix() {
        let prefix = Prefix::parse(":tmi.twitch.tv").unwrap();
        assert_eq!(
            prefix,
            Prefix::Server {
                host: "tmi.twitch.tv".into(),
            },
        )
    }

    #[test]
    fn missing_colon_prefix() {
        for input in &["museun!museun@museun.tmi.twitch.tv", "tmi.twitch.tv"] {
            assert!(Prefix::parse(input).is_none());
        }
    }

    #[test]
    fn decode() {
        let input = ":foo!bar@baz PRIVMSG #test :this is a test\r\n:local.host PING :1234\r\n";
        let (next, _msg) = super::decode(input).unwrap();
        assert!(next > 0);

        // this should be the last message
        let (next, _msg) = super::decode(&input[next..]).unwrap();
        assert_eq!(next, 0);

        // try with a bad element at the end
        let input = ":foo!bar@baz PRIVMSG #test :this is a test\r\n:local.host PING :1234\r\nfoo";
        {
            let (next, _msg) = super::decode(input).unwrap();
            assert!(next > 0);

            let input = &input[next..];
            let (next, _msg) = super::decode(&input).unwrap();
            assert!(next > 0);

            // last one should be an error
            let input = &input[next..];
            super::decode(&input).unwrap_err();
        }
    }

    #[test]
    fn decode_many() {
        let input = ":foo!bar@baz PRIVMSG #test :this is a test\r\n:local.host PING :1234\r\nfoo";

        // try with the iterator
        let mut vec = super::decode_many(input).collect::<Vec<_>>();
        assert_eq!(vec.len(), 3);

        // last one should be an error
        vec.pop().unwrap().unwrap_err();
        // rest should be okay
        while let Some(ok) = vec.pop() {
            ok.unwrap();
        }

        // remove all of the bad ones, only keep the 'success'
        let vec = super::decode_many(input).flatten().collect::<Vec<_>>();
        assert_eq!(vec.len(), 2);
    }
}
