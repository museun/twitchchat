use crate::*;

/// User leave message
///
/// The happens when a user (yourself included) leaves a channel
#[derive(Clone, PartialEq)]
pub struct Part<'a> {
    raw: Str<'a>,
    name: StrIndex,
    channel: StrIndex,
}

impl<'a> Part<'a> {
    raw!();
    str_field!(
        /// Name of the user that left the channel
        name
    );
    str_field!(
        /// Channel which they left
        channel
    );
}

impl<'a> FromIrcMessage<'a> for Part<'a> {
    type Error = InvalidMessage;

    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::PART)?;

        let this = Self {
            channel: msg.expect_arg_index(0)?,
            name: msg.expect_nick()?,
            raw: msg.raw,
        };

        Ok(this)
    }

    into_inner_raw!();
}

into_owned!(Part { raw, name, channel });
impl_custom_debug!(Part { raw, name, channel });
serde_struct!(Part { raw, name, channel });

#[cfg(test)]
mod tests {
    use super::*;
    use crate::irc;

    #[test]
    #[cfg(feature = "serde")]
    fn part_serde() {
        let input = ":test!test@test PART #museun\r\n";
        crate::serde::round_trip_json::<Part>(input);
        crate::serde::round_trip_rmp::<Part>(input);
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
