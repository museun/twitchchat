use super::*;

#[derive(Copy, Clone, Debug)]
pub struct SlowOff {}

impl Encodable for SlowOff {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        command("/slowoff").encode(writer)
    }
}

pub fn slow_off() -> SlowOff {
    SlowOff {}
}
