use super::{FromIrcMessage, IrcMessage};

#[derive(Debug, Clone)]
pub struct ClearChat<'a> {
    s: &'a (),
}
