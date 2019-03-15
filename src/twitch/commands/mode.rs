/// When a user gains or loses moderator (operator) status in a channel.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mode {
    /// The channel this event happened on
    pub channel: String,
    /// Whether they lost or gained the status
    pub status: ModeStatus,
    /// Which user was effected by this
    pub user: String,
}

/// Status of gaining or losing moderator (operator) status
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ModeStatus {
    Gained,
    Lost,
}
