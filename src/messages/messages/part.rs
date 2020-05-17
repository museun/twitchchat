use super::*;

/// User leave message
///
/// The happens when a user (yourself included) leaves a channel
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Part<'t> {
    /// Name of the user that left the channel
    pub name: Cow<'t, str>,
    /// Channel which they left
    pub channel: Cow<'t, str>,
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for Part<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("PART")?;
        Ok(Self {
            name: msg.expect_nick()?,
            channel: msg.expect_arg(0)?,
        })
    }
}

impl<'t> AsOwned for Part<'t> {
    type Owned = Part<'static>;
    fn as_owned(&self) -> Self::Owned {
        Part {
            name: self.name.as_owned(),
            channel: self.channel.as_owned(),
        }
    }
}
