use super::*;

/// When a user departs from a channel.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Part {
    /// The IRC user leaving
    pub prefix: Option<Prefix>,
    /// The channel they are leaving
    pub channel: String,
}
