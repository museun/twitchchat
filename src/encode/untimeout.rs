use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Untimeout<'a> {
    pub(crate) username: &'a str,
}

impl<'a> Encodable for Untimeout<'a> {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        command(&format!("/untimeout {}", self.username)).encode(writer)
    }
}

pub fn untimeout<'a>(username: &'a str) -> Untimeout<'a> {
    Untimeout { username }
}
