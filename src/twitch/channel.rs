use super::*;

/// Simple channel wrapper.
///
/// This ensures the twitch channels align with IRC naming scheme.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Channel(String);

/// A trait to convert types into [`Channel`](./struct.Channel.html)
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
    pub(crate) fn validate(name: impl ToString) -> Result<Channel, Error> {
        let name = name.to_string();
        if name.is_empty() {
            return Err(Error::EmptyChannelName);
        }

        let name = name.to_lowercase();
        let name = if !name.starts_with('#') {
            ["#", name.as_str()].concat()
        } else {
            name.to_string()
        };
        Ok(Channel(name))
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
        if let Error::EmptyChannelName = err {
        } else {
            panic!("wrong error: {}", err)
        }
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
