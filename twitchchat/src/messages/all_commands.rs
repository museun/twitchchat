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
        crate::serde::round_trip_json::<AllCommands>(input);
    }

    #[test]
    fn ensure_const_match() {
        let input = ":test!test@test PRIVMSG #museun :this is a test\r\n";
        let msg = IrcMessage::parse(Str::Borrowed(input)).unwrap();
        let all = AllCommands::from_irc(msg).unwrap();
        assert!(matches!(all, AllCommands::Privmsg{..}));
    }
}
