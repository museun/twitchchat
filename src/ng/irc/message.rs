use super::super::{Str, StrIndex};

use super::{parser::Parser, Prefix, PrefixIndex, Tags};

#[derive(Clone)]
pub struct IrcMessage<'a> {
    pub raw: Str<'a>,
    pub tags: Option<StrIndex>,
    // TODO make this less weird to use
    pub prefix: Option<PrefixIndex>,
    pub command: StrIndex,
    pub args: Option<StrIndex>,
    pub data: Option<StrIndex>,
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

    pub fn get_raw(&self) -> &str {
        &*self.raw
    }

    pub fn get_tags(&self) -> Option<&str> {
        self.tags.map(|index| &self.raw[index])
    }

    pub fn get_prefix(&self) -> Option<&str> {
        self.prefix.map(|index| &self.raw[index.as_index()])
    }

    pub fn get_command(&self) -> &str {
        &self.raw[self.command]
    }

    pub fn get_args(&self) -> Option<&str> {
        self.args.map(|index| &self.raw[index])
    }

    pub fn get_data(&self) -> Option<&str> {
        self.data.map(|index| &self.raw[index])
    }

    pub fn nth_arg(&self, nth: usize) -> Option<&str> {
        self.args
            .map(|index| &self.raw[index])?
            .split_ascii_whitespace()
            .nth(nth)
    }
}

impl<'a> std::fmt::Debug for IrcMessage<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IrcMessage")
            .field("raw", &&*self.raw)
            .field("tags", &self.tags.map(|index| &self.raw[index]))
            .field(
                "prefix",
                &self.prefix.map(|index| Prefix {
                    data: &self.raw,
                    index,
                }),
            )
            .field("command", &&self.raw[self.command])
            .field("args", &self.args.map(|index| &self.raw[index]))
            .field("data", &self.data.map(|index| &self.raw[index]))
            .finish()
    }
}
