use super::*;

/// When a single message has been removed from a channel.
///
/// This is triggered via /delete <target-msg-id> on IRC.
#[derive(Debug, PartialEq, Clone)]
pub struct ClearMsg {
    pub tags: Tags,
    /// The channel this event happened on
    pub channel: String,
    /// The message being moreved
    pub message: Option<String>,
}

impl ClearMsg {
    /// Name of the user who sent the message.
    pub fn login(&self) -> Option<&str> {
        self.get("login")
    }
    /// The message.
    pub fn message(&self) -> Option<&str> {
        self.get("message")
    }
    /// UUID of the message.
    pub fn target_msg_id(&self) -> Option<uuid::Uuid> {
        self.get_parsed("target-msg-id")
    }
}

impl Tag for ClearMsg {
    fn get(&self, key: &str) -> Option<&str> {
        self.tags.get(key).map(AsRef::as_ref)
    }
}
