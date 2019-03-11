/// Configuration used to complete the 'registration' with the irc server
pub struct UserConfig {
    /// OAuth token from twitch, it must have the
    /// [scopes](https://dev.twitch.tv/docs/authentication/#scopes):
    /// `chat:read`, `chat:edit`
    pub token: String,
    /// Username to use on twitch. (must be associated with the oauth token)
    pub nick: String,
}
