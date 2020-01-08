use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Slow {
    pub(crate) duration: Option<usize>,
}

impl Encodable for Slow {
    fn encode<W: ?Sized + Write>(&self, writer: &mut W) -> std::io::Result<()> {
        command(&format!("/slow {}", self.duration.unwrap_or_else(|| 120))).encode(writer)
    }
}

// TODO use `time` here
pub fn slow(duration: impl Into<Option<usize>>) -> Slow {
    Slow {
        duration: duration.into(),
    }
}
