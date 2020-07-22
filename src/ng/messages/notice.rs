use super::{FromIrcMessage, IrcMessage};

#[derive(Debug, Clone)]
pub struct Notice<'a> {
    s: &'a (),
}
