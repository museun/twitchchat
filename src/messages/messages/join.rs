use super::*;

/// User join message
///
/// The happens when a user (yourself included) joins a channel
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Join<'t> {
    /// Name of the user that joined the channel
    pub name: Cow<'t, str>,
    /// Channel which they joined
    pub channel: Cow<'t, str>,
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for Join<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("JOIN")?;
        Ok(Self {
            name: msg.expect_nick()?,
            channel: msg.expect_arg(0)?,
        })
    }
}

impl<'t> AsOwned for Join<'t> {
    type Owned = Join<'static>;
    fn as_owned(&self) -> Self::Owned {
        Join {
            name: self.name.as_owned(),
            channel: self.channel.as_owned(),
        }
    }
}
