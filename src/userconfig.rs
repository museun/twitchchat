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

impl UserConfig {
    /// Create a new config from token and nick with capabilities: `Membership`+`Commands`+`Tags`
    pub fn new_with_default_caps<S>(token: S, nick: S) -> Self
    where
        S: ToString,
    {
        Self {
            token: token.to_string(),
            nick: nick.to_string(),
            caps: vec![
                Capability::Membership,
                Capability::Commands,
                Capability::Tags,
            ],
        }
    }
}
