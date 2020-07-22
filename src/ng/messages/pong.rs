use super::{FromIrcMessage, IrcMessage};

#[derive(Debug, Clone)]
pub struct Pong<'a> {
    s: &'a (),
}
