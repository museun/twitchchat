macro_rules! into_owned {
    ($ty:ident { $($field:ident),* $(,)? }) => {
        impl<'a> $crate::IntoOwned<'a> for $ty<'a> {
            type Output = $ty<'static>;
            fn into_owned(self) -> Self::Output {
                $ty { $( $field: self.$field.into_owned(),)* }
            }
        }
    };
}

macro_rules! serde_struct {
    (@one $($x:tt)*) => { () };
    (@len $($e:expr),*) => { <[()]>::len(&[$(serde_struct!(@one $e)),*]) };

    ($ty:ident { $($field:ident),* $(,)? }) => {
        #[cfg(feature = "serde")]
        impl<'a> ::serde::Serialize for $ty<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                use ::serde::ser::SerializeMap as _;
                let len = serde_struct!(@len $($field),*);
                let mut s = serializer.serialize_map(Some(len))?;
                $( s.serialize_entry(stringify!($field), &self.$field())?; )*
                s.end()
            }
        }

        serde_struct!{ @de $ty }
    };

    (@de $ty:ident) => {
        #[cfg(feature = "serde")]
        impl<'de, 'a> ::serde::Deserialize<'de> for $ty<'a> {
            fn deserialize<D>(deserializer: D) -> Result<$ty<'a>, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                deserializer.deserialize_map($crate::serde::RawVisitor::default())
            }
        }
    };
}

macro_rules! impl_custom_debug {
    ($ty:ident { $($field:ident),* $(,)? }) => {
        impl<'a> std::fmt::Debug for $ty<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($ty))
                    $( .field(stringify!($field), &self.$field()) )*
                .finish()
            }
        }
    };
}

// TODO get rid of these macros
macro_rules! raw {
    () => {
        /// Get the raw message
        pub fn raw(&self) -> &str {
            &*self.raw
        }
    };
}

macro_rules! into_inner_raw {
    () => {
        /// Consumes the message, returning the raw [`MaybeOwned<'_>`](./enum.Str.html)
        fn into_inner(self) -> MaybeOwned<'a> {
            self.raw
        }
    };
}

macro_rules! tags {
    () => {
        /// Get a view of parsable tags
        pub fn tags(&self) -> $crate::irc::Tags<'_> {
            Tags {
                data: &self.raw,
                indices: &self.tags,
            }
        }
    };
}

macro_rules! str_field {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        pub fn $name(&self) -> &str {
            &self.raw[self.$name]
        }
    };
    ($name:ident) => {
        pub fn $name(&self) -> &str {
            &self.raw[self.$name]
        }
    };
}

macro_rules! opt_str_field {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        pub fn $name(&self) -> Option<&str> {
            self.$name.map(|index| &self.raw[index])
        }
    };

    ($name:ident) => {
        pub fn $name(&self) -> Option<&str> {
            self.$name.map(|index| &self.raw[index])
        }
    };
}
