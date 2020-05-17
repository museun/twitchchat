use super::*;

/// Happens when the IRC connection has been succesfully established
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrcReady<'t> {
    /// The name the server will refer to you as
    pub nickname: Cow<'t, str>,
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for IrcReady<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("001")?;
        msg.expect_arg(0).map(|nickname| Self { nickname })
    }
}

impl<'t> AsOwned for IrcReady<'t> {
    type Owned = IrcReady<'static>;
    fn as_owned(&self) -> Self::Owned {
        IrcReady {
            nickname: self.nickname.as_owned(),
        }
    }
}
