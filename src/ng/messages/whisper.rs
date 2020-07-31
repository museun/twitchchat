use crate::ng::{
    FromIrcMessage, InvalidMessage, IrcMessage, Str, StrIndex, TagIndices, Tags, Validator,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Whisper<'t> {
    raw: Str<'t>,
    tags: TagIndices,
    name: StrIndex,
    data: StrIndex,
}

impl<'t> Whisper<'t> {
    raw!();
    tags!();
    str_field!(name);
    str_field!(data);
}

impl<'t> FromIrcMessage<'t> for Whisper<'t> {
    type Error = InvalidMessage;

    fn from_irc(msg: IrcMessage<'t>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::WHISPER)?;

        // :sender WHISPER target :data
        // we're the target, so ignore it

        let this = Self {
            name: msg.expect_nick()?,
            data: msg.expect_data_index()?,
            tags: msg.parse_tags(),
            raw: msg.raw,
        };

        Ok(this)
    }
}

serde_struct!(Whisper {
    raw,
    tags,
    name,
    data,
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ng::irc;

    #[test]
    #[cfg(feature = "serde")]
    fn whisper_serde() {
        let input = ":test!user@host WHISPER museun :this is a test\r\n";
        crate::ng::round_trip_json::<Whisper>(input)
    }

    #[test]
    fn whisper() {
        let input = ":test!user@host WHISPER museun :this is a test\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let msg = Whisper::from_irc(msg).unwrap();

            assert_eq!(msg.name(), "test");
            assert_eq!(msg.data(), "this is a test");
        }
    }
}
