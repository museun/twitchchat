use super::super::{Str, StrIndex};
use super::{IrcMessage, PrefixIndex};

pub(super) struct Parser<'a> {
    pub(super) input: &'a str,
    pub(super) pos: usize,
}

impl<'a> Parser<'a> {
    fn mark_index(&mut self, tail: usize, adv: usize) -> StrIndex {
        let index = StrIndex::raw(self.pos, self.pos + tail);
        self.pos += adv;
        index
    }

    pub(super) fn tags(&mut self) -> Option<StrIndex> {
        let input = self.input.get(self.pos..)?;
        if input.starts_with('@') {
            if let Some(end) = input.find(' ') {
                return Some(self.mark_index(end, end + 1));
            }
        }
        None
    }

    pub(super) fn prefix(&mut self) -> Option<PrefixIndex> {
        let input = self.input.get(self.pos..)?;
        if input.starts_with(':') {
            if let Some(pos) = input.find(' ') {
                self.pos += 1;
                let prefix = match input.find('!') {
                    Some(bang) => PrefixIndex::User {
                        nick: self.mark_index(bang - 1, pos),
                    },
                    None => PrefixIndex::Server {
                        host: self.mark_index(pos - 1, pos),
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
        self.mark_index(pos, pos + 1)
    }

    pub(super) fn args(&mut self) -> Option<StrIndex> {
        if self.pos > self.input.len() || self.input[self.pos..].starts_with(':') {
            return None;
        }

        let input = self.input.get(self.pos..)?;
        let pos = input.find(" :").unwrap_or_else(|| input.len());
        Some(self.mark_index(pos, pos))
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
        self.data
            .get(pos..index)
            .map(Str::from)
            .map(IrcMessage::parse)
            .map(Ok)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     #[test]
//     fn parse_components() {
//         let inputs = &[
//             "@tag1=val;tag2=bar :foo!bar@baz COMMAND arg1 arg2 arg3 :data\r\n",
//             ":foo!bar@baz COMMAND arg1 arg2 arg3 :data\r\n",
//             ":foo COMMAND arg1 arg2 arg3 :data\r\n",
//             ":foo COMMAND arg1 arg2\r\n",
//             ":foo COMMAND arg1\r\n",
//             ":foo COMMAND :data\r\n",
//             ":foo COMMAND arg1 :data with spaces\r\n",
//         ];

//         for input in inputs.iter().copied().map(Str::from) {
//             eprintln!("{:#?}\n\n", IrcMessage::parse(input));
//         }
//     }
// }
