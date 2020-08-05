use super::*;
use crate::ng::{FromIrcMessage, InvalidMessage};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub enum AllCommands<'a> {
    /// An raw event occured
    Raw(IrcMessage<'a>),
    /// A capabilities event occured
    IrcReady(IrcReady<'a>),
    /// A ClearChat event occured
    Ready(Ready<'a>),
    /// A ClearMsg event occured
    Cap(Cap<'a>),
    /// A GlobalUserState event occured
    ClearChat(ClearChat<'a>),
    /// A HostTarget event occured
    ClearMsg(ClearMsg<'a>),
    /// A IrcReady event occured
    GlobalUserState(GlobalUserState<'a>),
    /// A Join event occured
    HostTarget(HostTarget<'a>),
    /// A Notice event occured
    Join(Join<'a>),
    /// A Part event occured
    Notice(Notice<'a>),
    /// A Ping event occured
    Part(Part<'a>),
    /// A Pong event occured
    Ping(Ping<'a>),
    /// A Privmsg event occured
    Pong(Pong<'a>),
    /// A Ready event occured
    Privmsg(Privmsg<'a>),
    /// A Reconnect event occured
    Reconnect(Reconnect<'a>),
    /// A RoomState event occured
    RoomState(RoomState<'a>),
    /// A UserNotice event occured
    UserNotice(UserNotice<'a>),
    /// A UserState event occured
    UserState(UserState<'a>),
    /// A Whisper event occured
    Whisper(Whisper<'a>),
}

impl<'a> FromIrcMessage<'a> for AllCommands<'a> {
    type Error = InvalidMessage;

    fn from_irc(msg: IrcMessage<'a>) -> Result<Self, Self::Error> {
        macro_rules! map {
            ($ident:ident) => {
                Self::$ident($ident::from_irc(msg)?)
            };
        }

        use IrcMessage as M;
        let this = match msg.get_command() {
            M::IRCREADY => map!(IrcReady),
            M::READY => map!(Ready),
            M::CAP => map!(Cap),
            M::CLEARCHAT => map!(ClearChat),
            M::CLEARMSG => map!(ClearMsg),
            M::GLOBALUSERSTATE => map!(GlobalUserState),
            M::HOSTTARGET => map!(HostTarget),
            M::JOIN => map!(Join),
            M::NOTICE => map!(Notice),
            M::PART => map!(Part),
            M::PING => map!(Ping),
            M::PONG => map!(Pong),
            M::PRIVMSG => map!(Privmsg),
            M::RECONNECT => map!(Reconnect),
            M::ROOMSTATE => map!(RoomState),
            M::USERNOTICE => map!(UserNotice),
            M::USERSTATE => map!(UserState),
            M::WHISPER => map!(Whisper),
            _ => Self::Raw(IrcMessage::from_irc(msg).expect("infallible conversion")),
        };

        Ok(this)
    }
}

macro_rules! from_other {
    ($($ident:tt)*) => {
        $(
            impl<'t> From<$ident<'t>> for AllCommands<'t> {
                fn from(msg: $ident<'t>) -> Self {
                    Self::$ident(msg)
                }
            }
        )*
    };
}

type Raw<'t> = IrcMessage<'t>;

from_other! {
    Raw
    IrcReady
    Ready
    Cap
    ClearChat
    ClearMsg
    GlobalUserState
    HostTarget
    Join
    Notice
    Part
    Ping
    Pong
    Privmsg
    Reconnect
    RoomState
    UserNotice
    UserState
    Whisper
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "serde")]
    fn all_commands_serde() {
        let input = ":test!test@test PRIVMSG #museun :this is a test\r\n";
        crate::ng::serde::round_trip_json::<AllCommands>(input);
    }

    #[test]
    fn ensure_const_match() {
        let input = ":test!test@test PRIVMSG #museun :this is a test\r\n";
        let msg = IrcMessage::parse(Str::Borrowed(input));
        let all = AllCommands::from_irc(msg).unwrap();
        assert!(matches!(all, AllCommands::Privmsg{..}));
    }
}
