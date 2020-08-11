use crate::{
    messages::*, DispatchError, EventMap, EventStream, FromIrcMessage, IntoOwned, IrcMessage,
};

use async_mutex::Mutex;
use std::sync::Arc;

/// A message dispatcher
#[derive(Clone, Default)]
pub struct AsyncDispatcher {
    map: Arc<Mutex<EventMap>>,
    system: Arc<Mutex<EventMap>>,
}

impl AsyncDispatcher {
    /// Create a new message dispatcher
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribe to the provided message type, giving you an stream (and iterator) over any future messages.
    pub async fn subscribe<T>(&self) -> EventStream<T>
    where
        T: Send + Sync + Clone + 'static,
    {
        self.map.lock().await.register_stream()
    }

    /// Subscrive to a message that cannot be cleared. These should be 'system' events, e.g. 'PING'
    pub async fn subscribe_system<T>(&self) -> EventStream<T>
    where
        T: Send + Sync + Clone + 'static,
    {
        self.system.lock().await.register_stream()
    }

    /// Dispatch this `IrcMessage`
    pub async fn dispatch(&self, message: IrcMessage<'_>) -> Result<(), DispatchError> {
        use IrcMessage as M;

        let mut map = self.map.lock().await;
        let mut system = self.system.lock().await;

        let msg = message.into_owned();
        macro_rules! dispatch {
            ($ty:ty) => {
                Self::dispatch_static::<$ty>(msg, &mut system, &mut map)?
            };
        }

        if Self::has_any_listener::<IrcMessage>(&system, &map) {
            Self::dispatch_static::<IrcMessage>(msg.clone(), &mut system, &mut map)
                .expect("identity conversion should be upheld");
        }

        if Self::has_any_listener::<AllCommands>(&system, &map) {
            Self::dispatch_static::<AllCommands>(msg.clone(), &mut system, &mut map)
                .expect("identity conversion should be upheld");
        }

        match msg.get_command() {
            M::IRC_READY => dispatch!(IrcReady),
            M::READY => dispatch!(Ready),
            M::CAP => dispatch!(Cap),
            M::CLEAR_CHAT => dispatch!(ClearChat),
            M::CLEAR_MSG => dispatch!(ClearMsg),
            M::GLOBAL_USER_STATE => dispatch!(GlobalUserState),
            M::HOST_TARGET => dispatch!(HostTarget),
            M::JOIN => dispatch!(Join),
            M::NOTICE => dispatch!(Notice),
            M::PART => dispatch!(Part),
            M::PING => dispatch!(Ping),
            M::PONG => dispatch!(Pong),
            M::PRIVMSG => dispatch!(Privmsg),
            M::RECONNECT => dispatch!(Reconnect),
            M::ROOM_STATE => dispatch!(RoomState),
            M::USER_NOTICE => dispatch!(UserNotice),
            M::USER_STATE => dispatch!(UserState),
            M::WHISPER => dispatch!(Whisper),
            _ => {
                // TODO user-defined messages
            }
        };

        Ok(())
    }

    /// Reset the dispatcher, this will cause all `EventStreams` previously produce via subscription to eventually return None.
    ///
    /// You'll have to re-subscribe to events after this.
    ///
    /// This is a way to stop any polling event handlers
    pub async fn reset(&self) {
        self.map.lock().await.reset()
    }

    fn has_any_listener<T: 'static>(system: &EventMap, map: &EventMap) -> bool {
        !system.is_empty::<T>() || !map.is_empty::<T>()
    }

    fn dispatch_static<T>(
        message: IrcMessage<'static>,
        system: &mut EventMap,
        map: &mut EventMap,
    ) -> Result<(), DispatchError>
    where
        T: FromIrcMessage<'static>,
        T: Send + Sync + Clone + 'static,
        DispatchError: From<T::Error>,
    {
        let msg = T::from_irc(message)?;

        // only clone if we're actually listening for it
        if !system.is_empty::<T>() {
            system.send(msg.clone());
        }

        if !map.is_empty::<T>() {
            map.send(msg);
        }

        Ok(())
    }
}
