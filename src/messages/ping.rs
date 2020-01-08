use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Ping<T = String>
where
    T: StringMarker,
{
    /// Token associated with the PING event
    pub token: T,
}

impl<'a> TryFrom<&'a Message<&'a str>> for Ping<&'a str> {
    type Error = InvalidMessage;

    fn try_from(msg: &'a Message<&'a str>) -> Result<Self, Self::Error> {
        msg.expect_command("PING")
            .and_then(|_| msg.expect_data().map(|token| Self { token }))
    }
}

as_owned!(for Ping {
    token
});
