use super::*;

/// Signals that you should reconnect and rejoin channels after a restart.
///
/// Twitch IRC processes occasionally need to be restarted. When this happens,
/// clients that have requested the IRC v3 twitch.tv/commands capability are
/// issued a RECONNECT. After a short time, the connection is closed. In this
/// case, reconnect and rejoin channels that were on the connection, as you
/// would normally.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Reconnect {}

impl<'a: 't, 't> Parse<&'a Message<'t>> for Reconnect {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("RECONNECT").map(|_| Self {})
    }
}

impl AsOwned for Reconnect {
    type Owned = Self;
    fn as_owned(&self) -> Self::Owned {
        Self {}
    }
}
