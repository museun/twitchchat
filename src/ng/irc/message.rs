use super::super::{AsOwned, Reborrow, Str};
use super::{parser::Parser, Prefix, Tags};

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct IrcMessage<'a> {
    pub raw: Str<'a>,
    pub tags: Option<Tags<'a>>,
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
            tags: p.tags().map(Tags::parse),
            prefix: p.prefix(),
            command: p.command(),
            args: p.args(),
            data: p.data(),
        }
    }

    pub fn get_data(&self) -> Option<Str<'_>> {
        self.data.as_ref().map(Str::reborrow)
    }

    pub fn nth_arg(&self, nth: usize) -> Option<Str<'_>> {
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

reborrow_and_asowned!(IrcMessage {
    raw,
    tags,
    prefix,
    command,
    args,
    data
});

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_cap() {
        let input = ":tmi.twitch.tv CAP * ACK :twitch.tv/membership\r\n";

        let msg = IrcMessage::parse(input);
        assert_eq!(msg.raw, input);
        assert_eq!(msg.tags, None);
        assert_eq!(
            msg.prefix,
            Some(Prefix::Server {
                host: "tmi.twitch.tv".into()
            })
        );
        assert_eq!(msg.command, "CAP");
        assert_eq!(msg.args, Some("* ACK".into()));
        assert_eq!(msg.data, Some("twitch.tv/membership".into()));
    }
}
