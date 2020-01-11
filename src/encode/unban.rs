use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Unban<'a> {
    pub(crate) username: &'a str,
}

impl<'a> Encodable for Unban<'a> {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        command(&format!("/unban {}", self.username)).encode(writer)
    }
}

pub fn unban<'a>(username: &'a str) -> Unban<'a> {
    Unban { username }
}
