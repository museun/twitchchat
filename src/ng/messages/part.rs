use super::{FromIrcMessage, IrcMessage};

#[derive(Debug, Clone)]
pub struct Part<'a> {
    s: &'a (),
}
