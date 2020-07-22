use super::{FromIrcMessage, IrcMessage};

#[derive(Debug, Clone)]
pub struct GlobalUserState<'a> {
    s: &'a (),
}
