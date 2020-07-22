use super::{FromIrcMessage, IrcMessage};

#[derive(Debug, Clone)]
pub struct Ready<'a> {
    s: &'a (),
}
