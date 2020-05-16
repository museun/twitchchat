use super::*;

/// When a channel starts to host another channel
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HostTarget<'t> {
    /// Source channel (the one doing the hosting).
    pub source: Cow<'t, str>,
    /// How many viewers are going along
    pub viewers: Option<usize>,
    /// What kind of event this was. e.g. `Start` or `End`
    pub kind: HostTargetKind<'t>,
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for HostTarget<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        msg.expect_command("HOSTTARGET")?;
        let source = msg.expect_arg(0)?;
        let (kind, viewers) = if let Ok(target) = msg.expect_arg(1) {
            let viewers = msg.expect_arg(2).ok().and_then(|data| data.parse().ok());
            (HostTargetKind::Start { target }, viewers)
        } else {
            let data = msg.expect_data()?;
            if !data.starts_with('-') {
                return Err(InvalidMessage::ExpectedData);
            }
            let viewers = data.get(2..).and_then(|s| s.parse().ok());
            (HostTargetKind::End, viewers)
        };
        Ok(Self {
            source,
            kind,
            viewers,
        })
    }
}

impl<'t> AsOwned for HostTarget<'t> {
    type Owned = HostTarget<'static>;
    fn as_owned(&self) -> Self::Owned {
        HostTarget {
            source: self.source.as_owned(),
            viewers: self.viewers.as_owned(),
            kind: self.kind.as_owned(),
        }
    }
}

/// Event kind for determine when a Host event beings or end
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HostTargetKind<'t> {
    /// The host event started
    Start {
        /// Target channel that is being hosted
        target: Cow<'t, str>,
    },
    /// The host event ended
    End,
}

impl<'t> AsOwned for HostTargetKind<'t> {
    type Owned = HostTargetKind<'static>;
    fn as_owned(&self) -> Self::Owned {
        match self {
            HostTargetKind::Start { target } => HostTargetKind::Start {
                target: target.as_owned(),
            },
            HostTargetKind::End => HostTargetKind::End,
        }
    }
}
