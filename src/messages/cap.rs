use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Cap<T = String>
where
    T: StringMarker,
{
    pub capability: T,
    pub acknowledged: bool,
}

impl<'a> TryFrom<&'a Message<&'a str>> for Cap<&'a str> {
    type Error = InvalidMessage;

    fn try_from(msg: &'a Message<&'a str>) -> Result<Self, Self::Error> {
        msg.expect_command("CAP").and_then(|_| {
            let acknowledged = msg.expect_arg(1)? == "ACK";
            let capability = msg.expect_data()?;
            Ok(Self {
                capability,
                acknowledged,
            })
        })
    }
}

as_owned!(for Cap {
    capability,
    acknowledged
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ack() {
        use std::convert::TryInto;

        let input = ":tmi.twitch.tv CAP * ACK :twitch.tv/membership\r\n\
                     :tmi.twitch.tv CAP * ACK :twitch.tv/tags\r\n\
                     :tmi.twitch.tv CAP * ACK :twitch.tv/commands\r\n";

        let expected = &[
            "twitch.tv/membership",
            "twitch.tv/tags",
            "twitch.tv/commands",
        ];
        for (msg, expected) in crate::decode_many(&input)
            .map(|s| s.unwrap())
            .zip(expected.into_iter())
        {
            let msg: Cap<&str> = (&msg).try_into().unwrap();
            assert!(msg.acknowledged);
            assert_eq!(msg.capability, *expected);
        }
    }

    #[test]
    fn parse_nal() {
        use std::convert::TryInto;

        let input = ":tmi.twitch.tv CAP * NAK :foobar\r\n";

        let msg = crate::decode_many(&input).next().unwrap().unwrap();
        let msg: Cap<&str> = (&msg).try_into().unwrap();

        assert!(!msg.acknowledged);
        assert_eq!(msg.capability, "foobar");
    }
}
