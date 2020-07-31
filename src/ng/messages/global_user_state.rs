use crate::ng::{FromIrcMessage, InvalidMessage, IrcMessage, Str, TagIndices, Tags, Validator};
use crate::{color::Color, Badge};

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalUserState<'t> {
    raw: Str<'t>,
    tags: TagIndices,
    pub user_id: Str<'t>,
    pub display_name: Option<Str<'t>>,
    pub color: Color,
}

impl<'t> FromIrcMessage<'t> for GlobalUserState<'t> {
    type Error = InvalidMessage;

    fn from_irc(msg: IrcMessage<'t>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::GLOBALUSERSTATE)?;

        let tag_index = msg.parse_tags();
        let tags = Tags {
            data: &msg.raw,
            indices: &tag_index,
        };

        let user_id = tags
            .get("user-id")
            .ok_or_else(|| InvalidMessage::ExpectedTag {
                name: "user-id".to_string(),
            })
            .map(Str::from)
            .map(Str::into_owned)?;

        let display_name = tags.get("display-name").map(Str::from).map(Str::into_owned);

        let color = tags
            .get("color")
            .map(std::str::FromStr::from_str)
            .transpose()
            .map_err(|err| InvalidMessage::CannotParseTag {
                name: "color".into(),
                error: Box::new(err),
            })?
            .unwrap_or_default();

        let this = Self {
            user_id,
            display_name,
            color,
            tags: tag_index,
            raw: msg.raw,
        };

        Ok(this)
    }
}

impl<'t> GlobalUserState<'t> {
    raw!();
    tags!();

    pub fn emote_sets(&self) -> Vec<&str> {
        self.tags()
            .get("emote-sets")
            .map(|s| s.split(',').collect())
            .unwrap_or_else(|| vec!["0"])
    }

    pub fn badges(&self) -> Vec<Badge<'_>> {
        self.tags()
            .get("badges")
            .map(|s| s.split(',').filter_map(Badge::parse).collect())
            .unwrap_or_default()
    }

    pub fn user_id(&self) -> &str {
        &*self.user_id
    }

    pub fn display_name(&self) -> Option<&str> {
        self.display_name.as_deref()
    }

    pub fn color(&self) -> Color {
        self.color
    }
}

serde_struct!(GlobalUserState {
    raw,
    tags,
    user_id,
    display_name,
    color,
    badges,
    emote_sets,
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ng::irc;

    #[test]
    #[cfg(feature = "serde")]
    fn global_user_state_serde() {
        let input = "@badge-info=;badges=;color=#FF69B4;display-name=shaken_bot;emote-sets=0;user-id=241015868;user-type= :tmi.twitch.tv GLOBALUSERSTATE\r\n";
        crate::ng::serde::round_trip_json::<GlobalUserState>(input);
    }

    #[test]
    fn global_user_state() {
        let input = "@badge-info=;badges=;color=#FF69B4;display-name=shaken_bot;emote-sets=0;user-id=241015868;user-type= :tmi.twitch.tv GLOBALUSERSTATE\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let msg = GlobalUserState::from_irc(msg).unwrap();

            assert_eq!(msg.user_id, "241015868");
            assert_eq!(msg.user_id(), "241015868");

            assert_eq!(msg.display_name.as_ref().unwrap(), "shaken_bot");
            assert_eq!(msg.display_name().unwrap(), "shaken_bot");

            let color = "#FF69B4".parse().unwrap();
            assert_eq!(msg.color, color);
            assert_eq!(msg.color(), color);

            assert_eq!(msg.emote_sets(), vec!["0"]);
        }
    }
}
