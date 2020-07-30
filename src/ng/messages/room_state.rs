use crate::ng::{FromIrcMessage, InvalidMessage, Validator};
use crate::ng::{IrcMessage, Str, StrIndex, TagIndices, Tags};

#[derive(Debug, Clone, PartialEq)]
pub struct RoomState<'t> {
    raw: Str<'t>,
    tags: TagIndices,
    channel: StrIndex,
}

impl<'a> FromIrcMessage<'a> for RoomState<'a> {
    type Error = InvalidMessage;
    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::ROOMSTATE)?;

        let this = Self {
            tags: msg.parse_tags(),
            channel: msg.expect_arg_index(0)?,
            raw: msg.raw,
        };

        Ok(this)
    }
}

impl<'t> RoomState<'t> {
    raw!();
    tags!();
    str_field!(channel);
}

serde_struct!(RoomState { raw, tags, channel });

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ng::irc;

    #[test]
    fn user_state_serde() {
        let input = ":tmi.twitch.tv ROOMSTATE #museun\r\n";
        crate::ng::serde::round_trip_json::<RoomState>(input);
    }

    #[test]
    fn user_state() {
        let input = ":tmi.twitch.tv ROOMSTATE #museun\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let msg = RoomState::from_irc(msg).unwrap();
            assert_eq!(msg.channel(), "#museun");
        }
    }
}
