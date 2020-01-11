use super::*;

#[derive(Copy, Clone, Debug)]
pub struct GiveMod<'a> {
    pub(crate) username: &'a str,
}

impl<'a> Encodable for GiveMod<'a> {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        command(&format!("/mod {}", self.username,)).encode(writer)
    }
}

pub fn give_mod<'a>(username: &'a str) -> GiveMod<'a> {
    GiveMod { username }
}
