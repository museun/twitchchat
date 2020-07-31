use crate::ng::{FromIrcMessage, InvalidMessage, Validator};
use crate::ng::{IrcMessage, Str, StrIndex};

#[derive(Debug, Clone, PartialEq)]
pub struct Part<'t> {
    raw: Str<'t>,
    name: StrIndex,
    channel: StrIndex,
}

impl<'t> Part<'t> {
    raw!();
    str_field!(name);
    str_field!(channel);
}

impl<'t> FromIrcMessage<'t> for Part<'t> {
    type Error = InvalidMessage;

    fn from_irc(msg: IrcMessage<'t>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::PART)?;

        let this = Self {
            channel: msg.expect_arg_index(0)?,
            name: msg.expect_nick()?,
            raw: msg.raw,
        };

        Ok(this)
    }
}

serde_struct!(Part { raw, name, channel });

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ng::irc;

    #[test]
    #[cfg(feature = "serde")]
    fn part_serde() {
        let input = ":test!test@test PART #museun\r\n";
        crate::ng::serde::round_trip_json::<Part>(input);
    }

    #[test]
    fn part() {
        let input = ":test!test@test PART #museun\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let msg = Part::from_irc(msg).unwrap();
            assert_eq!(msg.name(), "test");
            assert_eq!(msg.channel(), "#museun");
        }
    }
}
