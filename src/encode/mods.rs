use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Mods {}

impl Encodable for Mods {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        command("/mods").encode(writer)
    }
}

pub fn mods() -> Mods {
    Mods {}
}
