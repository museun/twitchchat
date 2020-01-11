use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Ban<'a> {
    pub(crate) username: &'a str,
    pub(crate) reason: Option<&'a str>,
}

impl<'a> Encodable for Ban<'a> {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let data = match self.reason {
            Some(reason) => format!("/ban {} {}", self.username, &reason),
            None => format!("/ban {}", self.username),
        };
        command(&data).encode(writer)
    }
}

pub fn ban<'a>(username: &'a str, reason: impl Into<Option<&'a str>>) -> Ban<'a> {
    Ban {
        username,
        reason: reason.into(),
    }
}
