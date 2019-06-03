use super::*;

/// When a user gains or loses moderator (operator) status in a channel.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mode {
    pub(super) channel: Channel,
    pub(super) status: ModeStatus,
    pub(super) user: String,
}

impl Mode {
    /// The channel this event happened on
    pub fn channel(&self) -> &Channel {
        &self.channel
    }
    /// Whether they lost or gained the status
    pub fn status(&self) -> ModeStatus {
        self.status
    }
    /// Which user was effected by this
    pub fn user(&self) -> &str {
        &self.user
    }
}

/// Status of gaining or losing moderator (operator) status
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ModeStatus {
    /// Moderator status gained
    Gained,
    /// Moderator status lost
    Lost,
}
