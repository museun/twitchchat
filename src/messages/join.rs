use crate::{FromIrcMessage, IrcError, Validator};
use crate::{IrcMessage, Str, StrIndex};

/// User join message
///
/// The happens when a user (yourself included) joins a channel
#[derive(Debug, Clone, PartialEq)]
pub struct Join<'t> {
    raw: Str<'t>,
    name: StrIndex,
    channel: StrIndex,
}

impl<'t> Join<'t> {
    raw!();
    str_field!(
        /// Name of the user that joined the channel
        name
    );
    str_field!(
        /// Channel which they joined
        channel
    );
}

impl<'t> FromIrcMessage<'t> for Join<'t> {
    type Error = IrcError;

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

into_owned!(Join { raw, name, channel });

serde_struct!(Join { raw, name, channel });

#[cfg(test)]
mod tests {
    use super::*;
    use crate::irc;

    #[test]
    #[cfg(feature = "serde")]
    fn join_serde() {
        let input = ":test!test@test JOIN #foo\r\n";
        crate::serde::round_trip_json::<Join>(input);
    }

    #[test]
    fn join_bad_command() {
        let input = ":tmi.twitch.tv NOT_JOIN #foo\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let err = Join::from_irc(msg).unwrap_err();
            assert!(matches!(err,IrcError::InvalidCommand { .. }))
        }
    }

    #[test]
    fn join_bad_nick() {
        let input = ":tmi.twitch.tv JOIN #foo\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let err = Join::from_irc(msg).unwrap_err();
            assert!(matches!(err, IrcError::ExpectedNick))
        }
    }

    #[test]
    fn join_bad_channel() {
        let input = ":tmi.twitch.tv JOIN\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let err = Join::from_irc(msg).unwrap_err();
            assert!(matches!(dbg!(err), IrcError::ExpectedArg { pos: 0 }))
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
