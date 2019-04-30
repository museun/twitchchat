use super::*;

/// When a user departs from a channel.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Part {
    pub(super) user: String,
    pub(super) channel: Channel,
}

impl Part {
    /// The name of the user leaving
    pub fn user(&self) -> &str {
        &self.user
    }
    /// The channel they are leaving
    pub fn channel(&self) -> &Channel {
        &self.channel
    }
}
