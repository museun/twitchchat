use super::*;

/// When a user's message(s) have been purged.
///
/// Typically after a user is banned from chat or timed out.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ClearChat {
    pub tags: Tags,
    /// The channel this event happened on
    pub channel: String,
    /// The owner of the message. Empty if its the entire channel
    pub user: Option<String>,
}

impl ClearChat {
    /// (Optional) Duration of the timeout, in seconds. If omitted, the ban is permanent.
    pub fn ban_duration(&self) -> Option<u64> {
        self.get("ban-duration")?.parse().ok()
    }
}

impl Tag for ClearChat {
    fn get(&self, key: &str) -> Option<&str> {
        self.tags.get(key).map(AsRef::as_ref)
    }
}
