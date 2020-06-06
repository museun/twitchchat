use {super::*, crate::*};

use std::future::Future;
use std::sync::Arc;

use futures::prelude::*;

use tokio::prelude::*;
use tokio::prelude::{AsyncRead, AsyncWrite};

use tokio::sync::Mutex;
use tokio::time::Duration;

// 45 seconds after a receive we'll send a ping
const PING_INACTIVITY: Duration = Duration::from_secs(45);

// and then wait 10 seconds for a pong resposne
const PING_WINDOW: Duration = Duration::from_secs(10);

type BoxFuture<'a, T> = std::pin::Pin<Box<dyn Future<Output = T> + 'a + Send>>;
type ConnectFuture<IO> = BoxFuture<'static, Result<IO, std::io::Error>>;

/// A connector type that acts as a factory for connecting to Twitch
pub struct Connector<IO> {
    connect: Arc<dyn Fn() -> ConnectFuture<IO> + Send + Sync + 'static>,
}

impl<IO> Connector<IO>
where
    IO: AsyncRead + AsyncWrite,
    IO: Send + Sync + 'static,
{
    /// Create a new connector with this factory function
    pub fn new<F, R>(connect_func: F) -> Self
    where
        F: Fn() -> R + Send + Sync + 'static,
        R: Future<Output = Result<IO, std::io::Error>> + Send + Sync + 'static,
    {
        Self {
            connect: Arc::new(move || Box::pin(connect_func())),
        }
    }
}

impl<IO> Clone for Connector<IO> {
    fn clone(&self) -> Self {
        Self {
            connect: Arc::clone(&self.connect),
        }
    }
}

impl<IO> std::fmt::Debug for Connector<IO> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Connector").finish()
    }
}

/// Some common retry strategies.
///
/// These are used with [`Runner::run_with_retry`][retry].
///
/// You can provide your own by simplying having an async function with the same
/// signature.
///
/// That is `async fn(result: Result<Status, Error>) -> Result<bool, Error>`.
///
/// Return one of:
/// * `Ok(true)` to cause it to reconnect.
/// * `Ok(false)` will gracefully exit with `Ok(Status::Eof)`
/// * `Err(err)` will return that error
#[derive(Copy, Clone, Debug, Default)]
pub struct RetryStrategy;

impl RetryStrategy {
    /// Reconnect immediately unless the `Status` was `Canceled`
    pub async fn immediately(result: Result<Status, Error>) -> Result<bool, Error> {
        if let Ok(Status::Canceled) = result {
            return Ok(false);
        }
        Ok(true)
    }

    /// Retries if `Status` was a **Timeout**, otherwise return the `Err` or `false` (to stop the connection loop).
    pub async fn on_timeout(result: Result<Status, Error>) -> Result<bool, Error> {
        let status = if let Status::Timeout = result? {
            true
        } else {
            false
        };

        Ok(status)
    }

    /// Retries if the `Result` was an error
    pub async fn on_error(result: Result<Status, Error>) -> Result<bool, Error> {
        Ok(result.is_err())
    }
}

/// A type that drive the __event loop__ to completion, and optionally retries on an return condition.
///
/// This type is used to 'drive' the dispatcher and internal read/write futures.
pub struct Runner {
    dispatcher: Dispatcher,
    receiver: Rx,
    writer: Writer,
    abort: abort::Abort,
    ready: Arc<tokio::sync::Notify>,
}

impl Runner {
    /// Create a Runner with the provided dispatcher with the default rate limiter
    ///
    /// # Returns
    /// This returns the [`Runner`] and a [`Control`] type that'll let you interact with the Runner.
    ///
    /// [`Runner`]: ./struct.Runner.html
    /// [`Control`]: ./struct.Control.html
    pub fn new(dispatcher: Dispatcher) -> (Runner, Control) {
        Self::new_with_rate_limit(dispatcher, RateLimit::default())
    }

    /// Create a Runner without a rate limiter.
    ///
    /// # Warning
    /// This is not advisable and goes against the 'rules' of the API.
    ///
    /// You should prefer to use [`Runner::new`](#method.new) to use a default rate limiter
    ///
    /// Or, [`Runner::new_with_rate_limit`](#method.new_with_rate_limit) if you
    /// know your 'bot' status is higher than normal.
    ///
    /// # Returns
    /// This returns the [`Runner`] and a [`Control`] type that'll let you interact with the Runner.
    ///
    /// [`Runner`]: ./struct.Runner.html
    /// [`Control`]: ./struct.Control.html
    pub fn new_without_rate_limit(dispatcher: Dispatcher) -> (Runner, Control) {
        let (sender, receiver) = mpsc::channel(64);
        let stop = abort::Abort::default();
        let writer = Writer::new(writer::MpscWriter::new(sender));
        let ready = Arc::new(tokio::sync::Notify::default());

        let this = Self {
            dispatcher,
            receiver,
            abort: stop.clone(),
            writer: writer.clone(),
            ready: ready.clone(),
        };

        let control = Control {
            writer,
            stop,
            ready,
        };
        (this, control)
    }

    /// Crate a new Runner with the provided dispatcher and rate limiter
    ///
    /// # Returns
    /// This returns the [`Runner`] and a [`Control`] type that'll let you interact with the Runner.
    ///
    /// [`Runner`]: ./struct.Runner.html
    /// [`Control`]: ./struct.Control.html
    ///
    pub fn new_with_rate_limit(dispatcher: Dispatcher, rate_limit: RateLimit) -> (Runner, Control) {
        let (sender, receiver) = mpsc::channel(64);
        let stop = abort::Abort::default();

        let writer = Writer::new(writer::MpscWriter::new(sender))
            .with_rate_limiter(Arc::new(Mutex::new(rate_limit)));

        let ready = Arc::new(tokio::sync::Notify::default());

        let this = Self {
            dispatcher,
            receiver,
            abort: stop.clone(),
            writer: writer.clone(),
            ready: ready.clone(),
        };

        let control = Control {
            writer,
            stop,
            ready,
        };

        (this, control)
    }

    /// Run to completion.
    ///
    /// This takes a [`Connector`][connector] which acts a factory for producing IO types.
    ///
    /// This will only call the connector factory once. If you want to reconnect
    /// automatically, refer to [`Runner::run_with_retry`][retry]. That function takes in
    /// a retry strategy for determining how to continue on disconnection.
    ///
    /// The follow happens during the operation of this future
    /// * Connects using the provided [`Connector`][connector]
    /// * Automatically `PING`s the connection when a `PONG` is received
    /// * Checks for timeouts.
    /// * Reads from the IO type, parsing and dispatching messages
    /// * Reads from the writer and forwards it to the IO type
    /// * Listens for user cancellation from the [`Control::stop`][stop] method.
    ///
    /// # Returns a future that resolves to..
    /// * An [error] if one was encountered while in operation
    /// * [`Ok(Status::Eof)`][eof] if it ran to completion
    /// * [`Ok(Status::Canceled)`][cancel] if the associated [`Control::stop`][stop] was called
    ///
    /// [connector]: ./struct.Connector.html
    /// [error]: ./enum.Error.html
    /// [eof]: ./enum.Status.html#variant.Eof
    /// [cancel]: ./enum.Status.html#variant.Canceled
    /// [stop]: ./struct.Control.html#method.stop    
    /// [retry]: #method.run_with_retry
    ///
    pub async fn run_to_completion<IO>(&mut self, connector: Connector<IO>) -> Result<Status, Error>
    where
        IO: AsyncRead + AsyncWrite,
        IO: Unpin + Send + Sync + 'static,
    {
        let io = (connector.connect)().await.map_err(Error::Io)?;

        let mut stream = tokio::io::BufStream::new(io);
        let mut buffer = String::with_capacity(1024);

        let mut ping = self
            .dispatcher
            .subscribe_internal::<crate::events::Ping>(true);

        struct Token(Arc<tokio::sync::Notify>, Arc<tokio::sync::Notify>);
        impl Drop for Token {
            fn drop(&mut self) {
                self.0.notify();
                self.1.notify();
            }
        }

        let restart = Arc::new(tokio::sync::Notify::default());

        // when this drops, the check_connection loop will exit
        let _token = Token(restart.clone(), self.ready.clone());

        let mut out = self.writer.clone();

        // we start a 2nd loop that runs outside of the main loop
        // this sends a ping if we've not sent a message with a window defined by PING_INACTIVITY
        // and if we didn't a PONG response within PING_WINDOW we'll consider the connection stale and exit
        let (mut check_timeout, timeout_delay, timeout_task) =
            check_connection(restart, &self.dispatcher, out.clone());

        loop {
            tokio::select! {
                // Abort notification
                _ = self.abort.wait_for() => {
                    log::debug!("received signal from user to stop");
                    let _ = self.dispatcher.clear_subscriptions_all();
                    break Ok(Status::Canceled)
                }

                // Auto-ping
                Some(msg) = ping.next() => {
                    if out.pong(&msg.token).await.is_err() {
                        log::debug!("cannot send pong");
                        break Ok(Status::Eof);
                    }
                }

                // Read half
                Ok(n) = &mut stream.read_line(&mut buffer) => {
                    if n == 0 {
                        log::info!("read 0 bytes. this is an EOF");
                        break Ok(Status::Eof)
                    }

                    let mut visited = false;
                    for msg in decode(&buffer) {
                        let msg = msg?;
                        log::trace!(target: "twitchchat::runner::read", "< {}", msg.raw.escape_debug());
                        self.dispatcher.dispatch(&msg);
                        visited = true;
                    }

                    // if we didn't parse a message then we should signal that this was EOF
                    // twitch sometimes just stops writing to the client
                    if !visited {
                        log::warn!("twitch sent an incomplete message");
                        break Ok(Status::Eof)
                    }
                    buffer.clear();

                    let _ = check_timeout.send(()).await;
                },

                // Write half
                Some(data) = &mut self.receiver.next() => {
                    log::trace!(target: "twitchchat::runner::write", "> {}", std::str::from_utf8(&data).unwrap().escape_debug());
                    stream.write_all(&data).await?;
                    // flush after each line -- people probably prefer messages sent early
                    stream.flush().await?
                },

                // We received a timeout
                _ = timeout_delay.notified() => {
                    log::warn!(target: "twitchchat::runner::timeout", "timeout detected, quitting loop");
                    // force the loop to exit (we could also use the 'restart' notify here)
                    drop(check_timeout);
                    // and wait for the task to join
                    timeout_task.await;
                    break Ok(Status::Timeout);
                },

                // All of the futures are dead, so the loop should end
                else => {
                    log::info!("all futures are dead. ending loop");
                    break Ok(Status::Eof)
                }
            }
        }
    }
}

fn check_connection(
    dispatcher: &Dispatcher,
    mut writer: Writer,
) -> (
    tokio::sync::mpsc::Sender<()>,
    Arc<tokio::sync::Notify>,
    impl Future,
) {
    use tokio::sync::{mpsc, Notify};

    let mut pong = dispatcher.subscribe_internal::<crate::events::Pong>(true);
    let timeout_notify = Arc::new(Notify::new());
    let (tx, mut rx) = mpsc::channel(1);

    let timeout = timeout_notify.clone();
    let task = async move {
        loop {
            tokio::select! {
                _ = tokio::time::delay_for(PING_INACTIVITY) => {
                    log::debug!(target: "twitchchat::runner::timeout", "inactivity detected of {:?}, sending a ping", PING_INACTIVITY);

                    let ts = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .expect("time to not go backwards")
                        .as_secs();

                    if writer.ping(&format!("{}", ts)).await.is_err() {
                        timeout.notify();
                        log::error!(target: "twitchchat::runner::timeout", "cannot send ping");
                        break;
                    }

    /// Run to completion and applies a retry functor to the result.
    ///
    /// This takes in a:
    /// * [`Connector`][connector] which acts a factory for producing IO types.
    /// * `retry_check` is a functor from `Result<Status, Error>` to a ___future___ of a `Result<bool, Error>`.
    ///
    /// You can pause in the `retry_check` to cause the next connection attempt to be delayed.
    ///
    /// `retry_check` return values:
    /// * `Ok(true)` will cause this to reconnect.
    /// * `Ok(false)` will cause this to exit with `Ok(Status::Eof)`
    /// * `Err(..)` will cause this to exit with `Err(err)`
    ///
    /// [connector]: ./struct.Connector.html     
    pub async fn run_with_retry<IO, F, R>(
        &mut self,
        connector: Connector<IO>,
        retry_check: F,
    ) -> Result<(), Error>
    where
        IO: AsyncRead + AsyncWrite,
        IO: Unpin + Send + Sync + 'static,

        F: Fn(Result<Status, Error>) -> R,
        R: Future<Output = Result<bool, Error>> + Send + Sync + 'static,
    {
        loop {
            let res = self.run_to_completion(connector.clone()).await;
            match retry_check(res).await {
                Err(err) => break Err(err),
                Ok(false) => break Ok(()),
                Ok(true) => {}
            }

            // reset our internal subscriptions to stop the leak
            self.dispatcher.reset_internal_subscriptions();
        }
    }
}

fn check_connection(
    restart: Arc<tokio::sync::Notify>,
    dispatcher: &Dispatcher,
    mut writer: Writer,
) -> (
    tokio::sync::mpsc::Sender<()>,
    Arc<tokio::sync::Notify>,
    impl Future,
) {
    use tokio::sync::{mpsc, Notify};

    let mut pong = dispatcher.subscribe_internal::<crate::events::Pong>(true);
    let timeout_notify = Arc::new(Notify::new());
    let (tx, mut rx) = mpsc::channel(1);

    let timeout = timeout_notify.clone();
    let task = async move {
        loop {
            tokio::select! {
                // check to see if we've sent a message within the window
                _ = tokio::time::delay_for(PING_INACTIVITY) => {
                    log::debug!(target: "twitchchat::runner::timeout", "inactivity detected of {:?}, sending a ping", PING_INACTIVITY);

                    let ts = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .expect("time to not go backwards")
                        .as_secs();

                    // try sending a ping
                    if writer.ping(&format!("{}", ts)).await.is_err() {
                        timeout.notify();
                        log::error!(target: "twitchchat::runner::timeout", "cannot send ping");
                        break;
                    }

                    // and if we didn't get a response in time
                    if tokio::time::timeout(PING_WINDOW, pong.next())
                        .await
                        .is_err()
                    {
                        // exit
                        timeout.notify();
                        log::error!(target: "twitchchat::runner::timeout", "did not get a ping after {:?}", PING_WINDOW);
                        break;
                    }
                }

                // we write something in time, do nothing
                Some(..) = rx.next() => { }

                // when the main loop drops, this is triggered
                _ = restart.notified() => { break }

                else => { break }
            }
        }
    };

    (tx, timeout_notify, tokio::task::spawn(task))
}

impl std::fmt::Debug for Runner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Runner").finish()
    }
}
