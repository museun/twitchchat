use super::super::{Str, StrIndex};
use super::{IrcMessage, PrefixIndex};

pub(super) struct Parser<'a> {
    pub(super) input: &'a str,
    pub(super) pos: usize,
}

impl<'a> Parser<'a> {
    fn mark_index(&mut self, pos: usize) -> StrIndex {
        let index = StrIndex::raw(self.pos, self.pos + pos);
        self.pos += pos + 1;
        index
    }

    pub(super) fn tags(&mut self) -> Option<StrIndex> {
        let input = self.input.get(self.pos..)?;
        if input.starts_with('@') {
            if let Some(pos) = input.find(' ') {
                return Some(self.mark_index(pos));
            }
        }
        None
    }

    pub(super) fn prefix(&mut self) -> Option<PrefixIndex> {
        let input = &self.input.get(self.pos..)?;
        if input.starts_with(':') {
            if let Some(pos) = input.find(' ') {
                let prefix = match input.find('!') {
                    Some(pos) => PrefixIndex::User {
                        nick: self.mark_index(pos),
                    },
                    None => PrefixIndex::Server {
                        host: self.mark_index(pos),
                    },
                };
                return Some(prefix);
            }
        }
        None
    }

    pub(super) fn command(&mut self) -> StrIndex {
        let input = &self.input[self.pos..];
        let pos = input.find(' ').unwrap_or_else(|| input.len());
        self.mark_index(pos)
    }

    pub(super) fn args(&mut self) -> Option<StrIndex> {
        if self.pos > self.input.len() || self.input[self.pos..].starts_with(':') {
            return None;
        }

        let input = self.input.get(self.pos..)?;
        let pos = input.find(" :").unwrap_or_else(|| input.len());
        Some(self.mark_index(pos))
    }

    pub(super) fn data(self) -> Option<StrIndex> {
        let pos = self.input.get(self.pos..).and_then(|s| s.find(':'))?;
        self.input
            .get(self.pos + pos + 1..)
            .filter(|s| !s.is_empty())
            .map(|_| StrIndex::raw(self.pos + pos + 1, self.input.len()))
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
