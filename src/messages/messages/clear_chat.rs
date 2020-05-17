use super::*;

/// When a user's message(s) have been purged.
///
/// Typically after a user is banned from chat or timed out
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClearChat<'t> {
    /// Tags attached to the message
    pub tags: Tags<'t>,
    /// The channel this event happened on
    pub channel: Cow<'t, str>,
    /// The user, if any, that was being purged
    pub name: Option<Cow<'t, str>>,
}

impl<'t> ClearChat<'t> {
    /// (Optional) Duration of the timeout, in seconds. If omitted, the ban is permanent.
    pub fn ban_duration(&self) -> Option<u64> {
        self.tags.get_parsed("ban-duration")
    }
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for ClearChat<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("CLEARCHAT")?;
        Ok(Self {
            tags: msg.tags.clone(),
            channel: msg.expect_arg(0)?,
            name: msg.expect_data().ok().cloned(),
        })
    }
}

impl<'t> AsOwned for ClearChat<'t> {
    type Owned = ClearChat<'static>;
    fn as_owned(&self) -> Self::Owned {
        ClearChat {
            tags: self.tags.as_owned(),
            channel: self.channel.as_owned(),
            name: self.name.as_owned(),
        }
    }
}
