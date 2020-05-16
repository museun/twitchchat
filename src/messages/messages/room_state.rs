use super::*;

/// Identifies the channel's chat settings (e.g., slow mode duration).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RoomState<'t> {
    /// Tags attached to this message
    pub tags: Tags<'t>,
    /// Channel this event is happening on
    pub channel: Cow<'t, str>,
}

impl<'t> RoomState<'t> {
    /// Whether this room is in emote only mode
    pub fn is_emote_only(&self) -> bool {
        self.tags.get_as_bool("emote-only")
    }

    /// Whether this room is in followers only mode
    pub fn is_followers_only(&self) -> Option<FollowersOnly> {
        self.tags
            .get_parsed::<_, isize>("followers-only")
            .map(|s| match s {
                -1 => FollowersOnly::Disabled,
                0 => FollowersOnly::All,
                d => FollowersOnly::Limit(d),
            })
    }

    /// Whether this room is in r9k mode
    pub fn is_r9k(&self) -> bool {
        self.tags.get_as_bool("r9k")
    }

    /// The id of the room this message was sent to
    pub fn room_id(&self) -> Option<u64> {
        self.tags.get_parsed("room-id")
    }

    /// Whether this room is in slow mode
    ///
    /// This returns the delay in which each message can be sent
    pub fn is_slow_mode(&self) -> Option<u64> {
        self.tags.get_parsed("slow").filter(|&s| s > 0)
    }

    /// Whether this room is in subs only mode
    pub fn is_subs_only(&self) -> bool {
        self.tags.get_as_bool("subs-only")
    }
}

/// The parameters for a room being in follower-only mode
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FollowersOnly {
    /// The mode is disbaled
    Disabled,
    /// All followers are allowed to speak
    All,
    /// Only those following for `n` days are allowed to speak
    Limit(isize),
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for RoomState<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("ROOMSTATE")?;
        Ok(Self {
            channel: msg.expect_arg(0)?,
            tags: msg.tags.clone(),
        })
    }
}

impl<'t> AsOwned for RoomState<'t> {
    type Owned = RoomState<'static>;
    fn as_owned(&self) -> Self::Owned {
        RoomState {
            tags: self.tags.as_owned(),
            channel: self.channel.as_owned(),
        }
    }
}
