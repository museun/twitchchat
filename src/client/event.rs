use crate::messages::InvalidMessage;
use std::convert::TryFrom;

pub trait Event<'a>: private::Sealed
where
    Self::Mapped: Clone + std::fmt::Debug,
    Self::Mapped: Send + Sync + 'static,
    Self::Mapped: TryFrom<&'a crate::decode::Message<&'a str>, Error = InvalidMessage>,
{
    type Mapped;
}

mod private {
    pub trait Sealed {}
    impl<'a, T> Sealed for T where T: super::Event<'a> {}
}
