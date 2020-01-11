/// A marker trait for Event subscription
pub trait Event<'a>: private::Sealed
where
    Self::Mapped: Clone + std::fmt::Debug,
    Self::Mapped: Send + Sync + 'static,
    Self::Mapped: crate::Parse<&'a crate::decode::Message<&'a str>>,
{
    /// Event message mapping
    type Mapped;
}

mod private {
    pub trait Sealed {}
    impl<'a, T> Sealed for T where T: super::Event<'a> {}
}
