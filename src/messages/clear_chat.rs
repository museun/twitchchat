use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct ClearChat<T = String>
where
    T: StringMarker,
{
    pub tags: Tags<T>,
    pub channel: T,
    pub user: Option<T>,
}

impl<'a> TryFrom<&'a Message<&'a str>> for ClearChat<&'a str> {
    type Error = InvalidMessage;

    fn try_from(msg: &'a Message<&'a str>) -> Result<Self, Self::Error> {
        msg.expect_command("CLEARCHAT").and_then(|_| {
            Ok(Self {
                tags: msg.tags.clone(),
                channel: msg.expect_arg(0)?,
                user: msg.expect_data().ok(),
            })
        })
    }
}

as_owned!(for ClearChat {
    tags,
    user,
    channel
});

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse() {
        let input = ":tmi.twitch.tv CLEARCHAT #museun :shaken_bot\r\n";

        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                ClearChat::<String>::try_from(&msg).unwrap(),
                ClearChat {
                    tags: Tags::default(),
                    channel: "#museun".into(),
                    user: Some("shaken_bot".into()),
                }
            );

            assert_eq!(
                ClearChat::<&str>::try_from(&msg).unwrap(),
                ClearChat {
                    tags: Tags::default(),
                    channel: "#museun",
                    user: Some("shaken_bot"),
                }
            )
        }
    }

    #[test]
    fn no_user() {
        let input = ":tmi.twitch.tv CLEARCHAT #museun\r\n";

        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                ClearChat::<String>::try_from(&msg).unwrap(),
                ClearChat {
                    tags: Tags::default(),
                    channel: "#museun".into(),
                    user: None,
                }
            );

            assert_eq!(
                ClearChat::<&str>::try_from(&msg).unwrap(),
                ClearChat {
                    tags: Tags::default(),
                    channel: "#museun",
                    user: None,
                }
            )
        }
    }
}
