mod tcpconn;
pub use self::tcpconn::TcpConn;

mod message;
mod prefix;
mod tags;

/// Collection of types used by the IRC client
pub mod types {
    pub use super::message::Message;
    pub use super::prefix::Prefix;
    pub use super::tags::Tags;
}
