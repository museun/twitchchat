use crate::ng::{FromIrcMessage, InvalidMessage, Validator};
use crate::ng::{IrcMessage, Str, StrIndex, TagIndices, Tags};

#[derive(Debug, Clone, PartialEq)]
pub struct ClearMsg<'t> {
    raw: Str<'t>,
    tags: TagIndices,
    channel: StrIndex,
    message: Option<StrIndex>,
}

impl<'t> ClearMsg<'t> {
    raw!();
    tags!();
    str_field!(channel);
    opt_str_field!(message);

    pub fn login(&self) -> Option<&str> {
        self.tags().get("login")
    }

    pub fn target_msg_id(&self) -> Option<&str> {
        self.tags().get("target-msg-id")
    }
}

impl<'t> FromIrcMessage<'t> for ClearMsg<'t> {
    type Error = InvalidMessage;

    fn from_irc(msg: IrcMessage<'t>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::CLEARMSG)?;

        let this = Self {
            tags: msg.parse_tags(),
            channel: msg.expect_arg_index(0)?,
            message: msg.data,
            raw: msg.raw,
        };

        Ok(this)
    }
}

serde_struct!(ClearMsg {
    raw,
    tags,
    channel,
    message,
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ng::irc;

    #[test]
    fn clear_msg_serde() {
        let input = ":tmi.twitch.tv CLEARMSG #museun :HeyGuys\r\n";
        crate::ng::serde::round_trip_json::<ClearMsg>(input);
    }

    #[test]
    fn clear_msg() {
        let input = ":tmi.twitch.tv CLEARMSG #museun :HeyGuys\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let cm = ClearMsg::from_irc(msg).unwrap();
            assert_eq!(cm.channel(), "#museun");
            assert_eq!(cm.message().unwrap(), "HeyGuys");
        }
    }

    #[test]
    fn clear_msg_empty() {
        let input = ":tmi.twitch.tv CLEARMSG #museun\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let cm = ClearMsg::from_irc(msg).unwrap();
            assert_eq!(cm.channel(), "#museun");
            assert!(cm.message().is_none());
        }
    }
}
