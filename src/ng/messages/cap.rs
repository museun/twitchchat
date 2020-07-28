use super::{AsOwned, FromIrcMessage, InvalidMessage, IrcMessage, Reborrow, Str, Validator};

/// Acknowledgement (or not) for a CAPS request
#[derive(Debug, Clone)]
pub struct Cap<'t> {
    /// The capability name
    pub capability: Str<'t>,
    /// Whether it was acknowledged
    pub acknowledged: bool,
}

impl<'a> FromIrcMessage<'a> for Cap<'a> {
    type Error = InvalidMessage;

    fn from_irc(msg: &'a IrcMessage<'a>) -> Result<Self, Self::Error> {
        msg.expect_command("CAP")?;

        let this = Self {
            acknowledged: msg.expect_arg(1)? == "ACK",
            capability: msg.expect_data()?,
        };

        Ok(this)
    }
}

reborrow_and_asowned!(Cap {
    capability,
    acknowledged
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ng::irc;

    #[test]
    fn cap_acknowledged() {
        let input = ":tmi.twitch.tv CAP * ACK :twitch.tv/membership\r\n\
                     :tmi.twitch.tv CAP * ACK :twitch.tv/tags\r\n\
                     :tmi.twitch.tv CAP * ACK :twitch.tv/commands\r\n";
        let expected = &[
            "twitch.tv/membership",
            "twitch.tv/tags",
            "twitch.tv/commands",
        ];
        for (msg, expected) in irc::parse(&input).map(|s| s.unwrap()).zip(expected) {
            let msg = Cap::from_irc(&msg).unwrap();
            assert!(msg.acknowledged);
            assert_eq!(msg.capability, *expected);
        }
    }

    #[test]
    fn cap_failed() {
        let input = ":tmi.twitch.tv CAP * NAK :foobar\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let cap = Cap::from_irc(&msg).unwrap();
            assert!(!cap.acknowledged);
            assert_eq!(cap.capability, "foobar");
        }
    }
}
