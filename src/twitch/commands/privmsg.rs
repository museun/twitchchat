use super::*;

/// Send a message to a channel.
#[derive(Debug, PartialEq, Clone)]
pub struct PrivMsg {
    pub tags: Tags,
    /// The IRC user that sent this message
    pub prefix: Option<Prefix>,
    /// The channel this message was sent to
    pub channel: String,
    /// The message body
    pub message: String,
}

impl PrivMsg {
    /// List of badges attached to the user/message
    pub fn badges(&self) -> Vec<Badge> {
        badges(self.get("badges").unwrap_or_default())
    }
    /// How many bits were attached (0 for None)
    // TODO make this optional
    pub fn bits(&self) -> u64 {
        self.get_parsed("bits").unwrap()
    }
    /// The color of the user who sent this message, if set
    pub fn color(&self) -> Option<Color> {
        self.get("color").map(RGB::from_hex).map(Into::into)
    }
    /// The irc name of the user (generally same as their twitch account name)
    pub fn irc_name(&self) -> &str {
        if let Some(crate::irc::types::Prefix::User { ref nick, .. }) = self.prefix {
            &nick
        } else {
            unreachable!("must have a valid irc name")
        }
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
    pub fn id(&self) -> uuid::Uuid {
        self.get_parsed("id").unwrap()
    }
    /// The message body
    pub fn message(&self) -> &str {
        self.get("message").unwrap()
    }
    /// Whether this user was a moderator
    pub fn moderator(&self) -> bool {
        self.get_as_bool("mod")
    }
    /// The id for the room
    pub fn room_id(&self) -> u64 {
        self.get_parsed("room-id").unwrap()
    }
    /// The timestamp that this message was received by Twitch
    pub fn tmi_sent_ts(&self) -> u64 {
        self.get_parsed("tmi-sent-ts").unwrap()
    }
    /// The id of the user who sent this message
    pub fn user_id(&self) -> u64 {
        self.get_parsed("user-id").unwrap()
    }
}

impl Tag for PrivMsg {
    fn get(&self, key: &str) -> Option<&str> {
        self.tags.get(key).map(AsRef::as_ref)
    }
}
