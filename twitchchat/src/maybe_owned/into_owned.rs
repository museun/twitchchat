use super::{MaybeOwned, MaybeOwnedIndex};
use crate::{color::Color, UserConfig};

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

/// Converts a 'borrowed' type into an owned type. e.g. `'a` to `'static`
pub trait IntoOwned<'a> {
    /// The output type
    type Output: 'static;
    /// Consumes self, returning an owned version
    fn into_owned(self) -> Self::Output;
}

impl<'a> IntoOwned<'a> for MaybeOwned<'a> {
    type Output = MaybeOwned<'static>;
    fn into_owned(self) -> Self::Output {
        match self {
            Self::Owned(s) => MaybeOwned::Owned(s),
            Self::Borrowed(s) => MaybeOwned::Owned(s.to_string().into_boxed_str()),
        }
    }
}

impl IntoOwned<'static> for MaybeOwnedIndex {
    type Output = Self;
    fn into_owned(self) -> Self::Output {
        self
    }
}

impl IntoOwned<'static> for Color {
    type Output = Self;
    fn into_owned(self) -> Self::Output {
        self
    }
}

impl IntoOwned<'static> for UserConfig {
    type Output = Self;
    fn into_owned(self) -> Self::Output {
        self
    }
}

impl<'a, T: IntoOwned<'a> + Clone> IntoOwned<'a> for &'a T {
    type Output = T::Output;
    fn into_owned(self) -> Self::Output {
        self.clone().into_owned()
    }
}

impl<'a, T: IntoOwned<'a> + 'a> IntoOwned<'a> for Option<T> {
    type Output = Option<T::Output>;
    fn into_owned(self) -> Self::Output {
        self.map(IntoOwned::into_owned)
    }
}

macro_rules! into_owned_primitives {
    ($($ty:ty)*) => {
        $(
            impl IntoOwned<'static> for $ty {
                type Output = Self;
                fn into_owned(self) -> Self::Output {
                    self
                }
            }
        )*
    };
}

into_owned_primitives! {
    u8 u16 u32 u64 u128 usize
    i8 i16 i32 i64 i128 isize
    bool f32 f64
}
