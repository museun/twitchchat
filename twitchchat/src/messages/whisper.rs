use crate::*;

/// Message sent by a user
#[derive(Clone, PartialEq)]
pub struct Whisper<'a> {
    raw: Str<'a>,
    tags: TagIndices,
    name: StrIndex,
    data: StrIndex,
}

impl<'a> Whisper<'a> {
    raw!();
    tags!();
    str_field!(
        /// User who sent this messages
        name
    );
    str_field!(
        /// Data that the user provided
        data
    );

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
    pub fn display_name(&'a self) -> Option<&'a str> {
        self.tags().get("display-name")
    }

    /// Badges attached to this message
    pub fn badges(&'a self) -> Vec<Badge<'a>> {
        self.tags()
            .get("badges")
            .map(parse_badges)
            .unwrap_or_default()
    }

    /// Emotes attached to this message
    pub fn emotes(&self) -> Vec<Emotes> {
        self.tags()
            .get("emotes")
            .map(parse_emotes)
            .unwrap_or_default()
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

    /// The timestamp of when this message was received by Twitch
    pub fn tmi_sent_ts(&self) -> Option<u64> {
        self.tags().get_parsed("tmi-sent-ts")
    }

    /// The id of the user who sent this message
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

impl<'a> FromIrcMessage<'a> for Whisper<'a> {
    type Error = InvalidMessage;

    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::WHISPER)?;

        // :sender WHISPER target :data
        // we're the target, so ignore it

        let this = Self {
            name: msg.expect_nick()?,
            data: msg.expect_data_index()?,
            tags: msg.parse_tags(),
            raw: msg.raw,
        };

        Ok(this)
    }
}

into_owned!(Whisper {
    raw,
    tags,
    name,
    data,
});
impl_custom_debug!(Whisper {
    raw,
    tags,
    name,
    data,
});
serde_struct!(Whisper {
    raw,
    tags,
    name,
    data,
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::irc;

    #[test]
    #[cfg(feature = "serde")]
    fn whisper_serde() {
        let input = ":test!user@host WHISPER museun :this is a test\r\n";
        crate::serde::round_trip_json::<Whisper>(input)
    }

    #[test]
    fn whisper() {
        let input = ":test!user@host WHISPER museun :this is a test\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let msg = Whisper::from_irc(msg).unwrap();

            assert_eq!(msg.name(), "test");
            assert_eq!(msg.data(), "this is a test");
        }
    }
}
