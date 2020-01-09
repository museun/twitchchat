use {std::fmt::Debug, std::hash::Hash};

#[doc(hidden)]
pub trait StringMarker
where
    Self: Hash + Debug + Clone,
    Self: Eq + PartialEq + AsRef<str>,
    Self: private::string_marker::Sealed,
{
}

impl StringMarker for String {}
impl<'a> StringMarker for &'a str {}

#[doc(hidden)]
pub trait IntoOwned: private::into_owned::Sealed {
    type Target;
    fn into_owned(&self) -> Self::Target;
}

// impl IntoOwned for str {
//     type Target = String;
//     fn into_owned(&self) -> Self::Target {
//         self.to_string()
//     }
// }

impl IntoOwned for &str {
    type Target = String;
    fn into_owned(&self) -> Self::Target {
        (*self).to_string()
    }
}

impl IntoOwned for String {
    type Target = String;
    fn into_owned(&self) -> Self::Target {
        self.to_string()
    }
}

impl IntoOwned for crate::decode::Prefix<&str> {
    type Target = crate::decode::Prefix<String>;
    fn into_owned(&self) -> Self::Target {
        self.into_owned()
    }
}

impl IntoOwned for crate::Tags<&str> {
    type Target = crate::Tags<String>;
    fn into_owned(&self) -> Self::Target {
        crate::Tags(
            self.clone()
                .into_inner()
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        )
    }
}

impl<T> IntoOwned for Option<T>
where
    T: IntoOwned,
{
    type Target = Option<T::Target>;
    fn into_owned(&self) -> Self::Target {
        self.as_ref().map(|s| (*s).into_owned())
    }
}

impl<T> IntoOwned for Vec<T>
where
    T: IntoOwned,
{
    type Target = Vec<T::Target>;
    fn into_owned(&self) -> Self::Target {
        self.clone()
            .into_iter()
            .map(IntoOwned::into_owned)
            .collect()
    }
}

impl IntoOwned for bool {
    type Target = bool;
    fn into_owned(&self) -> Self::Target {
        *self
    }
}

impl IntoOwned for usize {
    type Target = usize;
    fn into_owned(&self) -> Self::Target {
        *self
    }
}

impl IntoOwned for crate::color::Color {
    type Target = crate::color::Color;
    fn into_owned(&self) -> Self::Target {
        *self
    }
}

impl IntoOwned for crate::Badge {
    type Target = crate::Badge;
    fn into_owned(&self) -> Self::Target {
        self.clone()
    }
}

mod private {
    pub mod into_owned {
        pub trait Sealed {}
        impl<T> Sealed for T where T: crate::internal::IntoOwned {}
    }

    pub mod string_marker {
        pub trait Sealed {}
        impl<T> Sealed for T where T: crate::internal::StringMarker {}
    }
}
