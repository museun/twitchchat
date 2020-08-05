use crate::ng::{FromIrcMessage, InvalidMessage, Validator};
use crate::ng::{IrcMessage, Str, StrIndex};

/// A pong response sent from the server
///
/// This should be a response to sending a PING to the server
#[derive(Debug, Clone, PartialEq)]
pub struct Pong<'t> {
    raw: Str<'t>,
    token: StrIndex,
}

impl<'t> Pong<'t> {
    raw!();
    str_field!(
        /// Token associated with the PONG event
        token
    );
}

impl<'t> FromIrcMessage<'t> for Pong<'t> {
    type Error = InvalidMessage;

    fn from_irc(msg: IrcMessage<'t>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::PONG)?;

        let this = Self {
            token: msg.expect_data_index()?,
            raw: msg.raw,
        };

        Ok(this)
    }
}

serde_struct!(Pong { raw, token });

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ng::irc;

    #[test]
    #[cfg(feature = "serde")]
    fn pong_serde() {
        let input = "PONG :1234567890\r\n";
        crate::ng::serde::round_trip_json::<Pong>(input);
    }

    #[test]
    fn pong() {
        let input = "PONG :1234567890\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let msg = Pong::from_irc(msg).unwrap();
            assert_eq!(msg.token(), "1234567890");
        }
    }
}
