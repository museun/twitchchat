use super::*;

/// Simple channel wrapper.
///
/// This ensures the twitch channels align with IRC naming scheme.
#[derive(Debug, PartialEq)]
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

impl From<String> for Channel {
    fn from(s: String) -> Self {
        if s.is_empty() {
            return Self("".into());
        }

        let s = s.to_lowercase();
        let s = if !s.starts_with('#') {
            ["#", s.as_str()].concat()
        } else {
            s.to_string()
        };

        Self(s)
    }
}

impl From<&str> for Channel {
    fn from(s: &str) -> Self {
        s.to_string().into()
    }
}

impl std::ops::Deref for Channel {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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
}
