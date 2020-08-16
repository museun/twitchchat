use crate::*;

/// User join message
///
/// The happens when a user (yourself included) joins a channel
#[derive(Clone, PartialEq)]
pub struct Join<'a> {
    raw: Str<'a>,
    name: StrIndex,
    channel: StrIndex,
}

impl<'a> Join<'a> {
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

impl<'a> FromIrcMessage<'a> for Join<'a> {
    type Error = InvalidMessage;

    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error> {
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
impl_custom_debug!(Join { raw, name, channel });
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
