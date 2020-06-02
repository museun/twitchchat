use super::*;

/// Acknowledgement (or not) on a CAPS request
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cap<'t> {
    /// The capability name
    pub capability: Cow<'t, str>,
    /// Whether it was acknowledged
    pub acknowledged: bool,
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for Cap<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("CAP")?;
        let acknowledged = msg.expect_arg(1)? == "ACK";
        let capability = msg.expect_data()?;
        Ok(Self {
            capability,
            acknowledged,
        })
    }
}

impl<'t> AsOwned for Cap<'t> {
    type Owned = Cap<'static>;
    fn as_owned(&self) -> Self::Owned {
        Cap {
            capability: self.capability.as_owned(),
            acknowledged: self.acknowledged.as_owned(),
        }
    }
}
