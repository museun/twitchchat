use super::*;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Simple channel wrapper.
///
/// This ensures the twitch channels align with IRC naming scheme.
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Channel(String);

impl Channel {
    pub(crate) fn validate<C: Into<Channel>>(channel: C) -> Result<Channel, Error> {
        let channel = channel.into();
        if channel.0.is_empty() {
            return Err(Error::EmptyChannelName);
        }
        Ok(channel)
    }
}

impl<T> From<T> for Channel
where
    T: ToString,
{
    fn from(name: T) -> Self {
        let name = name.to_string();
        if name.is_empty() {
            return Self("".into());
        }

        let name = name.to_lowercase();
        let name = if !name.starts_with('#') {
            ["#", name.as_str()].concat()
        } else {
            name.to_string()
        };

        Self(name)
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
            s.as_str().into(),
            (&s).into(),
            s.clone().into(),
            s.into(),
            "museun".into(),
            String::from("museun").into(),
            (&String::from("museun")).into(),
        ];

        for name in channels {
            assert_eq!(*name, "#museun");
        }
    }
}
