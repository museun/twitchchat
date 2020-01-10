/// Used for StringMarker to -> String
macro_rules! fast_to_string {
    ($expr:expr) => {
        (*($expr.as_ref())).to_string()
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

macro_rules! parse {
    ($ty:tt { $($field:ident),* $(,)? } => $body:expr) => {
        impl<'a> TryFrom<&'a Message<&'a str>> for $ty<&'a str> {
            type Error = InvalidMessage;
            fn try_from(msg: &'a Message<&'a str>) -> Result<Self, Self::Error> {
                $body(msg)
            }
        }
        as_owned!(for $ty { $($field),* });
    };

    ($ty:tt => $body:expr) => {
        impl<'a> TryFrom<&'a Message<&'a str>> for $ty {
            type Error = InvalidMessage;
            fn try_from(msg: &'a Message<&'a str>) -> Result<Self, Self::Error> {
                $body(msg)
            }
        }
        as_owned!(for $ty);
    };
}

macro_rules! as_owned {
    (for $ty:tt { $($field:ident),* $(,)? }) => {
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

    (for $ty:tt) => {
        impl IntoOwned for $ty {
            type Target = $ty;

            fn into_owned(&self) -> Self::Target {
                self.clone()
            }
        }
    }
}

macro_rules! make_event {
    (@DOC $($doc:expr)* => $item:tt) => {
        $(#[doc = $doc])*
        #[non_exhaustive]
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
