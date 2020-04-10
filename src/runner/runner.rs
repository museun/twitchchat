use {super::*, crate::*};

use std::sync::Arc;
use tokio::prelude::*;
use tokio::sync::Mutex;
use tokio::time::Duration;

// 45 seconds after a receive we'll send a ping
const PING_INACTIVITY: Duration = Duration::from_secs(45);
// and then wait 10 seconds for a pong resposne
const PING_WINDOW: Duration = Duration::from_secs(10);

/**
The runner is the main "event loop" of this crate.

It is created with a [Dispatcher][dispatcher]. It returns the new runner and the
[Control][control] type.

Once you're ready to start reading from the **Reader** and processing **Writes** you should call [Runner::run](#method.run).

# Returns
- A [`future`][future] which resolves to a [Status][status] once the Runner has finished.

Interacting with the `Runner` is done via the [Control][control] type.

# Example
```rust
# use tokio::spawn;
# tokio::runtime::Runtime::new().unwrap().block_on(async {
# let conn = tokio_test::io::Builder::new().wait(std::time::Duration::from_millis(10000)).build();
use twitchchat::{Dispatcher, Status, Runner, RateLimit};
// make a dispatcher
let dispatcher = Dispatcher::new();
// do stuff with the dispatcher (its clonable)
// ..

// create a new runner
let (runner, control) = Runner::new(dispatcher, RateLimit::default());

// spawn a task that kills the runner after some time
let ctl = control.clone();
spawn(async move {
    // pretend some time has passed
    ctl.stop()
});

// run, blocking the task.
// you can spawn this in a task and await on that join handle if you prefer
match runner.run(conn).await {
    // for the doc test
    Ok(Status::Canceled) => { assert!(true) }
    Ok(Status::Eof) => { panic!("eof") }
    Ok(Status::Timeout) => { panic!("timeout") }
    Err(err) => { panic!("{}", err) }
};
# });
```

[dispatcher]: ./struct.Dispatcher.html
[control]: ./struct.Control.html
[status]: ./enum.Status.html
[future]: https://doc.rust-lang.org/std/future/trait.Future.html
*/
pub struct Runner {
    dispatcher: Dispatcher,
    receiver: Rx,
    writer: Writer,
    abort: abort::Abort,
}

impl Runner {
    /**
    Create a new client runner with this [`Dispatcher`][dispatcher]

    # Returns
    The [`Runner`]() and a [`Control`][control] type

    [control]: ./struct.Control.html
    [dispatcher]: ./struct.Dispatcher.html
    */
    pub fn new(dispatcher: Dispatcher, rate_limit: RateLimit) -> (Self, Control) {
        let (sender, receiver) = mpsc::channel(64);
        let abort = abort::Abort::default();

        let writer = Writer::new(writer::MpscWriter::new(sender))
            .with_rate_limiter(Arc::new(Mutex::new(rate_limit)));

        let control = Control {
            writer: writer.clone(),
            stop: abort.clone(),
        };

        let this = Self {
            receiver,
            dispatcher,
            writer,
            abort,
        };

        (this, control)
    }

    /**
    Run to completion, dispatching messages to the subscribers.

    This returns a future. You should await this future at the end of your code
    to keep the runtime active until the client closes.

    # Interacting with the runner
    You can interact with the runner via the `Control` type returned by [`Runner::new`](#method.new).

    To _stop_ this early, you can use the [`Control::stop`][stop] method.

    To get a _writer_, you can use the [`Control::writer`][writer] method.

    # Returns after resolving the future
    * An [error][error] if one was encountered while in operation
    * [`Ok(Status::Eof)`][eof] if it ran to completion
    * [`Ok(Status::Canceled)`][cancel] if the associated [`Control::stop`][stop] was called

    [error]: ./enum.Error.html
    [eof]: ./enum.Status.html#variant.Eof
    [cancel]: ./enum.Status.html#variant.Canceled
    [stop]: ./struct.Control.html#method.stop
    [writer]: ./struct.Control.html#method.writer
    */
    pub async fn run<IO>(mut self, io: IO) -> Result<Status, Error>
    where
        IO: AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static,
    {
        use futures::prelude::*;
        let mut stream = tokio::io::BufStream::new(io);
        let mut buffer = String::with_capacity(1024);

        let mut ping = self
            .dispatcher
            .subscribe_internal::<crate::events::Ping>(true);

        let mut out = self.writer;

        let (mut check_timeout, timeout_delay) =
            Self::check_connection(&self.dispatcher, out.clone());

        loop {
            tokio::select! {
                _ = timeout_delay.notified() => {
                    log::warn!("timeout detected, quitting loop");
                    break Ok(Status::Timeout);
                }

                // Abort notification
                _ = self.abort.wait_for() => {
                    let _ = self.dispatcher.clear_subscriptions_all();
                    break Ok(Status::Canceled)
                }

                // Auto-ping
                Some(msg) = ping.next() => {
                    if out.pong(&msg.token).await.is_err() {
                        break Ok(Status::Eof);
                    }
                }

                // Read half
                Ok(n) = &mut stream.read_line(&mut buffer) => {
                    if n == 0 {
                        break Ok(Status::Eof)
                    }

                    let mut visited = false;
                    for msg in decode(&buffer) {
                        let msg = msg?;
                        log::trace!("< {}", msg.raw.escape_debug());
                        self.dispatcher.dispatch(&msg);
                        visited = true;
                    }

                    // if we didn't parse a message then we should signal that this was EOF
                    // twitch sometimes just stops writing to the client
                    if !visited {
                        break Ok(Status::Eof)
                    }

                    buffer.clear();

                    let _ = check_timeout.send(()).await;
                },

                // Write half
                Some(data) = &mut self.receiver.next() => {
                    log::trace!("> {}", std::str::from_utf8(&data).unwrap().escape_debug());
                    stream.write_all(&data).await?;
                    stream.flush().await?
                },

                // All of the futures are dead, so the loop should end
                else => { break Ok(Status::Eof) }
            }
        }
    }

    fn check_connection(
        dispatcher: &Dispatcher,
        mut writer: Writer,
    ) -> (tokio::sync::mpsc::Sender<()>, Arc<tokio::sync::Notify>) {
        use futures::prelude::*;
        use tokio::sync::{mpsc, Notify};

        let mut pong = dispatcher.subscribe_internal::<crate::events::Pong>(true);
        let timeout = Arc::new(Notify::new());
        let (tx, mut rx) = mpsc::channel(1);

        tokio::task::spawn({
            let timeout = timeout.clone();
            async move {
                loop {
                    tokio::select! {
                        _ = tokio::time::delay_for(PING_INACTIVITY) => {
                            log::debug!("inactivity detected of {:?}, sending a ping", PING_INACTIVITY);
                            if writer.ping(&format!(
                                "{}",
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .expect("time to not go backwards")
                                    .as_secs()
                            ))
                            .await.is_err() {
                                timeout.notify();
                                log::error!("cannot send ping");
                                break;
                            }

                            if tokio::time::timeout(PING_WINDOW, pong.next())
                                .await
                                .is_err()
                            {
                                timeout.notify();
                                log::error!("did not get a ping after {:?}", PING_WINDOW);
                                break;
                            }
                        }
                        Some(..) = rx.next() => { }
                    }
                }
            }
        });

        (tx, timeout)
    }
}

impl std::fmt::Debug for Runner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Runner").finish()
    }
}
