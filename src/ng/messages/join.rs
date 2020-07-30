use crate::ng::{FromIrcMessage, InvalidMessage, Validator};
use crate::ng::{IrcMessage, Str, StrIndex};

#[derive(Debug, Clone, PartialEq)]
pub struct Join<'t> {
    raw: Str<'t>,
    name: StrIndex,
    channel: StrIndex,
}

impl<'t> Join<'t> {
    raw!();
    str_field!(name);
    str_field!(channel);
}

impl<'t> FromIrcMessage<'t> for Join<'t> {
    type Error = InvalidMessage;

    fn from_irc(msg: IrcMessage<'t>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::JOIN)?;

        let this = Self {
            channel: msg.expect_arg_index(0)?,
            name: msg.expect_nick()?,
            raw: msg.raw,
        };

        Ok(this)
    }
}

serde_struct!(Join { raw, name, channel });

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ng::irc;

    #[test]
    fn join_serde() {
        let input = ":test!test@test JOIN #foo\r\n";
        crate::ng::serde::round_trip_json::<Join>(input);
    }

    #[test]
    fn join_bad_command() {
        let input = ":tmi.twitch.tv NOT_JOIN #foo\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let err = Join::from_irc(msg).unwrap_err();
            assert!(matches!(err,InvalidMessage::InvalidCommand { .. }))
        }
    }

    #[test]
    fn join_bad_nick() {
        let input = ":tmi.twitch.tv JOIN #foo\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let err = Join::from_irc(msg).unwrap_err();
            assert!(matches!(err, InvalidMessage::ExpectedNick))
        }
    }

    #[test]
    fn join_bad_channel() {
        let input = ":tmi.twitch.tv JOIN\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let err = Join::from_irc(msg).unwrap_err();
            assert!(matches!(dbg!(err), InvalidMessage::ExpectedArg { pos: 0 }))
        }
    }

    #[test]
    fn join() {
        let input = ":test!test@test JOIN #foo\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let msg = Join::from_irc(msg).unwrap();
            assert_eq!(msg.name(), "test");
            assert_eq!(msg.channel(), "#foo");
        }
    }
}
