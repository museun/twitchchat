use super::*;

/// When a user joins a channel
#[derive(Debug, PartialEq, Clone)]
pub struct Join {
    /// IRC User that joined
    pub prefix: Option<Prefix>,
    /// The channel that they joined
    pub channel: String,
}
