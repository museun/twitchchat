use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Unvip<'a> {
    pub(crate) username: &'a str,
}

impl<'a> Encodable for Unvip<'a> {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        command(&format!("/unvip {}", self.username)).encode(writer)
    }
}

pub fn unvip<'a>(username: &'a str) -> Unvip<'a> {
    Unvip { username }
}
