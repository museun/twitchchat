use crate::ng::{FromIrcMessage, InvalidMessage, Validator};
use crate::ng::{IrcMessage, Str, StrIndex, TagIndices, Tags};

#[derive(Debug, Clone, PartialEq)]
pub struct ClearChat<'t> {
    raw: Str<'t>,
    tags: TagIndices,
    channel: StrIndex,
    name: Option<StrIndex>,
}

impl<'t> ClearChat<'t> {
    raw!();
    str_field!(channel);
    opt_str_field!(name);
    tags!();

    pub fn ban_duration(&self) -> Option<u64> {
        self.tags().get_parsed("ban-duration")
    }
}

impl<'t> FromIrcMessage<'t> for ClearChat<'t> {
    type Error = InvalidMessage;

    fn from_irc(msg: IrcMessage<'t>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::CLEARCHAT)?;

        let this = Self {
            tags: msg.parse_tags(),
            channel: msg.expect_arg_index(0)?,
            name: msg.data,
            raw: msg.raw,
        };

        Ok(this)
    }
}

serde_struct!(ClearChat {
    raw,
    tags,
    channel,
    name
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ng::irc;

    #[test]
    fn clear_chat_serde() {
        let input = ":tmi.twitch.tv CLEARCHAT #museun :shaken_bot\r\n";
        crate::ng::serde::round_trip_json::<ClearChat>(input);
    }

    #[test]
    fn clear_chat() {
        let input = ":tmi.twitch.tv CLEARCHAT #museun :shaken_bot\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let cc = ClearChat::from_irc(msg).unwrap();
            assert_eq!(cc.channel(), "#museun");
            assert_eq!(cc.name().unwrap(), "shaken_bot");
        }
    }

    #[test]
    fn clear_chat_empty() {
        let input = ":tmi.twitch.tv CLEARCHAT #museun\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let cc = ClearChat::from_irc(msg).unwrap();
            assert_eq!(cc.channel(), "#museun");
            assert!(cc.name().is_none());
        }
    }
}
