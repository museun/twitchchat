use super::{AsOwned, FromIrcMessage, InvalidMessage, IrcMessage, Reborrow, Str, Validator};

#[derive(Debug, Clone)]
pub struct RoomState<'a> {
    s: &'a (),
}
