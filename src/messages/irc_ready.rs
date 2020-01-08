use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct IrcReady<T = String>
where
    T: StringMarker,
{
    pub nickname: T,
}

impl<'a> TryFrom<&'a Message<&'a str>> for IrcReady<&'a str> {
    type Error = InvalidMessage;

    fn try_from(msg: &'a Message<&'a str>) -> Result<Self, Self::Error> {
        msg.expect_command("001")
            .and_then(|_| msg.expect_arg(0).map(|nickname| Self { nickname }))
    }
}

as_owned!(for IrcReady {
    nickname
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let good = ":tmi.twitch.tv 001 shaken_bot :Welcome, GLHF!\r\n";
        let msg = crate::decode(good).unwrap().1;

        assert_eq!(
            IrcReady::<String>::try_from(&msg).unwrap(),
            IrcReady {
                nickname: "shaken_bot".into()
            }
        );

        assert_eq!(
            IrcReady::<&str>::try_from(&msg).unwrap(),
            IrcReady {
                nickname: "shaken_bot"
            }
        )
    }
}
