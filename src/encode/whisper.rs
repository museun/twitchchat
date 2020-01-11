use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Whisper<'a> {
    pub(crate) username: &'a str,
    pub(crate) message: &'a str,
}

impl<'a> Encodable for Whisper<'a> {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        command(&format!("/w {} {}", self.username, self.message)).encode(writer)
    }
}

pub fn whisper<'a>(username: &'a str, message: &'a str) -> Whisper<'a> {
    Whisper { username, message }
}
