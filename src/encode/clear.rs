use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Clear {}

impl Encodable for Clear {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        command("/clear").encode(writer)
    }
}

pub fn clear() -> Clear {
    Clear {}
}
