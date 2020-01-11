use super::*;

#[derive(Copy, Clone, Debug)]
pub struct R9kBetaOff {}

impl Encodable for R9kBetaOff {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        command("/r9kbetaoff").encode(writer)
    }
}

pub fn r9k_beta_off() -> R9kBetaOff {
    R9kBetaOff {}
}
