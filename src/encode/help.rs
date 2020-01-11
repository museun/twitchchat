use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Help {}

impl Encodable for Help {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        command("/help").encode(writer)
    }
}

pub fn help() -> Help {
    Help {}
}
