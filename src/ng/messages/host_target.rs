use crate::ng::{FromIrcMessage, InvalidMessage, Validator};
use crate::ng::{IrcMessage, Str, StrIndex};

#[derive(Debug, Clone, PartialEq)]
//
#[derive(::serde::Serialize, ::serde::Deserialize)]
pub enum HostTargetKind<'a> {
    Start { target: &'a str },
    End,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HostTarget<'t> {
    raw: Str<'t>,
    source: StrIndex,
    viewers: Option<usize>,
    target: Option<StrIndex>,
}

impl<'t> HostTarget<'t> {
    raw!();
    str_field!(source);

    pub fn viewers(&self) -> Option<usize> {
        self.viewers
    }

    pub fn host_target_kind(&self) -> HostTargetKind<'_> {
        match self.target {
            Some(index) => HostTargetKind::Start {
                target: &self.raw[index],
            },
            None => HostTargetKind::End,
        }
    }
}

impl<'t> FromIrcMessage<'t> for HostTarget<'t> {
    type Error = InvalidMessage;

    fn from_irc(msg: IrcMessage<'t>) -> Result<Self, Self::Error> {
        msg.expect_command(IrcMessage::HOSTTARGET)?;

        // [- | <target>] number?

        let mut data = msg.expect_data()?.splitn(2, char::is_whitespace);
        let target = match data.next() {
            Some("-") => None,
            Some(t) => {
                let kind = msg.expect_data_index()?.resize(t.len());
                Some(kind)
            }
            None => return Err(InvalidMessage::ExpectedData),
        };

        let viewers = data.next().and_then(|s| s.parse().ok());

        // TODO assert iterator is empty?

        let this = Self {
            source: msg.expect_arg_index(0)?,
            viewers,
            target,
            raw: msg.raw,
        };

        Ok(this)
    }
}

serde_struct!(HostTarget {
    raw,
    source,
    viewers,
    host_target_kind
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ng::irc;

    #[test]
    fn host_target_serde() {
        let input = ":tmi.twitch.tv HOSTTARGET #shaken_bot :museun 1024\r\n";
        crate::ng::serde::round_trip_json::<HostTarget>(input);
    }

    #[test]
    fn host_target_start() {
        let input = ":tmi.twitch.tv HOSTTARGET #shaken_bot :museun 1024\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let ht = HostTarget::from_irc(msg).unwrap();
            assert_eq!(ht.source(), "#shaken_bot");
            assert_eq!(ht.viewers().unwrap(), 1024);
            assert_eq!(
                ht.host_target_kind(),
                HostTargetKind::Start { target: "museun" }
            );
        }
    }

    #[test]
    fn host_target_start_none() {
        let input = ":tmi.twitch.tv HOSTTARGET #shaken_bot :museun -\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let ht = HostTarget::from_irc(msg).unwrap();
            assert_eq!(ht.source(), "#shaken_bot");
            assert!(ht.viewers().is_none());
            assert_eq!(
                ht.host_target_kind(),
                HostTargetKind::Start { target: "museun" }
            );
        }
    }

    #[test]
    fn host_target_end() {
        let input = ":tmi.twitch.tv HOSTTARGET #shaken_bot :- 1024\r\n";
        for msg in irc::parse(input).map(|s| s.unwrap()) {
            let ht = HostTarget::from_irc(msg).unwrap();
            assert_eq!(ht.source(), "#shaken_bot");
            assert_eq!(ht.viewers().unwrap(), 1024);
            assert_eq!(ht.host_target_kind(), HostTargetKind::End);
        }
    }
}