use crate::{
    connector::Connector,
    messages::{Ping, Pong},
    rate_limit::AsyncBlocker,
    util::timestamp,
    util::Either::{self, Left, Right},
    *,
};

use runner::{ResetConfig, RunnerError, Status};

use futures_lite::{pin, AsyncRead, AsyncWrite, StreamExt};
use futures_timer::Delay;
use writer::{AsyncWriter, MpscWriter};

use std::{
    future::Future,
    time::{Duration, Instant},
};

const WINDOW: Duration = Duration::from_secs(45);
const TIMEOUT: Duration = Duration::from_secs(10);

/// A trait alias to make function signatures smaller
pub trait ConnectorSafe: AsyncRead + AsyncWrite + Unpin + Send + Sync {}

/// An async runner. This will act as a main loop, if you want one.
pub struct AsyncRunner {
    dispatcher: AsyncDispatcher,
    writer: (Sender<Vec<u8>>, Receiver<Vec<u8>>),
    activity: (Sender<()>, Receiver<()>),
    quit: (Sender<()>, Receiver<()>),
}

impl AsyncRunner {
    /// Create a new async runner with this dispatcher
    pub fn new(dispatcher: AsyncDispatcher) -> Self {
        Self {
            dispatcher,
            writer: channel::bounded(64),
            activity: channel::bounded(32),
            quit: channel::bounded(1),
        }
    }

    /// Get a **clonable** `Writer` with the provided `rate limiter` and `async blocker`
    pub fn writer<R, B>(&self, rate_limit: R, blocker: B) -> AsyncWriter<MpscWriter>
    where
        R: Into<Option<RateLimit>>,
        B: AsyncBlocker,
    {
        let (tx, rx) = (self.activity.0.clone(), self.quit.1.clone());
        let writer = MpscWriter::new(self.writer.0.clone());
        AsyncWriter::new(writer, tx, rx, rate_limit, blocker)
    }

    /// Get a mutable borrow to the dispatcher
    pub fn dispatcher(&mut self) -> &mut AsyncDispatcher {
        &mut self.dispatcher
    }

    /// Get a channel you can use to have the main loop exit early.
    pub fn quit_signal(&self) -> Receiver<()> {
        let (_, rx) = &self.quit;
        rx.clone()
    }

    /// Using this connector, retry strategy and reset config try to reconnect
    /// based on the retry strategy.
    ///
    /// This will act like run to completion in a loop with a configurable
    /// criteria for when a reconnect should happen.
    ///
    /// The reset configuration allows you to determine (and have a way to be
    /// notified when you should resubscribe, if you want to.)
    pub async fn run_with_retry<C, F, R, E>(
        &mut self,
        user_config: &UserConfig,
        connector: C,
        retry: F,
        reset_config: E,
    ) -> Result<(), RunnerError>
    where
        C: Connector,
        for<'a> &'a C::Output: ConnectorSafe,

        F: Fn(Result<Status, RunnerError>) -> R + Send + Sync,
        R: Future<Output = Result<bool, RunnerError>> + Send + Sync,
        E: Into<Option<ResetConfig>> + Send + Sync,
    {
        let mut reset_config = reset_config.into();

        loop {
            let status = self
                .run_to_completion(&user_config, connector.clone())
                .await;

            match retry(status).await {
                Err(err) => break Err(err),
                Ok(false) => break Ok(()),
                Ok(true) => {
                    // if we have a reset config, reset the handlers and send the signal
                    // otherwise just restart without resetting the handlers
                    if let Some(config) = &reset_config {
                        self.dispatcher.reset().await;

                        // if they dropped the receiver assume they don't want to reset any more
                        // so clear the option
                        if config.reset_handlers.send(()).await.is_err() {
                            reset_config.take();
                        }
                    };
                }
            }
        }
    }

    /// Using this connector, run the loop to completion.
    pub async fn run_to_completion<C>(
        &mut self,
        user_config: &UserConfig,
        connector: C,
    ) -> Result<Status, RunnerError>
    where
        C: Connector,
        for<'a> &'a C::Output: ConnectorSafe,
    {
        let mut connector = connector;

        let stream = connector.connect().await?;
        let stream = async_dup::Arc::new(stream);

        let mut ping = self.dispatcher.subscribe_system::<Ping>().await;
        let mut pong = self.dispatcher.subscribe_system::<Pong>().await;

        let (mut reader, mut writer) = (
            AsyncDecoder::new(stream.clone()), //
            AsyncEncoder::new(stream),
        );

        // register with the connection
        writer
            .encode(crate::commands::register(user_config))
            .await?;

        let mut state = TimeoutState::Start;

        let (_, rx) = &self.activity;
        let (_, write) = &self.writer;

        // this is awful. but look. no select!{}
        let status = loop {
            let (read, activity) = (reader.read_message(), rx.recv());
            let (ping, pong) = (ping.next(), pong.next());
            pin!(read);
            pin!(ping);
            pin!(activity);
            pin!(pong);

            // Bind all 4 interesting events together
            let (left, right) = (Either::select(read, ping), Either::select(activity, pong));
            pin!(left);
            pin!(right);

            let notification = Either::select(left, right);
            pin!(notification);

            let write = write.recv();
            pin!(write);

            let notification = Either::select(notification, write);
            pin!(notification);

            // and bind them with the timeout
            let timeout = Delay::new(WINDOW);
            pin!(timeout);

            // and select the first one
            match Either::select(notification, timeout).await {
                // we read a message
                Left(Left(Left(Left(read)))) => {
                    let msg = match read {
                        Err(DecodeError::Eof) => {
                            log::info!("got an EOF, exiting main loop");
                            break Status::Eof;
                        }
                        Err(err) => return Err(err.into()),
                        Ok(msg) => msg,
                    };
                    log::trace!("dispatching: {:#?}", msg);
                    self.dispatcher.dispatch(msg).await?;
                    state = TimeoutState::Activity(Instant::now())
                }

                // we get a ping
                Left(Left(Left(Right(Some(ping))))) => {
                    let token = ping.token();
                    log::debug!(
                        "got a ping from the server. responding with token '{}'",
                        token
                    );
                    let pong = crate::commands::pong(token);
                    writer.encode(pong).await?;
                    state = TimeoutState::activity();
                }

                // they wrote a message
                Left(Left(Right(Left(_write)))) => {
                    state = TimeoutState::activity();
                }

                // we got a pong
                Left(Left(Right(Right(Some(_pong))))) => {
                    if let TimeoutState::WaitingForPong(_ts) = state {
                        state = TimeoutState::activity();
                    }
                }

                // our future timed out, send a ping
                Left(Right(write)) => {
                    if let Some(write) = write {
                        writer.encode(write).await?;
                    } else {
                        log::warn!("no more writers detected");
                    }
                }

                Right(_timeout) => {
                    log::info!("idle connectiond detected, sending a ping");
                    let ts = timestamp().to_string();
                    writer.encode(crate::commands::ping(&ts)).await?;
                    state = TimeoutState::waiting_for_pong();
                }

                // we have a dead future -- they should all be alive unless we're shutting down
                _ => break Status::Eof,
            };

            match state {
                TimeoutState::WaitingForPong(dt) => {
                    if dt.elapsed() > TIMEOUT {
                        log::warn!("PING timeout detected, exiting");
                        break Status::TimedOut;
                    }
                }
                TimeoutState::Activity(dt) => {
                    if dt.elapsed() > WINDOW {
                        log::warn!("idle connectiond detected, sending a PING");
                        let ts = timestamp().to_string();
                        writer.encode(crate::commands::ping(&ts)).await?;
                        state = TimeoutState::waiting_for_pong();
                    }
                }
                TimeoutState::Start => {}
            }
        };

        let (tx, _) = &self.quit;
        // send the quit signal
        let _ = tx.send(()).await;

        // TODO: determine if we want to wait for all writers to finish
        // it wouldn't make much sense, twitch closes the connection as soon as
        // it reads the QUIT message
        //
        // but this could 'spin' on writers (or if we don't give quit_tx to
        // writers), or some other 'spawned' task
        // while !self.quit_tx.is_closed() {
        //     futures_lite::future::yield_now().await;
        // }

        Ok(status)
    }
}

#[derive(Copy, Clone, Debug)]
enum TimeoutState {
    WaitingForPong(Instant),
    Activity(Instant),
    Start,
}

impl TimeoutState {
    fn activity() -> Self {
        Self::Activity(Instant::now())
    }

    fn waiting_for_pong() -> Self {
        Self::WaitingForPong(Instant::now())
    }
}
