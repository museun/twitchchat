use super::{FromIrcMessage, IrcMessage};

#[derive(Debug, Clone)]
pub struct Ping<'a> {
    s: &'a (),
}
