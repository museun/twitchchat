use super::*;

/// When a single message has been removed from a channel.
///
/// This is triggered via /delete <target-msg-id> on IRC.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClearMsg {
    /// IRC tags
    pub tags: Tags,
    /// The channel this event happened on
    pub channel: String,
    /// The message being removed
    pub message: Option<String>,
}

impl ClearMsg {
    /// The channel this event happened on
    pub fn channel(&self) -> &str {
        &self.channel
    }
}

impl ClearMsg {
    /// Name of the user who sent the message.
    pub fn login(&self) -> Option<&str> {
        self.get("login")
    }
    /// The message.
    pub fn message(&self) -> Option<&str> {
        self.get("message")
            .or_else(|| self.message.as_ref().map(|s| s.as_str()))
    }
    /// UUID of the message.
    pub fn target_msg_id(&self) -> Option<&str> {
        self.get("target-msg-id")
    }
}

impl Tag for ClearMsg {
    fn get(&self, key: &str) -> Option<&str> {
        self.tags.get(key).map(AsRef::as_ref)
    }
}
