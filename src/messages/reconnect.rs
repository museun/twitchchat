use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Reconnect {}

impl<'a> TryFrom<&'a Message<&'a str>> for Reconnect {
    type Error = InvalidMessage;

    fn try_from(msg: &'a Message<&'a str>) -> Result<Self, Self::Error> {
        msg.expect_command("RECONNECT").map(|_| Self {})
    }
}

as_owned!(for Reconnect);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse() {
        let input = ":tmi.twitch.tv RECONNECT\r\n";

        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(Reconnect::try_from(&msg).unwrap(), Reconnect {});
        }
    }
}
