use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Unhost {}

impl Encodable for Unhost {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        command("/unhost").encode(writer)
    }
}

pub fn unhost() -> Unhost {
    Unhost {}
}
