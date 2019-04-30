use super::*;

/// Identifies the channel's chat settings (e.g., slow mode duration).
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RoomState {
    /// IRC tags
    pub tags: Tags,
    /// The channel this event came from
    pub channel: Channel,
}

impl RoomState {
    /// The channel this event came from
    pub fn channel(&self) -> &Channel {
        &self.channel
    }
}

impl RoomState {
    /// Whether this room is in emote-only mode
    pub fn emote_only(&self) -> bool {
        self.get_as_bool("emote-only")
    }
    /// Whether this room is in followers-only mode
    pub fn followers_only(&self) -> FollowersOnly {
        self.get("followers-only")
            .and_then(|k| k.parse().ok())
            .map(|k| match k {
                -1 => FollowersOnly::Disabled,
                0 => FollowersOnly::All,
                d => FollowersOnly::Limit(d),
            })
            .unwrap_or_else(|| FollowersOnly::All)
    }
    /// Whether this room is in r9k mode
    pub fn r9k(&self) -> bool {
        self.get_as_bool("r9k")
    }
    /// Whether this room is in slow mode
    pub fn slow(&self) -> u64 {
        self.get_parsed("slow").unwrap_or_else(|| 0)
    }
    /// Whether this room is in subs-only mode
    pub fn subs_only(&self) -> bool {
        self.get_as_bool("subs-only")
    }
}

impl Tag for RoomState {
    fn get(&self, key: &str) -> Option<&str> {
        self.tags.get(key).map(AsRef::as_ref)
    }
}

/// Followers-only mode
#[derive(Debug, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FollowersOnly {
    /// `Disabled` signifies that anyone can chat
    Disabled,
    /// `All` signifies that only followers can talk
    All,
    /// `Limit` signifies that followers can only talk if they've been following
    /// for the specified number of minutes
    Limit(i64), // maybe make this a Duration
}
