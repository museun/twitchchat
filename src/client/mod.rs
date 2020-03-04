/*!
The client for reading/writing messages to Twitch

# Event handling.

You can [get] a [Dispatcher] from the [Client].

A dispatcher can be [subscribed] to. Subscribing will then produce a [Stream] of discrete [messages] for the [events].

## Stream cancellation
When the dispatcher is dropped, all of the streams will produce ***None***.

You can clear event subscriptions for [specific events][specific] or for [all events][all]

[get]: ./struct.Client.html#method.dispatcher
[Client]: ./struct.Client.html
[Dispatcher]: ./struct.Dispatcher.html
[subscribed]: ./struct.Dispatcher.html#method.subscribe
[Stream]: https://docs.rs/futures/0.3.1/futures/stream/trait.Stream.html
[messages]: ../messages/index.html
[events]: ../events/index.html
[specific]: ./struct.Dispatcher.html#method.clear_subscriptions
[all]: ./struct.Dispatcher.html#method.clear_subscriptions_all
*/

use futures::stream::*;
use std::sync::Arc;
use tokio::prelude::*;
use tokio::sync::{mpsc, Mutex};

use crate::rate_limit::*;

mod dispatcher;
pub use dispatcher::Dispatcher;

mod stream;
pub use stream::EventStream;

mod event;
pub use event::SimpleEvent;
#[doc(hidden)]
pub use event::{Event, EventMapped};

use crate::error::Error;

mod writer;
pub use writer::Writer;

mod reader;

type Tx<T = Vec<u8>> = mpsc::Sender<T>;
type Rx<T = Vec<u8>> = mpsc::Receiver<T>;

/// Status of the client after running
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Status {
    /// It ran to completion
    Eof,
    /// It was canceled
    Canceled,
}

/**
Client allows for reading and writing.

It reads from an [tokio::io::AsyncRead][AsyncRead] and allows [subscription] to [events] via a [Dispatcher].

It allows encoding of messages to a [tokio::io::AsyncWrite][AsyncWrite]

Event subcription returns a [Stream] of [Messages] for that event.

The client allows you to get a [Writer] which lets you send messages to the connection.

# Using the client

You drive a [Client] to completion with an [AsyncRead] and [AsyncWrite] pair by calling [run].

`run` will read from the connection end an error or EOF is received, dispatching messages to any event subscribers.

The [Writer] uses the AsyncWrite part to encode messages to it.

# Example
```rust,ignore
// make a client
let mut client = Client::new();
// get a cloneable writer
let mut writer = client.writer()
// subscribe to the join events (multiple subscriptions to the same event is allowed)
let mut join_stream = client.dispatcher().await.subscribe::<event::Join>();
tokio::task::spawn(async move {
    // will read until the client drops, or the event subscription is cleared
    while let Some(join_msg) = join_stream.next().await {
        // join_msg is a twitchchat::messages::JoinMessage
    }
});

// wait for the client to read to the end, driving any subscriptions
client.run(read_impl, write_impl).await;
```

[Dispatcher]: ../client/struct.Dispatcher.html
[Writer]: ../client/struct.Writer.html
[Client]: ../client/struct.Client.html
[AsyncRead]: https://docs.rs/tokio/0.2.6/tokio/io/trait.AsyncRead.html
[AsyncWrite]: https://docs.rs/tokio/0.2.6/tokio/io/trait.AsyncWrite.html
[Stream]: https://docs.rs/futures/0.3.1/futures/stream/trait.Stream.html
[run]: ./struct.Client.html#method.run
[subscription]: ./struct.Dispatcher.html#method.subscribe
[events]: ../events/index.html
[Messages]: ../events/index.html
*/
#[derive(Clone)]
pub struct Client {
    sender: Tx,

    // TODO make this something users can attach/deatch
    ready: SimpleEvent<crate::messages::Ready<'static>>,
    irc_ready: SimpleEvent<crate::messages::IrcReady<'static>>,
    global_state: SimpleEvent<crate::messages::GlobalUserState<'static>>,

    dispatcher: Arc<Mutex<Dispatcher>>,
    receiver: Arc<Mutex<Option<Rx>>>,
    abort: Arc<Mutex<Option<futures::future::AbortHandle>>>,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client").finish()
    }
}

impl Default for Client {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel(64);

        Self {
            sender,
            ready: SimpleEvent::new(),
            irc_ready: SimpleEvent::new(),
            global_state: SimpleEvent::new(),

            receiver: Arc::new(Mutex::new(Some(receiver))),
            dispatcher: Arc::new(Mutex::new(Dispatcher::new())),
            abort: Default::default(),
        }
    }
}

impl Client {
    /// Create a new client
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the dispatcher
    pub async fn dispatcher(&self) -> tokio::sync::MutexGuard<'_, Dispatcher> {
        self.dispatcher.lock().await
    }

    /// Get a new writer
    pub fn writer(&self) -> Writer {
        Writer::new(writer::DisjointWriter::new(self.sender.clone()))
    }

    /// Stops the running task
    ///
    /// This will cause `Client::run` to return [`Ok(Status::Canceled)`](./enum.Status.html#variant.Canceled)
    pub async fn stop(&self) -> Result<(), Error> {
        self.abort
            .lock()
            .await
            .take()
            .ok_or_else(|| Error::NotRunning)?
            .abort();
        Ok(())
    }

    /// Waits for the client to be in the `Ready` state
    ///
    /// # Returns
    /// * [`Ok(Ready)`][ok] if the client is ready to proceed
    /// * [`Err(Error::ClientDisconnect)`][error] if the client never became ready (within reason)
    ///
    /// [ok]: ../messages/struct.Ready.html
    /// [error]: ./enum.Error.html#variant.ClientDisconnect
    pub async fn wait_for_ready(&self) -> Result<crate::messages::Ready<'static>, Error> {
        self.ready.wait_for().await
    }

    /// Waits for the client to be in the `IrcReady` state
    ///
    /// # Returns
    /// * [`Ok(IrcReady)`][ok] if the client is ready to proceed
    /// * [`Err(Error::ClientDisconnect)`][error] if the client never became ready (within reason)
    ///
    /// [ok]: ../messages/struct.IrcReady.html
    /// [error]: ./enum.Error.html#variant.ClientDisconnect
    pub async fn wait_for_irc_ready(&self) -> Result<crate::messages::IrcReady<'static>, Error> {
        self.irc_ready.wait_for().await
    }

    /// waits for the client to receiver a `GlobalUserState`
    ///
    /// # Returns
    /// * [`Ok(GlobalUserState)`][ok] if the client is ready to proceed
    /// * [`Err(Error::ClientDisconnect)`][error] if the client never became ready (within reason)
    ///
    /// [ok]: ../messages/struct.GlobalUserState.html
    /// [error]: ./enum.Error.html#variant.ClientDisconnect
    pub async fn wait_for_global_user_state(
        &self,
    ) -> Result<crate::messages::GlobalUserState<'static>, Error> {
        self.global_state.wait_for().await
    }

    /// This allow you provide a custom Rate Limit configuration
    ///
    /// See [Client::run](./struct.Client.html#method.run)
    ///
    /// # Warning
    /// Having the wrong 'rate limit' will likely cause the server to drop you silently.
    ///
    /// Use this with caution, or if you know what you're doing
    pub async fn run_with_user_rate_limit<R, W>(
        &self,
        read: R,
        write: W,
        rate: RateLimit,
    ) -> Result<Status, Error>
    where
        R: AsyncRead + Send + Sync + Unpin + 'static,
        W: AsyncWrite + Send + Sync + Unpin + 'static,
    {
        if self.abort.lock().await.is_some() {
            return Err(Error::AlreadyRunning);
        }

        self.initialize_handlers().await;

        let dispatcher = Arc::clone(&self.dispatcher);
        let receiver = self
            .receiver
            .lock()
            .await
            .take()
            .expect("receiver to exist");

        let read = tokio::task::spawn(reader::read_loop(read, dispatcher));
        let write = tokio::task::spawn(writer::write_loop(write, rate, receiver));

        let fut = futures::future::try_select(read, write);
        let (handle, token) = futures::future::AbortHandle::new_pair();
        let future = futures::future::Abortable::new(fut, token);
        debug_assert!(
            self.abort.lock().await.replace(handle).is_none(),
            "client shouldn't have been running"
        );

        match future.await {
            Ok(Ok(res)) => res.factor_first().0,
            Ok(Err(err)) => panic!("panic in read/write handler: {}", err.factor_first().0),
            Err(..) => Ok(Status::Canceled),
        }
    }

    /// Run the client to completion, dispatching messages to the subscribers
    ///
    /// This returns a future. You should await this future at the end of your code to keep the runtime active until the client closes
    ///
    /// # Note
    /// This enables an internal rate limit of 50 messages sent per 30 seconds    
    ///
    /// # Returns after resolving the future
    /// * An [error][error] if one was encountered while in operation
    /// * [`Ok(Status::Eof)`][eof] if it ran to completion
    /// * [`Ok(Status::Canceled)`][cancel] if `stop` was called
    ///
    /// [error]: ./enum.Error.html
    /// [eof]: ../client/enum.Status.html#variant.Eof
    /// [cancel]: ../client/enum.Status.html#variant.Canceled
    // TODO allow for customization of the rate limiting
    pub fn run<R, W>(
        &self,
        read: R,
        write: W,
    ) -> impl std::future::Future<Output = Result<Status, Error>> + '_
    where
        R: AsyncRead + Send + Sync + Unpin + 'static,
        W: AsyncWrite + Send + Sync + Unpin + 'static,
    {
        use futures::prelude::*;
        let rate = RateLimit::from_class(RateClass::Known);
        let this = self.clone();
        tokio::task::spawn(async move { this.run_with_user_rate_limit(read, write, rate).await })
            .map_ok_or_else(|_err| Err(Error::ClientDisconnect), |ok| ok)
    }

    async fn initialize_handlers(&self) {
        let mut dispatcher = self.dispatcher().await;
        let mut writer = self.writer();

        // set up the auto PING
        let mut stream = dispatcher.subscribe_internal::<crate::events::Ping>(true);
        let ping = async move {
            while let Some(msg) = stream.next().await {
                if writer.pong(&msg.token).await.is_err() {
                    break;
                }
            }
        };
        tokio::task::spawn(ping);

        // set up ready
        let stream = dispatcher.subscribe_internal::<crate::events::Ready>(true);
        debug_assert!(self.ready.register(stream).await);

        // set up irc ready
        let stream = dispatcher.subscribe_internal::<crate::events::IrcReady>(true);
        debug_assert!(self.irc_ready.register(stream).await);

        // set up global user state
        let stream = dispatcher.subscribe_internal::<crate::events::GlobalUserState>(true);
        debug_assert!(self.global_state.register(stream).await);
    }
}
