use super::{FromIrcMessage, InvalidMessage, IrcMessage, Str, StrIndex, Validator};

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

impl<'t> serde::Serialize for Ready<'t> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct as _;
        let mut s = serializer.serialize_struct("Ready", 3)?;
        s.serialize_field("raw", &self.raw)?;
        s.serialize_field("username", &self.raw[self.username])?;
        s.end()
    }
}

impl<'t, 'de: 't> serde::Deserialize<'de> for Ready<'t> {
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
    fn ready() {
        let input = ":tmi.twitch.tv 376 shaken_bot :>\r\n";
        for irc in irc::parse(input).map(|s| s.unwrap()) {
            let msg = Ready::from_irc(irc).unwrap();
            assert_eq!(msg.username(), "shaken_bot")
        }
    }
}
