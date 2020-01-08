use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Host<'a> {
    pub(crate) channel: &'a str,
}

impl<'a> Encodable for Host<'a> {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        command(&format!("/host {}", self.channel)).encode(writer)
    }
}

pub fn host(channel: &str) -> Host<'_> {
    Host { channel }
}
