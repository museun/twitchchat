use crate::{
    channel,
    connector::Connector,
    messages::*,
    rate_limit::*,
    runner::*,
    util::*,
    writer::{AsyncWriter, MpscWriter},
    *,
};

use futures_lite::{AsyncRead, AsyncWrite, Stream, StreamExt};
use futures_timer::Delay;

use super::{wait_for::CapsStatus, RateLimitedChannels, WaitFor};
use async_mutex::Mutex;
use simple_event_map::EventStream;
use std::{
    collections::HashSet,
    future::Future,
    sync::Arc,
    time::{Duration, Instant},
};

const WINDOW: Duration = Duration::from_secs(45);
const TIMEOUT: Duration = Duration::from_secs(10);

/// An Error returned by [`AsyncRunner::join][join`] or [`AsyncRunner::part`][part]
///
/// [join]: ./struct.AsyncRunner.html#method.join
/// [part]: ./struct.AsyncRunner.html#method.part
#[derive(Debug)]
#[non_exhaustive]
pub enum ChannelError {
    /// You could not leave this channel because you were not on it.
    NotOnChannel(String),
    /// You could not join this channel because you were already on it.
    AlreadyOnChannel(String),
}

impl std::fmt::Display for ChannelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotOnChannel(channel) => write!(f, "Not on channel: {}", channel),
            Self::AlreadyOnChannel(channel) => write!(f, "Already on channel: {}", channel),
        }
    }
}

impl std::error::Error for ChannelError {}

/// An async runner. This will act as a main loop, if you want one.
#[derive(Clone)]
pub struct AsyncRunner<C>
where
    C: Connector,
{
    dispatcher: AsyncDispatcher,
    writer: AsyncWriter<MpscWriter>,

    writer_rx: Receiver<Box<[u8]>>,
    activity_rx: Receiver<()>,
    quit_handle: NotifyHandle,
    user_config: UserConfig,

    inner: Arc<Mutex<Inner<C>>>,
}

struct Inner<C>
where
    C: Connector,
{
    quit_notify: Notify,

    wait_for: WaitFor,
    rate_limit: RateLimitedChannels,

    channels: HashSet<String>,

    connector: C,
    stream: Option<async_dup::Arc<C::Output>>,

    // this is the name twitch gave us
    user_name: Option<String>,
}

impl<C> std::fmt::Debug for AsyncRunner<C>
where
    C: Connector,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncRunner").finish()
    }
}

impl<C> AsyncRunner<C>
where
    C: Connector,
    for<'a> &'a C::Output: AsyncRead + AsyncWrite + Send + Sync,
{
    /// Create a new async runner with this dispatcher
    pub fn new(
        dispatcher: AsyncDispatcher,
        user_config: UserConfig,
        rate_class: RateClass,
        connector: C,
    ) -> Self {
        let (writer_tx, writer_rx) = channel::bounded(64);
        let (activity_tx, activity_rx) = channel::bounded(32);

        let (quit_notify, quit_handle) = Notify::new();

        let writer = MpscWriter::new(writer_tx);
        let writer = AsyncWriter::new(writer, activity_tx, quit_handle.clone());

        let wait_for = WaitFor::new(user_config.is_anonymous());
        let rate_limit = RateLimitedChannels::new(RateLimit::from_class(rate_class));

        let inner = Inner {
            quit_notify,
            wait_for,
            rate_limit,
            channels: HashSet::new(),
            connector,
            stream: None,
            user_name: None,
        };

        Self {
            dispatcher,
            writer,

            writer_rx,
            activity_rx,
            quit_handle,
            user_config,

            inner: Arc::new(Mutex::new(inner)),
        }
    }

    /// Get a **clonable** `Writer`.
    pub fn writer(&self) -> AsyncWriter<MpscWriter> {
        self.writer.clone()
    }

    /// Attempts to join the provided channel. Blocking until you join
    ///
    /// This returns an error if you were already on the channel.
    pub async fn join(&mut self, channel: &str) -> Result<(), ChannelError> {
        if self.inner.lock().await.channels.contains(channel) {
            return Err(ChannelError::AlreadyOnChannel(channel.to_string()));
        }

        self.writer.encode(commands::join(channel)).await.unwrap();

        // this will deadlock if they aren't running the main loop in another
        // thread
        while !self.inner.lock().await.channels.contains(channel) {
            futures_lite::future::yield_now().await;
        }

        Ok(())
    }

    /// Attempts to leave the provided channel. Blocking until you leave
    ///
    /// This returns an error if you weren't on the channel
    pub async fn part(&mut self, channel: &str) -> Result<(), ChannelError> {
        if !self.inner.lock().await.channels.contains(channel) {
            return Err(ChannelError::NotOnChannel(channel.to_string()));
        }

        self.writer.encode(commands::part(channel)).await.unwrap();

        // this will deadlock if they aren't running the main loop in another
        // thread
        while self.inner.lock().await.channels.contains(channel) {
            futures_lite::future::yield_now().await;
        }

        Ok(())
    }

    /// Get a borrow of the dispatcher
    pub fn dispatcher(&self) -> &AsyncDispatcher {
        &self.dispatcher
    }

    /// Get a handle you can use to notify that the main loop should exit early.
    pub fn quit_signal(&self) -> NotifyHandle {
        self.quit_handle.clone()
    }

    /// Set the global rate limiter 'class'. This is used for 'general' messages
    ///
    /// If you don't have special bot status, don't change this.
    pub async fn set_custom_rate_limit(&mut self, rate_class: RateClass) {
        self.inner
            .lock()
            .await
            .rate_limit
            .set_global_rate_limit(rate_class);
    }

    /// Attempts to set a specific rate class for a channel.
    ///
    /// This returns 'false' if the AsyncRunner is not tracking that channel.
    ///
    /// # Note
    /// If you abuse/misuse this Twitch can/may ban you.
    ///
    /// If you're not a 'VIP' or 'Moderator' on the channel, or you don't have a special bot status ***don't use this***.    
    pub async fn set_rate_class_for_channel(
        &mut self,
        channel: &str,
        rate_class: RateClass,
    ) -> bool {
        match self.inner.lock().await.rate_limit.get_channel(channel) {
            Some(ch) => {
                ch.change_rate_class(rate_class);
                true
            }
            None => false,
        }
    }

    /// Wait for a specific message.
    ///
    /// This acts as a temporary main loop.
    ///
    /// You'd use this to wait for an event to come in before switching to the
    /// 'run' style loops.
    pub async fn wait_for_ready<T>(&mut self) -> Result<T, RunnerError>
    where
        T: ReadyMessage<'static> + Send + Sync + 'static,
        DispatchError: From<T::Error>,
    {
        let mut inner = self.inner.lock().await;
        log::debug!("registering to wait for '{}'", trim_type_name::<T>());
        inner.wait_for.register::<T>()?;

        let should_register = match inner.stream.is_some() {
            true => false,
            false => {
                log::debug!(
                    "initializing stream in wait_for_ready ({})",
                    trim_type_name::<T>(),
                );
                let stream = inner.connector.connect().await?;
                let stream = async_dup::Arc::new(stream);
                inner.stream.replace(stream);
                true
            }
        };

        drop(inner);

        let mut step_state = StepState::build(self.clone(), &mut self.dispatcher.clone()).await;
        if should_register {
            Self::register(&self.user_config, &mut step_state.writer).await?;
        }

        loop {
            match self.inner.lock().await.wait_for.check_queue::<T>() {
                CapsStatus::RequiredCap(cap) => {
                    log::error!(
                        "we reached ready state, but we will never get this message ('{}') because the '{:?}' capability isn't acknowledged", 
                        trim_type_name::<T>(),
                        cap
                    );
                    return Err(RunnerError::RequiredCaps(cap, trim_type_name::<T>()));
                }

                CapsStatus::Seen(msg) => {
                    log::debug!("done waiting for '{}'", trim_type_name::<T>(),);
                    return T::from_irc(msg)
                        .map_err(DispatchError::from)
                        .map_err(Into::into);
                }

                CapsStatus::NotSeen => {}
            }

            match self.step(&mut step_state).await? {
                StepResult::Continue => {}
                // we got a break signal for an error, report it
                StepResult::Break(_) => {
                    log::warn!("breaking out of a wait_for_message -- this means the connection was closed early.");
                    return Err(RunnerError::WaitForMessage(trim_type_name::<T>()));
                }
            }
        }
    }

    /// Using this connector try to reconnect based on the Retry strategy
    ///
    /// This will act like run to completion in a loop with a configurable
    /// criteria for when a reconnect should happen.
    pub async fn run_with_retry<Retry, Fut>(&mut self, retry: Retry) -> Result<(), Error>
    where
        Retry: Fn(Result<Status, Error>) -> Fut + Send + Sync,
        Fut: Future<Output = Result<bool, Error>> + Send + Sync,
    {
        self.run_with_retry_advanced(retry, |_| async move { Ok(()) }, None)
            .await
    }

    /// Using this connector, retry strategy and reset config try to reconnect
    /// based on the retry strategy.
    ///
    /// The Connect async closure allows you 'wait' for events before the main loop starts.
    ///
    /// This will act like run to completion in a loop with a configurable
    /// criteria for when a reconnect should happen.
    ///
    /// The reset configuration allows you to determine (and have a way to be
    /// notified when you should resubscribe, if you want to.)
    pub async fn run_with_retry_advanced<Retry, RetryRes, Connect, ConnectRes, Reset>(
        &mut self,
        retry: Retry,
        on_connect: Connect,
        reset_config: Reset,
    ) -> Result<(), Error>
    where
        Retry: Fn(Result<Status, Error>) -> RetryRes + Send + Sync,
        RetryRes: Future<Output = Result<bool, Error>> + Send + Sync,

        Connect: Fn(&mut Self) -> ConnectRes + Send + Sync,
        ConnectRes: Future<Output = std::io::Result<()>> + Send + Sync,

        Reset: Into<Option<ResetConfig>> + Send + Sync,
    {
        let mut reset_config = reset_config.into();

        loop {
            on_connect(self).await?;

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

            let mut inner = self.inner.lock().await;
            inner.wait_for = WaitFor::new(self.user_config.is_anonymous());
            inner.stream.take();
        }
    }

    /// Using this connector, run the loop to completion.
    pub async fn run_to_completion(&mut self) -> Result<Status, Error> {
        let should_register = {
            let mut inner = self.inner.lock().await;
            match inner.stream.is_some() {
                true => false,
                false => {
                    log::debug!("initializing stream in run_to_completion",);
                    let stream = inner.connector.connect().await?;
                    let stream = async_dup::Arc::new(stream);
                    inner.stream.replace(stream);
                    true
                }
            }
        };

        let mut step_state = StepState::build(self.clone(), &mut self.dispatcher).await;

        if should_register {
            Self::register(&self.user_config, &mut step_state.writer).await?;
        }

        let status = loop {
            match self.step(&mut step_state).await? {
                StepResult::Continue => {}
                StepResult::Break(done) => break done,
            }
        };

        // and see if we triggered it
        match self
            .inner
            .lock()
            .await
            .quit_notify
            .wait()
            .either(futures_lite::future::ready(()))
            .await
        {
            Left(_cancelled) => Ok(Status::Cancelled),
            Right(_nope) => Ok(status),
        }
    }

    async fn step(&mut self, step_state: &mut StepState<C>) -> Result<StepResult, Error> {
        let StepState {
            events,
            reader,
            writer,
            state,
        } = step_state;

        let select = reader
            .read_message()
            .either(self.activity_rx.recv())
            .either(events.ping.next())
            .either(events.pong.next())
            .either(events.notice.next())
            .either(events.join.next())
            .either(events.part.next())
            .either(events.ready.next())
            .either(events.roomstate.next())
            .either(self.writer_rx.recv())
            .either(Delay::new(WINDOW))
            .await;

        match select {
            Left(Left(Left(Left(Left(Left(Left(Left(Left(Left(read)))))))))) => {
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

                self.inner.lock().await.wait_for.maybe_add(&msg);
                self.dispatcher.dispatch(msg).await?;
                *state = TimeoutState::Activity(Instant::now())
            }

            Left(Left(Left(Left(Left(Left(Left(Left(Left(Right(Some(_activity))))))))))) => {
                *state = TimeoutState::activity();
            }

            Left(Left(Left(Left(Left(Left(Left(Left(Right(Some(ping)))))))))) => {
                let token = ping.token();
                log::debug!(
                    "got a ping from the server. responding with token '{}'",
                    token
                );
                let pong = crate::commands::pong(token);
                writer.encode(pong).await?;
                *state = TimeoutState::activity();
            }

            Left(Left(Left(Left(Left(Left(Left(Right(Some(_pong))))))))) => {
                if let TimeoutState::WaitingForPong(_ts) = state {
                    *state = TimeoutState::activity();
                }
            }

            Left(Left(Left(Left(Left(Left(Right(Some(notice)))))))) => {
                let channel = notice.channel();
                match notice.msg_id() {
                    Some(MessageId::SlowOn) => {
                        self.inner.lock().await.rate_limit.enable_slow(channel);
                    }

                    Some(MessageId::SlowOff) => {
                        self.inner.lock().await.rate_limit.disable_slow(channel);
                    }

                    Some(MessageId::MsgRatelimit) => {
                        self.inner
                            .lock()
                            .await
                            .rate_limit
                            .rate_limited_on_channel(channel);
                    }

                    // we already handle slow mode duration via an external event
                    _ => {}
                }
            }

            Left(Left(Left(Left(Left(Right(Some(join))))))) => {
                let mut inner = self.inner.lock().await;
                let user_name = inner
                    .user_name
                    .as_deref()
                    .ok_or_else(|| RunnerError::InvalidConnection)?;

                if join.name() == user_name {
                    log::debug!("starting tracking channel for '{}'", join.channel());
                    inner.channels.insert(join.channel().to_string());
                    // this uses the default rate limiter, they can adjust it later
                    inner.rate_limit.add_channel(join.channel(), <_>::default());
                }
            }

            Left(Left(Left(Left(Right(Some(part)))))) => {
                let mut inner = self.inner.lock().await;

                let user_name = inner
                    .user_name
                    .as_deref()
                    .ok_or_else(|| RunnerError::InvalidConnection)?;

                if part.name() == user_name {
                    log::debug!("stopping tracking of channel '{}'", part.channel());
                    inner.channels.remove(part.channel());
                    // drop any queued messages if we're not on the channel
                    inner.rate_limit.remove_channel(part.channel());
                }
            }

            Left(Left(Left(Right(Some(ready))))) => {
                log::info!("setting our internal username to: {}", ready.username());
                self.inner
                    .lock()
                    .await
                    .user_name
                    .replace(ready.username().to_string());
            }

            Left(Left(Right(Some(roomstate)))) => {
                if let Some(n) = roomstate.is_slow_mode() {
                    let channel = roomstate.channel();
                    self.inner
                        .lock()
                        .await
                        .rate_limit
                        .set_slow_duration(channel, n);
                }
            }

            Left(Right(Some(write_data))) => {
                let msg = std::str::from_utf8(&write_data).unwrap();
                log::trace!("> {}", msg.escape_debug());

                match IrcMessage::parse(crate::Str::Borrowed(msg)) {
                    // if we're sending a PRIVMSG
                    Ok(msg) if msg.get_command() == IrcMessage::PRIVMSG => {
                        // and its a valid PRIVMSG
                        if let Some(channel) = msg.nth_arg(0) {
                            let channel = channel.to_string();
                            // queue the message on that channel
                            self.inner
                                .lock()
                                .await
                                .rate_limit
                                .enqueue_for(channel, write_data, writer)
                                .await?;
                        } else {
                            // otherwise use the global queue
                            self.inner
                                .lock()
                                .await
                                .rate_limit
                                .global_enqueue(write_data, writer)
                                .await?
                        }
                    }
                    // otherwise use the global queue
                    _ => {
                        self.inner
                            .lock()
                            .await
                            .rate_limit
                            .global_enqueue(write_data, writer)
                            .await?
                    }
                }
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
    for<'a> &'a C::Output: AsyncRead + AsyncWrite + Sync + Send,
{
    events: Events,

    reader: AsyncDecoder<async_dup::Arc<C::Output>>,
    writer: AsyncEncoder<async_dup::Arc<C::Output>>,

    state: TimeoutState,
}

impl<C> StepState<C>
where
    C: Connector,
    for<'a> &'a C::Output: AsyncRead + AsyncWrite + Send + Sync,
{
    async fn build(runner: AsyncRunner<C>, dispatcher: &mut AsyncDispatcher) -> Self {
        let stream = runner
            .inner
            .lock()
            .await
            .stream
            .as_ref()
            .map(Clone::clone)
            .expect("connect must be called first");

        let events = Events::build(dispatcher).await;

        let (reader, writer) = (
            AsyncDecoder::new(stream.clone()), //
            AsyncEncoder::new(stream),
        );

        Self {
            events,

            reader,
            writer,
            state: TimeoutState::Start,
        }
    }
}

struct Events {
    ping: EventStream<Ping<'static>>,
    pong: EventStream<Pong<'static>>,
    notice: EventStream<Notice<'static>>,
    join: EventStream<Join<'static>>,
    part: EventStream<Part<'static>>,
    ready: EventStream<Ready<'static>>,
    roomstate: EventStream<RoomState<'static>>,
}

impl Events {
    async fn build(dispatcher: &mut AsyncDispatcher) -> Self {
        let ping = dispatcher.subscribe_system().await;
        let pong = dispatcher.subscribe_system().await;
        let notice = dispatcher.subscribe_system().await;
        let join = dispatcher.subscribe_system().await;
        let part = dispatcher.subscribe_system().await;
        let ready = dispatcher.subscribe_system().await;
        let roomstate = dispatcher.subscribe_system().await;

        Self {
            ping,
            pong,
            notice,
            join,
            part,
            ready,
            roomstate,
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
