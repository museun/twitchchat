use super::*;

/// Identifies a user's chat settings or properties (e.g., chat color)..
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UserState {
    /// IRC tags
    pub tags: Tags,
    /// Channel this even happened on
    pub channel: String,
}

impl UserState {
    /// Channel this even happened on
    pub fn channel(&self) -> &str {
        &self.channel
    }
}

impl UserState {
    /// Metadata related to the chat badges
    ///
    /// Currently used only for `subscriber`, to indicate the exact number of months the user has been a subscriber.
    pub fn badge_info(&self) -> Vec<BadgeInfo> {
        badges(self.get("badge-info").unwrap_or_default())
    }

    /// Badges attached to this message
    pub fn badges(&self) -> Vec<Badge> {
        badges(self.get("badges").unwrap_or_default())
    }
    /// The user's color, if set
    pub fn color(&self) -> Option<Color> {
        self.get("color")
            .and_then(|s| s.parse::<RGB>().ok())
            .map(Into::into)
    }
    /// The user's display name, if set
    pub fn display_name(&self) -> Option<&str> {
        self.get("display-name")
    }
    /// Emotes attached to this message
    pub fn emotes(&self) -> Vec<Emotes> {
        emotes(self.get("emotes").unwrap_or_default())
    }
    /// Whether this user is a moderator
    pub fn moderator(&self) -> bool {
        self.get_as_bool("mod")
    }
}

impl Tag for UserState {
    fn get(&self, key: &str) -> Option<&str> {
        self.tags.get(key).map(AsRef::as_ref)
    }
}
