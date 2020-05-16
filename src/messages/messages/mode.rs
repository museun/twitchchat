#![allow(deprecated)]
use super::*;

/// When a user gains or loses moderator (operator) status in a channel.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[deprecated(
    since = "0.10.2",
    note = "Twitch has deprecated this event. see https://discuss.dev.twitch.tv/t/irc-update-removing-mode-and-names-capabilities/25568"
)]
pub struct Mode<'t> {
    /// The channel this event happened on
    pub channel: Cow<'t, str>,
    /// The status. gained, or lost
    pub status: ModeStatus,
    /// The user this applies too
    pub name: Cow<'t, str>,
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for Mode<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("MODE")?;
        let channel = msg.expect_arg(0)?;
        let status = match msg.expect_arg(1)?.chars().next().unwrap() {
            '+' => ModeStatus::Gained,
            '-' => ModeStatus::Lost,
            _ => unreachable!(),
        };
        let name = msg.expect_arg(2)?;
        Ok(Self {
            channel,
            status,
            name,
        })
    }
}

impl<'t> AsOwned for Mode<'t> {
    type Owned = Mode<'static>;
    fn as_owned(&self) -> Self::Owned {
        Mode {
            channel: self.channel.as_owned(),
            status: self.status.as_owned(),
            name: self.name.as_owned(),
        }
    }
}

/// Status of gaining or losing moderator (operator) status
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[deprecated(
    since = "0.10.2",
    note = "Twitch has deprecated this event. see https://discuss.dev.twitch.tv/t/irc-update-removing-mode-and-names-capabilities/25568"
)]
pub enum ModeStatus {
    /// Moderator status gained
    Gained,
    /// Moderator status lost
    Lost,
}

impl<'t> AsOwned for ModeStatus {
    type Owned = Self;
    fn as_owned(&self) -> Self::Owned {
        *self
    }
}
