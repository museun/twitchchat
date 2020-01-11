use super::*;

#[derive(Copy, Clone, Debug)]
pub struct R9kBeta {}

impl Encodable for R9kBeta {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        command("/r9kbeta").encode(writer)
    }
}

pub fn r9k_beta() -> R9kBeta {
    R9kBeta {}
}
