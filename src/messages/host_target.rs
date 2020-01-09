use super::*;

#[derive(Debug, Clone, PartialEq)]
pub enum HostTargetKind<T = String>
where
    T: StringMarker,
{
    Start { target: T },
    End,
}

impl<'a> IntoOwned for HostTargetKind<&'a str> {
    type Target = HostTargetKind<String>;

    fn into_owned(&self) -> Self::Target {
        match self {
            HostTargetKind::Start { target } => HostTargetKind::Start {
                target: target.to_string(),
            },
            HostTargetKind::End => HostTargetKind::End,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HostTarget<T = String>
where
    T: StringMarker,
{
    pub source: T,
    pub viewers: Option<usize>,
    pub kind: HostTargetKind<T>,
}

impl<'a> TryFrom<&'a Message<&'a str>> for HostTarget<&'a str> {
    type Error = InvalidMessage;

    fn try_from(msg: &'a Message<&'a str>) -> Result<Self, Self::Error> {
        msg.expect_command("HOSTTARGET").and_then(|_| {
            let source = msg.expect_arg(0)?;
            let (kind, viewers) = if let Ok(target) = msg.expect_arg(1) {
                let viewers = msg.expect_arg(2).ok().and_then(|data| data.parse().ok());
                (HostTargetKind::Start { target }, viewers)
            } else {
                let data = msg.expect_data()?;
                if !data.starts_with("-") {
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
        })
    }
}

as_owned!(for HostTarget {
    source,
    viewers,
    kind
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_start_viewers() {
        let input = ":tmi.twitch.tv HOSTTARGET #shaken_bot #museun 1024\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                HostTarget::<String>::try_from(&msg).unwrap(),
                HostTarget {
                    source: "#shaken_bot".into(),
                    viewers: Some(1024),
                    kind: HostTargetKind::Start {
                        target: "#museun".into()
                    },
                }
            );
            assert_eq!(
                HostTarget::<&str>::try_from(&msg).unwrap(),
                HostTarget {
                    source: "#shaken_bot",
                    viewers: Some(1024),
                    kind: HostTargetKind::Start { target: "#museun" },
                }
            )
        }
    }

    #[test]
    fn parse_start_no_viewers() {
        let input = ":tmi.twitch.tv HOSTTARGET #shaken_bot #museun\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                HostTarget::<String>::try_from(&msg).unwrap(),
                HostTarget {
                    source: "#shaken_bot".into(),
                    viewers: None,
                    kind: HostTargetKind::Start {
                        target: "#museun".into()
                    },
                }
            );
            assert_eq!(
                HostTarget::<&str>::try_from(&msg).unwrap(),
                HostTarget {
                    source: "#shaken_bot",
                    viewers: None,
                    kind: HostTargetKind::Start { target: "#museun" },
                }
            )
        }
    }

    #[test]
    fn parse_end_viewers() {
        let input = ":tmi.twitch.tv HOSTTARGET #shaken_bot :- 1024\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                HostTarget::<String>::try_from(&msg).unwrap(),
                HostTarget {
                    source: "#shaken_bot".into(),
                    viewers: Some(1024),
                    kind: HostTargetKind::End,
                }
            );
            assert_eq!(
                HostTarget::<&str>::try_from(&msg).unwrap(),
                HostTarget {
                    source: "#shaken_bot",
                    viewers: Some(1024),
                    kind: HostTargetKind::End,
                }
            )
        }
    }

    #[test]
    fn parse_end_no_viewers() {
        let input = ":tmi.twitch.tv HOSTTARGET #shaken_bot :-\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                HostTarget::<String>::try_from(&msg).unwrap(),
                HostTarget {
                    source: "#shaken_bot".into(),
                    viewers: None,
                    kind: HostTargetKind::End,
                }
            );
            assert_eq!(
                HostTarget::<&str>::try_from(&msg).unwrap(),
                HostTarget {
                    source: "#shaken_bot",
                    viewers: None,
                    kind: HostTargetKind::End,
                }
            )
        }
    }
}
