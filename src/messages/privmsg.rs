use crate::{irc::*, MaybeOwned, MaybeOwnedIndex, Validator};

use crate::twitch::{
    parse_badges, parse_badges_iter, parse_emotes, Badge, BadgeInfo, BadgeKind, Color, Emotes,
};

/// Some PRIVMSGs are considered 'CTCP' (client-to-client protocol)
///
/// This is a tag-type for determining what kind of CTCP it was
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub enum Ctcp<'a> {
    /// An action CTCP, sent by the user when they do `/me` or `/action`
    Action,
    /// An unknown CTCP
    Unknown {
        /// The unknown CTCP command
        command: &'a str,
    },
}

/// Message sent by a user
#[derive(Clone, PartialEq)]
pub struct Privmsg<'a> {
    raw: MaybeOwned<'a>,
    tags: TagIndices,
    name: MaybeOwnedIndex,
    channel: MaybeOwnedIndex,
    data: MaybeOwnedIndex,
    ctcp: Option<MaybeOwnedIndex>,
}

/// An iterator over badges
#[derive(Debug)]
pub struct BadgesIter<'a> {
    items: Option<std::str::Split<'a, char>>,
}

impl<'a> Iterator for BadgesIter<'a> {
    type Item = Badge<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.items.as_mut()?.next() {
            Badge::parse(item)
        } else {
            None
        }
    }
}

/// An iterator over emotes
#[derive(Debug)]
pub struct EmotesIter<'a> {
    items: Option<std::str::SplitTerminator<'a, char>>,
}

impl<'a> Iterator for EmotesIter<'a> {
    type Item = Emotes;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.items.as_mut()?.next() {
            Emotes::parse_item(item)
        } else {
            None
        }
    }
}

impl<'a> Privmsg<'a> {
    raw!();
    tags!();
    str_field!(
        /// User who sent this messages
        name
    );
    str_field!(
        /// Channel this message was sent on
        channel
    );
    str_field!(
        /// Data that the user provided
        data
    );

    /// Iterator alternative to `Privmsg::badges()`
    pub fn iter_badges(&self) -> BadgesIter {
        BadgesIter {
            items: self.tags().get("badges").map(|s| s.split(',')),
        }
    }

    /// Iterator alternative to `Privmsg::emotes()`
    pub fn iter_emotes(&self) -> EmotesIter {
        EmotesIter {
            items: self.tags().get("emotes").map(|s| s.split_terminator('/')),
        }
    }

    /// Gets the 'CTCP' kind associated with this message, if any
    pub fn ctcp(&self) -> Option<Ctcp<'_>> {
        const ACTION: &str = "ACTION";
        let command = &self.raw[self.ctcp?];
        if command == ACTION {
            Some(Ctcp::Action)
        } else {
            Some(Ctcp::Unknown { command })
        }
    }

    /// Whether this message was an Action (a `/me` or `/action`)
    pub fn is_action(&self) -> bool {
        matches!(self.ctcp(), Some(Ctcp::Action))
    }

    /// Metadata related to the chat badges
    ///
    /// Currently used only for `subscriber`, to indicate the exact number of
    /// months the user has been a subscriber
    pub fn badge_info(&'a self) -> Vec<BadgeInfo<'a>> {
        self.tags()
            .get("badge-info")
            .map(parse_badges)
            .unwrap_or_default()
    }

    /// Badges attached to this message
    pub fn badges(&'a self) -> Vec<Badge<'a>> {
        self.tags()
            .get("badges")
            .map(parse_badges)
            .unwrap_or_default()
    }

    /// How many bits were attached to this message
    pub fn bits(&self) -> Option<u64> {
        self.tags().get_parsed("bits")
    }

    /// The color of the user who sent this message, if set
    pub fn color(&self) -> Option<Color> {
        self.tags().get_parsed("color")
    }

    /// Returns the display name of the user, if set.
    ///
    /// Users can changed the casing and encoding of their names, if they choose
    /// to.
    ///
    /// By default, their display name is not set. If the user **foo** changes
    /// their display name to **FOO** then this'll return that **FOO**.
    ///
    /// Otherwise it'll return `None`.
    pub fn display_name(&'a self) -> Option<&str> {
        self.tags().get("display-name")
    }

    /// Emotes attached to this message
    pub fn emotes(&self) -> Vec<Emotes> {
        self.tags()
            .get("emotes")
            .map(parse_emotes)
            .unwrap_or_default()
    }

    /// Whether the user sending this message was a broadcaster
    pub fn is_broadcaster(&self) -> bool {
        self.contains_badge(BadgeKind::Broadcaster)
    }

    /// Whether the user sending this message was a moderator
    pub fn is_moderator(&self) -> bool {
        self.contains_badge(BadgeKind::Moderator)
    }

    /// Whether the user sending this message was a vip
    pub fn is_vip(&self) -> bool {
        self.contains_badge(BadgeKind::Broadcaster)
    }

    /// Whether the user sending this message was a susbcriber
    pub fn is_subscriber(&self) -> bool {
        self.contains_badge(BadgeKind::Subscriber)
    }

    /// Whether the user sending this message was a staff member
    pub fn is_staff(&self) -> bool {
        self.contains_badge(BadgeKind::Staff)
    }

    /// Whether the user sending this message had turbo
    pub fn is_turbo(&self) -> bool {
        self.contains_badge(BadgeKind::Turbo)
    }

    /// Whether the user sending this message was a global moderator
    pub fn is_global_moderator(&self) -> bool {
        self.contains_badge(BadgeKind::GlobalMod)
    }

    /// The id of the room this message was sent to
    pub fn room_id(&self) -> Option<u64> {
        self.tags().get_parsed("room-id")
    }

    /// The timestamp of when this message was received by Twitch
    pub fn tmi_sent_ts(&self) -> Option<u64> {
        self.tags().get_parsed("tmi-sent-ts")
    }

    /// The id of the user who sent this message
    pub fn user_id(&self) -> Option<u64> {
        self.tags().get_parsed("user-id")
    }

    /// `custom-reward-id` is returned on custom rewards set by broadcaster.
    ///
    /// **NOTE** From the new community points rewards.
    ///
    /// With no api from Twitch to retrieve proper name, looks like a UUID.
    pub fn custom_reward_id(&self) -> Option<&str> {
        self.tags().get("custom-reward-id")
    }

    /// The name of the custom channel reward.
    ///
    /// For example, a highlighted message would be `highlighted-message`
    ///
    /// **NOTE** From the new community points rewards.
    pub fn msg_id(&self) -> Option<&str> {
        self.tags().get("msg-id")
    }

    fn contains_badge(&self, badge: BadgeKind<'_>) -> bool {
        self.tags()
            .get("badges")
            .into_iter()
            .flat_map(parse_badges_iter)
            .any(|x| x.kind == badge)
    }
}

impl<'a> FromIrcMessage<'a> for Privmsg<'a> {
    type Error = MessageError;

    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error> {
        const CTCP_MARKER: char = '\x01';
        msg.expect_command(IrcMessage::PRIVMSG)?;

        let mut index = msg.expect_data_index()?;
        let mut ctcp = None;

        let data = &msg.raw[index];
        if data.starts_with(CTCP_MARKER) && data.ends_with(CTCP_MARKER) {
            let len = data.chars().map(char::len_utf8).sum::<usize>();
            match data[1..len - 1].find(' ') {
                Some(pos) => {
                    // TODO refactor this so the casting is done in 1 canonical place
                    //
                    // skip the first byte
                    let head = index.start + 1;
                    let ctcp_index = MaybeOwnedIndex::raw(head as usize, (head as usize) + pos);

                    // for the byte + space
                    index.start += (pos as u16) + 2;
                    index.end -= 1;
                    ctcp.replace(ctcp_index);
                }
                None => return Err(MessageError::ExpectedData),
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

    into_inner_raw!();
}

into_owned!(Privmsg {
    raw,
    tags,
    name,
    channel,
    data,
    ctcp,
});

impl_custom_debug!(Privmsg {
    raw,
    tags,
    name,
    channel,
    data,
    ctcp,
    // TODO decide /how/ much should be in the debug, all of this is in the tags
    // is_action,
    // badge_info,
    // badges,
    // bits,
    // color,
    // display_name,
    // emotes,
    // is_broadcaster,
    // is_moderator,
    // is_vip,
    // is_subscriber,
    // is_staff,
    // is_turbo,
    // is_global_moderator,
    // room_id,
    // tmi_sent_ts,
    // user_id,
    // custom_reward_id,
    // msg_id,
});

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
            crate::serde::round_trip_json::<Privmsg>(input);
            crate::serde::round_trip_rmp::<Privmsg>(input);
        }
    }

    #[test]
    fn privmsg() {
        let input = ":test!user@host PRIVMSG #museun :this is a test\r\n";
        for msg in parse(input).map(|s| s.unwrap()) {
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
        for msg in parse(input).map(|s| s.unwrap()) {
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
        for msg in parse(input).map(|s| s.unwrap()) {
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
        for msg in parse(input).map(|s| s.unwrap()) {
            let msg = Privmsg::from_irc(msg).unwrap();

            assert_eq!(msg.name(), "test");
            assert_eq!(msg.channel(), "#museun");
            assert_eq!(msg.data(), "this is a test");
            assert_eq!(msg.ctcp().unwrap(), Ctcp::Unknown { command: "FOOBAR" });
        }
    }

    #[test]
    fn privmsg_community_rewards() {
        let input = "@custom-reward-id=abc-123-foo;msg-id=highlighted-message :test!user@host PRIVMSG #museun :Notice me!\r\n";
        for msg in parse(input).map(|s| s.unwrap()) {
            let msg = Privmsg::from_irc(msg).unwrap();
            assert_eq!(msg.name(), "test");
            assert_eq!(msg.channel(), "#museun");
            assert_eq!(msg.data(), "Notice me!");
            assert_eq!(msg.custom_reward_id().unwrap(), "abc-123-foo");
            assert_eq!(msg.msg_id().unwrap(), "highlighted-message");
        }
    }
}
