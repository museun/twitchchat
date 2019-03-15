/// List current chatters in a channel. (marks the end)
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NamesEnd {
    /// Your user for this event
    pub user: String,
    /// The channel this event happened on
    pub channel: String,
}
