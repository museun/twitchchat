use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalUserState<T = String>
where
    T: StringMarker,
{
    pub user_id: T,
    pub display_name: Option<T>,
    pub color: crate::color::Color,
    pub emote_sets: Vec<T>,
    pub badges: Vec<crate::Badge>,
}

impl<'a> TryFrom<&'a Message<&'a str>> for GlobalUserState<&'a str> {
    type Error = InvalidMessage;

    fn try_from(msg: &'a Message<&'a str>) -> Result<Self, Self::Error> {
        msg.expect_command("GLOBALUSERSTATE").map(|_| {
            let user_id = msg
                .tags
                .get("user-id")
                .expect("user-id attached to message");
            let display_name = msg.tags.get("display-name");
            let color = msg
                .tags
                .get("color")
                .and_then(|s| s.parse().ok())
                .unwrap_or_default();
            let emote_sets = msg
                .tags
                .get("emotes-set")
                .map(|s| s.split(',').collect())
                .unwrap_or_else(|| vec!["0"]);
            let badges = msg
                .tags
                .get("badges")
                .map(|s| s.split(',').filter_map(crate::Badge::parse).collect())
                .unwrap_or_default();

            Self {
                user_id,
                display_name,
                color,
                emote_sets,
                badges,
            }
        })
    }
}

as_owned!(for GlobalUserState {
    user_id,
    display_name,
    color,
    emote_sets,
    badges
});

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse() {
        let input = "@badge-info=;badges=;color=#FF69B4;display-name=shaken_bot;emote-sets=0;user-id=241015868;user-type= :tmi.twitch.tv GLOBALUSERSTATE\r\n";

        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                GlobalUserState::<String>::try_from(&msg).unwrap(),
                GlobalUserState {
                    user_id: "241015868".to_string(),
                    display_name: Some("shaken_bot".to_string()),
                    color: "#FF69B4".parse().unwrap(),
                    emote_sets: vec!["0".to_string()],
                    badges: vec![],
                }
            );

            assert_eq!(
                GlobalUserState::<&str>::try_from(&msg).unwrap(),
                GlobalUserState {
                    user_id: "241015868",
                    display_name: Some("shaken_bot"),
                    color: "#FF69B4".parse().unwrap(),
                    emote_sets: vec!["0"],
                    badges: vec![],
                }
            )
        }
    }
}
