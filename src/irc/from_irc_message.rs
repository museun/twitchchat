use crate::IrcMessage;
use std::convert::Infallible;

pub trait FromIrcMessage<'a>: Sized {
    type Error;
    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error>;
}

impl<'a> FromIrcMessage<'a> for IrcMessage<'a> {
    type Error = Infallible;
    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error> {
        Ok(msg)
    }
}
