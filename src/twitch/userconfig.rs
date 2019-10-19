use crate::twitch::Capability;
use std::collections::BTreeSet;

pub(crate) const JUSTINFAN1234: &str = "justinfan1234";

/// Configuration used to complete the 'registration' with the irc server
#[derive(Clone)]
pub struct UserConfig {
    /// OAuth token from twitch, it must have the
    /// [scopes](https://dev.twitch.tv/docs/authentication/#scopes):
    /// `chat:read`, `chat:edit`
    pub token: String,
    /// Username to use on twitch. (must be associated with the oauth token)
    pub nick: String,
    /// Which capabilites to enable
    pub caps: Vec<Capability>,
}

impl std::fmt::Debug for UserConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserConfig")
            .field("nick", &self.nick)
            .field("token", &"************ (redacted)")
            .field("caps", &self.caps)
            .finish()
    }
}

impl UserConfig {
    /// Create a [`UserConfigBuilder`](./userconfig/struct.UserConfigBuilder.html), defaults with all of the [`Capabilities`](./enum.Capability.html) disabled
    pub fn builder() -> UserConfigBuilder {
        UserConfigBuilder::default()
    }

    /// Create a [`UserConfigBuilder`](./userconfig/struct.UserConfigBuilder.html), with all of the [`Capabilities`](./enum.Capability.html) enabled
    pub fn with_caps() -> UserConfigBuilder {
        UserConfigBuilder::default().membership().commands().tags()
    }
}

/// A _builder_ type to create a [`UserConfig`](./struct.UserConfig.html) without dumb errors (like swapping nick/token)
pub struct UserConfigBuilder {
    nick: Option<String>,
    token: Option<String>,
    caps: BTreeSet<Capability>,
}

impl Default for UserConfigBuilder {
    fn default() -> Self {
        Self {
            nick: None,
            token: None,
            caps: BTreeSet::default(),
        }
    }
}

impl UserConfigBuilder {
    /// Use this nickname in the configuration
    pub fn nick<S: ToString>(mut self, nick: S) -> Self {
        let _ = self.nick.replace(nick.to_string());
        self
    }

    /// Use this oauth token in the configuration
    // check for the leading 'oauth:'
    // and probably the length (its probably 64 bytes)
    pub fn token<S: ToString>(mut self, token: S) -> Self {
        let token = token.to_string();
        if token == JUSTINFAN1234 || (token.starts_with("oauth") && token.len() == 36) {
            let _ = self.token.replace(token.to_string());
        }
        self
    }

    /// Enable or disable the membership capability
    ///
    /// Disabled by default
    pub fn membership(mut self) -> Self {
        self.toggle_cap(Capability::Membership);
        self
    }

    /// Enable or disable the commands capability
    ///
    /// Disabled by default
    pub fn commands(mut self) -> Self {
        self.toggle_cap(Capability::Commands);
        self
    }

    /// Enable or disable the tags capability
    ///
    /// Disabled by default
    pub fn tags(mut self) -> Self {
        self.toggle_cap(Capability::Tags);
        self
    }

    /// Build the `UserConfig`
    ///
    /// Returns None if `token` isn't 36-characters and prefixed by oauth:
    /// **note** panics if `nick` was empty
    pub fn build(self) -> Option<UserConfig> {
        Some(UserConfig {
            nick: self.nick.unwrap(),
            token: self.token?,
            caps: self.caps.into_iter().collect(),
        })
    }

    fn toggle_cap(&mut self, cap: Capability) {
        if self.caps.contains(&cap) {
            let _ = self.caps.remove(&cap);
        } else {
            let _ = self.caps.insert(cap);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_token() {
        assert!(
            UserConfig::builder()
                .nick("justinfan12345")
                .token("oauth:12345678901234567890123456789")
                .build()
                .is_none(),
            "token is too short"
        );
        assert!(
            UserConfig::builder()
                .nick("justinfan12345")
                .token("123456789012345678901234567890123456")
                .build()
                .is_none(),
            "no prefix"
        );
    }

    #[test]
    fn print_userconfig() {
        let c = UserConfig::builder()
            .nick("justinfan12345")
            .token("oauth:123456789012345678901234567890")
            .tags()
            .membership()
            .commands()
            .build()
            .unwrap();

        let good = r#"UserConfig { nick: "justinfan12345", token: "************ (redacted)", caps: [Membership, Commands, Tags] }"#;
        assert_eq!(format!("{:?}", c), good);
    }
}
