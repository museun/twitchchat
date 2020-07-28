use super::{AsOwned, FromIrcMessage, InvalidMessage, IrcMessage, Reborrow, Str, Validator};
use crate::ng::Tags;

/// When a single message has been removed from a channel.
///
/// This is triggered via /delete on IRC.
#[derive(Debug, Clone, PartialEq)]
pub struct ClearMsg<'a> {
    /// Tags attached to the message
    pub tags: Tags<'a>,
    /// The channel this event happened on
    pub channel: Str<'a>,
    /// The message that was deleted
    pub message: Option<Str<'a>>,
}

impl<'a> ClearMsg<'a> {
    /// Name of the user who sent the message
    pub fn login(&'a self) -> Option<Str<'a>> {
        self.tags.get("login")
    }

    /// UUID of the message
    pub fn target_msg_id(&'a self) -> Option<Str<'a>> {
        self.tags.get("target-msg-id")
    }
}

impl<'a> FromIrcMessage<'a> for ClearMsg<'a> {
    type Error = InvalidMessage;

    fn from_irc(msg: &'a IrcMessage<'a>) -> Result<Self, Self::Error> {
        msg.expect_command("CLEARMSG")?;
        Ok(Self {
            tags: msg.parse_tags(),
            channel: msg.expect_arg(0)?,
            message: msg.expect_data().ok(),
        })
    }
}

reborrow_and_asowned!(ClearMsg {
    tags,
    channel,
    message,
});
