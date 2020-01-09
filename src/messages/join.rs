use super::*;

/// User join message
///
/// The happens when a user (yourself included) joins a channel
#[derive(Debug, Clone, PartialEq)]
pub struct Join<T = String>
where
    T: StringMarker,
{
    /// Name of the user that joined the channel
    pub user: T,
    /// Channel which they joined
    pub channel: T,
}

impl<'a> TryFrom<&'a Message<&'a str>> for Join<&'a str> {
    type Error = InvalidMessage;

    fn try_from(msg: &'a Message<&'a str>) -> Result<Self, Self::Error> {
        msg.expect_command("JOIN").and_then(|_| {
            Ok(Self {
                user: msg.expect_nick()?,
                channel: msg.expect_arg(0)?,
            })
        })
    }
}

as_owned!(for Join {
    user,
    channel
});

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse() {
        let input = ":test!test@test JOIN #foo\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                Join::<String>::try_from(&msg).unwrap(),
                Join {
                    user: "test".into(),
                    channel: "#foo".into()
                }
            );
            assert_eq!(
                Join::<&str>::try_from(&msg).unwrap(),
                Join {
                    user: "test",
                    channel: "#foo"
                }
            )
        }
    }

    #[test]
    fn bad_command() {
        let input = crate::decode_many(":tmi.twitch.tv NOT_JOIN #foo\r\n")
            .flatten()
            .next()
            .unwrap();

        let err = Join::<&str>::try_from(&input).unwrap_err();
        matches::matches!(
            err,
            InvalidMessage::InvalidCommand {..}
        );
    }

    #[test]
    fn bad_nick() {
        let input = crate::decode_many(":tmi.twitch.tv JOIN #foo\r\n")
            .flatten()
            .next()
            .unwrap();

        let err = Join::<&str>::try_from(&input).unwrap_err();
        matches::matches!(err, InvalidMessage::ExpectedNick);
    }

    #[test]
    fn bad_channel() {
        let input = crate::decode_many(":tmi.twitch.tv JOIN\r\n")
            .flatten()
            .next()
            .unwrap();

        let err = Join::<&str>::try_from(&input).unwrap_err();
        matches::matches!(err, InvalidMessage::ExpectedArg { pos: 0 });
    }
}
