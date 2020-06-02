use super::*;

/// Sent on successful login, if TAGs caps have been sent beforehand
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GlobalUserState<'t> {
    /// Your user-id
    pub user_id: Cow<'t, str>,
    /// Your display name, if set   
    pub display_name: Option<Cow<'t, str>>,
    /// Your color, if set. Defaults to `white`
    pub color: crate::color::Color,
    /// Your available emote sets, always contains atleast '0'
    pub emote_sets: Vec<Cow<'t, str>>,
    /// Any badges you have
    pub badges: Vec<crate::Badge<'t>>,
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for GlobalUserState<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("GLOBALUSERSTATE")?;

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
            .get_ref("emotes-set")
            .map(|s| s.split(',').map(Into::into).collect())
            .unwrap_or_else(|| vec!["0".into()]);

        let badges = msg
            .tags
            .get_ref("badges")
            .map(|s| s.split(',').filter_map(crate::Badge::parse).collect())
            .unwrap_or_default();

        Ok(Self {
            user_id,
            display_name,
            color,
            emote_sets,
            badges,
        })
    }
}

impl<'t> AsOwned for GlobalUserState<'t> {
    type Owned = GlobalUserState<'static>;
    fn as_owned(&self) -> Self::Owned {
        GlobalUserState {
            user_id: self.user_id.as_owned(),
            display_name: self.display_name.as_owned(),
            color: self.color.as_owned(),
            emote_sets: self.emote_sets.as_owned(),
            badges: self.badges.as_owned(),
        }
    }
}
