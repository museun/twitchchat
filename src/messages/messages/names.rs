#![allow(deprecated)]
use super::*;

/// The names event
///
/// This'll will list people on a channel for your user
///
/// The `kind` field lets you determine if its still 'happening'
///
/// Your should keep a list of the names from the `Start` variant
///
/// And once you receive an End you'll have the complete lost
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[deprecated(
    since = "0.10.2",
    note = "Twitch has deprecated this event. see https://discuss.dev.twitch.tv/t/irc-update-removing-mode-and-names-capabilities/25568"
)]
pub struct Names<'t> {
    /// Your username
    pub name: Cow<'t, str>,
    /// The channel this event is happening for
    pub channel: Cow<'t, str>,
    /// The state of the event
    pub kind: NamesKind<'t>,
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for Names<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        let kind = match &*msg.command {
            "353" => {
                let users = msg.expect_data()?.split_whitespace();
                let users = users.map(Cow::Borrowed).collect();
                NamesKind::Start { users }
            }
            "366" => NamesKind::End,
            unknown => {
                return Err(InvalidMessage::InvalidCommand {
                    expected: "353 or 366".to_string(),
                    got: unknown.to_string(),
                })
            }
        };
        let name = msg.expect_arg(0)?;
        let channel = match msg.expect_arg(1)? {
            d if d == "=" => msg.expect_arg(2)?,
            channel => channel,
        };
        Ok(Self {
            name,
            channel,
            kind,
        })
    }
}

impl<'t> AsOwned for Names<'t> {
    type Owned = Names<'static>;
    fn as_owned(&self) -> Self::Owned {
        Names {
            name: self.name.as_owned(),
            channel: self.channel.as_owned(),
            kind: self.kind.as_owned(),
        }
    }
}

/// The kind of the Names event
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[deprecated(
    since = "0.10.2",
    note = "Twitch has deprecated this event. see https://discuss.dev.twitch.tv/t/irc-update-removing-mode-and-names-capabilities/25568"
)]
pub enum NamesKind<'t> {
    /// Names begins, this'll continue until `End` is recieved
    Start {
        /// A list of user names
        users: Vec<Cow<'t, str>>,
    },
    /// Names end, this'll mark the end of the event
    End,
}

impl<'t> AsOwned for NamesKind<'t> {
    type Owned = NamesKind<'static>;
    fn as_owned(&self) -> Self::Owned {
        match self {
            NamesKind::Start { users } => NamesKind::Start {
                users: users.as_owned(),
            },
            NamesKind::End => NamesKind::End,
        }
    }
}
