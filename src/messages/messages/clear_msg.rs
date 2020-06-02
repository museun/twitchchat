use super::*;

/// When a single message has been removed from a channel.
///
/// This is triggered via /delete on IRC.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClearMsg<'t> {
    /// Tags attached to the message
    pub tags: Tags<'t>,
    /// The channel this event happened on
    pub channel: Cow<'t, str>,
    /// The message that was deleted
    pub message: Option<Cow<'t, str>>,
}

impl<'t> ClearMsg<'t> {
    /// Name of the user who sent the message
    pub fn login(&'t self) -> Option<Cow<'t, str>> {
        self.tags.get("login").reborrow()
    }

    /// UUID of the message
    pub fn target_msg_id(&'t self) -> Option<Cow<'t, str>> {
        self.tags.get("target-msg-id").reborrow()
    }
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for ClearMsg<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("CLEARMSG")?;
        Ok(Self {
            tags: msg.tags.clone(),
            channel: msg.expect_arg(0)?,
            message: msg.expect_data().ok().cloned(),
        })
    }
}

impl<'t> AsOwned for ClearMsg<'t> {
    type Owned = ClearMsg<'static>;
    fn as_owned(&self) -> Self::Owned {
        ClearMsg {
            tags: self.tags.as_owned(),
            channel: self.channel.as_owned(),
            message: self.message.as_owned(),
        }
    }
}
