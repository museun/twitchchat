use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Vip<'a> {
    pub(crate) username: &'a str,
}

impl<'a> Encodable for Vip<'a> {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        command(&format!("/vip {}", self.username)).encode(writer)
    }
}

pub fn vip<'a>(username: &'a str) -> Vip<'a> {
    Vip { username }
}
