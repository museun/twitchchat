use super::*;

/// Message sent by a user
#[derive(Debug, Clone, PartialEq)]
pub struct Privmsg<T = String>
where
    T: StringMarker,
{
    /// User who sent this messages
    pub user: T,
    /// Channel this message was sent on
    pub channel: T,
    /// Data that the user provided
    pub data: T,
    /// Tags attached to the message
    pub tags: Tags<T>,
}

impl<'a> TryFrom<&'a Message<&'a str>> for Privmsg<&'a str> {
    type Error = InvalidMessage;

    fn try_from(msg: &'a Message<&'a str>) -> Result<Self, Self::Error> {
        msg.expect_command("PRIVMSG").and_then(|_| {
            Ok(Self {
                user: msg.expect_nick()?,
                channel: msg.expect_arg(0)?,
                data: msg.expect_data()?,
                tags: msg.tags.clone(),
            })
        })
    }
}

as_owned!(for Privmsg {
    user,
    channel,
    data,
    tags,
});
