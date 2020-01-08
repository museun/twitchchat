use super::*;

/// User leave message
///
/// The happens when a user (yourself included) leaves a channel
#[derive(Debug, Clone, PartialEq)]
pub struct Part<T = String>
where
    T: StringMarker,
{
    /// Name of the user that left the channel
    pub user: T,
    /// Channel which they left
    pub channel: T,
}

impl<'a> TryFrom<&'a Message<&'a str>> for Part<&'a str> {
    type Error = InvalidMessage;

    fn try_from(msg: &'a Message<&'a str>) -> Result<Self, Self::Error> {
        msg.expect_command("PART").and_then(|_| {
            Ok(Self {
                user: msg.expect_nick()?,
                channel: msg.expect_arg(0)?,
            })
        })
    }
}

as_owned!(for Part {
    user,
    channel
});

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse() {
        let input = ":test!test@test PART #foo\r\n:test!test@test PART #foo :with a message\r\n";

        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                Part::<String>::try_from(&msg).unwrap(),
                Part {
                    user: "test".into(),
                    channel: "#foo".into()
                }
            );

            assert_eq!(
                Part::<&str>::try_from(&msg).unwrap(),
                Part {
                    user: "test",
                    channel: "#foo"
                }
            )
        }
    }

    #[test]
    fn bad_command() {
        let input = crate::decode_many(":tmi.twitch.tv NOT_PART #foo\r\n")
            .flatten()
            .next()
            .unwrap();

        let err = Part::<&str>::try_from(&input).unwrap_err();
        matches::matches!(
            err,
            InvalidMessage::InvalidCommand {..}
        );
    }

    #[test]
    fn bad_nick() {
        let input = crate::decode_many(":tmi.twitch.tv PART #foo\r\n")
            .flatten()
            .next()
            .unwrap();

        let err = Part::<&str>::try_from(&input).unwrap_err();
        matches::matches!(err, InvalidMessage::ExpectedNick);
    }

    #[test]
    fn bad_channel() {
        let input = crate::decode_many(":tmi.twitch.tv PART\r\n")
            .flatten()
            .next()
            .unwrap();

        let err = Part::<&str>::try_from(&input).unwrap_err();
        matches::matches!(err, InvalidMessage::ExpectedArg { pos: 0 });
    }
}
