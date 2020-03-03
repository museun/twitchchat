use super::{Error, EventStream};
use crate::AsOwned;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::{watch, Mutex};

mod private {
    use super::{Event, EventMapped};

    pub trait EventSealed {}
    impl<'a, T: Event<'a>> EventSealed for T {}

    pub trait EventMappedSealed<E> {}
    impl<'a, T: EventMapped<'a, E>, E: Event<'a>> EventMappedSealed<E> for T {}
}

/// A marker trait for Event subscription
pub trait Event<'a>: private::EventSealed {
    /// Event message parsing
    type Parsed: crate::Parse<&'a crate::decode::Message<'a>> + AsOwned;
}

/// A trait to convert an Event::Parsed to a 'static type
pub trait EventMapped<'a, T>: private::EventMappedSealed<T>
where
    T: Event<'a>,
{
    /// Event message mapping
    type Owned: Clone + Debug + Send + Sync + 'static;
    /// Converts this to the owned representation
    fn into_owned(data: T::Parsed) -> Self::Owned;
}

impl<'a, T> EventMapped<'a, T> for T
where
    T: Event<'a>,
    <T::Parsed as AsOwned>::Owned: Clone + Debug + Send + Sync + 'static,
{
    type Owned = <T::Parsed as AsOwned>::Owned;
    fn into_owned(data: T::Parsed) -> Self::Owned {
        <T::Parsed as AsOwned>::as_owned(&data)
    }
}

#[derive(Clone, Debug)]
enum ReadyState<T, E = ()> {
    Ready(T),
    NotReady,
    Error(E),
}

impl<T> Default for ReadyState<T> {
    fn default() -> Self {
        Self::NotReady
    }
}

/// A simple event wrapper
///
/// This wraps an `EventStream` and provides a 'MRU'-style memorized container that can be awaited on.
///
/// When this `EventStream` produces a `message` the SimpleEvent's `wait_for` will return it
///
/// Repeated calls to `wait_for` will return the 'current' message.
///
/// If no message has been received yet, it will block until one is available.
///
/// ```rust,no_run
/// # use twitchchat::{events, messages};
/// # use twitchchat::{Client, client::SimpleEvent};
/// # let client = Client::new();
/// # tokio::runtime::Runtime::new().unwrap().block_on(async move {
/// // subscribe to the event you want
/// let stream = client.dispatcher().await.subscribe::<events::IrcReady>();
/// // make a simple event (make sure the mappings match..)
/// let event: SimpleEvent<messages::IrcReady<'static>> = SimpleEvent::new();
/// // register with the stream
/// assert!(event.register(stream).await);
///
/// // block until we get a message from the event stream
/// let msg: messages::IrcReady<'static> = event.wait_for().await.unwrap();
/// // repeated calls will return the 'most recent' one
/// # });
/// ```
#[derive(Clone)]
pub struct SimpleEvent<T>
where
    T: Clone + Send + Sync + 'static,
{
    tx: Arc<Mutex<Option<watch::Sender<ReadyState<T>>>>>,
    rx: watch::Receiver<ReadyState<T>>,
}

impl<T> Debug for SimpleEvent<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleEvent")
            .field("kind", &std::any::type_name::<T>())
            .finish()
    }
}

impl<T> SimpleEvent<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Create a new SimpleEvent for the message `T`
    ///
    /// This should be one of the types from [`messages`](../messages/index.html)
    pub fn new() -> Self {
        let (tx, rx) = watch::channel(Default::default());
        Self {
            tx: Arc::new(Mutex::new(Some(tx))),
            rx,
        }
    }

    /// Wait for the next message
    ///
    /// If a message hasn't been received at all, it'll block until it gets one
    ///
    /// Otherwise it'll return the most recent one
    pub async fn wait_for(&self) -> Result<T, Error> {
        let mut ready = self.rx.clone();
        loop {
            match ready.recv().await {
                Some(ReadyState::Ready(val)) => return Ok(val),
                Some(ReadyState::Error(_)) | None => return Err(Error::ClientDisconnect),
                Some(ReadyState::NotReady) => continue,
            }
        }
    }

    /// Registers this `SimpleEvent` with an `EventStream`
    ///
    /// Returns `false` if this has already been called
    pub async fn register(&self, mut stream: EventStream<Arc<T>>) -> bool {
        use tokio::stream::StreamExt as _;
        let tx = match self.tx.lock().await.take() {
            Some(tx) => tx,
            None => return false,
        };

        let ready = async move {
            let ready = match stream.next().await {
                Some(ready) => {
                    let ready = unwrap_arc(ready);
                    ReadyState::Ready(ready)
                }
                None => ReadyState::Error(()),
            };
            if tx.broadcast(ready).is_err() {
                return;
            }
        };
        tokio::task::spawn(ready);
        true
    }
}

#[inline]
fn unwrap_arc<T: Clone>(d: Arc<T>) -> T {
    Arc::try_unwrap(d).map_or_else(|d| (&*d).clone(), |d| d)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn event_mapped() {
        fn e<'a, T>(msg: &'a crate::decode::Message<'a>) -> T::Owned
        where
            T: Event<'a> + 'static,
            T: EventMapped<'a, T>,
        {
            use crate::Parse as _;
            T::into_owned(T::Parsed::parse(msg).unwrap())
        }

        let msg = crate::decode("PING :1234567890\r\n")
            .next()
            .unwrap()
            .unwrap();

        let msg: crate::messages::Ping<'static> = e::<crate::events::Ping>(&msg);
        assert_eq!(msg.token, "1234567890")
    }
}
