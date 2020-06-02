use super::*;

/// Message sent by a user
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Privmsg<'t> {
    /// User who sent this messages
    pub name: Cow<'t, str>,
    /// Channel this message was sent on
    pub channel: Cow<'t, str>,
    /// Data that the user provided
    pub data: Cow<'t, str>,
    /// Tags attached to the message
    pub tags: Tags<'t>,
}

impl<'t> Privmsg<'t> {
    /// Metadata related to the chat badges
    ///
    /// Currently used only for `subscriber`, to indicate the exact number of months the user has been a subscriber
    pub fn badge_info(&'t self) -> Vec<crate::BadgeInfo<'t>> {
        self.tags
            .get("badge-info")
            .map(|s| crate::parse_badges(s))
            .unwrap_or_default()
    }

    /// Badges attached to this message
    pub fn badges(&'t self) -> Vec<crate::Badge<'t>> {
        self.tags
            .get("badges")
            .map(|s| crate::parse_badges(s))
            .unwrap_or_default()
    }

    /// How many bits were attached to this message
    pub fn bits(&self) -> Option<u64> {
        self.tags.get_parsed("bits")
    }

    /// The color of the user who sent this message, if set
    pub fn color(&self) -> Option<crate::color::Color> {
        self.tags.get_parsed("color")
    }

    /// Returns the display name of the user, if set.
    ///
    /// Users can changed the casing and encoding of their names, if they choose to.
    ///
    /// By default, their display name is not set. If the user **foo** changes their display name to **FOO** then this'll return that **FOO**. Otherwise it'll return `None`. This also applies to users who have decided to user a localized version of their name.
    ///
    /// You can get their username with the field [`name`](#structfield.name).
    ///
    /// ```rust
    /// # use twitchchat::{*, messages::*};
    /// // without their display name set
    /// let data = ":foo!foo@foo PRIVMSG #testing :this is a test.\r\n";
    /// let msg = decode(data).next().unwrap().unwrap();
    /// let pm = Privmsg::parse(&msg).unwrap();
    /// assert_eq!(pm.name, "foo");
    /// assert!(pm.display_name().is_none());
    ///
    /// // with their display name set
    /// let data = "@display-name=FOO :foo!foo@foo PRIVMSG #testing :this is a test.\r\n";
    /// let msg = decode(data).next().unwrap().unwrap();
    /// let pm = Privmsg::parse(&msg).unwrap();
    /// assert_eq!(pm.name, "foo");
    /// assert_eq!(pm.display_name().unwrap(), "FOO");
    /// ```
    ///
    /// A useful thing to do is to try to get the `display_name` and fallback to the `username`.
    ///
    /// ```rust
    /// # use twitchchat::{*, messages::*};
    /// use std::borrow::Cow;
    /// fn get_user_or_display<'a>(msg: &'a Privmsg<'_>) -> Cow<'a, str> {
    ///     msg.display_name()
    ///         .unwrap_or_else(|| Cow::Borrowed(&*msg.name))
    /// }
    ///
    /// let data = ":foo!foo@foo PRIVMSG #testing :this is a test.\r\n";
    /// let msg = decode(data).next().unwrap().unwrap();
    /// let pm = Privmsg::parse(&msg).unwrap();
    ///
    /// let name = get_user_or_display(&pm);
    /// assert_eq!(name, "foo");
    ///
    /// let data = "@display-name=FOO :foo!foo@foo PRIVMSG #testing :this is a test.\r\n";
    /// let msg = decode(data).next().unwrap().unwrap();
    /// let pm = Privmsg::parse(&msg).unwrap();
    ///
    /// let name = get_user_or_display(&pm);
    /// assert_eq!(name, "FOO");    
    /// ```
    pub fn display_name(&'t self) -> Option<Cow<'t, str>> {
        self.tags.get("display-name").reborrow()
    }

    /// Emotes attached to this message
    pub fn emotes(&self) -> Vec<crate::Emotes> {
        self.tags
            .get("emotes")
            .map(|s| crate::parse_emotes(s))
            .unwrap_or_default()
    }

    /// Whether the user sending this message was a broadcaster
    pub fn is_broadcaster(&self) -> bool {
        self.badges()
            .iter()
            .any(|x| x.kind == crate::BadgeKind::Broadcaster)
    }

    /// Whether the user sending this message was a moderator
    pub fn is_moderator(&self) -> bool {
        self.tags.get_as_bool("mod")
    }

    /// Whether the user sending this message was a vip
    pub fn is_vip(&self) -> bool {
        self.badges()
            .iter()
            .any(|x| x.kind == crate::BadgeKind::Broadcaster)
    }

    /// Whether the user sending this message was a susbcriber
    pub fn is_subscriber(&self) -> bool {
        self.badges()
            .iter()
            .any(|x| x.kind == crate::BadgeKind::Subscriber)
    }

    /// Whether the user sending this message was a staff member
    pub fn is_staff(&self) -> bool {
        self.badges()
            .iter()
            .any(|x| x.kind == crate::BadgeKind::Staff)
    }

    /// Whether the user sending this message had turbo
    pub fn is_turbo(&self) -> bool {
        self.badges()
            .iter()
            .any(|x| x.kind == crate::BadgeKind::Turbo)
    }

    /// Whether the user sending this message was a global moderator
    pub fn is_global_moderator(&self) -> bool {
        self.badges()
            .iter()
            .any(|x| x.kind == crate::BadgeKind::GlobalMod)
    }

    /// The id of the room this message was sent to
    pub fn room_id(&self) -> Option<u64> {
        self.tags.get_parsed("room-id")
    }

    /// The timestamp of when this message was received by Twitch
    pub fn tmi_sent_ts(&self) -> Option<u64> {
        self.tags.get_parsed("tmi-sent-ts")
    }

    /// The id of the user who sent this message
    pub fn user_id(&self) -> Option<u64> {
        self.tags.get_parsed("user-id")
    }
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for Privmsg<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("PRIVMSG")?;
        Ok(Self {
            name: msg.expect_nick()?,
            channel: msg.expect_arg(0)?,
            data: msg.expect_data()?,
            tags: msg.tags.clone(),
        })
    }
}

impl<'t> AsOwned for Privmsg<'t> {
    type Owned = Privmsg<'static>;
    fn as_owned(&self) -> Self::Owned {
        Privmsg {
            name: self.name.as_owned(),
            channel: self.channel.as_owned(),
            data: self.data.as_owned(),
            tags: self.tags.as_owned(),
        }
    }
}
