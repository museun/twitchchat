use super::*;

/// When a user's message(s) have been purged.
///
/// Typically after a user is banned from chat or timed out.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClearChat {
    /// IRC tags
    pub tags: Tags,
    /// The channel this event happened on
    pub channel: String,
    /// The owner of the message. Empty if its the entire channel
    pub user: Option<String>,
}

impl ClearChat {
    /// The owner of the message. Empty if its the entire channel
    pub fn user(&self) -> Option<&str> {
        self.user.as_ref().map(String::as_str)
    }
    /// The channel this event happened on
    pub fn channel(&self) -> &str {
        &self.channel
    }
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
