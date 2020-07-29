use super::{FromIrcMessage, InvalidMessage, IrcMessage, Str, Validator};

#[derive(Debug, Clone)]
pub struct Ready<'a> {
    s: &'a (),
}
