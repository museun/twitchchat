use crate::{
    color::Color,
    ng::{FromIrcMessage, InvalidMessage, IrcMessage, Str, StrIndex, TagIndices, Tags, Validator},
    parse_badges, parse_badges_iter, parse_emotes, Badge, BadgeInfo, BadgeKind, Emotes,
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
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

    pub fn is_action(&self) -> bool {
        match self.ctcp() {
            Some(Ctcp::Action) => true,
            _ => false,
        }
    }

    pub fn badge_info(&'t self) -> Vec<BadgeInfo<'t>> {
        self.tags()
            .get("badge-info")
            .map(parse_badges)
            .unwrap_or_default()
    }

    pub fn badges(&'t self) -> Vec<Badge<'t>> {
        self.tags()
            .get("badges")
            .map(parse_badges)
            .unwrap_or_default()
    }

    pub fn bits(&self) -> Option<u64> {
        self.tags().get_parsed("bits")
    }

    pub fn color(&self) -> Option<Color> {
        self.tags().get_parsed("color")
    }

    pub fn display_name(&'t self) -> Option<&str> {
        self.tags().get("display-name")
    }

    pub fn emotes(&self) -> Vec<Emotes> {
        self.tags()
            .get("emotes")
            .map(parse_emotes)
            .unwrap_or_default()
    }

    pub fn is_broadcaster(&self) -> bool {
        self.contains_badge(BadgeKind::Broadcaster)
    }

    pub fn is_moderator(&self) -> bool {
        self.tags().get_as_bool("mod")
    }

    pub fn is_vip(&self) -> bool {
        self.contains_badge(BadgeKind::Broadcaster)
    }

    pub fn is_subscriber(&self) -> bool {
        self.contains_badge(BadgeKind::Subscriber)
    }

    pub fn is_staff(&self) -> bool {
        self.contains_badge(BadgeKind::Staff)
    }

    pub fn is_turbo(&self) -> bool {
        self.contains_badge(BadgeKind::Turbo)
    }

    pub fn is_global_moderator(&self) -> bool {
        self.contains_badge(BadgeKind::GlobalMod)
    }

    pub fn room_id(&self) -> Option<u64> {
        self.tags().get_parsed("room-id")
    }

    pub fn tmi_sent_ts(&self) -> Option<u64> {
        self.tags().get_parsed("tmi-sent-ts")
    }

    pub fn user_id(&self) -> Option<u64> {
        self.tags().get_parsed("user-id")
    }

    fn contains_badge(&self, badge: BadgeKind) -> bool {
        self.tags()
            .get("badges")
            .into_iter()
            .flat_map(parse_badges_iter)
            .any(|x| x.kind == badge)
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
    #[cfg(feature = "serde")]
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
