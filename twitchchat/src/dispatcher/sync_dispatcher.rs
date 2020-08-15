use crate::{messages::*, DispatchError, EventMap, FromIrcMessage, IntoOwned, IrcMessage};
use simple_event_map::EventIter;
use std::sync::Arc;

/// A message dispatcher
#[derive(Clone, Default)]
pub struct SyncDispatcher {
    map: Arc<std::sync::Mutex<EventMap>>,
    system: Arc<std::sync::Mutex<EventMap>>,
}

impl std::fmt::Debug for SyncDispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncDispatcher").finish()
    }
}

impl SyncDispatcher {
    /// Create a new message dispatcher
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribe to the provided message type, giving you an stream (and iterator) over any future messages.
    pub fn subscribe<T>(&mut self) -> EventIter<T>
    where
        T: Send + Sync + Clone + 'static,
        T: FromIrcMessage<'static>,
        DispatchError: From<T::Error>,
    {
        self.map.lock().unwrap().register_iter()
    }

    /// Subscrive to a message that cannot be cleared. These should be 'system' events, e.g. 'PING'
    pub fn subscribe_system<T>(&mut self) -> EventIter<T>
    where
        T: Send + Sync + Clone + 'static,
        T: FromIrcMessage<'static>,
        DispatchError: From<T::Error>,
    {
        self.system.lock().unwrap().register_iter()
    }

    /// Dispatch this `IrcMessage`
    pub fn dispatch(&mut self, message: IrcMessage<'_>) -> Result<(), DispatchError> {
        use IrcMessage as M;

        let mut map = self.map.lock().unwrap();
        let mut system = self.system.lock().unwrap();

        macro_rules! dispatch {
            ($ty:ty) => {
                return Self::dispatch_static::<$ty, _>(message, &mut system, &mut map);
            };
        }

        // we should always dispatch these 2
        Self::dispatch_static::<AllCommands, _>(&message, &mut system, &mut map)?;
        Self::dispatch_static::<IrcMessage, _>(&message, &mut system, &mut map)?;

        // and then conditionally dispatch these
        match message.get_command() {
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
            _ => {}
        };

        Ok(())
    }

    /// Reset the dispatcher, this will cause all `EventStreams` previously produce via subscription to eventually return None.
    ///
    /// You'll have to re-subscribe to events after this.
    ///
    /// This is a way to stop any polling event handlers
    pub fn reset(&mut self) {
        self.map.lock().unwrap().reset()
    }

    fn dispatch_static<'a, T, O>(
        message: O,
        system: &mut EventMap,
        map: &mut EventMap,
    ) -> Result<(), DispatchError>
    where
        O: IntoOwned<'a, Output = IrcMessage<'static>>,
        T: FromIrcMessage<'static>,
        T: Send + Sync + Clone + 'static,
        DispatchError: From<T::Error>,
    {
        let (want_system, want_map) = (!system.is_empty::<T>(), !map.is_empty::<T>());
        // no one is listening, so don't even parse it
        if !want_system && !want_map {
            return Ok(());
        }

        let message = message.into_owned();
        let msg = T::from_irc(message)?;

        match (want_system, want_map) {
            (true, true) => {
                system.send(msg.clone());
                map.send(msg);
            }
            (true, false) => {
                system.send(msg);
            }
            (false, true) => {
                map.send(msg);
            }
            _ => {}
        };

        Ok(())
    }
}
