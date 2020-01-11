use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Vips {}

impl Encodable for Vips {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        command("/vips").encode(writer)
    }
}

pub fn vips() -> Vips {
    Vips {}
}
