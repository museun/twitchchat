/// Simple channel wrapper.
///
/// This ensures the twitch channels align with IRC naming scheme.
#[derive(Debug, Clone, PartialEq)]
pub struct Channel(String);

impl AsRef<str> for Channel {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// An error produced by invalid channel names
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum Error {
    /// Channel name was empty
    EmptyChannelName,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyChannelName => f.write_str("empty channel name"),
        }
    }
}

impl std::error::Error for Error {}

/// A trait to convert types into [`Channel`](./struct.Channel.html)
///
/// ```rust
/// # use twitchchat::*;
/// let channel: Channel = "museun".into_channel().unwrap();
/// assert_eq!(*channel, "#museun");
///
/// let channel: Channel = "#museun".into_channel().unwrap();
/// assert_eq!(*channel, "#museun");
/// ```
pub trait IntoChannel {
    /// Tries to convert this type a channel
    fn into_channel(self) -> Result<Channel, Error>;
}

impl<T> IntoChannel for T
where
    T: ToString,
{
    fn into_channel(self) -> Result<Channel, Error> {
        Channel::validate(self.to_string())
    }
}

impl Channel {
    pub(crate) fn validate(name: impl ToString) -> Result<Self, Error> {
        let name = name.to_string();
        if name.is_empty() {
            return Err(Error::EmptyChannelName);
        }

        let name = name.to_lowercase();
        let name = if !name.starts_with('#') {
            ["#", name.as_str()].concat()
        } else {
            name
        };
        Ok(Self(name))
    }
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq<str> for Channel {
    fn eq(&self, other: &str) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<&str> for Channel {
    fn eq(&self, other: &&str) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<String> for Channel {
    fn eq(&self, other: &String) -> bool {
        self.0.eq(other)
    }
}

impl std::ops::Deref for Channel {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn good_channel() {
        assert_eq!(Channel::validate("museun").unwrap().0, "#museun");
    }

    #[test]
    fn equals() {
        let s = "foobar";
        let ch: Channel = s.into_channel().unwrap();
        assert!(ch == "#foobar");
        assert!(ch == "#foobar".to_string());
    }

    #[test]
    fn bad_channel() {
        let err = Channel::validate("").unwrap_err();
        matches::assert_matches!(err, Error::EmptyChannelName);
    }

    #[test]
    fn into_channel() {
        let s = String::from("museun");

        let channels: Vec<Channel> = vec![
            s.as_str().into_channel().unwrap(),
            s.clone().into_channel().unwrap(),
            s.into_channel().unwrap(),
            "museun".into_channel().unwrap(),
            String::from("museun").into_channel().unwrap(),
            std::sync::Arc::new(String::from("museun"))
                .into_channel()
                .unwrap(),
            std::rc::Rc::new(String::from("museun"))
                .into_channel()
                .unwrap(),
            String::from("museun")
                .into_channel()
                .unwrap()
                .into_channel()
                .unwrap(),
        ];

        for name in channels {
            assert_eq!(*name, "#museun");
        }
    }
}
