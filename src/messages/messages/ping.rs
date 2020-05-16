use super::*;

/// A ping request from the server
///
/// This is sent periodically, and handled by the `Client` internally
///
/// But you can use them however you want
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ping<'t> {
    /// Token associated with the PING event
    pub token: Cow<'t, str>,
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for Ping<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("PING")?;
        msg.expect_data().map(|token| Self {
            token: token.clone(),
        })
    }
}

impl<'t> AsOwned for Ping<'t> {
    type Owned = Ping<'static>;
    fn as_owned(&self) -> Self::Owned {
        Ping {
            token: self.token.as_owned(),
        }
    }
}
