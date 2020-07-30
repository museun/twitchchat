use super::{IrcMessage, Str, StrIndex};
use crate::ng::{FromIrcMessage, InvalidMessage, Validator};

#[derive(Debug, Clone, PartialEq)]
pub struct Cap<'t> {
    raw: Str<'t>,
    capability: StrIndex,
    acknowledged: bool,
}

impl<'t> Cap<'t> {
    raw!();
    str_field!(capability);

    pub fn acknowledged(&self) -> bool {
        self.acknowledged
    }
}

impl<'a> FromIrcMessage<'a> for Cap<'a> {
    type Error = InvalidMessage;

    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error> {
        const ACK: &str = "ACK";

        msg.expect_command(IrcMessage::CAP)?;

        let this = Self {
            capability: msg.expect_data_index()?,
            acknowledged: msg.expect_arg(1)? == ACK,
            raw: msg.raw,
        };

        Ok(this)
    }
}

serde_struct!(Cap {
    raw,
    capability,
    acknowledged,
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ng::irc;

    #[test]
    fn cap_serde() {
        let input = ":tmi.twitch.tv CAP * ACK :twitch.tv/membership\r\n";
        crate::ng::serde::round_trip_json::<Cap>(input);
    }

    #[test]
    fn cap_acknowledged() {
        let input = ":tmi.twitch.tv CAP * ACK :twitch.tv/membership\r\n\
                     :tmi.twitch.tv CAP * ACK :twitch.tv/tags\r\n\
                     :tmi.twitch.tv CAP * ACK :twitch.tv/commands\r\n";
        let expected = &[
            "twitch.tv/membership",
            "twitch.tv/tags",
            "twitch.tv/commands",
        ];
        for (msg, expected) in irc::parse(&input).map(|s| s.unwrap()).zip(expected) {
            let msg = Cap::from_irc(msg).unwrap();
            assert!(msg.acknowledged());
            assert_eq!(msg.capability(), *expected);
        }
    }

    #[test]
    fn cap_failed() {
        let input = ":tmi.twitch.tv CAP * NAK :foobar\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let cap = Cap::from_irc(msg).unwrap();
            assert!(!cap.acknowledged());
            assert_eq!(cap.capability(), "foobar");
        }
    }
}
