use super::*;

/// When a user joins a channel
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Join {
    /// Name of the user that joined
    pub user: String,
    /// The channel that they joined
    pub channel: Channel,
}

impl Join {
    /// Name of the user that joined
    pub fn user(&self) -> &str {
        &self.user
    }
    /// The channel that they joined
    pub fn channel(&self) -> &Channel {
        &self.channel
    }
}
