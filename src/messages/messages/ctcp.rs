use super::*;

/// Some PRIVMSGs are considered 'CTCP' (client-to-client protocol)
///
/// This is a tag-type for determining what kind of CTCP it was
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Ctcp<'t> {
    /// An action CTCP, sent by the user when they do `/me` or `/action`
    Action,
    /// An unknown CTCP
    Unknown {
        /// The unknown CTCP command
        command: Cow<'t, str>,
    },
}

impl<'t> AsOwned for Ctcp<'t> {
    type Owned = Ctcp<'static>;
    fn as_owned(&self) -> Self::Owned {
        match self {
            Ctcp::Action => Ctcp::Action,
            Ctcp::Unknown { command } => Ctcp::Unknown {
                command: command.as_owned(),
            },
        }
    }
}
