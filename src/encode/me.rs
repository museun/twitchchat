use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Me<'a> {
    pub(crate) channel: &'a str,
    pub(crate) message: &'a str,
}

impl<'a> Encodable for Me<'a> {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        privmsg(self.channel, &format!("/me {}", self.message)).encode(writer)
    }
}

pub fn me<'a>(channel: &'a str, message: &'a str) -> Me<'a> {
    Me { channel, message }
}
