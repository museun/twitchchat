use super::*;

/// Information gathered during the [`GLOBALUSERSTATE`](./commands/struct.GlobalUserState.html) event
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LocalUser {
    /// Your user id
    pub user_id: u64,
    /// Your display name, if set.
    pub display_name: Option<String>,
    /// The name you've provided to UserConfig (incase display_name isn't set)
    pub name: String,
    /// Your color, if set
    pub color: Option<Color>,
    /// Your badges
    pub badges: Vec<Badge>,
    /// Your list of emote sets
    pub emote_sets: Vec<u64>,
    /// The capabilities the server acknowledged
    pub caps: Vec<Capability>,
}

impl LocalUser {
    pub(crate) fn from_global_user_state(
        state: &crate::commands::GlobalUserState,
        name: String,
        caps: impl IntoIterator<Item = Capability>,
    ) -> Self {
        LocalUser {
            user_id: state.user_id(),
            display_name: state.display_name().map(ToString::to_string),
            name,
            color: state.color(),
            badges: state.badges(),
            emote_sets: state.emote_sets(),
            caps: caps.into_iter().collect(),
        }
    }
}
