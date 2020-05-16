use super::*;

/// This is a collection of all possible message types
///
/// Subscribing to [events::All][all] will produce this, so you can have a single stream for multiple messages.
///
/// [all]: ../events/struct.All.html
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AllCommands<'t> {
    /// An unknown event occured
    Unknown(Raw<'t>),
    /// A capabilities event occured
    Cap(Cap<'t>),
    /// A ClearChat event occured
    ClearChat(ClearChat<'t>),
    /// A ClearMsg event occured
    ClearMsg(ClearMsg<'t>),
    /// A GlobalUserState event occured
    GlobalUserState(GlobalUserState<'t>),
    /// A HostTarget event occured
    HostTarget(HostTarget<'t>),
    /// A IrcReady event occured
    IrcReady(IrcReady<'t>),
    /// A Join event occured
    Join(Join<'t>),
    /// A Mode event occured
    Mode(Mode<'t>),
    /// A Names event occured
    Names(Names<'t>),
    /// A Notice event occured
    Notice(Notice<'t>),
    /// A Part event occured
    Part(Part<'t>),
    /// A Ping event occured
    Ping(Ping<'t>),
    /// A Pong event occured
    Pong(Pong<'t>),
    /// A Privmsg event occured
    Privmsg(Privmsg<'t>),
    /// A Ready event occured
    Ready(Ready<'t>),
    /// A Reconnect event occured
    Reconnect(Reconnect),
    /// A RoomState event occured
    RoomState(RoomState<'t>),
    /// A UserNotice event occured
    UserNotice(UserNotice<'t>),
    /// A UserState event occured
    UserState(UserState<'t>),
}

impl<'a: 't, 't> Parse<&'a Message<'t>> for AllCommands<'t> {
    fn parse(msg: &'a Message<'t>) -> Result<Self, InvalidMessage> {
        let out = match &*msg.command {
            "001" => IrcReady::parse(msg)?.into(),
            "353" => Names::parse(msg)?.into(),
            "366" => Names::parse(msg)?.into(),
            "376" => Ready::parse(msg)?.into(),
            "CAP" => Cap::parse(msg)?.into(),
            "CLEARCHAT" => ClearChat::parse(msg)?.into(),
            "CLEARMSG" => ClearMsg::parse(msg)?.into(),
            "GLOBALUSERSTATE" => GlobalUserState::parse(msg)?.into(),
            "HOSTARGET" => HostTarget::parse(msg)?.into(),
            "JOIN" => Join::parse(msg)?.into(),
            "MODE" => Mode::parse(msg)?.into(),
            "NOTICE" => Notice::parse(msg)?.into(),
            "PART" => Part::parse(msg)?.into(),
            "PING" => Ping::parse(msg)?.into(),
            "PONG" => Pong::parse(msg)?.into(),
            "PRIVMSG" => Privmsg::parse(msg)?.into(),
            "RECONNECT" => Reconnect::parse(msg)?.into(),
            "ROOMSTATE" => RoomState::parse(msg)?.into(),
            "USERNOTICE" => UserNotice::parse(msg)?.into(),
            "USERSTATE" => UserState::parse(msg)?.into(),
            _ => msg.clone().into(),
        };
        Ok(out)
    }
}

impl<'t> AsOwned for AllCommands<'t> {
    type Owned = AllCommands<'static>;
    fn as_owned(&self) -> Self::Owned {
        match self {
            AllCommands::Unknown(inner) => AllCommands::Unknown(inner.as_owned()),
            AllCommands::Cap(inner) => AllCommands::Cap(inner.as_owned()),
            AllCommands::ClearChat(inner) => AllCommands::ClearChat(inner.as_owned()),
            AllCommands::ClearMsg(inner) => AllCommands::ClearMsg(inner.as_owned()),
            AllCommands::GlobalUserState(inner) => AllCommands::GlobalUserState(inner.as_owned()),
            AllCommands::HostTarget(inner) => AllCommands::HostTarget(inner.as_owned()),
            AllCommands::IrcReady(inner) => AllCommands::IrcReady(inner.as_owned()),
            AllCommands::Join(inner) => AllCommands::Join(inner.as_owned()),
            AllCommands::Mode(inner) => AllCommands::Mode(inner.as_owned()),
            AllCommands::Names(inner) => AllCommands::Names(inner.as_owned()),
            AllCommands::Notice(inner) => AllCommands::Notice(inner.as_owned()),
            AllCommands::Part(inner) => AllCommands::Part(inner.as_owned()),
            AllCommands::Ping(inner) => AllCommands::Ping(inner.as_owned()),
            AllCommands::Pong(inner) => AllCommands::Pong(inner.as_owned()),
            AllCommands::Privmsg(inner) => AllCommands::Privmsg(inner.as_owned()),
            AllCommands::Ready(inner) => AllCommands::Ready(inner.as_owned()),
            AllCommands::Reconnect(inner) => AllCommands::Reconnect(inner.as_owned()),
            AllCommands::RoomState(inner) => AllCommands::RoomState(inner.as_owned()),
            AllCommands::UserNotice(inner) => AllCommands::UserNotice(inner.as_owned()),
            AllCommands::UserState(inner) => AllCommands::UserState(inner.as_owned()),
        }
    }
}

// manual impls because they are different
impl<'t> From<Raw<'t>> for AllCommands<'t> {
    fn from(msg: Raw<'t>) -> Self {
        Self::Unknown(msg)
    }
}

impl<'t> From<Reconnect> for AllCommands<'t> {
    fn from(msg: Reconnect) -> Self {
        Self::Reconnect(msg)
    }
}

macro_rules! from {
    ($($ident:tt),* $(,)?) => {
        $(
            impl<'t> From<$ident<'t>> for AllCommands<'t> {
                fn from(msg: $ident<'t>) -> Self {
                    Self::$ident(msg)
                }
            }
        )*
    };
}

// rote implementation
from! {
    Cap,
    ClearChat,
    ClearMsg,
    GlobalUserState,
    HostTarget,
    IrcReady,
    Join,
    Mode,
    Names,
    Notice,
    Part,
    Ping,
    Pong,
    Privmsg,
    Ready,
    RoomState,
    UserNotice,
    UserState,
}
