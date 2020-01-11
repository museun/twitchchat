use super::*;

#[derive(Copy, Clone, Debug)]
pub struct EmoteOnlyOff {}

impl Encodable for EmoteOnlyOff {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        command("/emoteonlyoff").encode(writer)
    }
}

pub fn emote_only_off() -> EmoteOnlyOff {
    EmoteOnlyOff {}
}
