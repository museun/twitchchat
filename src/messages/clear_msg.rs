use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct ClearMsg<T = String>
where
    T: StringMarker,
{
    pub tags: Tags<T>,
    pub channel: T,
    pub message: Option<T>,
}

impl<'a> TryFrom<&'a Message<&'a str>> for ClearMsg<&'a str> {
    type Error = InvalidMessage;

    fn try_from(msg: &'a Message<&'a str>) -> Result<Self, Self::Error> {
        msg.expect_command("CLEARMSG").and_then(|_| {
            Ok(Self {
                tags: msg.tags.clone(),
                channel: msg.expect_arg(0)?,
                message: msg.expect_data().ok(),
            })
        })
    }
}

as_owned!(for ClearMsg {
    tags,
    message,
    channel
});

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse() {
        let input = ":tmi.twitch.tv CLEARMSG #museun :HeyGuys\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                ClearMsg::<String>::try_from(&msg).unwrap(),
                ClearMsg {
                    tags: Tags::default(),
                    channel: "#museun".into(),
                    message: Some("HeyGuys".into()),
                }
            );

            assert_eq!(
                ClearMsg::<&str>::try_from(&msg).unwrap(),
                ClearMsg {
                    tags: Tags::default(),
                    channel: "#museun",
                    message: Some("HeyGuys"),
                }
            )
        }
    }

    #[test]
    fn no_message() {
        let input = ":tmi.twitch.tv CLEARMSG #museun\r\n";

        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                ClearMsg::<String>::try_from(&msg).unwrap(),
                ClearMsg {
                    tags: Tags::default(),
                    channel: "#museun".into(),
                    message: None,
                }
            );

            assert_eq!(
                ClearMsg::<&str>::try_from(&msg).unwrap(),
                ClearMsg {
                    tags: Tags::default(),
                    channel: "#museun",
                    message: None,
                }
            )
        }
    }
}
