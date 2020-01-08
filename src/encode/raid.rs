use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Raid<'a> {
    pub(crate) channel: &'a str,
}

impl<'a> Encodable for Raid<'a> {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        command(&format!("/raid {}", self.channel)).encode(writer)
    }
}

pub fn raid<'a>(channel: &'a str) -> Raid<'a> {
    Raid { channel }
}
