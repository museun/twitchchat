use std::collections::HashSet;

use crate::{
    decoder::DecodeError,
    identity::{Identity, YourCapabilities},
    irc::IrcMessage,
    twitch::{Capability, UserConfig},
    util::is_blocking_error,
};

/// An event to wait for
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[non_exhaustive]
pub enum Event {
    /// Raw event
    Raw,
    /// IrcReady event
    IrcReady,
    /// Ready event
    Ready,
    /// Capabilities event
    Cap,
    /// ClearChat event
    ClearChat,
    /// ClearMsg event
    ClearMsg,
    /// GlobalUserState event
    GlobalUserState,
    /// HostTarget event
    HostTarget,
    /// Join event
    Join,
    /// Notice event
    Notice,
    /// Part event
    Part,
    /// Ping event
    Ping,
    /// Pong event
    Pong,
    /// Privmsg event
    Privmsg,
    /// Reconnect event
    Reconnect,
    /// RoomState event
    RoomState,
    /// UserNotice event
    UserNotice,
    /// UserState event
    UserState,
    /// Whisper event
    Whisper,
}

impl Event {
    pub(crate) fn from_raw(s: &str) -> Self {
        use IrcMessage as M;
        match s {
            M::IRC_READY => Self::IrcReady,
            M::READY => Self::Ready,
            M::CAP => Self::Cap,
            M::CLEAR_CHAT => Self::ClearChat,
            M::CLEAR_MSG => Self::ClearMsg,
            M::GLOBAL_USER_STATE => Self::GlobalUserState,
            M::HOST_TARGET => Self::HostTarget,
            M::JOIN => Self::Join,
            M::NOTICE => Self::Notice,
            M::PART => Self::Part,
            M::PING => Self::Ping,
            M::PONG => Self::Pong,
            M::PRIVMSG => Self::Privmsg,
            M::RECONNECT => Self::Reconnect,
            M::ROOM_STATE => Self::RoomState,
            M::USER_NOTICE => Self::UserNotice,
            M::USER_STATE => Self::UserState,
            M::WHISPER => Self::Whisper,
            _ => Self::Raw,
        }
    }
}

pub(crate) enum State<'a> {
    Done(IrcMessage<'a>),
    Requeue(IrcMessage<'a>),
    Yield,
}

pub(crate) fn wait_inner(
    result: Result<IrcMessage<'_>, DecodeError>,
    event: Event,
) -> Result<State<'_>, DecodeError> {
    let msg = match result {
        Err(DecodeError::Io(err)) if is_blocking_error(&err) => return Ok(State::Yield),
        Err(err) => return Err(err),
        Ok(msg) => msg,
    };

    if Event::from_raw(msg.get_command()) == event {
        Ok(State::Done(msg))
    } else {
        Ok(State::Requeue(msg))
    }
}

pub(crate) fn check_message(
    msg: &IrcMessage<'_>,
    state: &mut ReadyState,
) -> Result<StepState, CheckError> {
    const FAILURE: &str = "Login authentication failed";

    use crate::{
        messages::Capability as TwitchCap, //
        messages::Commands as C,
        FromIrcMessage as _,
    };

    match C::from_irc(msg.clone()).unwrap() {
        C::Notice(msg) if msg.message() == FAILURE => Err(CheckError::BadPass),

        C::Ready(msg) => {
            state.our_name.replace(msg.username().to_string());

            // if we aren't going to be receiving tags, then we
            // won't be looking for any more messages

            // if we're anonymous, we won't get GLOBALUSERSTATE even
            // if we do send Tags
            if state.is_anonymous {
                return Ok(StepState::Identity(Identity::Anonymous {
                    caps: std::mem::take(&mut state.caps),
                }));
            }

            // if we're not looking for any more caps and we won't be
            // getting a GlobalUserState just give them the Basic Identity
            if state.looking_for.is_empty() && !state.will_be_getting_global_user_state_hopefully {
                return Ok(StepState::Identity(Identity::Basic {
                    name: state.our_name.take().unwrap(),
                    caps: std::mem::take(&mut state.caps),
                }));
            }

            Ok(StepState::Continue)
        }

        C::Cap(msg) => {
            match msg.capability() {
                TwitchCap::Acknowledged(name) => {
                    use crate::twitch::Capability as Cap;

                    let cap = match name.parse() {
                        Ok(cap) => cap,
                        // Twitch sent us an unknown capability
                        Err(..) => {
                            state.caps.unknown.insert(name.to_string());
                            return Ok(StepState::Continue);
                        }
                    };

                    *match cap {
                        Cap::Tags => &mut state.caps.tags,
                        Cap::Membership => &mut state.caps.membership,
                        Cap::Commands => &mut state.caps.commands,
                    } = true;

                    state.looking_for.remove(&cap);
                }

                TwitchCap::NotAcknowledged(name) => {
                    return Err(CheckError::InvalidCap(name.to_string()))
                }
            }

            Ok(StepState::Continue)
        }

        // NOTE: This will only be sent when there's both Commands and atleast
        // one other CAP requested
        C::GlobalUserState(msg) => {
            // TODO: this is so shitty.
            let id = match msg.user_id {
                Some(id) => id.parse().unwrap(),
                // XXX: we can get this message without any tags
                None => {
                    return Ok(StepState::Identity(Identity::Basic {
                        name: state.our_name.take().unwrap(),
                        caps: std::mem::take(&mut state.caps),
                    }))
                }
            };

            Ok(StepState::Identity(Identity::Full {
                // these unwraps should be safe because we'll have all of the TAGs here
                name: state.our_name.take().unwrap(),
                user_id: id,
                display_name: msg.display_name.map(|s| s.to_string()),
                color: msg.color,
                caps: std::mem::take(&mut state.caps),
            }))
        }

        // Reply to any PINGs while waiting. Although Twitch doesn't
        // currently send a PING for spoof detection on initial
        // handshake, one day they may. Most IRC servers do this
        // already
        C::Ping(msg) => Ok(StepState::ShouldPong(msg.token().to_string())),

        // Skip this message because its a synthetic response
        C::Pong(_) => Ok(StepState::Skip),

        C::Reconnect(_) => Err(CheckError::ShouldReconnect),

        // we have our name, but we won't be getting GlobalUserState and we've
        // got all of our Caps
        _ if state.our_name.is_some()
            && !state.will_be_getting_global_user_state_hopefully
            && state.looking_for.is_empty() =>
        {
            Ok(StepState::Identity(Identity::Basic {
                name: state.our_name.take().unwrap(),
                caps: std::mem::take(&mut state.caps),
            }))
        }

        _ => Ok(StepState::Continue),
    }
}

#[derive(Debug)]
pub(crate) enum CheckError {
    InvalidCap(String),
    BadPass,
    ShouldReconnect,
}

pub(crate) enum StepState {
    Skip,
    Continue,
    ShouldPong(String),
    Identity(Identity),
}

pub(crate) struct ReadyState {
    pub(crate) looking_for: HashSet<Capability>,
    pub(crate) caps: YourCapabilities,
    pub(crate) our_name: Option<String>,
    pub(crate) is_anonymous: bool,
    pub(crate) will_be_getting_global_user_state_hopefully: bool,
}

impl ReadyState {
    pub(crate) fn new(user_config: &UserConfig) -> Self {
        let is_anonymous = user_config.is_anonymous();

        let will_be_getting_global_user_state_hopefully =
            user_config.capabilities.contains(&Capability::Tags)
                && user_config.capabilities.contains(&Capability::Commands);

        Self {
            looking_for: user_config.capabilities.iter().copied().collect(),
            caps: YourCapabilities::default(),
            our_name: None,
            is_anonymous,
            will_be_getting_global_user_state_hopefully,
        }
    }
}

// TODO record a bunch of twitch handshakes and test them here
