use {super::*, crate::*};

/// An enum of all possible Twitch messages.
///
/// This is useful if you just want to subscribe to ***all** messages.
#[non_exhaustive]
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

impl<'a> AllCommands<'a> {
    /// Get the raw string out of this
    pub fn raw(&'a self) -> &'a str {
        match self {
            Self::Raw(msg) => msg.get_raw(),
            Self::IrcReady(msg) => msg.raw(),
            Self::Ready(msg) => msg.raw(),
            Self::Cap(msg) => msg.raw(),
            Self::ClearChat(msg) => msg.raw(),
            Self::ClearMsg(msg) => msg.raw(),
            Self::GlobalUserState(msg) => msg.raw(),
            Self::HostTarget(msg) => msg.raw(),
            Self::Join(msg) => msg.raw(),
            Self::Notice(msg) => msg.raw(),
            Self::Part(msg) => msg.raw(),
            Self::Ping(msg) => msg.raw(),
            Self::Pong(msg) => msg.raw(),
            Self::Privmsg(msg) => msg.raw(),
            Self::Reconnect(msg) => msg.raw(),
            Self::RoomState(msg) => msg.raw(),
            Self::UserNotice(msg) => msg.raw(),
            Self::UserState(msg) => msg.raw(),
            Self::Whisper(msg) => msg.raw(),
        }
    }
}

impl<'a> IntoOwned<'a> for AllCommands<'a> {
    type Output = AllCommands<'static>;

    fn into_owned(self) -> Self::Output {
        match self {
            Self::Raw(s) => AllCommands::Raw(s.into_owned()),
            Self::IrcReady(s) => AllCommands::IrcReady(s.into_owned()),
            Self::Ready(s) => AllCommands::Ready(s.into_owned()),
            Self::Cap(s) => AllCommands::Cap(s.into_owned()),
            Self::ClearChat(s) => AllCommands::ClearChat(s.into_owned()),
            Self::ClearMsg(s) => AllCommands::ClearMsg(s.into_owned()),
            Self::GlobalUserState(s) => AllCommands::GlobalUserState(s.into_owned()),
            Self::HostTarget(s) => AllCommands::HostTarget(s.into_owned()),
            Self::Join(s) => AllCommands::Join(s.into_owned()),
            Self::Notice(s) => AllCommands::Notice(s.into_owned()),
            Self::Part(s) => AllCommands::Part(s.into_owned()),
            Self::Ping(s) => AllCommands::Ping(s.into_owned()),
            Self::Pong(s) => AllCommands::Pong(s.into_owned()),
            Self::Privmsg(s) => AllCommands::Privmsg(s.into_owned()),
            Self::Reconnect(s) => AllCommands::Reconnect(s.into_owned()),
            Self::RoomState(s) => AllCommands::RoomState(s.into_owned()),
            Self::UserNotice(s) => AllCommands::UserNotice(s.into_owned()),
            Self::UserState(s) => AllCommands::UserState(s.into_owned()),
            Self::Whisper(s) => AllCommands::Whisper(s.into_owned()),
        }
    }
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
            M::IRC_READY => map!(IrcReady),
            M::READY => map!(Ready),
            M::CAP => map!(Cap),
            M::CLEAR_CHAT => map!(ClearChat),
            M::CLEAR_MSG => map!(ClearMsg),
            M::GLOBAL_USER_STATE => map!(GlobalUserState),
            M::HOST_TARGET => map!(HostTarget),
            M::JOIN => map!(Join),
            M::NOTICE => map!(Notice),
            M::PART => map!(Part),
            M::PING => map!(Ping),
            M::PONG => map!(Pong),
            M::PRIVMSG => map!(Privmsg),
            M::RECONNECT => map!(Reconnect),
            M::ROOM_STATE => map!(RoomState),
            M::USER_NOTICE => map!(UserNotice),
            M::USER_STATE => map!(UserState),
            M::WHISPER => map!(Whisper),
            _ => Self::Raw(IrcMessage::from_irc(msg).expect("infallible conversion")),
        };

        Ok(this)
    }

    /// Consumes this wrapper and returns the raw `Str<'a>`
    fn into_inner(self) -> Str<'a> {
        match self {
            Self::Raw(msg) => msg.into_inner(),
            Self::IrcReady(msg) => msg.into_inner(),
            Self::Ready(msg) => msg.into_inner(),
            Self::Cap(msg) => msg.into_inner(),
            Self::ClearChat(msg) => msg.into_inner(),
            Self::ClearMsg(msg) => msg.into_inner(),
            Self::GlobalUserState(msg) => msg.into_inner(),
            Self::HostTarget(msg) => msg.into_inner(),
            Self::Join(msg) => msg.into_inner(),
            Self::Notice(msg) => msg.into_inner(),
            Self::Part(msg) => msg.into_inner(),
            Self::Ping(msg) => msg.into_inner(),
            Self::Pong(msg) => msg.into_inner(),
            Self::Privmsg(msg) => msg.into_inner(),
            Self::Reconnect(msg) => msg.into_inner(),
            Self::RoomState(msg) => msg.into_inner(),
            Self::UserNotice(msg) => msg.into_inner(),
            Self::UserState(msg) => msg.into_inner(),
            Self::Whisper(msg) => msg.into_inner(),
        }
    }
}

macro_rules! from_other {
    ($($ident:tt)*) => {
        $(impl<'a> From<$ident<'a>> for AllCommands<'a> {
            fn from(msg: $ident<'a>) -> Self {
                Self::$ident(msg)
            }
        })*
    };
}

type Raw<'a> = IrcMessage<'a>;

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
        crate::serde::round_trip_json::<AllCommands>(input);
        crate::serde::round_trip_rmp::<AllCommands>(input);
    }

    #[test]
    fn ensure_const_match() {
        let input = ":test!test@test PRIVMSG #museun :this is a test\r\n";
        let msg = IrcMessage::parse(Str::Borrowed(input)).unwrap();
        let all = AllCommands::from_irc(msg).unwrap();
        assert!(matches!(all, AllCommands::Privmsg{..}));
    }
}
