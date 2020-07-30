use super::{FromIrcMessage, InvalidMessage, IrcMessage, Str, StrIndex, Validator};

#[derive(Debug, Clone, PartialEq)]
pub struct IrcReady<'t> {
    raw: Str<'t>,
    nickname: StrIndex,
}

impl<'t> IrcReady<'t> {
    raw!();
    str_field!(nickname);
}

impl<'t> FromIrcMessage<'t> for IrcReady<'t> {
    type Error = InvalidMessage;

    fn from_irc(msg: IrcMessage<'t>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::IRCREADY)?;

        let this = Self {
            nickname: msg.expect_arg_index(0)?,
            raw: msg.raw,
        };

        Ok(this)
    }
}

impl<'t> serde::Serialize for IrcReady<'t> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct as _;
        let mut s = serializer.serialize_struct("Cap", 3)?;
        s.serialize_field("raw", &self.raw)?;
        s.serialize_field("nickname", &self.raw[self.nickname])?;
        s.end()
    }
}

impl<'t, 'de: 't> serde::Deserialize<'de> for IrcReady<'t> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(crate::ng::RawVisitor::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ng::irc;

    #[test]
    fn irc_ready() {
        let input = ":tmi.twitch.tv 001 shaken_bot :Welcome, GLHF!\r\n";
        for irc in irc::parse(input).map(|s| s.unwrap()) {
            let msg = IrcReady::from_irc(irc).unwrap();
            assert_eq!(msg.nickname(), "shaken_bot")
        }
    }
}
