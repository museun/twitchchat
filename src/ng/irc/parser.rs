use super::super::Str;
use super::{IrcMessage, Prefix};

pub(super) struct Parser<'a> {
    pub(super) input: &'a str,
    pub(super) pos: usize,
}

impl<'a> Parser<'a> {
    pub(super) fn tags(&mut self) -> Option<Str<'a>> {
        let input = self.input.get(self.pos..)?;
        if input.starts_with('@') {
            if let Some(pos) = input.find(' ') {
                self.pos += pos + 1;
                return input.get(..pos).map(Str::from);
            }
        }
        None
    }

    pub(super) fn prefix(&mut self) -> Option<Prefix<'a>> {
        let input = &self.input.get(self.pos..)?;
        if input.starts_with(':') {
            if let Some(pos) = input.find(' ') {
                self.pos += pos + 1;
                let prefix = match input.find('!') {
                    Some(pos) => Prefix::User {
                        nick: input.get(1..pos).map(Str::from)?,
                    },
                    None => Prefix::Server {
                        host: input.get(1..pos).map(Str::from)?,
                    },
                };
                return Some(prefix);
            }
        }
        None
    }

    pub(super) fn command(&mut self) -> Str<'a> {
        let input = &self.input[self.pos..];
        let pos = input.find(' ').unwrap_or_else(|| input.len());
        self.pos += pos + 1;
        Str::from(&input[..pos])
    }

    pub(super) fn args(&mut self) -> Option<Str<'a>> {
        if self.pos > self.input.len() || self.input[self.pos..].starts_with(':') {
            return None;
        }

        let input = self.input.get(self.pos..)?;
        let pos = input.find(" :").unwrap_or_else(|| input.len());
        self.pos += pos + 1;
        input.get(..pos).map(Str::from)
    }

    pub(super) fn data(self) -> Option<Str<'a>> {
        let pos = self.input.get(self.pos..).and_then(|s| s.find(':'))?;
        self.input
            .get(self.pos + pos + 1..)
            .filter(|s| !s.is_empty())
            .map(Str::from)
    }
}

pub struct IrcParserIter<'a> {
    data: &'a str,
    pos: usize,
}

impl<'a> IrcParserIter<'a> {
    pub(crate) const fn new(data: &'a str) -> Self {
        Self { data, pos: 0 }
    }
}

impl<'a> Iterator for IrcParserIter<'a> {
    type Item = Result<IrcMessage<'a>, super::Error>;
    fn next(&mut self) -> Option<Self::Item> {
        const CRLF: &str = "\r\n";
        if self.pos == self.data.len() {
            return None;
        }

        let index = match self.data.get(self.pos..)?.find(CRLF) {
            Some(index) => index + CRLF.len() + self.pos,
            None => {
                let err = Err(super::Error::IncompleteMessage { pos: self.pos });
                self.pos = self.data.len();
                return err.into();
            }
        };

        let pos = std::mem::replace(&mut self.pos, index);
        self.data.get(pos..index).map(IrcMessage::parse).map(Ok)
    }
}
