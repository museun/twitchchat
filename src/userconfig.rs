use crate::twitch::Capability;

/// Configuration used to complete the 'registration' with the irc server
pub struct UserConfig {
    /// OAuth token from twitch, it must have the
    /// [scopes](https://dev.twitch.tv/docs/authentication/#scopes):
    /// `chat:read`, `chat:edit`
    pub token: String,
    /// Username to use on twitch. (must be associated with the oauth token)
    pub nick: String,
    //
    // TODO allow for TLS configuration here
    //
    /// Which capabilites to enable
    pub caps: Vec<Capability>,
}

/// A _builder_ type to create a `UserConfig` without dumb errors (like swapping nick/token)
pub struct UserConfigBuilder {
    nick: Option<String>,
    token: Option<String>,
    caps: hashbrown::HashSet<Capability>,
}

impl Default for UserConfigBuilder {
    fn default() -> Self {
        Self {
            nick: None,
            token: None,
            caps: [
                Capability::Membership,
                Capability::Commands,
                Capability::Tags,
            ]
            .iter()
            .cloned()
            .collect(),
        }
    }
}

impl UserConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Use this nickname in the configuration
    pub fn nick<S: ToString>(mut self, nick: S) -> Self {
        self.nick.replace(nick.to_string());
        self
    }

    /// Use this oauth token in the configuration
    // check for the leading 'oauth:'
    // and probably the length (its probably 64 bytes)
    pub fn token<S: ToString>(mut self, token: S) -> Self {
        self.token.replace(token.to_string());
        self
    }

    /// Enable or disable the membership capability
    ///
    /// Enabled by default
    pub fn membership(mut self) -> Self {
        self.toggle_cap(Capability::Membership);
        self
    }

    /// Enable or disable the commands capability
    ///
    /// Enabled by default

    pub fn commands(mut self) -> Self {
        self.toggle_cap(Capability::Commands);
        self
    }

    /// Enable or disable the tags capability
    ///
    /// Enabled by default
    pub fn tags(mut self) -> Self {
        self.toggle_cap(Capability::Tags);
        self
    }

    /// Build the `UserConfig`
    ///
    /// Returns None if nick or token are invalid
    pub fn build(self) -> Option<UserConfig> {
        Some(UserConfig {
            nick: self.nick?,
            token: self.token?,
            caps: self.caps.into_iter().collect(),
        })
    }

    fn toggle_cap(&mut self, cap: Capability) {
        if self.caps.contains(&cap) {
            self.caps.remove(&cap);
        } else {
            self.caps.insert(cap);
        }
    }
}

impl UserConfig {
    /// Create a `UserConfigBuilder`, defaults with all of the `Capability` enabled
    pub fn builder() -> UserConfigBuilder {
        UserConfigBuilder::new()
    }
}
