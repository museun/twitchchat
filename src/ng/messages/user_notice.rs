use super::{FromIrcMessage, IrcMessage};

#[derive(Debug, Clone)]
pub struct UserNotice<'a> {
    s: &'a (),
}
