use super::{AsOwned, FromIrcMessage, InvalidMessage, IrcMessage, Reborrow, Str, Validator};
use crate::ng::Tags;

/// When a user's message(s) have been purged.
///
/// Typically after a user is banned from chat or timed out
#[derive(Debug, Clone, PartialEq)]
pub struct ClearChat<'a> {
    /// Tags attached to the message
    pub tags: Tags<'a>,
    /// The channel this event happened on
    pub channel: Str<'a>,
    /// The user, if any, that was being purged
    pub name: Option<Str<'a>>,
}

impl<'a> ClearChat<'a> {
    /// (Optional) Duration of the timeout, in seconds. If omitted, the ban is permanent.
    pub fn ban_duration(&self) -> Option<u64> {
        self.tags.get_parsed("ban-duration")
    }
}

impl<'a> FromIrcMessage<'a> for ClearChat<'a> {
    type Error = InvalidMessage;

    fn from_irc(msg: &'a IrcMessage<'a>) -> Result<Self, Self::Error> {
        msg.expect_command("CLEARCHAT")?;
        Ok(Self {
            tags: msg.parse_tags(),
            channel: msg.expect_arg(0)?,
            name: msg.expect_data().ok(),
        })
    }
}

reborrow_and_asowned!(ClearChat {
    tags,
    channel,
    name,
});
