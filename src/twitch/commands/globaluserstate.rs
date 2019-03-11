use super::*;

/// Sent on successful login, if TAGs caps have been sent beforehand
#[derive(Debug, PartialEq, Clone)]
pub struct GlobalUserState {
    pub(crate) tags: Tags,
}

impl GlobalUserState {
    /// List of badges your user has
    pub fn badges(&self) -> Vec<Badge> {
        badges(self.get("badges").unwrap_or_default())
    }
    /// Your color, if set
    pub fn color(&self) -> Option<Color> {
        self.get("color").map(RGB::from_hex).map(Into::into)
    }
    /// Your dusplay name, if set
    pub fn display_name(&self) -> Option<&str> {
        self.get("display-name")
    }
    /// Your emote sets
    pub fn emote_sets(&self) -> Vec<u64> {
        self.get("emote-sets")
            .map(|s| {
                s.split_terminator(',')
                    .filter_map(|d| d.parse().ok())
                    .collect()
            })
            .unwrap_or_default()
    }
    /// Your user id
    pub fn user_id(&self) -> u64 {
        self.get_parsed("user-id").unwrap()
    }
}

impl Tag for GlobalUserState {
    fn get(&self, key: &str) -> Option<&str> {
        self.tags.get(key).map(AsRef::as_ref)
    }
}
