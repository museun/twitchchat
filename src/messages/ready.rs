use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Ready<T = String>
where
    T: StringMarker,
{
    pub username: T,
}

impl<'a> TryFrom<&'a Message<&'a str>> for Ready<&'a str> {
    type Error = InvalidMessage;

    fn try_from(msg: &'a Message<&'a str>) -> Result<Self, Self::Error> {
        msg.expect_command("376")
            .and_then(|_| msg.expect_arg(0).map(|username| Self { username }))
    }
}

as_owned!(for Ready {
    username
});

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse() {
        let input = ":tmi.twitch.tv 376 shaken_bot :>\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                Ready::<String>::try_from(&msg).unwrap(),
                Ready {
                    username: "shaken_bot".into(),
                }
            );
            assert_eq!(
                Ready::<&str>::try_from(&msg).unwrap(),
                Ready {
                    username: "shaken_bot",
                }
            )
        }
    }
}
