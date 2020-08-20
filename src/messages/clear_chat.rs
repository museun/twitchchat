use crate::*;

/// When a user's message(s) have been purged.
///
/// Typically after a user is banned from chat or timed out
#[derive(Clone, PartialEq)]
pub struct ClearChat<'a> {
    raw: Str<'a>,
    tags: TagIndices,
    channel: StrIndex,
    name: Option<StrIndex>,
}

impl<'a> ClearChat<'a> {
    raw!();
    tags!();

    str_field!(
        /// The channel this event happened on
        channel
    );
    opt_str_field!(
        /// The user, if any, that was being purged
        name
    );

    /// (Optional) Duration of the timeout, in seconds. If omitted, the ban is permanent.
    pub fn ban_duration(&self) -> Option<u64> {
        self.tags().get_parsed("ban-duration")
    }

    /// The room id this event happened on
    pub fn room_id(&self) -> Option<&str> {
        self.tags().get("room-id")
    }
}

impl<'a> FromIrcMessage<'a> for ClearChat<'a> {
    type Error = InvalidMessage;

    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::CLEAR_CHAT)?;

        let this = Self {
            tags: msg.parse_tags(),
            channel: msg.expect_arg_index(0)?,
            name: msg.data,
            raw: msg.raw,
        };

        Ok(this)
    }

    into_inner_raw!();
}

into_owned!(ClearChat {
    raw,
    tags,
    channel,
    name
});

impl_custom_debug!(ClearChat {
    raw,
    tags,
    channel,
    name,
    ban_duration,
    room_id,
});

serde_struct!(ClearChat {
    raw,
    tags,
    channel,
    name
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::irc;

    #[test]
    #[cfg(feature = "serde")]
    fn clear_chat_serde() {
        let input = ":tmi.twitch.tv CLEARCHAT #museun :shaken_bot\r\n";
        crate::serde::round_trip_json::<ClearChat>(input);
        crate::serde::round_trip_rmp::<ClearChat>(input);
    }

    #[test]
    fn clear_chat() {
        let input = ":tmi.twitch.tv CLEARCHAT #museun :shaken_bot\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let cc = ClearChat::from_irc(msg).unwrap();
            assert_eq!(cc.channel(), "#museun");
            assert_eq!(cc.name().unwrap(), "shaken_bot");
        }
    }

    #[test]
    fn clear_chat_empty() {
        let input = ":tmi.twitch.tv CLEARCHAT #museun\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let cc = ClearChat::from_irc(msg).unwrap();
            assert_eq!(cc.channel(), "#museun");
            assert!(cc.name().is_none());
        }
    }
}
