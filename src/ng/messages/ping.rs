use crate::ng::{FromIrcMessage, InvalidMessage, Validator};
use crate::ng::{IrcMessage, Str, StrIndex};

#[derive(Debug, Clone, PartialEq)]
pub struct Ping<'t> {
    raw: Str<'t>,
    token: StrIndex,
}

impl<'t> Ping<'t> {
    raw!();
    str_field!(token);
}

impl<'t> FromIrcMessage<'t> for Ping<'t> {
    type Error = InvalidMessage;

    fn from_irc(msg: IrcMessage<'t>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::PING)?;

        let this = Self {
            token: msg.expect_data_index()?,
            raw: msg.raw,
        };

        Ok(this)
    }
}

serde_struct!(Ping { raw, token });

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ng::irc;

    #[test]
    fn ping_serde() {
        let input = "PING :1234567890\r\n";
        crate::ng::serde::round_trip_json::<Ping>(input);
    }

    #[test]
    fn ping() {
        let input = "PING :1234567890\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let msg = Ping::from_irc(msg).unwrap();
            assert_eq!(msg.token(), "1234567890");
        }
    }
}
