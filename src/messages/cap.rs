use crate::{irc::*, MaybeOwned, MaybeOwnedIndex, Validator};

/// A parsed Capability
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub enum Capability<'a> {
    /// This Capability was Acknowledged
    Acknowledged(
        /// The name of the requested capability
        &'a str,
    ),
    /// This Capability was not acknowledged
    NotAcknowledged(
        /// The name of the requested capability
        &'a str,
    ),
}

/// Acknowledgement (or not) on a **CAPS** request
#[derive(Clone, PartialEq)]
pub struct Cap<'a> {
    raw: MaybeOwned<'a>,
    capability: MaybeOwnedIndex,
    acknowledged: bool,
}

impl<'a> Cap<'a> {
    raw!();

    /// The parsed capability
    pub fn capability(&self) -> Capability<'_> {
        let cap = &self.raw[self.capability];
        if self.acknowledged {
            Capability::Acknowledged(cap)
        } else {
            Capability::NotAcknowledged(cap)
        }
    }
}

impl<'a> FromIrcMessage<'a> for Cap<'a> {
    type Error = MessageError;

    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error> {
        const ACK: &str = "ACK";

        msg.expect_command(IrcMessage::CAP)?;

        let this = Self {
            capability: msg.expect_data_index()?,
            acknowledged: msg.expect_arg(1)? == ACK,
            raw: msg.raw,
        };

        Ok(this)
    }

    into_inner_raw!();
}

into_owned!(Cap {
    raw,
    capability,
    acknowledged
});

impl_custom_debug!(Cap { raw, capability });

serde_struct!(Cap { raw, capability });

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "serde")]
    fn cap_serde() {
        let input = ":tmi.twitch.tv CAP * ACK :twitch.tv/membership\r\n";
        crate::serde::round_trip_json::<Cap>(input);
        crate::serde::round_trip_rmp::<Cap>(input);
    }

    #[test]
    fn cap_acknowledged() {
        let input = ":tmi.twitch.tv CAP * ACK :twitch.tv/membership\r\n\
                     :tmi.twitch.tv CAP * ACK :twitch.tv/tags\r\n\
                     :tmi.twitch.tv CAP * ACK :twitch.tv/commands\r\n";
        let expected = &[
            "twitch.tv/membership",
            "twitch.tv/tags",
            "twitch.tv/commands",
        ];
        for (msg, expected) in parse(input).map(|s| s.unwrap()).zip(expected) {
            let msg = Cap::from_irc(msg).unwrap();
            assert_eq!(msg.capability(), Capability::Acknowledged(expected));
        }
    }

    #[test]
    fn cap_failed() {
        let input = ":tmi.twitch.tv CAP * NAK :foobar\r\n";
        for msg in parse(input).map(|s| s.unwrap()) {
            let cap = Cap::from_irc(msg).unwrap();
            assert_eq!(cap.capability(), Capability::NotAcknowledged("foobar"));
        }
    }
}
