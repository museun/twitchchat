use super::{FromIrcMessage, IrcMessage};

#[derive(Debug, Clone)]
pub struct ClearMsg<'a> {
    s: &'a (),
}
