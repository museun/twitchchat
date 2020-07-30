use crate::ng::{
    FromIrcMessage, InvalidMessage, IrcMessage, Str, StrIndex, TagIndices, Tags, Validator,
};

#[derive(Debug, Clone, PartialEq)]
//
#[derive(::serde::Serialize, ::serde::Deserialize)]
pub enum Ctcp<'a> {
    Action,
    Unknown { command: &'a str },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Privmsg<'t> {
    raw: Str<'t>,
    tags: TagIndices,
    name: StrIndex,
    channel: StrIndex,
    data: StrIndex,

    // hmm
    ctcp: Option<StrIndex>,
}

impl<'t> Privmsg<'t> {
    raw!();
    tags!();
    str_field!(name);
    str_field!(channel);
    str_field!(data);

    pub fn ctcp(&self) -> Option<Ctcp<'_>> {
        const ACTION: &str = "ACTION";
        let command = &self.raw[self.ctcp?];
        if command == ACTION {
            Some(Ctcp::Action)
        } else {
            Some(Ctcp::Unknown { command })
        }
    }
}

impl<'t> FromIrcMessage<'t> for Privmsg<'t> {
    type Error = InvalidMessage;

    fn from_irc(msg: IrcMessage<'t>) -> Result<Self, Self::Error> {
        const CTCP_MARKER: char = '\x01';
        msg.expect_command(IrcMessage::PRIVMSG)?;

        let mut index = msg.expect_data_index()?;
        let mut ctcp = None;

        let data = &msg.raw[index];
        if data.starts_with(CTCP_MARKER) && data.ends_with(CTCP_MARKER) {
            let len = data.chars().map(char::len_utf8).sum::<usize>();
            match data[1..len - 1].find(' ') {
                Some(pos) => {
                    // skip the first byte
                    let head = index.start + 1;
                    let ctcp_index = StrIndex::raw(head, head + pos);

                    // for the byte + space
                    index.start += pos + 2;
                    index.end -= 1;
                    ctcp.replace(ctcp_index);
                }
                None => return Err(InvalidMessage::ExpectedData),
            }
        }

        let this = Self {
            tags: msg.parse_tags(),
            name: msg.expect_nick()?,
            channel: msg.expect_arg_index(0)?,
            data: index,
            ctcp,
            raw: msg.raw,
        };
        Ok(this)
    }
}

serde_struct!(Privmsg {
    raw,
    tags,
    name,
    channel,
    data,
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ng::irc;

    #[test]
    fn privmsg_serde() {
        let input = &[
            ":test!user@host PRIVMSG #museun :this is a test\r\n",
            ":test!user@host PRIVMSG #museun :\u{FFFD}\u{1F468}\r\n",
            ":test!user@host PRIVMSG #museun :\x01ACTION this is a test\x01\r\n",
            ":test!user@host PRIVMSG #museun :\x01FOOBAR this is a test\x01\r\n",
        ];

        for input in input {
            crate::ng::serde::round_trip_json::<Privmsg>(input);
        }
    }

    #[test]
    fn privmsg() {
        let input = ":test!user@host PRIVMSG #museun :this is a test\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let msg = Privmsg::from_irc(msg).unwrap();

            assert_eq!(msg.name(), "test");
            assert_eq!(msg.channel(), "#museun");
            assert_eq!(msg.data(), "this is a test");
            assert_eq!(msg.ctcp(), None);
        }
    }

    #[test]
    fn privmsg_boundary() {
        let input = ":test!user@host PRIVMSG #museun :\u{FFFD}\u{1F468}\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let msg = Privmsg::from_irc(msg).unwrap();

            assert_eq!(msg.name(), "test");
            assert_eq!(msg.channel(), "#museun");
            assert_eq!(msg.data(), "\u{FFFD}\u{1F468}");
            assert_eq!(msg.ctcp(), None);
        }
    }

    #[test]
    fn privmsg_action() {
        let input = ":test!user@host PRIVMSG #museun :\x01ACTION this is a test\x01\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let msg = Privmsg::from_irc(msg).unwrap();

            assert_eq!(msg.name(), "test");
            assert_eq!(msg.channel(), "#museun");
            assert_eq!(msg.data(), "this is a test");
            assert_eq!(msg.ctcp().unwrap(), Ctcp::Action);
        }
    }

    #[test]
    fn privmsg_unknown() {
        let input = ":test!user@host PRIVMSG #museun :\x01FOOBAR this is a test\x01\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let msg = Privmsg::from_irc(msg).unwrap();

            assert_eq!(msg.name(), "test");
            assert_eq!(msg.channel(), "#museun");
            assert_eq!(msg.data(), "this is a test");
            assert_eq!(msg.ctcp().unwrap(), Ctcp::Unknown { command: "FOOBAR" });
        }
    }
}
