use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Disconnect {}

impl Encodable for Disconnect {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        command("/disconnect").encode(writer)
    }
}

pub fn disconnect() -> Disconnect {
    Disconnect {}
}
