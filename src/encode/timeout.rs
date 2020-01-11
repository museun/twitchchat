use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Timeout<'a> {
    pub(crate) username: &'a str,
    pub(crate) duration: Option<&'a str>,
    pub(crate) message: Option<&'a str>,
}

impl<'a> Encodable for Timeout<'a> {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let data = match (self.duration, self.message) {
            (Some(dur), Some(reason)) => format!("/timeout {} {} {}", self.username, dur, reason),
            (None, Some(reason)) => format!("/timeout {} {}", self.username, reason),
            (Some(dur), None) => format!("/timeout {} {}", self.username, dur),
            (None, None) => format!("/timeout {}", self.username),
        };
        command(&data).encode(writer)
    }
}

// TODO use `time` here
pub fn timeout<'a>(
    username: &'a str,
    duration: impl Into<Option<&'a str>>,
    message: impl Into<Option<&'a str>>,
) -> Timeout<'a> {
    Timeout {
        username,
        duration: duration.into(),
        message: message.into(),
    }
}
