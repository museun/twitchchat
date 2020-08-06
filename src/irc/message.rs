use super::{parser::Parser, Error, Prefix, PrefixIndex};
use crate::{Str, StrIndex};

/// A raw irc message `@tags :prefix COMMAND args :data\r\n`
#[derive(Clone, PartialEq)]
pub struct IrcMessage<'a> {
    /// The raw string
    pub raw: Str<'a>,
    /// Index of the tags
    pub tags: Option<StrIndex>,
    /// Index of the prefix
    pub prefix: Option<PrefixIndex>,
    /// Index of the command
    pub command: StrIndex,
    /// Index of the args
    pub args: Option<StrIndex>,
    /// Index of the data
    pub data: Option<StrIndex>,
}

impl<'a> IrcMessage<'a> {
    pub(crate) fn parse(input: Str<'a>) -> Result<Self, Error> {
        // trim any \r\n off incase this was directly called
        let data = if input.ends_with("\r\n") {
            &input.as_ref()[..input.len() - 2]
        } else {
            input.as_ref()
        };

        let data = data.trim();
        if data.is_empty() {
            return Err(super::Error::EmptyMessage);
        }

        let mut p = Parser {
            input: data,
            pos: 0,
        };

        let this = Self {
            tags: p.tags(),
            prefix: p.prefix(),
            command: p.command(),
            args: p.args(),
            data: p.data(),
            raw: input, // NOTE: this stores the original input string, not the trimmed string
        };
        Ok(this)
    }

    /// Get the raw string
    pub fn get_raw(&self) -> &str {
        &*self.raw
    }

    /// Get the raw tags
    pub fn get_tags(&self) -> Option<&str> {
        self.tags.map(|index| &self.raw[index])
    }

    /// Get the raw prefix
    pub fn get_prefix(&self) -> Option<&str> {
        self.prefix.map(|index| &self.raw[index.as_index()])
    }

    /// Get the raw command
    pub fn get_command(&self) -> &str {
        &self.raw[self.command]
    }

    /// Get the raw args
    pub fn get_args(&self) -> Option<&str> {
        self.args.map(|index| &self.raw[index])
    }

    /// Get the raw data
    pub fn get_data(&self) -> Option<&str> {
        self.data.map(|index| &self.raw[index])
    }

    /// Get the raw 'nth' argument
    pub fn nth_arg(&self, nth: usize) -> Option<&str> {
        self.args
            .map(|index| &self.raw[index])?
            .split_ascii_whitespace()
            .nth(nth)
    }

    /// Get the index of the 'nth' argumnet
    pub fn nth_arg_index(&self, nth: usize) -> Option<StrIndex> {
        let index = self.args?;
        let args = &self.raw[index];

        let mut seen = 0;
        let (mut head, mut tail) = (index.start, index.start);

        for ch in args.chars() {
            if ch.is_ascii_whitespace() {
                if seen == nth {
                    return Some(StrIndex::raw(head as _, tail as _));
                }

                // skip the space
                head = tail + 1;
                seen += 1;
            }

            tail += 1;
        }

        if seen == nth {
            return Some(StrIndex::raw(head as _, tail as _));
        }

        None
    }
}

impl<'a> IrcMessage<'a> {
    pub const IRC_READY: &'static str = "001";
    pub const READY: &'static str = "376";
    pub const CAP: &'static str = "CAP";
    pub const CLEAR_CHAT: &'static str = "CLEARCHAT";
    pub const CLEAR_MSG: &'static str = "CLEARMSG";
    pub const GLOBAL_USER_STATE: &'static str = "GLOBALUSERSTATE";
    pub const HOST_TARGET: &'static str = "HOSTTARGET";
    pub const JOIN: &'static str = "JOIN";
    pub const NOTICE: &'static str = "NOTICE";
    pub const PART: &'static str = "PART";
    pub const PING: &'static str = "PING";
    pub const PONG: &'static str = "PONG";
    pub const PRIVMSG: &'static str = "PRIVMSG";
    pub const RECONNECT: &'static str = "RECONNECT";
    pub const ROOM_STATE: &'static str = "ROOMSTATE";
    pub const USER_NOTICE: &'static str = "USERNOTICE";
    pub const USER_STATE: &'static str = "USERSTATE";
    pub const WHISPER: &'static str = "WHISPER";
}

into_owned! {
    IrcMessage {
        raw,
        tags,
        prefix,
        command,
        args,
        data,
    }
}

impl<'a> std::fmt::Debug for IrcMessage<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IrcMessage")
            .field("raw", &&*self.raw)
            .field("tags", &self.get_tags())
            .field(
                "prefix",
                &self.prefix.map(|index| Prefix {
                    data: &self.raw,
                    index,
                }),
            )
            .field("command", &self.get_command())
            .field("args", &self.get_args())
            .field("data", &self.get_data())
            .finish()
    }
}

#[cfg(feature = "serde")]
impl<'t> ::serde::Serialize for IrcMessage<'t> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        use ::serde::ser::SerializeStruct as _;

        let mut s = serializer.serialize_struct("IrcMessage", 6)?;
        s.serialize_field("raw", &&*self.raw)?;
        s.serialize_field("tags", &self.get_tags())?;
        s.serialize_field("prefix", &self.get_prefix())?;
        s.serialize_field("command", &self.get_command())?;
        s.serialize_field("args", &self.get_args())?;
        s.serialize_field("data", &self.get_data())?;
        s.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, 't> ::serde::Deserialize<'de> for IrcMessage<'t> {
    fn deserialize<D>(deserializer: D) -> Result<IrcMessage<'t>, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(crate::serde::RawVisitor::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "serde")]
    fn irc_message_serde() {
        let input = ":test!test@test PRIVMSG #museun :this is a test\r\n";
        crate::serde::round_trip_json::<IrcMessage>(input);
    }

    #[test]
    fn parse_empty_spaces() {
        for i in 0..10 {
            let s: Str<'_> = format!("{}\r\n", " ".repeat(i)).into();
            let err = IrcMessage::parse(s).unwrap_err();
            assert!(matches!(err, Error::EmptyMessage))
        }
    }
}
