use super::{FromIrcMessage, InvalidMessage, IrcMessage, Str, Validator};

#[derive(Debug, Clone)]
pub struct IrcReady<'a> {
    s: &'a (),
}
