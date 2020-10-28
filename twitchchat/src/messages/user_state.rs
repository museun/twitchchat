use crate::twitch::{parse_badges, parse_emotes, Badge, BadgeInfo, Color, Emotes};
use crate::{irc::*, MaybeOwned, MaybeOwnedIndex, Validator};

/// Identifies a user's chat settings or properties (e.g., chat color)..
#[derive(Clone, PartialEq)]
pub struct UserState<'a> {
    raw: MaybeOwned<'a>,
    tags: TagIndices,
    channel: MaybeOwnedIndex,
}

impl<'a> UserState<'a> {
    raw!();
    tags!();
    str_field!(
        /// Channel this event happened on
        channel
    );

    /// Metadata related to the chat badges
    ///
    /// Currently used only for `subscriber`, to indicate the exact number of
    /// months the user has been a subscriber
    pub fn badge_info(&self) -> Vec<BadgeInfo<'_>> {
        self.tags()
            .get("badge-info")
            .map(parse_badges)
            .unwrap_or_default()
    }

    /// Badges attached to this message
    pub fn badges(&self) -> Vec<Badge<'_>> {
        self.tags()
            .get("badges")
            .map(parse_badges)
            .unwrap_or_default()
    }

    /// The user's color, if set
    pub fn color(&self) -> Option<Color> {
        self.tags().get_parsed("color")
    }

    /// The user's display name, if set
    pub fn display_name(&self) -> Option<&str> {
        self.tags().get("display-name")
    }

    /// Emotes attached to this message
    pub fn emotes(&self) -> Vec<Emotes> {
        self.tags()
            .get("emotes")
            .map(parse_emotes)
            .unwrap_or_default()
    }

    /// Whether this user is a moderator
    pub fn is_moderator(&self) -> bool {
        self.tags().get_as_bool("mod")
    }
}

impl<'a> FromIrcMessage<'a> for UserState<'a> {
    type Error = MessageError;

    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::USER_STATE)?;

        let this = Self {
            tags: msg.parse_tags(),
            channel: msg.expect_arg_index(0)?,
            raw: msg.raw,
        };

        Ok(this)
    }

    into_inner_raw!();
}

into_owned!(UserState { raw, tags, channel });
impl_custom_debug!(UserState { raw, tags, channel });
serde_struct!(UserState { raw, tags, channel });

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "serde")]
    fn user_state_serde() {
        let input = "@badges=bits/1000;badge-info=moderator :tmi.twitch.tv USERSTATE #museun\r\n";
        crate::serde::round_trip_json::<UserState>(input);
        crate::serde::round_trip_rmp::<UserState>(input);
    }

    #[test]
    fn user_state() {
        let input = ":tmi.twitch.tv USERSTATE #museun\r\n";
        for msg in parse(input).map(|s| s.unwrap()) {
            let msg = UserState::from_irc(msg).unwrap();
            assert_eq!(msg.channel(), "#museun");
        }
    }
}
