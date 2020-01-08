use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Commercial {
    pub(crate) length: Option<usize>,
}

impl Encodable for Commercial {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match self.length {
            Some(length) => command(&format!("/commercial {}", length)).encode(writer),
            None => command("/commercial").encode(writer),
        }
    }
}

pub fn commercial(length: impl Into<Option<usize>>) -> Commercial {
    Commercial {
        length: length.into(),
    }
}
