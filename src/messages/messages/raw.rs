use super::*;

/// A raw IRC message
pub type Raw<'t> = Message<'t>;

impl<'a: 't, 't> Parse<&'a Message<'t>> for Raw<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        Ok(msg.clone())
    }
}
