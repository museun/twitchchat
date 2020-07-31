use crate::ng::{FromIrcMessage, InvalidMessage, Validator};
use crate::ng::{IrcMessage, Str, StrIndex};

#[derive(Debug, Clone, PartialEq)]
pub struct Ready<'t> {
    raw: Str<'t>,
    username: StrIndex,
}

impl<'t> Ready<'t> {
    raw!();
    str_field!(username);
}

impl<'t> FromIrcMessage<'t> for Ready<'t> {
    type Error = InvalidMessage;

    fn from_irc(msg: IrcMessage<'t>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::READY)?;

        let this = Self {
            username: msg.expect_arg_index(0)?,
            raw: msg.raw,
        };

        Ok(this)
    }
}

serde_struct!(Ready { raw, username });

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ng::irc;

    #[test]
    #[cfg(feature = "serde")]
    fn ready_serde() {
        let input = ":tmi.twitch.tv 376 shaken_bot :>\r\n";
        crate::ng::serde::round_trip_json::<Ready>(input);
    }

    #[test]
    fn ready() {
        let input = ":tmi.twitch.tv 376 shaken_bot :>\r\n";
        for irc in irc::parse(input).map(|s| s.unwrap()) {
            let msg = Ready::from_irc(irc).unwrap();
            assert_eq!(msg.username(), "shaken_bot")
        }
    }
}
