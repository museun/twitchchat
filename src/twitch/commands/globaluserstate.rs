use super::*;

/// Sent on successful login, if TAGs caps have been sent beforehand
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GlobalUserState {
    pub(super) tags: Tags,
}

impl GlobalUserState {
    /// IRC tags
    pub fn tags(&self) -> &Tags {
        &self.tags
    }
    /// List of badges your user has
    pub fn badges(&self) -> Vec<Badge> {
        badges(self.get("badges").unwrap_or_default())
    }
    /// Your color, if set
    pub fn color(&self) -> Option<Color> {
        self.get("color")
            .and_then(|s| s.parse::<RGB>().ok())
            .map(Into::into)
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
        self.get_parsed("user-id")
            .expect("user-id on globaluserstate")
    }
}

impl Tag for GlobalUserState {
    fn get(&self, key: &str) -> Option<&str> {
        self.tags.get(key).map(AsRef::as_ref)
    }
}
