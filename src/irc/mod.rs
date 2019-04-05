mod message;
mod prefix;

/// Collection of types used by the IRC client
pub mod types {
    pub use super::message::Message;
    pub use super::prefix::Prefix;
}
