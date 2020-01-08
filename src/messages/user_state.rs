use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct UserState<T = String>
where
    T: StringMarker,
{
    pub tags: Tags<T>,
    pub channel: T,
}

impl<'a> TryFrom<&'a Message<&'a str>> for UserState<&'a str> {
    type Error = InvalidMessage;

    fn try_from(msg: &'a Message<&'a str>) -> Result<Self, Self::Error> {
        msg.expect_command("USERSTATE").and_then(|_| {
            msg.expect_arg(0).map(|channel| Self {
                channel,
                tags: msg.tags.clone(),
            })
        })
    }
}

as_owned!(for UserState{
    tags,
    channel
});

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse() {
        let input = ":tmi.twitch.tv USERSTATE #museun\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                UserState::<String>::try_from(&msg).unwrap(),
                UserState {
                    channel: "#museun".into(),
                    tags: Tags::default()
                }
            );
            assert_eq!(
                UserState::<&str>::try_from(&msg).unwrap(),
                UserState {
                    channel: "#museun",
                    tags: Tags::default()
                }
            )
        }
    }
}
