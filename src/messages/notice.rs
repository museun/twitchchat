use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Notice<T = String>
where
    T: StringMarker,
{
    pub tags: Tags<T>,
    pub channel: T,
    pub message: T,
}

impl<'a> TryFrom<&'a Message<&'a str>> for Notice<&'a str> {
    type Error = InvalidMessage;

    fn try_from(msg: &'a Message<&'a str>) -> Result<Self, Self::Error> {
        msg.expect_command("NOTICE").and_then(|_| {
            Ok(Self {
                tags: msg.tags.clone(),
                channel: msg.expect_arg(0)?,
                message: msg.expect_data()?,
            })
        })
    }
}

as_owned!(for Notice {
    tags,
    message,
    channel
});

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse() {
        let input = ":tmi.twitch.tv NOTICE #museun :This room is no longer in slow mode.\r\n";

        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                Notice::<String>::try_from(&msg).unwrap(),
                Notice {
                    tags: Tags::default(),
                    channel: "#museun".into(),
                    message: "This room is no longer in slow mode.".into(),
                }
            );

            assert_eq!(
                Notice::<&str>::try_from(&msg).unwrap(),
                Notice {
                    tags: Tags::default(),
                    channel: "#museun",
                    message: "This room is no longer in slow mode.",
                }
            )
        }
    }
}
