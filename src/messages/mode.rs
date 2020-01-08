use super::*;

/// Status of gaining or losing moderator (operator) status
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ModeStatus {
    /// Moderator status gained
    Gained,
    /// Moderator status lost
    Lost,
}

as_owned!(for ModeStatus);

#[derive(Debug, Clone, PartialEq)]
pub struct Mode<T = String>
where
    T: StringMarker,
{
    pub channel: T,
    pub status: ModeStatus,
    pub user: T,
}

impl<'a> TryFrom<&'a Message<&'a str>> for Mode<&'a str> {
    type Error = InvalidMessage;

    fn try_from(msg: &'a Message<&'a str>) -> Result<Self, Self::Error> {
        msg.expect_command("MODE").and_then(|_| {
            let channel = msg.expect_arg(0)?;
            let status = match msg.expect_arg(1)?.chars().nth(0).unwrap() {
                '+' => ModeStatus::Gained,
                '-' => ModeStatus::Lost,
                _ => unreachable!(),
            };
            let user = msg.expect_arg(2)?;
            Ok(Self {
                channel,
                status,
                user,
            })
        })
    }
}

as_owned!(for Mode{
    channel,
    status,
    user,
});

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse() {
        let input = ":jtv MODE #museun +o shaken_bot\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                Mode::<String>::try_from(&msg).unwrap(),
                Mode {
                    channel: "#museun".into(),
                    status: ModeStatus::Gained,
                    user: "shaken_bot".into()
                }
            );
            assert_eq!(
                Mode::<&str>::try_from(&msg).unwrap(),
                Mode {
                    channel: "#museun",
                    status: ModeStatus::Gained,
                    user: "shaken_bot"
                }
            )
        }

        let input = ":jtv MODE #museun -o shaken_bot\r\n";
        for msg in crate::decode_many(input).map(|s| s.unwrap()) {
            assert_eq!(
                Mode::<String>::try_from(&msg).unwrap(),
                Mode {
                    channel: "#museun".into(),
                    status: ModeStatus::Lost,
                    user: "shaken_bot".into()
                }
            );
            assert_eq!(
                Mode::<&str>::try_from(&msg).unwrap(),
                Mode {
                    channel: "#museun",
                    status: ModeStatus::Lost,
                    user: "shaken_bot"
                }
            )
        }
    }
}
