use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Pong<T = String>
where
    T: StringMarker,
{
    /// Token associated with the PONG event
    pub token: T,
}

impl<'a> TryFrom<&'a Message<&'a str>> for Pong<&'a str> {
    type Error = InvalidMessage;

    fn try_from(msg: &'a Message<&'a str>) -> Result<Self, Self::Error> {
        msg.expect_command("PONG")
            .and_then(|_| msg.expect_data().map(|token| Self { token }))
    }
}

as_owned!(for Pong {
   token    
});
