use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Unraid {}

impl Encodable for Unraid {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        command("/unraid").encode(writer)
    }
}

pub fn unraid() -> Unraid {
    Unraid {}
}
