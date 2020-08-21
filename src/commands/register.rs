use super::Encodable;
use crate::twitch::UserConfig;

use std::io::Write;

/// Registers with Twitch. This writes the `UserConfig` out
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Register<'a> {
    pub(crate) user_config: UserConfig,
    #[cfg_attr(feature = "serde", serde(skip, default))]
    _marker: std::marker::PhantomData<&'a ()>,
}

/// Write the User Registration to the connection.
///
/// This is required to be done before you do anything else.
pub fn register(user_config: &UserConfig) -> Register<'_> {
    // TODO serde really doesn't like this type, so lets clone it
    let user_config = user_config.clone();
    Register {
        user_config,
        _marker: std::marker::PhantomData,
    }
}

impl<'a> Encodable for Register<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> std::io::Result<()> {
        let UserConfig {
            name,
            token,
            capabilities,
        } = &self.user_config;

        // the caps have to be written first
        for cap in capabilities {
            let cap = cap.encode_as_str();
            write!(buf, "{}\r\n", cap)?;
        }

        write!(buf, "PASS {}\r\n", token)?;
        write!(buf, "NICK {}\r\n", name)?;
        buf.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn register_encode() {
        let config = UserConfig::builder()
            .anonymous()
            .enable_all_capabilities()
            .build()
            .unwrap();

        test_encode(
            register(&config),
            "CAP REQ :twitch.tv/membership\r\n\
            CAP REQ :twitch.tv/tags\r\n\
            CAP REQ :twitch.tv/commands\r\n\
            PASS justinfan1234\r\n\
            NICK justinfan1234\r\n",
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn register_serde() {
        let config = UserConfig::builder()
            .anonymous()
            .enable_all_capabilities()
            .build()
            .unwrap();

        test_serde(
            register(&config),
            "CAP REQ :twitch.tv/membership\r\n\
            CAP REQ :twitch.tv/tags\r\n\
            CAP REQ :twitch.tv/commands\r\n\
            PASS justinfan1234\r\n\
            NICK justinfan1234\r\n",
        )
    }
}
