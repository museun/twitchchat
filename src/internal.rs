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

pub(crate) trait IntoString {
    type Target;
    fn into_string(&self) -> Self::Target;
}

impl IntoString for str {
    type Target = String;
    fn into_string(&self) -> Self::Target {
        self.to_string()
    }
}

impl IntoString for &str {
    type Target = String;
    fn into_string(&self) -> Self::Target {
        (*self).to_string()
    }
}

impl IntoString for String {
    type Target = String;
    fn into_string(&self) -> Self::Target {
        self.to_string()
    }
}

impl IntoString for crate::decode::Prefix<&str> {
    type Target = crate::decode::Prefix<String>;
    fn into_string(&self) -> Self::Target {
        self.into_owned()
    }
}

impl IntoString for crate::Tags<&str> {
    type Target = crate::Tags<String>;
    fn into_string(&self) -> Self::Target {
        crate::Tags(
            self.clone()
                .into_inner()
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        )
    }
}

impl<T> IntoString for Option<T>
where
    T: IntoString,
{
    type Target = Option<T::Target>;
    fn into_string(&self) -> Self::Target {
        self.as_ref().map(|s| s.into_string())
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
