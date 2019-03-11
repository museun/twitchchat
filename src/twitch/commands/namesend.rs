/// List current chatters in a channel. (marks the end)
#[derive(Debug, PartialEq, Clone)]
pub struct NamesEnd {
    /// Your user for this event
    pub user: String,
    /// The channel this event happened on
    pub channel: String,
}
