//! Twitch messages that can be parsed from `IrcMessage`, or subscribed to from the `Dispatcher`
//!
//!
//! # Converting from an `IrcMessage` to a specific message
//!
//! ```
//! let input = "user!user@user PRIVMSG #test_channel :this is some data\r\n";
//! let irc_msg = crate::irc::parse(input).next().unwrap().unwrap();
//!
//! // this is implemented for all of the tyupes in this module
//! use twitchchat::FromIrcMessage;
//! use twitchchat::messages::Privmsg;
//! // this will produce an error if its not this type of message
//! let pm = Privmsg::from_irc(msg).unwrap();
//! assert_eq!(pm.data(), "this is some data");
//! ```
//!
//! # Converting an `IrcMessage` to an enum of all possible messages
//!
//! ```
//! let input = "user!user@user PRIVMSG #test_channel :this is some data\r\n";
//! let irc_msg = crate::irc::parse(input).next().unwrap().unwrap();
//!
//! // this is implemented for all of the tyupes in this module
//! use twitchchat::FromIrcMessage;
//! use twitchchat::messages::Commands;
//!
//! let all = Commands::from_irc(msg);
//! assert!(matches!(all, Commands::Privmsg{..}));
//! ```
//!

mod commands;
pub use commands::Commands;

mod irc_ready;
pub use irc_ready::IrcReady;

mod ready;
pub use ready::Ready;

mod cap;
pub use cap::{Cap, Capability};

mod clear_chat;
pub use clear_chat::ClearChat;

mod clear_msg;
pub use clear_msg::ClearMsg;

mod global_user_state;
pub use global_user_state::GlobalUserState;

mod host_target;
pub use host_target::{HostTarget, HostTargetKind};

mod join;
pub use join::Join;

mod notice;
pub use notice::{MessageId, Notice};

mod part;
pub use part::Part;

mod ping;
pub use ping::Ping;

mod pong;
pub use pong::Pong;

mod privmsg;
pub use privmsg::Privmsg;

mod reconnect;
pub use reconnect::Reconnect;

mod room_state;
pub use room_state::{FollowersOnly, RoomState};

mod user_notice;
pub use user_notice::{NoticeType, SubPlan, UserNotice};

mod user_state;
pub use user_state::UserState;

mod whisper;
pub use whisper::Whisper;

pub use crate::irc::IrcMessage;
