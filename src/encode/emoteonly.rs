use super::*;

#[derive(Copy, Clone, Debug)]
pub struct EmoteOnly {}

impl Encodable for EmoteOnly {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        command("/emoteonly").encode(writer)
    }
}

pub fn emote_only() -> EmoteOnly {
    EmoteOnly {}
}
