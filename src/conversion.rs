use crate::irc::Message as IrcMessage;
use std::collections::HashMap;

/// Convert an IRC-like message type into something that the Twitch commands can be parsed from
///
/// Refer to this form when implementing this trait:
///
/// raw string form: `@tags :prefix command args :data\r\n`
///
/// Example:
/** ```
use twitchchat::conversion::{TagType, ArgsType};
use std::collections::HashMap;
struct MyPrivMsg {
    tags: HashMap<String, String>,
    sender: String,
    channel: String,
    data: String,
}
impl MyPrivMsg {
    pub fn new<S: ToString>(channel: S, sender: S, data: S, tags: &[(S, S)]) -> Self {
        Self {
            tags: tags
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            channel: channel.to_string(),
            sender: sender.to_string(),
            data: data.to_string(),
        }
    }
}

impl twitchchat::ToMessage for MyPrivMsg {
    fn tags(&self) -> Option<TagType<'_>> {
        Some(TagType::Map(&self.tags))
    }
    fn prefix(&self) -> Option<&str> {
        Some(self.sender.as_str())
    }
    fn command(&self) -> Option<&str> {
        Some("PRIVMSG")
    }
    fn args(&self) -> Option<ArgsType<'_>> {
        Some(ArgsType::Raw(self.channel.as_str()))
    }
    fn data(&self) -> Option<&str> {
        Some(self.data.as_str())
    }
}

let msg = MyPrivMsg::new(
    "test_channel",
    "museun",
    "hello world",
    &[("color", "#FF4500"), ("display-name", "Museun")],
);
let twitch_msg = twitchchat::Message::parse(msg);
let pm = match twitch_msg {
    twitchchat::Message::PrivMsg(pm) => pm,
    _ => unreachable!(),
};
assert_eq!(pm.user(), "museun");
assert_eq!(pm.channel(), "#test_channel");
assert_eq!(pm.message(), "hello world");
assert_eq!(pm.color().unwrap().kind, twitchchat::TwitchColor::OrangeRed);
```
**/

pub trait ToMessage {
    /// Get the tags portion of the IRC message
    fn tags(&self) -> Option<TagType<'_>>;
    /// Get the prefix portion of the IRC message
    fn prefix(&self) -> Option<&str>;
    /// Get the command portion of the IRC message
    fn command(&self) -> Option<&str>;
    /// Get the args portion of the IRC message
    fn args(&self) -> Option<ArgsType<'_>>;
    /// Get the data portion of the IRC message
    fn data(&self) -> Option<&str>;
}

/// A representation of IRCv3 tags, a raw string or a Vec of Key-Vals
pub enum TagType<'a> {
    /// Raw string
    Raw(&'a str),
    /// List of Key -> Values (owned)
    List(&'a Vec<(String, String)>),
    /// Map of Key -> Values (owned)
    Map(&'a HashMap<String, String>),
}

/// A representation of the args list portion of the IRC message
pub enum ArgsType<'a> {
    /// A raw string
    Raw(&'a str),
    /// A list of parts parsed from the whitespace-separated raw string
    List(&'a Vec<String>),
}

impl ToMessage for IrcMessage {
    fn tags(&self) -> Option<TagType<'_>> {
        match self {
            IrcMessage::Unknown { tags, .. } => Some(TagType::Map(&tags.0)),
            _ => None,
        }
    }
    fn prefix(&self) -> Option<&str> {
        match self {
            IrcMessage::Unknown {
                prefix: Some(crate::irc::Prefix::User { nick, .. }),
                ..
            } => Some(&nick),
            _ => None,
        }
    }
    fn command(&self) -> Option<&str> {
        match self {
            IrcMessage::Unknown { head, .. } => Some(&head),
            _ => None,
        }
    }
    fn args(&self) -> Option<ArgsType<'_>> {
        match self {
            IrcMessage::Unknown { args, .. } => Some(ArgsType::List(&args)),
            _ => None,
        }
    }
    fn data(&self) -> Option<&str> {
        match self {
            IrcMessage::Unknown { tail, .. } => tail.as_ref().map(String::as_str),
            _ => None,
        }
    }
}
