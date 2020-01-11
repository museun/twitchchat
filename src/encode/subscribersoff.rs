use super::*;

#[derive(Copy, Clone, Debug)]
pub struct SubscribersOff {}

impl Encodable for SubscribersOff {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        command("/subscribersoff").encode(writer)
    }
}

pub fn subscribers_off() -> SubscribersOff {
    SubscribersOff {}
}
