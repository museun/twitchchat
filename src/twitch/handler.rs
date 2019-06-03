use log::*;
use std::sync::Arc;

use crate::twitch::commands::*;
use crate::twitch::Message;

use super::{Token, TokenGen};

#[derive(Default)]
pub(crate) struct Handlers(Vec<(Token, Box<dyn Handler>)>, TokenGen);

impl Handlers {
    pub fn add<H>(&mut self, h: H) -> Token
    where
        H: Handler + 'static + Send + Sync, // probably not needed
    {
        let tok = self.1.next();
        self.0.push((tok, Box::new(h)));
        tok
    }

    pub fn remove(&mut self, token: Token) -> bool {
        if let Some(pos) = self.0.iter().position(|(id, _)| *id == token) {
            let _ = self.0.remove(pos);
            return true;
        }
        false
    }

    // its seeing this after the macro has been expanded.
    #[allow(
        clippy::unknown_clippy_lints,
        clippy::cyclomatic_complexity,
        clippy::cognitive_complexity
    )]
    pub fn handle(&mut self, msg: Message) {
        macro_rules! dispatch {
            ($msg:expr, $func:tt) => {{
                let msg = Arc::new($msg);
                for (id, hn) in self.0.iter_mut() {
                    trace!("dispatching to: {} -> {}", id, stringify!($func));
                    Handler::$func(&mut **hn, Arc::clone(&msg))
                }
            }};
        }

        match msg.clone() {
            Message::Irc(msg) => dispatch!(msg, on_irc_message),
            Message::Join(msg) => dispatch!(msg, on_join),
            Message::Part(msg) => dispatch!(msg, on_part),
            Message::PrivMsg(msg) => dispatch!(msg, on_priv_msg),
            Message::Mode(msg) => dispatch!(msg, on_mode),
            Message::NamesStart(msg) => dispatch!(msg, on_names_start),
            Message::NamesEnd(msg) => dispatch!(msg, on_names_end),
            Message::ClearChat(msg) => dispatch!(msg, on_clear_chat),
            Message::ClearMsg(msg) => dispatch!(msg, on_clear_msg),
            Message::HostTargetStart(msg) => dispatch!(msg, on_host_target_start),
            Message::HostTargetEnd(msg) => dispatch!(msg, on_host_target_end),
            Message::Notice(msg) => dispatch!(msg, on_notice),
            Message::Reconnect(msg) => dispatch!(msg, on_reconnect),
            Message::RoomState(msg) => dispatch!(msg, on_room_state),
            Message::UserNotice(msg) => dispatch!(msg, on_user_notice),
            Message::UserState(msg) => dispatch!(msg, on_user_state),
            Message::GlobalUserState(msg) => dispatch!(msg, on_global_user_state),
            Message::__Nonexhaustive => unreachable!(),
        };

        dispatch!(msg, on_message)
    }
}
/// Handler allows you to implement message filtering with a struct
#[allow(unused_variables)]
pub trait Handler {
    /// Called when a [`Message`](./enum.Message.html) message is received
    #[inline]
    fn on_message(&mut self, msg: Arc<Message>) {
        debug!("on_message: {:?}", msg)
    }
    /// Called when a [`irc::types::Message`](./irc/types/enum.Message.html) message is received
    #[inline]
    fn on_irc_message(&mut self, msg: Arc<crate::irc::types::Message>) {
        debug!("on_irc_message: {:?}", msg)
    }
    /// Called when a [`Join`](./commands/struct.Join.html) message is received
    #[inline]
    fn on_join(&mut self, msg: Arc<Join>) {
        debug!("on_join: {:?}", msg)
    }
    /// Called when a [`Part`](./commands/struct.Part.html) message is received
    #[inline]
    fn on_part(&mut self, msg: Arc<Part>) {
        debug!("on_part: {:?}", msg)
    }
    /// Called when a [`PrivMsg`](./commands/struct.PrivMsg.html) message is received
    #[inline]
    fn on_priv_msg(&mut self, msg: Arc<PrivMsg>) {
        debug!("on_priv_msg: {:?}", msg)
    }
    /// Called when a [`Mode`](./commands/struct.Mode.html) message is received
    #[inline]
    fn on_mode(&mut self, msg: Arc<Mode>) {
        debug!("on_mode: {:?}", msg)
    }
    /// Called when a [`NamesStart`](./commands/struct.NamesStart.html) message is received
    #[inline]
    fn on_names_start(&mut self, msg: Arc<NamesStart>) {
        debug!("on_names_start: {:?}", msg)
    }
    /// Called when a [`NamesEnd`](./commands/struct.NamesEnd.html) message is received
    #[inline]
    fn on_names_end(&mut self, msg: Arc<NamesEnd>) {
        debug!("on_names_end: {:?}", msg)
    }
    /// Called when a [`ClearChat`](./commands/struct.ClearChat.html) message is received
    #[inline]
    fn on_clear_chat(&mut self, msg: Arc<ClearChat>) {
        debug!("on_clear_chat: {:?}", msg)
    }
    /// Called when a [`ClearMsg`](./commands/struct.ClearMsg.html) message is received
    #[inline]
    fn on_clear_msg(&mut self, msg: Arc<ClearMsg>) {
        debug!("on_clear_msg: {:?}", msg)
    }
    /// Called when a [`HostTargetStart`](./commands/struct.HostTargetStart.html) message is received
    #[inline]
    fn on_host_target_start(&mut self, msg: Arc<HostTargetStart>) {
        debug!("on_host_target_start: {:?}", msg)
    }
    /// Called when a [`HostTargetEnd`](./commands/struct.HostTargetEnd.html) message is received
    #[inline]
    fn on_host_target_end(&mut self, msg: Arc<HostTargetEnd>) {
        debug!("on_host_target_end: {:?}", msg)
    }
    /// Called when a [`Notice`](./commands/struct.Notice.html) message is received
    #[inline]
    fn on_notice(&mut self, msg: Arc<Notice>) {
        debug!("on_notice: {:?}", msg)
    }
    /// Called when a [`Reconnect`](./commands/struct.Reconnect.html) message is received
    #[inline]
    fn on_reconnect(&mut self, msg: Arc<Reconnect>) {
        debug!("on_reconnect: {:?}", msg)
    }
    /// Called when a [`RoomState`](./commands/struct.RoomState.html) message is received
    #[inline]
    fn on_room_state(&mut self, msg: Arc<RoomState>) {
        debug!("on_room_state: {:?}", msg)
    }
    /// Called when a [`UserNotice`](./commands/struct.UserNotice.html) message is received
    #[inline]
    fn on_user_notice(&mut self, msg: Arc<UserNotice>) {
        debug!("on_user_notice: {:?}", msg)
    }
    /// Called when a [`UserState`](./commands/struct.UserState.html) message is received
    #[inline]
    fn on_user_state(&mut self, msg: Arc<UserState>) {
        debug!("on_user_state: {:?}", msg)
    }
    /// Called when a [`GlobalUserState`](./commands/struct.GlobalUserState.html) message is received
    #[inline]
    fn on_global_user_state(&mut self, msg: Arc<GlobalUserState>) {
        debug!("on_global_user_state: {:?}", msg)
    }
}
