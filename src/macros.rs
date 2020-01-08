macro_rules! as_owned {
    (for $ty:tt {
        $($field:ident),* $(,)?
    }) => {
        impl<'a> TryFrom<&'a Message<&'a str>> for $ty<String> {
            type Error = InvalidMessage;

            fn try_from(msg: &'a Message<&'a str>) -> Result<Self, Self::Error> {
                $ty::<&'a str>::try_from(msg).map(|ok| ok.into_owned())
            }
        }

        impl IntoOwned for $ty<&str> {
            type Target = $ty<String>;

            fn into_owned(&self) -> Self::Target {
                Self::Target {
                    $( $field: self.$field.into_owned(), )*
                }
            }
        }
    };
}

macro_rules! make_event {
    (@DOC $($doc:expr)* => $item:tt) => {
        $(#[doc = $doc])*
        #[non_exhaustive]
        pub struct $item;
    };

    ($event:ident => $message:path) => {
        make_event!(@DOC
            concat!("Used to get a [", stringify!($message), "](../messages/struct.", stringify!($event), ".html)")
            =>
            $event
        );

        impl<'a> crate::client::Event<'a> for $event {
            type Mapped = $message;
        }
    };
}

macro_rules! cfg_async {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "async")]
            #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
            $item
        )*
    }
}
