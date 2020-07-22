use super::{FromIrcMessage, IrcMessage};

#[derive(Debug, Clone)]
pub struct Privmsg<'a> {
    s: &'a (),
}
