use crate::*;

/// Happens when the IRC connection has been succesfully established
#[derive(Debug, Clone, PartialEq)]
pub struct IrcReady<'a> {
    raw: Str<'a>,
    nickname: StrIndex,
}

impl<'a> IrcReady<'a> {
    raw!();
    str_field!(
        /// The name the server will refer to you as
        nickname
    );
}

impl<'a> FromIrcMessage<'a> for IrcReady<'a> {
    type Error = InvalidMessage;

    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::IRC_READY)?;

        let this = Self {
            nickname: msg.expect_arg_index(0)?,
            raw: msg.raw,
        };

        Ok(this)
    }
}

into_owned!(IrcReady { raw, nickname });

serde_struct!(IrcReady { raw, nickname });

#[cfg(test)]
mod tests {
    use super::*;
    use crate::irc;

    #[test]
    #[cfg(feature = "serde")]
    fn irc_ready_serde() {
        let input = ":tmi.twitch.tv 001 shaken_bot :Welcome, GLHF!\r\n";
        crate::serde::round_trip_json::<IrcReady>(input);
    }

    #[test]
    fn irc_ready() {
        let input = ":tmi.twitch.tv 001 shaken_bot :Welcome, GLHF!\r\n";
        for irc in irc::parse(input).map(|s| s.unwrap()) {
            let msg = IrcReady::from_irc(irc).unwrap();
            assert_eq!(msg.nickname(), "shaken_bot")
        }
    }
}
