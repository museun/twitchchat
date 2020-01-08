use crate::Capability;
use std::collections::BTreeSet;

/**
User configuration for 'registering' with Twitch

This is used to *register* the connection with the server

It contains your user name, your OAuth token and the capabilities you want to request.

# example using a builder
```
# use twitchchat::{Capability, UserConfig};
# std::env::set_var("TWITCH_NAME", "foo");
# std::env::set_var("TWITCH_TOKEN", format!("oauth:{}", "a".repeat(30)));
// as anonymous
let config = UserConfig::builder().anonymous().build().unwrap();
// or with a name/token
let name = std::env::var("TWITCH_NAME").unwrap();
let token = std::env::var("TWITCH_TOKEN").unwrap();
let config = UserConfig::builder()
    .name(name)
    .token(token)
    .capabilities(&[Capability::Tags])
    .build()
    .unwrap();
```
*/
#[derive(Debug, Clone, PartialEq)]
pub struct UserConfig {
    /// Requested name of your user
    pub name: String,
    /// OAuth token of the user
    pub token: String,
    /// Capabilities to be requested from the server
    pub capabilities: Vec<Capability>,
}

impl UserConfig {
    /// Create a builder to make a [Config]
    ///
    /// [Config]: ./struct.UserConfig.html
    pub fn builder() -> UserConfigBuilder {
        UserConfigBuilder::default()
    }

    /// Determines whether this config was requested as anonymous
    pub fn is_anonymous(&self) -> bool {
        self.name == crate::JUSTINFAN1234 && self.token == crate::JUSTINFAN1234
    }
}

impl crate::Encodable for UserConfig {
    fn encode<W: ?Sized + std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let UserConfig {
            name,
            token,
            capabilities,
        } = self;

        for cap in capabilities {
            writer.write_all(cap.encode_as_str().as_bytes())?;
            writer.write_all(b"\r\n")?;
        }

        writer.write_all(format!("PASS {}\r\n", token).as_bytes())?;
        writer.write_all(format!("NICK {}\r\n", name).as_bytes())?;

        Ok(())
    }
}

/// User config error returned by the [UserConfigBuilder]
///
/// [UserConfigBuilder]: ./struct.UserConfigBuilder.html
#[non_exhaustive]
#[derive(Debug)]
pub enum UserConfigError {
    /// An invalid name was provided
    InvalidName,
    /// An invalid token was provided.
    InvalidToken,
    /// Anonymous login was requested with a user-provided name or token
    PartialAnonymous,
}

impl std::fmt::Display for UserConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserConfigError::InvalidName => f.write_str("invalid name"),
            UserConfigError::InvalidToken => {
                f.write_str("invalid token. token must start with oauth: and be 36 characters")
            }
            UserConfigError::PartialAnonymous => f.write_str(
                "user provided name or token provided when an anonymous login was requested",
            ),
        }
    }
}

impl std::error::Error for UserConfigError {}

/// Builder for making a [UserConfig]
///
/// [UserConfig]: ./struct.UserConfig.html
#[derive(Default)]
pub struct UserConfigBuilder {
    capabilities: BTreeSet<Capability>,
    name: Option<String>,
    token: Option<String>,
}

impl UserConfigBuilder {
    /// Name for the connection
    pub fn name(mut self, name: impl ToString) -> Self {
        self.name.replace(name.to_string());
        self
    }

    /// Token for the connection
    ///
    /// This is an oauth token. It must have atleast two [scopes]: `chat:read`, `chat:edit`
    ///
    /// [scopes]: https://dev.twitch.tv/docs/authentication/#scopes
    pub fn token(mut self, token: impl ToString) -> Self {
        self.token.replace(token.to_string());
        self
    }

    /// Uses an anonymous login
    ///
    /// This uses `"justin1234"` as the name and token
    ///
    /// Capabilites cannot be used with this login
    pub fn anonymous(self) -> Self {
        let (name, token) = crate::ANONYMOUS_LOGIN;
        self.name(name).token(token)
    }

    /// Capabilities to enable
    ///
    /// This will produce an [error] on [build] if the name/pass were ***anonymous***
    ///
    /// [error]: ./enum.UserConfigError.html#variant.AnonymousCapabilities
    /// [build]: ./struct.UserConfigBuilder.html#method.build
    pub fn capabilities(mut self, caps: &[Capability]) -> Self {
        self.capabilities.extend(caps);
        self
    }

    /// Tries to build the UserConfig
    ///
    /// This returns an error if the name or token are invalid
    pub fn build(mut self) -> Result<UserConfig, UserConfigError> {
        let name = self
            .name
            .filter(|s| validate_name(&s))
            .ok_or_else(|| UserConfigError::InvalidName)?;

        let token = self
            .token
            .filter(|s| validate_token(&s))
            .ok_or_else(|| UserConfigError::InvalidToken)?;

        match (name.as_str(), token.as_str()) {
            (crate::JUSTINFAN1234, crate::JUSTINFAN1234) => {
                self.capabilities.clear();
            }
            (crate::JUSTINFAN1234, ..) | (.., crate::JUSTINFAN1234) => {
                return Err(UserConfigError::PartialAnonymous)
            }
            _ => {}
        }

        Ok(UserConfig {
            name,
            token,
            capabilities: self.capabilities.into_iter().collect(),
        })
    }
}

#[inline]
fn validate_name(s: &str) -> bool {
    !s.is_empty()
}

#[inline]
fn validate_token(s: &str) -> bool {
    if s == crate::JUSTINFAN1234 {
        return true;
    }
    !s.is_empty() && s.len() == 36 && &s[..6] == "oauth:"
}

#[cfg(test)]
mod tests {
    use super::*;
    use matches::matches;

    #[test]
    fn valid_user_config_no_caps() {
        let config = UserConfig::builder()
            .name("foo")
            .token(format!("oauth:{}", "a".repeat(30)))
            .build()
            .unwrap();

        assert_eq!(
            config,
            UserConfig {
                name: "foo".to_string(),
                token: format!("oauth:{}", "a".repeat(30)),
                capabilities: vec![],
            }
        )
    }

    #[test]
    fn valid_user_config() {
        let config = UserConfig::builder()
            .name("foo")
            .token(format!("oauth:{}", "a".repeat(30)))
            .capabilities(&[Capability::ChatRooms])
            .capabilities(&[Capability::Tags, Capability::Tags])
            .capabilities(&[Capability::Membership, Capability::ChatRooms])
            .build()
            .unwrap();

        assert_eq!(
            config,
            UserConfig {
                name: "foo".to_string(),
                token: format!("oauth:{}", "a".repeat(30)),
                capabilities: vec![
                    Capability::Membership,
                    Capability::Tags,
                    Capability::ChatRooms,
                ],
            }
        )
    }

    #[test]
    fn valid_user_config_anonymous() {
        let config = UserConfig::builder().anonymous().build().unwrap();

        assert_eq!(
            config,
            UserConfig {
                name: crate::JUSTINFAN1234.to_string(),
                token: crate::JUSTINFAN1234.to_string(),
                capabilities: vec![],
            }
        );

        assert!(config.is_anonymous());
    }

    #[test]
    fn invalid_name_missing() {
        let err = UserConfig::builder().build().unwrap_err();
        matches!(err, UserConfigError::InvalidName);
    }

    #[test]
    fn invalid_partial_login_name() {
        let err = UserConfig::builder()
            .anonymous()
            .name("foo")
            .build()
            .unwrap_err();
        matches!(err, UserConfigError::PartialAnonymous);
    }

    #[test]
    fn invalid_partial_login_token() {
        let err = UserConfig::builder()
            .anonymous()
            .token(format!("oauth:{}", "a".repeat(30)))
            .build()
            .unwrap_err();
        matches!(err, UserConfigError::PartialAnonymous);
    }

    #[test]
    fn invalid_token_missing() {
        let err = UserConfig::builder().name("foobar").build().unwrap_err();
        matches!(err, UserConfigError::InvalidToken);
    }

    #[test]
    fn invalid_token_empty() {
        let err = UserConfig::builder()
            .name("foobar")
            .token("")
            .build()
            .unwrap_err();
        matches!(err, UserConfigError::InvalidToken);
    }

    #[test]
    fn invalid_token_short() {
        let err = UserConfig::builder()
            .name("foobar")
            .token("foo")
            .build()
            .unwrap_err();
        matches!(err, UserConfigError::InvalidToken);
    }

    #[test]
    fn invalid_token_no_oauth() {
        let err = UserConfig::builder()
            .name("foobar")
            .token("a".repeat(36))
            .build()
            .unwrap_err();
        matches!(err, UserConfigError::InvalidToken);
    }
}
