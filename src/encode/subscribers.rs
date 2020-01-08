use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Subscribers {}

impl Encodable for Subscribers {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        command("/subscribers").encode(writer)
    }
}

pub fn subscribers() -> Subscribers {
    Subscribers {}
}
