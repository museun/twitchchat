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

impl<'a> IrcMessage<'a>
where
    Self: 'a,
{
    // TODO should this be public?
    pub(crate) fn parse(input: Str<'a>) -> Self {
        // trim any \r\n off incase this was directly called
        let data = if input.ends_with("\r\n") {
            &input.as_ref()[..input.len() - 2]
        } else {
            input.as_ref()
        };

        let mut p = Parser {
            input: data,
            pos: 0,
        };

        Self {
            tags: p.tags(),
            prefix: p.prefix(),
            command: p.command(),
            args: p.args(),
            data: p.data(),
            raw: input,
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

    // TODO this is wrong. this is grabbing the 'COMMAND' as well
    pub fn nth_arg(&self, nth: usize) -> Option<&str> {
        self.args
            .map(|index| &self.raw[index])?
            .split_ascii_whitespace()
            .nth(nth)
    }

    pub fn nth_arg_index(&self, nth: usize) -> Option<StrIndex> {
        let index = self.args?;
        let args = &self.raw[index];

        let mut seen = 0;
        let (mut head, mut tail) = (index.start, index.start);

        for ch in args.chars() {
            if ch.is_ascii_whitespace() {
                if seen == nth {
                    return Some(StrIndex::raw(head, tail));
                }

                // skip the space
                head = tail + 1;
                seen += 1;
            }

            tail += 1;
        }

        if seen == nth {
            return Some(StrIndex::raw(head, tail));
        }

        None
    }
}

impl<'a> IrcMessage<'a> {
    pub const IRCREADY: &'static str = "001";
    pub const READY: &'static str = "376";
    pub const CAP: &'static str = "CAP";
    pub const CLEARCHAT: &'static str = "CLEARCHAT";
    pub const CLEARMSG: &'static str = "CLEARMSG";
    pub const GLOBALUSERSTATE: &'static str = "GLOBALUSERSTATE";
    pub const HOSTTARGET: &'static str = "HOSTTARGET";
    pub const JOIN: &'static str = "JOIN";
    pub const NOTICE: &'static str = "NOTICE";
    pub const PART: &'static str = "PART";
    pub const PING: &'static str = "PING";
    pub const PONG: &'static str = "PONG";
    pub const PRIVMSG: &'static str = "PRIVMSG";
    pub const RECONNECT: &'static str = "RECONNECT";
    pub const ROOMSTATE: &'static str = "ROOMSTATE";
    pub const USERNOTICE: &'static str = "USERNOTICE";
    pub const USERSTATE: &'static str = "USERSTATE";
    pub const WHISPER: &'static str = "WHISPER";
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
