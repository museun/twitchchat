use super::*;

/// Happens when the Twitch connection has been succesfully established
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ready<'t> {
    /// The name Twitch will refer to you as
    pub username: Cow<'t, str>,
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for Ready<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("376")?;
        msg.expect_arg(0).map(|username| Self { username })
    }
}

impl<'t> AsOwned for Ready<'t> {
    type Owned = Ready<'static>;
    fn as_owned(&self) -> Self::Owned {
        Ready {
            username: self.username.as_owned(),
        }
    }
}
