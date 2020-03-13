#![allow(unused_macros)]

macro_rules! cfg_async {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "async")]
            #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
            $item
        )*
    }
}

macro_rules! conversion {
    ($ty:tt { $($field:ident),* $(,)? }) => {
        impl<'t> AsOwned for $ty<'t> {
            type Owned = $ty<'static>;
            fn as_owned(&self) -> Self::Owned {
                $ty {
                    $( $field: self.$field.as_owned(), )*
                }
            }
        }
    };
    ($ty:tt) => {
        impl AsOwned for $ty {
            type Owned = $ty;
            fn as_owned(&self) -> Self::Owned {
                $ty { }
            }
        }
    };
}

macro_rules! parse {
    (bare $ty:tt { $($field:ident),* $(,)? } => $body:expr) => {
        impl<'a: 't, 't> Parse<&'a Message<'t>> for $ty<'t> {
            fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
                $body(msg)
            }
        }
    };

    ($ty:tt { $($field:ident),* $(,)? } => $body:expr) => {
        conversion!($ty { $($field,)* });
        parse!(bare $ty { $($field,)* } => $body);
    };

    ($ty:tt => $body:expr) => {
        conversion!($ty);
        impl<'a: 't, 't> Parse<&'a Message<'t>> for $ty {
            fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
                $body(msg)
            }
        }
    };
}
