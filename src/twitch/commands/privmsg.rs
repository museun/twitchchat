use super::*;

/// Send a message to a channel.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PrivMsg {
    /// IRC tags
    pub tags: Tags,
    /// The User that sent this message
    pub user: String,
    /// The channel this message was sent to
    pub channel: Channel,
    /// The message body
    pub message: String,
    /// Whether this message was an action (someone doing `/me message`)
    pub action: bool,
}

impl PrivMsg {
    /// The irc name of the user (generally same as their twitch account name)
    pub fn user(&self) -> &str {
        &self.user
    }
    /// The channel this message was sent to
    pub fn channel(&self) -> &Channel {
        &self.channel
    }
    /// The message body
    pub fn message(&self) -> &str {
        self.message.as_str()
    }
    /// Whether this message was an action (someone doing `/me message`)
    pub fn is_action(&self) -> bool {
        self.action
    }
}

impl PrivMsg {
    /// Metadata related to the chat badges
    ///
    /// Currently used only for `subscriber`, to indicate the exact number of months the user has been a subscriber.
    pub fn badge_info(&self) -> Vec<BadgeInfo> {
        badges(self.get("badge-info").unwrap_or_default())
    }
    /// List of badges attached to the user/message
    pub fn badges(&self) -> Vec<Badge> {
        badges(self.get("badges").unwrap_or_default())
    }
    /// How many bits were attached (0 for None)
    // TODO make this optional
    pub fn bits(&self) -> u64 {
        self.get_parsed("bits").unwrap_or_default()
    }
    /// The color of the user who sent this message, if set
    pub fn color(&self) -> Option<Color> {
        self.get("color")
            .and_then(|s| s.parse::<RGB>().ok())
            .map(Into::into)
    }
    /// The display name of the user, if set
    pub fn display_name(&self) -> Option<&str> {
        self.get("display-name")
    }
    /// List of emotes found in the message body.
    pub fn emotes(&self) -> Vec<Emotes> {
        emotes(self.get("emotes").unwrap_or_default())
    }
    /// The unique UUID for this mesage
    pub fn id(&self) -> Option<&str> {
        self.get("id")
    }
    /// Whether this user was a moderator
    pub fn moderator(&self) -> bool {
        self.get_as_bool("mod")
    }
    /// The id for the room
    pub fn room_id(&self) -> Option<u64> {
        self.get_parsed("room-id")
    }
    /// The timestamp that this message was received by Twitch
    pub fn tmi_sent_ts(&self) -> Option<u64> {
        self.get_parsed("tmi-sent-ts")
    }
    /// The id of the user who sent this message
    pub fn user_id(&self) -> Option<u64> {
        self.get_parsed("user-id")
    }
}

impl Tag for PrivMsg {
    fn get(&self, key: &str) -> Option<&str> {
        self.tags.get(key).map(AsRef::as_ref)
    }
}
