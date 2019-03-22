/// When a user departs from a channel.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Part {
    /// The name of the user leaving
    pub user: String,
    /// The channel they are leaving
    pub channel: String,
}
