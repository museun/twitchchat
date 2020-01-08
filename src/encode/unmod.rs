use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Unmod<'a> {
    pub(crate) username: &'a str,
}

impl<'a> Encodable for Unmod<'a> {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        command(&format!("/unmod {}", self.username)).encode(writer)
    }
}

pub fn unmod<'a>(username: &'a str) -> Unmod<'a> {
    Unmod { username }
}
