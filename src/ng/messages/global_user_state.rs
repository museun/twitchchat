use super::{FromIrcMessage, InvalidMessage, IrcMessage, Str, StrIndex, Validator};

/// Sent on successful login, if TAGs caps have been sent beforehand
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalUserState<'a> {
    /// Your user-id
    pub user_id: Str<'a>,
    /// Your display name, if set   
    pub display_name: Option<Str<'a>>,
    /// Your color, if set. Defaults to `white`
    pub color: crate::color::Color,
    /// Your available emote sets, always contains atleast '0'
    pub emote_sets: Vec<Str<'a>>,
    /// Any badges you have
    pub badges: Vec<crate::Badge<'a>>,
}

impl<'a> FromIrcMessage<'a> for GlobalUserState<'a> {
    type Error = InvalidMessage;

    fn from_irc(msg: &'a IrcMessage<'a>) -> Result<Self, Self::Error> {
        msg.expect_command("GLOBALUSERSTATE")?;

        let mut tags: Tags<'a> = msg.parse_tags();

        // TODO return an error
        let user_id = tags.remove("user-id").expect("user-id attached to message");
        let display_name = tags.remove("display-name");

        let color = tags
            .get("color")
            .and_then(|s| s.parse().ok())
            .unwrap_or_default();

        let mut v = vec![];
        let t = tags.remove("emotes-set");
        let t: Option<Str<'a>> = t.as_ref().map(Str::reborrow);
        let emote_sets = match t {
            Some(d) => {
                // 't
                // for el in d.split(',').map(Into::into) {
                //     v.push(el)
                // }
                v
            }
            None => vec!["0".into()],
        };
        // 'a
        //
        // .map(|s| s.split(',').map(Reborrow::reborrow).collect())
        // .unwrap_or_else(|| vec!["0".into()]);

        let badges = tags
            .remove("badges")
            .map(|s| s.split(',').filter_map(crate::Badge::parse).collect())
            .unwrap_or_default();

        Ok(Self {
            user_id,
            display_name,
            color,
            badges,
            emote_sets,
        })
    }
}

reborrow_and_asowned!(GlobalUserState {
    user_id,
    display_name,
    color,
    badges,
    emote_sets,
});
