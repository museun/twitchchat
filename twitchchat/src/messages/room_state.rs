use crate::{irc::*, MaybeOwned, MaybeOwnedIndex, Validator};

/// The parameters for a room being in follower-only mode
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub enum FollowersOnly {
    /// The mode is disabled
    Disabled,
    /// All followers are allowed to speak
    All,
    /// Only those following for `n` days are allowed to speak
    Limit(isize),
}

/// Identifies the channel's chat settings (e.g., slow mode duration).
#[derive(Clone, PartialEq)]
pub struct RoomState<'a> {
    raw: MaybeOwned<'a>,
    tags: TagIndices,
    channel: MaybeOwnedIndex,
}

impl<'a> FromIrcMessage<'a> for RoomState<'a> {
    type Error = MessageError;
    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::ROOM_STATE)?;

        let this = Self {
            tags: msg.parse_tags(),
            channel: msg.expect_arg_index(0)?,
            raw: msg.raw,
        };

        Ok(this)
    }

    into_inner_raw!();
}

impl<'a> RoomState<'a> {
    raw!();
    tags!();
    str_field!(
        /// The channel that this event is happening on
        channel
    );

    /// Whether this room is in emote only mode
    pub fn is_emote_only(&self) -> bool {
        self.tags().get_as_bool("emote-only")
    }

    /// Whether this room is in followers only mode
    pub fn is_followers_only(&self) -> Option<FollowersOnly> {
        self.tags()
            .get_parsed::<_, isize>("followers-only")
            .map(|s| match s {
                -1 => FollowersOnly::Disabled,
                0 => FollowersOnly::All,
                d => FollowersOnly::Limit(d),
            })
    }

    /// Whether this room is in r9k mode
    pub fn is_r9k(&self) -> bool {
        self.tags().get_as_bool("r9k")
    }

    /// The id of the room this message was sent to
    pub fn room_id(&self) -> Option<u64> {
        self.tags().get_parsed("room-id")
    }

    /// Whether this room is in slow mode
    ///
    /// This returns the delay in which each message can be sent
    pub fn is_slow_mode(&self) -> Option<u64> {
        self.tags().get_parsed("slow").filter(|&s| s > 0)
    }

    /// Whether this room is in subs only mode
    pub fn is_subs_only(&self) -> bool {
        self.tags().get_as_bool("subs-only")
    }
}

into_owned!(RoomState { raw, tags, channel });
impl_custom_debug!(RoomState { raw, tags, channel });
serde_struct!(RoomState { raw, tags, channel });

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "serde")]
    fn user_state_serde() {
        let input = ":tmi.twitch.tv ROOMSTATE #museun\r\n";
        crate::serde::round_trip_json::<RoomState>(input);
        crate::serde::round_trip_rmp::<RoomState>(input);
    }

    #[test]
    fn user_state() {
        let input = ":tmi.twitch.tv ROOMSTATE #museun\r\n";
        for msg in parse(input).map(|s| s.unwrap()) {
            let msg = RoomState::from_irc(msg).unwrap();
            assert_eq!(msg.channel(), "#museun");
        }
    }
}
