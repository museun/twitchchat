use crate::{irc::*, MaybeOwned, MaybeOwnedIndex, Validator};

/// An event that is produced when the Twitch connection has been succesfully
/// established
#[derive(Clone, PartialEq)]
pub struct Ready<'a> {
    raw: MaybeOwned<'a>,
    username: MaybeOwnedIndex,
}

impl<'a> Ready<'a> {
    raw!();
    str_field!(
        /// The name Twitch will refer to you as
        username
    );
}

impl<'a> FromIrcMessage<'a> for Ready<'a> {
    type Error = MessageError;

    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::READY)?;

        let this = Self {
            username: msg.expect_arg_index(0)?,
            raw: msg.raw,
        };

        Ok(this)
    }

    into_inner_raw!();
}

into_owned!(Ready { raw, username });
impl_custom_debug!(Ready { raw, username });
serde_struct!(Ready { raw, username });

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "serde")]
    fn ready_serde() {
        let input = ":tmi.twitch.tv 376 shaken_bot :>\r\n";
        crate::serde::round_trip_json::<Ready>(input);
        crate::serde::round_trip_rmp::<Ready>(input);
    }

    #[test]
    fn ready() {
        let input = ":tmi.twitch.tv 376 shaken_bot :>\r\n";
        for irc in parse(input).map(|s| s.unwrap()) {
            let msg = Ready::from_irc(irc).unwrap();
            assert_eq!(msg.username(), "shaken_bot")
        }
    }
}
