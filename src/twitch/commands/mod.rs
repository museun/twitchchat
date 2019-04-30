mod clearchat;
mod clearmsg;
mod globaluserstate;
mod hosttargetend;
mod hosttargetstart;
mod join;
mod mode;
mod namesend;
mod namesstart;
mod notice;
mod part;
mod privmsg;
mod reconnect;
mod roomstate;
mod usernotice;
mod userstate;

pub use self::clearchat::ClearChat;
pub use self::clearmsg::ClearMsg;
pub use self::globaluserstate::GlobalUserState;
pub use self::hosttargetend::HostTargetEnd;
pub use self::hosttargetstart::HostTargetStart;
pub use self::join::Join;
pub use self::mode::{Mode, ModeStatus};
pub use self::namesend::NamesEnd;
pub use self::namesstart::NamesStart;
pub use self::notice::{MessageId, Notice};
pub use self::part::Part;
pub use self::privmsg::PrivMsg;
pub use self::reconnect::Reconnect;
pub use self::roomstate::{FollowersOnly, RoomState};
pub use self::usernotice::{NoticeType, SubPlan, UserNotice};
pub use self::userstate::UserState;

use crate::irc::types::*;
use crate::twitch::{Badge, BadgeInfo, Channel, Color, Emotes, RGB};
use crate::Tags;

/// Tag allows access to the Tags part of the Message
pub trait Tag {
    /// Gets the `key` from the mapping, returning the value if found
    fn get(&self, key: &str) -> Option<&str>;
}

impl Tag for Tags {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(AsRef::as_ref)
    }
}

impl<T> Tagged for T where T: Tag {}

/// Tagged provides some helpers for getting tagged data out as reasonable types
pub trait Tagged: Tag {
    /// Tries to get the value for `key` as a bool, defaults to false if not found
    fn get_as_bool(&self, key: &str) -> bool {
        self.get(key).map(|k| k == "1").unwrap_or_else(|| false)
    }

    /// Tries to parse (with [FromStr::from_str](https://doc.rust-lang.org/std/str/trait.FromStr.html#tymethod.from_str)) the value at `key`
    fn get_parsed<E>(&self, key: &str) -> Option<E>
    where
        E: std::str::FromStr,
    {
        self.get(key)?.parse().ok()
    }
}

fn badges(s: &str) -> Vec<Badge> {
    s.split(',').filter_map(Badge::parse).collect()
}

fn emotes(s: &str) -> Vec<Emotes> {
    Emotes::parse(s).collect()
}

/// Parses a Twitch commands from an IRC Message
pub(crate) fn parse(msg: &Message) -> Option<super::Message> {
    use super::Message;

    struct Rev(Vec<String>);
    impl Rev {
        pub fn next(&mut self) -> Option<String> {
            if self.0.is_empty() {
                return None;
            }
            Some(self.0.remove(0))
        }
    }

    macro_rules! get_user {
        ($prefix:expr) => {
            match $prefix.unwrap() {
                crate::irc::types::Prefix::User { nick, .. } => nick,
                _ => unreachable!(),
            }
        };
    }

    if let crate::irc::types::Message::Unknown {
        prefix,
        tags,
        head,
        args,
        tail,
    } = msg.clone()
    {
        let mut args = Rev(args);
        let cmd = match head.as_str() {
            "JOIN" => Message::Join(Join {
                user: get_user!(prefix),
                channel: args.next()?.into(),
            }),
            "PART" => Message::Part(Part {
                user: get_user!(prefix),
                channel: args.next()?.into(),
            }),
            "PRIVMSG" => {
                let body = tail?;
                let (message, action) = if body.starts_with('\x01') {
                    (body[8..body.len() - 1].to_string(), true)
                } else {
                    (body, false)
                };

                Message::PrivMsg(PrivMsg {
                    user: get_user!(prefix),
                    tags,
                    channel: args.next()?.into(),
                    message,
                    action,
                })
            }
            "353" => {
                let user = args.next()?;
                let _ = args.next()?; // out here to ignore the
                Message::NamesStart(NamesStart {
                    user,
                    channel: args.next()?.into(),
                    users: tail?.split(' ').map(str::to_string).collect(),
                })
            }
            "366" => Message::NamesEnd(NamesEnd {
                user: args.next()?,
                channel: args.next()?.into(),
            }),
            "MODE" => Message::Mode(Mode {
                channel: args.next()?.into(),
                status: match args.next()?.as_str() {
                    "+o" => ModeStatus::Gained,
                    "-o" => ModeStatus::Lost,
                    _ => unreachable!(),
                },
                user: args.next()?,
            }),
            "CLEARCHAT" => Message::ClearChat(ClearChat {
                tags,
                channel: args.next()?.into(),
                user: tail,
            }),
            "CLEARMSG" => Message::ClearMsg(ClearMsg {
                tags,
                channel: args.next()?.into(),
                message: tail,
            }),
            "HOSTTARGET" => {
                let source = args.next()?;
                match args.next() {
                    Some(target) => Message::HostTargetStart(HostTargetStart {
                        source,
                        target,
                        viewers: args.next().and_then(|s| s.parse().ok()),
                    }),
                    None => Message::HostTargetEnd(HostTargetEnd {
                        source,
                        viewers: tail
                            .and_then(|s| s.split(' ').nth(1).map(str::to_string))
                            .and_then(|s| s.parse().ok()),
                    }),
                }
            }
            "NOTICE" => Message::Notice(Notice {
                tags,
                channel: args.next()?.into(),
                message: tail?,
            }),
            "RECONNECT" => Message::Reconnect(Reconnect),
            "ROOMSTATE" => Message::RoomState(RoomState {
                tags,
                channel: args.next()?.into(),
            }),
            "USERNOTICE" => Message::UserNotice(UserNotice {
                tags,
                channel: args.next()?.into(),
                message: tail,
            }),
            "USERSTATE" => Message::UserState(UserState {
                tags,
                channel: args.next()?.into(),
            }),
            "GLOBALUSERSTATE" => Message::GlobalUserState(GlobalUserState { tags }),
            _ => return None,
        };
        return Some(cmd);
    }

    None
}

#[cfg(test)]
mod tests;
