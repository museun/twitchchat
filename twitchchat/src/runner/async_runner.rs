use crate::{
    connector::Connector,
    messages::{Ping, Pong},
    rate_limit::{AsyncBlocker, NullBlocker},
    runner::{Error, ReadyMessage, ResetConfig, Status},
    util::Either::{Left, Right},
    util::{timestamp, FutExt},
    writer::{AsyncWriter, MpscWriter},
    *,
};

use futures_lite::{AsyncRead, AsyncWrite, StreamExt};
use futures_timer::Delay;

use std::{
    future::Future,
    time::{Duration, Instant},
};

const WINDOW: Duration = Duration::from_secs(45);
const TIMEOUT: Duration = Duration::from_secs(10);

/// An async runner. This will act as a main loop, if you want one.
pub struct AsyncRunner<C>
where
    C: Connector,
{
    dispatcher: AsyncDispatcher,
    writer: AsyncWriter<MpscWriter>,

    writer_rx: Receiver<Vec<u8>>,
    activity_rx: Receiver<()>,
    quit_tx: Sender<()>,
    quit_rx: Receiver<()>,

    wait_for: super::WaitFor,

    user_config: UserConfig,
    connector: C,
    stream: Option<async_dup::Arc<C::Output>>,
}

impl<C> AsyncRunner<C>
where
    C: Connector + Send + Sync,
    C::Output: AsyncRead + AsyncWrite + Unpin + Send + Sync,
    for<'a> &'a C::Output: AsyncRead + AsyncWrite + Unpin + Send + Sync,
{
    /// Create a new async runner with this dispatcher
    pub fn new(dispatcher: AsyncDispatcher, user_config: UserConfig, connector: C) -> Self {
        let (writer_tx, writer_rx) = channel::bounded(64);
        let (activity_tx, activity_rx) = channel::bounded(32);
        let (quit_tx, quit_rx) = channel::bounded(1);

        let writer = MpscWriter::new(writer_tx);
        let writer = AsyncWriter::new(
            writer,
            activity_tx,
            quit_rx.clone(),
            None,
            NullBlocker::default(),
        );

        Self {
            dispatcher,
            writer,

            writer_rx,
            activity_rx,
            quit_tx,
            quit_rx,

            wait_for: super::WaitFor::default(),

            user_config,
            connector,
            stream: None,
        }
    }

    /// Get a **clonable** `Writer` with the provided `rate limiter` and `async blocker`
    pub fn writer<R, B>(&self, rate_limit: R, blocker: B) -> AsyncWriter<MpscWriter>
    where
        R: Into<Option<RateLimit>>,
        B: AsyncBlocker,
    {
        self.writer.reconfigure(rate_limit, blocker)
    }

    /// Get a borrow of the dispatcher
    pub fn dispatcher(&self) -> &AsyncDispatcher {
        &self.dispatcher
    }

    /// Get a channel you can use to have the main loop exit early.
    pub fn quit_signal(&self) -> Receiver<()> {
        self.quit_rx.clone()
    }

    pub async fn wait_for_ready<T>(&mut self) -> Result<T, RunnerError>
    where
        T: ReadyMessage<'static> + Clone + Send + Sync + 'static,
        DispatchError: From<T::Error>,
    {
        self.wait_for.register::<T>();

        let mut should_register = false;
        if self.stream.is_none() {
            log::warn!("initializing stream in wait for ready");
            let stream = self.connector.connect().await?;
            let stream = async_dup::Arc::new(stream);
            self.stream.replace(stream);
            should_register = true;
        }

        let mut step_state = StepState::build(self).await;
        if should_register {
            Self::register(&self.user_config, &mut step_state.writer).await?;
        }

        loop {
            if let Some(msg) = self.wait_for.check_queue::<T>() {
                return T::from_irc(msg)
                    .map_err(DispatchError::from)
                    .map_err(Into::into);
            }

            log::warn!("stepping in wait for ready");
            match self.step(&mut step_state).await? {
                StepResult::Continue => {}
                StepResult::Break(_) => {
                    // this is an error
                }
            }
        }
    }

    /// Using this connector, retry strategy and reset config try to reconnect
    /// based on the retry strategy.
    ///
    /// This will act like run to completion in a loop with a configurable
    /// criteria for when a reconnect should happen.
    ///
    /// The reset configuration allows you to determine (and have a way to be
    /// notified when you should resubscribe, if you want to.)
    pub async fn run_with_retry<F, R, E>(&mut self, retry: F, reset_config: E) -> Result<(), Error>
    where
        F: Fn(Result<Status, Error>) -> R + Send + Sync,
        R: Future<Output = Result<bool, Error>> + Send + Sync,
        E: Into<Option<ResetConfig>> + Send + Sync,
    {
        let mut reset_config = reset_config.into();

        loop {
            let status = self.run_to_completion().await;

            match retry(status).await {
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
                    }
                }

                Ok(false) => break Ok(()),
                Err(err) => break Err(err),
            }
        }
    }

    /// Using this connector, run the loop to completion.
    pub async fn run_to_completion(&mut self) -> Result<Status, Error> {
        let stream = self.connector.connect().await?;
        let stream = async_dup::Arc::new(stream);

        self.stream.replace(stream);
        let mut step_state = StepState::build(self).await;

        Self::register(&self.user_config, &mut step_state.writer).await?;

        let status = loop {
            match self.step(&mut step_state).await? {
                StepResult::Continue => {}
                StepResult::Break(done) => break done,
            }
        };

        // send the quit signal
        let _ = self.quit_tx.send(()).await;

        Ok(status)
    }

    async fn step(&mut self, step_state: &mut StepState<C>) -> Result<StepResult, Error> {
        let StepState {
            ping,
            pong,
            reader,
            writer,
            state,
        } = step_state;

        let select = FutExt::either(reader.read_message(), self.activity_rx.recv())
            .either(ping.next())
            .either(pong.next())
            .either(self.writer_rx.recv())
            .either(Delay::new(WINDOW))
            .await;

        match select {
            Left(Left(Left(Left(Left(read))))) => {
                let msg = match read {
                    Err(DecodeError::Eof) => {
                        log::info!("got an EOF, exiting main loop");
                        return Ok(StepResult::Break(Status::Eof));
                    }
                    Err(err) => {
                        log::warn!("read an error: {}", err);
                        return Err(err.into());
                    }
                    Ok(msg) => msg,
                };

                self.wait_for.maybe_add(&msg);

                log::trace!("dispatching: {}", util::name(&msg));
                self.dispatcher.dispatch(msg).await?;
                *state = TimeoutState::Activity(Instant::now())
            }

            Left(Left(Left(Left(Right(Some(_activity)))))) => {
                *state = TimeoutState::activity();
            }

            Left(Left(Left(Right(Some(ping))))) => {
                let token = ping.token();
                log::debug!(
                    "got a ping from the server. responding with token '{}'",
                    token
                );
                let pong = crate::commands::pong(token);
                writer.encode(pong).await?;
                *state = TimeoutState::activity();
            }

            Left(Left(Right(Some(_pong)))) => {
                if let TimeoutState::WaitingForPong(_ts) = state {
                    *state = TimeoutState::activity();
                }
            }

            Left(Right(Some(write))) => {
                let msg = std::str::from_utf8(&write).unwrap().escape_debug();
                log::trace!("> {}", msg);
                writer.encode(write).await?;
            }

            Right(_timeout) => {
                log::info!("idle connection detected, sending a ping");
                let ts = timestamp().to_string();
                writer.encode(crate::commands::ping(&ts)).await?;
                *state = TimeoutState::waiting_for_pong();
            }

            _ => return Ok(StepResult::Break(Status::Eof)),
        }

        match state {
            TimeoutState::WaitingForPong(dt) => {
                if dt.elapsed() > TIMEOUT {
                    log::warn!("PING timeout detected, exiting");
                    return Ok(StepResult::Break(Status::TimedOut));
                }
            }
            TimeoutState::Activity(dt) => {
                if dt.elapsed() > WINDOW {
                    log::warn!("idle connectiond detected, sending a PING");
                    let ts = timestamp().to_string();
                    writer.encode(crate::commands::ping(&ts)).await?;
                    *state = TimeoutState::waiting_for_pong();
                }
            }
            TimeoutState::Start => {}
        }

        Ok(StepResult::Continue)
    }

    async fn register<W>(
        user_config: &UserConfig,
        writer: &mut AsyncEncoder<W>,
    ) -> Result<(), Error>
    where
        W: AsyncWrite + Send + Sync + Unpin,
    {
        for cap in &user_config.capabilities {
            log::info!("sending capability: '{}'", cap.encode_as_str());
        }

        log::info!(
            "sending PASS '{}' (redacted)",
            "*".repeat(user_config.token.len())
        );
        log::info!("sending NICK '{}'", &user_config.name);

        // register with the connection
        writer
            .encode(crate::commands::register(user_config))
            .await?;

        Ok(())
    }
}

struct StepState<C>
where
    C: Connector,
{
    ping: EventStream<Ping<'static>>,
    pong: EventStream<Pong<'static>>,

    reader: AsyncDecoder<async_dup::Arc<C::Output>>,
    writer: AsyncEncoder<async_dup::Arc<C::Output>>,

    state: TimeoutState,
}

impl<C> StepState<C>
where
    C: Connector,
    C::Output: AsyncRead + AsyncWrite + Unpin + Send + Sync,
    for<'a> &'a C::Output: AsyncRead + AsyncWrite + Unpin + Send + Sync,
{
    async fn build(runner: &mut AsyncRunner<C>) -> Self {
        let stream = runner
            .stream
            .as_ref()
            .map(Clone::clone)
            .expect("connect must be called first");

        let (reader, writer) = (
            AsyncDecoder::new(stream.clone()), //
            AsyncEncoder::new(stream),
        );

        let ping = runner.dispatcher.subscribe_system().await;
        let pong = runner.dispatcher.subscribe_system().await;

        Self {
            ping,
            pong,
            reader,
            writer,
            state: TimeoutState::Start,
        }
    }
}

enum StepResult {
    Continue,
    Break(Status),
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
