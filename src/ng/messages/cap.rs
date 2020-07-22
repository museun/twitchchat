use super::{FromIrcMessage, IrcMessage};

#[derive(Debug, Clone)]
pub struct Cap<'a> {
    s: &'a (),
}

impl<'a> FromIrcMessage<'a> for Cap<'a> {
    type Error = (); // TODO do I really want an error here?
    fn from_irc(msg: &IrcMessage<'a>) -> Result<Self, Self::Error>
    where
        Self: Sized + 'a,
    {
        todo!()
    }
}
