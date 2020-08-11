use crate::*;

/// Signals that you should reconnect and rejoin channels after a restart.
///
/// Twitch IRC processes occasionally need to be restarted. When this happens,
/// clients that have requested the IRC v3 `twitch.tv/commands` _capability_ are
/// issued a `RECONNECT`. After a short time, the connection is closed. In this
/// case, reconnect and rejoin channels that were on the connection, as you
/// would normally.
#[derive(Debug, Clone, PartialEq)]
pub struct Reconnect<'a> {
    raw: Str<'a>,
}

impl<'a> Reconnect<'a> {
    raw!();
}

impl<'a> FromIrcMessage<'a> for Reconnect<'a> {
    type Error = InvalidMessage;

    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::RECONNECT)?;
        Ok(Self { raw: msg.raw })
    }
}

into_owned!(Reconnect { raw });

serde_struct!(Reconnect { raw });

#[cfg(test)]
mod tests {
    use super::*;
    use crate::irc;

    #[test]
    #[cfg(feature = "serde")]
    fn reconnect_serde() {
        let input = ":tmi.twitch.tv RECONNECT\r\n";
        crate::serde::round_trip_json::<Reconnect>(input);
    }

    #[test]
    fn reconnect() {
        let input = ":tmi.twitch.tv RECONNECT\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let _msg = Reconnect::from_irc(msg).unwrap();
        }
    }
}
