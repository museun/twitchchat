mod tcpconn;
pub use self::tcpconn::TcpConn;

mod userconfig;
pub use self::userconfig::UserConfig;

mod message;
mod prefix;
mod tags;

mod capability;

/// Collection of types used by the IRC client
pub mod types {
    pub use super::message::Message;
    pub use super::prefix::Prefix;
    pub use super::tags::Tags;
}
