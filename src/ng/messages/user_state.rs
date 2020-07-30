use super::{FromIrcMessage, InvalidMessage, IrcMessage, Str, StrIndex, Validator};
use crate::{
    color::Color,
    ng::{TagIndices, Tags},
    parse_badges, parse_emotes, Badge, BadgeInfo, Emotes,
};

#[derive(Debug, Clone, PartialEq)]
pub struct UserState<'t> {
    raw: Str<'t>,
    tags: TagIndices,
    channel: StrIndex,
}

impl<'t> UserState<'t> {
    raw!();
    tags!();
    str_field!(channel);

    pub fn badge_info(&self) -> Vec<BadgeInfo<'_>> {
        self.tags()
            .get("badge-info")
            .map(parse_badges)
            .unwrap_or_default()
    }

    pub fn badges(&self) -> Vec<Badge<'_>> {
        self.tags()
            .get("badges")
            .map(parse_badges)
            .unwrap_or_default()
    }

    pub fn color(&self) -> Option<Color> {
        self.tags().get_parsed("color")
    }

    pub fn display_name(&self) -> Option<&str> {
        self.tags().get("display-name")
    }

    pub fn emotes(&self) -> Vec<Emotes> {
        self.tags()
            .get("emotes")
            .map(parse_emotes)
            .unwrap_or_default()
    }

    pub fn is_moderator(&self) -> bool {
        self.tags().get_as_bool("mod")
    }
}

impl<'t> FromIrcMessage<'t> for UserState<'t> {
    type Error = InvalidMessage;

    fn from_irc(msg: IrcMessage<'t>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::USERSTATE)?;

        let this = Self {
            tags: msg.parse_tags(),
            channel: msg.expect_arg_index(0)?,
            raw: msg.raw,
        };

        Ok(this)
    }
}

serde_struct!(UserState {
    raw,
    tags,
    channel,
    // TODO determine if we want to serialize these
    // display_name,
    // color,
    // emotes,
    // badges,
    // badge_info
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ng::irc;

    #[test]
    fn user_state_serde() {
        let input = "@badges=bits/1000;badge-info=moderator :tmi.twitch.tv USERSTATE #museun\r\n";
        crate::ng::serde::round_trip_json::<UserState>(input);
    }

    #[test]
    fn user_state() {
        let input = ":tmi.twitch.tv USERSTATE #museun\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let msg = UserState::from_irc(msg).unwrap();
            assert_eq!(msg.channel(), "#museun");
        }
    }
}
