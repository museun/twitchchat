use super::{FromIrcMessage, IrcMessage};

#[derive(Debug, Clone)]
pub struct RoomState<'a> {
    s: &'a (),
}
