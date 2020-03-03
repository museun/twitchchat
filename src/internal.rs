pub(crate) mod private {
    cfg_async! {
        pub(crate) mod event_marker {
            pub trait Sealed {}
            impl<'a, T> Sealed for T where T: crate::client::Event<'a> {}
        }

        pub(crate) mod mapped_marker {
            pub trait Sealed<E> {}
            impl<'a, T, E> Sealed<E> for T
            where
                T: crate::client::EventMapped<'a, E>,
                E: crate::client::Event<'a>,
            {
            }
        }
    }
}
