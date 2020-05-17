use super::*;

/// Identifies a user's chat settings or properties (e.g., chat color)..
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UserState<'t> {
    /// Tags attached to this message
    pub tags: Tags<'t>,
    /// Channel this event happened on
    pub channel: Cow<'t, str>,
}

impl<'t> UserState<'t> {
    /// Metadata related to the chat badges
    ///
    /// Currently used only for `subscriber`, to indicate the exact number of months the user has been a subscriber
    ///    
    pub fn badge_info(&'t self) -> Vec<crate::BadgeInfo<'t>> {
        self.tags
            .get("badge-info")
            .map(|s| crate::parse_badges(s))
            .unwrap_or_default()
    }

    /// Badges attached to this message
    ///    
    pub fn badges(&'t self) -> Vec<crate::Badge<'t>> {
        self.tags
            .get("badges")
            .map(|s| crate::parse_badges(s))
            .unwrap_or_default()
    }

    /// The user's color, if set
    pub fn color(&self) -> Option<crate::color::Color> {
        self.tags.get_parsed("color")
    }

    /// The user's display name, if set
    pub fn display_name(&'t self) -> Option<&'t Cow<'t, str>> {
        self.tags.get("display-name")
    }

    /// Emotes attached to this message
    pub fn emotes(&self) -> Vec<crate::Emotes> {
        self.tags
            .get("emotes")
            .map(|s| crate::parse_emotes(s))
            .unwrap_or_default()
    }

    /// Whether this user a is a moderator
    pub fn is_moderator(&self) -> bool {
        self.tags.get_as_bool("mod")
    }
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for UserState<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("USERSTATE")?;
        msg.expect_arg(0).map(|channel| Self {
            channel,
            tags: msg.tags.clone(),
        })
    }
}

impl<'t> AsOwned for UserState<'t> {
    type Owned = UserState<'static>;
    fn as_owned(&self) -> Self::Owned {
        UserState {
            tags: self.tags.as_owned(),
            channel: self.channel.as_owned(),
        }
    }
}
