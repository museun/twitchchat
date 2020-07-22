use super::super::{Reborrow, Str};
use super::{parser::Parser, Prefix};

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct IrcMessage<'a> {
    pub raw: Str<'a>,
    pub tags: Option<Str<'a>>,
    pub prefix: Option<Prefix<'a>>,
    pub command: Str<'a>,
    pub args: Option<Str<'a>>,
    pub data: Option<Str<'a>>,
}

impl<'a> IrcMessage<'a> {
    // TODO should this be public?
    pub(crate) fn parse(input: &'a str) -> Self {
        // trim any \r\n off incase this was directly called
        let data = if input.ends_with("\r\n") {
            &input[..input.len() - 2]
        } else {
            input
        };

        let mut p = Parser {
            input: data,
            pos: 0,
        };

        Self {
            raw: Str::from(input),
            tags: p.tags(),
            prefix: p.prefix(),
            command: p.command(),
            args: p.args(),
            data: p.data(),
        }
    }

    pub fn get_data<'b: 'a>(&'b self) -> Option<Str<'a>> {
        self.data.as_ref().map(Str::reborrow)
    }

    pub fn nth_arg<'b: 'a>(&'b self, nth: usize) -> Option<Str<'a>> {
        self.args
            .as_ref()?
            .split_ascii_whitespace()
            .nth(nth)
            .map(Str::from)
    }
}

impl<'a> std::fmt::Debug for IrcMessage<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IrcMessage")
            .field("raw", &&*self.raw)
            .field("tags", &self.tags)
            .field("prefix", &self.prefix)
            .field("command", &self.command)
            .field("args", &self.args)
            .field("data", &self.data)
            .finish()
    }
}

impl<'a> Reborrow<'a> for IrcMessage<'a> {
    fn reborrow<'b: 'a>(this: &'b Self) -> Self {
        let Self {
            raw,
            tags,
            prefix,
            command,
            args,
            data,
        } = this;

        IrcMessage {
            raw: Str::reborrow(raw),
            tags: tags.as_ref().map(Str::reborrow),
            prefix: prefix.as_ref().map(Prefix::reborrow),
            command: Str::reborrow(command),
            args: args.as_ref().map(Str::reborrow),
            data: data.as_ref().map(Str::reborrow),
        }
    }
}
