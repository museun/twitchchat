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
        impl<'a, T> Conversion<'a> for $ty<T>
        where
            T: StringMarker + Conversion<'a>,
            <T as Conversion<'a>>::Borrowed: StringMarker,
            <T as Conversion<'a>>::Owned: StringMarker,
        {
            type Owned = $ty<T::Owned>;
            type Borrowed = $ty<T::Borrowed>;

            fn as_borrowed(&'a self) -> Self::Borrowed {
                $ty {
                    $( $field: self.$field.as_borrowed(), )*
                }
            }

            fn as_owned(&self) -> Self::Owned {
                $ty {
                    $( $field: self.$field.as_owned(), )*
                }
            }
        }
    };
    ($ty:tt) => {
        impl<'a> Conversion<'a> for $ty
        {
            type Owned = $ty;
            type Borrowed = $ty;

            fn as_borrowed(&'a self) -> Self::Borrowed {
                $ty { }
            }

            fn as_owned(&self) -> Self::Owned {
                $ty { }
            }
        }
    };
}

macro_rules! parse {
    (bare $ty:tt { $($field:ident),* $(,)? } => $body:expr) => {
        impl<'a> Parse<&'a Message<&'a str>> for $ty<&'a str> {
            fn parse(msg: &'a Message<&'a str>) -> Result<Self, InvalidMessage> {
                $body(msg)
            }
        }

        impl<'a> Parse<&'a Message<&'a str>> for $ty<String> {
            fn parse(msg: &'a Message<&'a str>) -> Result<Self, InvalidMessage> {
                $ty::<&'a str>::parse(msg).map(|ok| ok.as_owned())
            }
        }
        // TODO more conversions
    };

    ($ty:tt { $($field:ident),* $(,)? } => $body:expr) => {
        conversion!($ty { $($field,)* });
        parse!(bare $ty { $($field,)* } => $body);
    };

    ($ty:tt => $body:expr) => {
        conversion!($ty);

        impl<'a> Parse<&'a Message<&'a str>> for $ty {
            fn parse(msg: &'a Message<&'a str>) -> Result<Self, InvalidMessage> {
                $body(msg)
            }
        }

        impl<'a> Parse<&'a Message<String>> for $ty {
            fn parse(msg: &'a Message<String>) -> Result<Self, InvalidMessage> {
                $body(&msg.as_borrowed()).map(|ok| ok.as_owned())
            }
        }
    };
}

macro_rules! make_event {
    (@DOC $($doc:expr)* => $item:tt) => {
        $(#[doc = $doc])*
        #[non_exhaustive]
        #[allow(missing_debug_implementations,missing_copy_implementations)]
        pub struct $item;
    };

    ($($event:ident => $message:path)*) => {
        $(
            make_event!(@DOC
                concat!("Used to get a [", stringify!($message), "](../messages/struct.", stringify!($event), ".html)")
                =>
                $event
            );

            impl<'a> crate::client::Event<'a> for $event {
                type Mapped = $message;
            }
        )*
    };
}

macro_rules! make_mapping {
    ($($event:expr => $ident:ident)*) => {
        pub(crate) fn dispatch<'a>(&mut self, msg: &'a Message<&'a str>) {
            match msg.command {
                $($event => self.try_send::<events::$ident>(&msg),)*
                _ => {},
            }
            self.try_send::<events::Raw>(&msg);
        }

        pub(crate) fn new() -> Self {
            Self { event_map: Default::default() }
            $( .add_event::<events::$ident>() )*
            .add_event::<events::Raw>()
        }
    };
}

macro_rules! export_modules {
    ($($module:ident)*) => {
        $( mod $module; pub use $module::*; )*
    };
}
