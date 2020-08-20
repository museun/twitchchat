use crate::*;

/// When a single message has been removed from a channel.
///
/// This is triggered via `/delete` on IRC.
#[derive(Clone, PartialEq)]
pub struct ClearMsg<'a> {
    raw: Str<'a>,
    tags: TagIndices,
    channel: StrIndex,
    message: Option<StrIndex>,
}

impl<'a> ClearMsg<'a> {
    raw!();
    tags!();
    str_field!(
        /// The channel this event happened on
        channel
    );
    opt_str_field!(
        /// The message that was deleted
        message
    );

    /// Name of the user who sent the message
    pub fn login(&self) -> Option<&str> {
        self.tags().get("login")
    }

    /// UUID of the message
    pub fn target_msg_id(&self) -> Option<&str> {
        self.tags().get("target-msg-id")
    }
}

impl<'a> FromIrcMessage<'a> for ClearMsg<'a> {
    type Error = InvalidMessage;

    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::CLEAR_MSG)?;

        let this = Self {
            tags: msg.parse_tags(),
            channel: msg.expect_arg_index(0)?,
            message: msg.data,
            raw: msg.raw,
        };

        Ok(this)
    }

    into_inner_raw!();
}

into_owned!(ClearMsg {
    raw,
    tags,
    channel,
    message,
});

impl_custom_debug!(ClearMsg {
    raw,
    tags,
    channel,
    message,
    login,
    target_msg_id,
});

serde_struct!(ClearMsg {
    raw,
    tags,
    channel,
    message,
    login,
    target_msg_id
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::irc;

    #[test]
    #[cfg(feature = "serde")]
    fn clear_msg_serde() {
        let input = ":tmi.twitch.tv CLEARMSG #museun :HeyGuys\r\n";
        crate::serde::round_trip_json::<ClearMsg>(input);
        crate::serde::round_trip_rmp::<ClearMsg>(input);
    }

    #[test]
    fn clear_msg() {
        let input = ":tmi.twitch.tv CLEARMSG #museun :HeyGuys\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let cm = ClearMsg::from_irc(msg).unwrap();
            assert_eq!(cm.channel(), "#museun");
            assert_eq!(cm.message().unwrap(), "HeyGuys");
        }
    }

    #[test]
    fn clear_msg_empty() {
        let input = ":tmi.twitch.tv CLEARMSG #museun\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let cm = ClearMsg::from_irc(msg).unwrap();
            assert_eq!(cm.channel(), "#museun");
            assert!(cm.message().is_none());
        }
    }

    #[test]
    fn clear_msg_uuid() {
        let input =
            "@login=ronni;target-msg-id=abc-123-def :tmi.twitch.tv CLEARMSG #dallas :HeyGuys\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let cm = ClearMsg::from_irc(msg).unwrap();
            assert_eq!(cm.channel(), "#dallas");
            assert_eq!(cm.message().unwrap(), "HeyGuys");
            assert_eq!(cm.target_msg_id().unwrap(), "abc-123-def");
        }
    }
}
