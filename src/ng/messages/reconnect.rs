use super::{FromIrcMessage, InvalidMessage, IrcMessage, Str, Validator};

#[derive(Debug, Clone, PartialEq)]
pub struct Reconnect<'t> {
    raw: Str<'t>,
}

impl<'t> Reconnect<'t> {
    raw!();
}

impl<'t> FromIrcMessage<'t> for Reconnect<'t> {
    type Error = InvalidMessage;

    fn from_irc(msg: IrcMessage<'t>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::RECONNECT)?;
        Ok(Self { raw: msg.raw })
    }
}

serde_struct!(Reconnect { raw });

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ng::irc;

    #[test]
    fn reconnect_serde() {
        let input = ":tmi.twitch.tv RECONNECT\r\n";
        crate::ng::serde::round_trip_json::<Reconnect>(input);
    }

    #[test]
    fn reconnect() {
        let input = ":tmi.twitch.tv RECONNECT\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let _msg = Reconnect::from_irc(msg).unwrap();
        }
    }
}
