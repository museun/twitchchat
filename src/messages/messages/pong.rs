use super::*;

/// A pong response sent from the server
///
/// This should be a response to sending a PING to the server
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pong<'t> {
    /// Token associated with the PONG event
    pub token: Cow<'t, str>,
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for Pong<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("PONG")?;
        msg.expect_data_ref().map(|token| Self {
            token: token.reborrow(),
        })
    }
}

impl<'t> AsOwned for Pong<'t> {
    type Owned = Pong<'static>;
    fn as_owned(&self) -> Self::Owned {
        Pong {
            token: self.token.as_owned(),
        }
    }
}
