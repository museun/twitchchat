use crate::{
    channel::Receiver,
    commands,
    connector::Connector,
    messages::{AllCommands, Capability, Join, MessageId, Part},
    rate_limit::RateLimit,
    util::{Either, Notify, NotifyHandle},
    writer::{AsyncWriter, MpscWriter},
    AsyncDecoder, AsyncEncoder, DecodeError, Encodable, FromIrcMessage, RateClass, UserConfig,
};

use super::{
    channel::Channels,
    timeout::{TimeoutState, RATE_LIMIT_WINDOW, TIMEOUT, WINDOW},
    Capabilities, Channel, Error, Identity, Status, StepResult,
};

use futures_lite::{AsyncRead, AsyncWrite, AsyncWriteExt};
use std::collections::{HashSet, VecDeque};

/// An asynchronous runner
pub struct AsyncRunner {
    /// You identity that Twitch gives when you connected
    pub identity: Identity,

    channels: Channels,

    activity_rx: Receiver<()>,
    writer_rx: Receiver<Box<[u8]>>,

    notify: Notify,
    // why don't we use this?
    notify_handle: NotifyHandle,

    timeout_state: TimeoutState,

    decoder: AsyncDecoder<Box<dyn AsyncRead + Send + Sync + Unpin>>,
    encoder: AsyncEncoder<Box<dyn AsyncWrite + Send + Sync + Unpin>>,

    writer: AsyncWriter<MpscWriter>,
    global_rate_limit: RateLimit,

    missed_messages: VecDeque<AllCommands<'static>>,
}

impl std::fmt::Debug for AsyncRunner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncRunner { .. }").finish()
    }
}

impl AsyncRunner {
    /// Connect with the provided connector and the provided UserConfig
    ///
    /// This returns the Runner with your identity set.    
    pub async fn connect<C>(connector: C, user_config: &UserConfig) -> Result<Self, Error>
    where
        C: Connector,
        for<'a> &'a C::Output: AsyncRead + AsyncWrite + Send + Sync + Unpin,
    {
        log::info!("connecting");
        let mut stream = { connector }.connect().await?;
        log::info!("connection established");

        log::info!("registering");
        let mut buf = vec![];
        commands::register(user_config).encode(&mut buf)?;
        stream.write_all(&buf).await?;
        log::info!("registered");

        let read = async_dup::Arc::new(stream);
        let write = read.clone();

        let read: Box<dyn AsyncRead + Send + Sync + Unpin> = Box::new(read);
        let write: Box<dyn AsyncWrite + Send + Sync + Unpin> = Box::new(write);

        let mut decoder = AsyncDecoder::new(read);
        let mut encoder = AsyncEncoder::new(write);

        log::info!("waiting for the connection to be ready");
        let identity = Self::wait_for_ready(&mut decoder, &mut encoder, user_config).await?;
        log::info!("connection is ready: {:?}", identity);

        let (writer_tx, writer_rx) = crate::channel::bounded(64);
        let (notify, notify_handle) = Notify::new();
        let (activity_tx, activity_rx) = crate::channel::bounded(32);

        let writer = AsyncWriter::new(MpscWriter::new(writer_tx), activity_tx);

        let timeout_state = TimeoutState::Start;
        let (channels, missed_messages) = <_>::default();

        let global_rate_limit = RateLimit::from_class(RateClass::Regular);

        Ok(Self {
            identity,
            channels,

            activity_rx,
            writer_rx,

            notify,
            notify_handle,

            timeout_state,

            decoder,
            encoder,

            writer,
            global_rate_limit,

            missed_messages,
        })
    }

    /// Check whether you're on this channel
    pub fn is_on_channel(&self, channel: &str) -> bool {
        self.channels.is_on(channel)
    }

    /// Get a specific channel.
    ///
    /// This is useful for changing the rate limit/state manually.
    pub fn get_channel_mut(&mut self, channel: &str) -> Option<&mut Channel> {
        self.channels.get_mut(channel)
    }

    /// Get a clonable writer you can use
    pub fn writer(&self) -> AsyncWriter<MpscWriter> {
        self.writer.clone()
    }

    /// Get a handle that you can trigger a normal 'quit'.
    ///
    /// You can also do `AsyncWriter::quit`.
    pub fn quit_handle(&self) -> NotifyHandle {
        self.notify_handle.clone()
    }

    /// Join `channel` and wait for it to complete    
    pub async fn join(&mut self, channel: &str) -> Result<(), Error> {
        if self.is_on_channel(channel) {
            return Err(Error::AlreadyOnChannel {
                channel: channel.to_string(),
            });
        }

        log::info!("joining '{}'", channel);
        self.encoder.encode(commands::join(channel)).await?;

        log::debug!("waiting for a response");
        let channel_and_us = |msg: &Join<'_>, this: &AsyncRunner| {
            msg.channel() == channel && msg.name() == this.identity.username()
        };
        self.wait_for_cmd(
            |cmd| match cmd {
                AllCommands::Join(msg) => Some(msg),
                _ => None,
            },
            channel_and_us,
        )
        .await?;
        log::info!("joined '{}'", channel);

        Ok(())
    }

    /// Part `channel` and wait for it to complete
    pub async fn part(&mut self, channel: &str) -> Result<(), Error> {
        if !self.is_on_channel(channel) {
            return Err(Error::NotOnChannel {
                channel: channel.to_string(),
            });
        }

        log::info!("leaving '{}'", channel);
        self.encoder.encode(commands::part(channel)).await?;

        log::debug!("waiting for a response");
        let channel_and_us = |msg: &Part<'_>, this: &AsyncRunner| {
            msg.channel() == channel && msg.name() == this.identity.username()
        };
        self.wait_for_cmd(
            |cmd| match cmd {
                AllCommands::Part(msg) => Some(msg),
                _ => None,
            },
            channel_and_us,
        )
        .await?;
        log::info!("left '{}'", channel);

        Ok(())
    }

    /// Get the next message. You'll usually want to call this in a loop
    pub async fn next_message(&mut self) -> Result<Status<'static>, Error> {
        use crate::util::{Either::*, FutExt as _};

        loop {
            match self.step().await? {
                StepResult::Nothing => continue,
                StepResult::Status(Status::Quit) => {
                    if let Left(_notified) = self.notify.wait().now_or_never().await {
                        // close everything
                        self.writer_rx.close();
                        self.activity_rx.close();

                        // and then drain any remaining items
                        while self.available_queued_messages() > 0 {
                            self.drain_queued_messages().await?;
                            futures_lite::future::yield_now().await;
                        }

                        // and finally send the quit
                        self.encoder.encode(commands::raw("QUIT\r\n")).await?;

                        // and signal that we've quit
                        break Ok(Status::Quit);
                    }
                }
                StepResult::Status(status) => break Ok(status),
            }
        }
    }

    /// Single step the loop. This is useful for testing.
    pub async fn step(&mut self) -> Result<StepResult<'static>, Error> {
        use crate::util::*;
        use crate::IntoOwned as _;

        if let Some(msg) = self.missed_messages.pop_front() {
            return Ok(StepResult::Status(Status::Message(msg)));
        }

        let select = self
            .decoder
            .read_message()
            .either(self.activity_rx.recv())
            .either(self.writer_rx.recv())
            .either(self.notify.wait())
            .either(super::timeout::next_delay())
            .await;

        match select {
            Left(Left(Left(Left(msg)))) => {
                let msg = match msg {
                    Err(DecodeError::Eof) => {
                        log::info!("got an EOF, exiting main loop");
                        return Ok(StepResult::Status(Status::Eof));
                    }
                    Err(err) => {
                        log::warn!("read an error: {}", err);
                        return Err(err.into());
                    }
                    Ok(msg) => msg,
                };

                self.timeout_state = TimeoutState::activity();

                let all = AllCommands::from_irc(msg) //
                    .expect("msg identity conversion should be upheld")
                    .into_owned();

                self.check_messages(&all).await?;

                return Ok(StepResult::Status(Status::Message(all)));
            }

            Left(Left(Left(Right(Some(_activity))))) => {
                self.timeout_state = TimeoutState::activity();
            }

            Left(Left(Right(Some(write_data)))) => {
                // TODO provide a 'bytes' flavored parser
                let msg = std::str::from_utf8(&*write_data).map_err(Error::InvalidUtf8)?;
                let msg = crate::IrcMessage::parse(crate::Str::Borrowed(msg))
                    .expect("encoder should produce valid IRC messages");

                if let crate::IrcMessage::PRIVMSG = msg.get_command() {
                    if let Some(ch) = msg.nth_arg(0) {
                        if !self.channels.is_on(ch) {
                            self.channels.add(ch)
                        }

                        let ch = self.channels.get_mut(ch).unwrap();
                        if ch.rated_limited_at.map(|s| s.elapsed()) > Some(RATE_LIMIT_WINDOW) {
                            ch.reset_rate_limit();
                        }

                        ch.rate_limited.enqueue(write_data)
                    }
                }
            }

            Left(Right(_notified)) => return Ok(StepResult::Status(Status::Quit)),

            Right(_timeout) => {
                log::info!("idle connection detected, sending a ping");
                let ts = timestamp().to_string();
                self.encoder.encode(commands::ping(&ts)).await?;
                self.timeout_state = TimeoutState::waiting_for_pong();
            }

            _ => {
                return Ok(StepResult::Status(Status::Eof));
            }
        }

        match self.timeout_state {
            TimeoutState::WaitingForPong(dt) => {
                if dt.elapsed() > TIMEOUT {
                    log::warn!("PING timeout detected, exiting");
                    return Err(Error::TimedOut);
                }
            }
            TimeoutState::Activity(dt) => {
                if dt.elapsed() > WINDOW {
                    log::warn!("idle connectiond detected, sending a PING");
                    let ts = timestamp().to_string();
                    self.encoder.encode(crate::commands::ping(&ts)).await?;
                    self.timeout_state = TimeoutState::waiting_for_pong();
                }
            }
            TimeoutState::Start => {}
        }

        log::trace!("draining messages");
        self.drain_queued_messages().await?;

        Ok(StepResult::Nothing)
    }

    async fn check_messages(&mut self, all: &AllCommands<'static>) -> Result<(), Error> {
        use {AllCommands::*, TimeoutState::*};

        match &all {
            Ping(msg) => {
                let token = msg.token();
                log::debug!(
                    "got a ping from the server. responding with token '{}'",
                    token
                );
                self.encoder.encode(commands::pong(token)).await?;
                self.timeout_state = TimeoutState::activity();
            }

            Pong(..) if matches!(self.timeout_state, WaitingForPong {..}) => {
                self.timeout_state = TimeoutState::activity()
            }

            Join(msg) if msg.name() == self.identity.username() => {
                log::debug!("starting tracking channel for '{}'", msg.channel());
                self.channels.add(msg.channel());
            }

            Part(msg) if msg.name() == self.identity.username() => {
                log::debug!("stopping tracking of channel '{}'", msg.channel());
                self.channels.remove(msg.channel());
            }

            RoomState(msg) => {
                if let Some(dur) = msg.is_slow_mode() {
                    if let Some(ch) = self.channels.get_mut(msg.channel()) {
                        ch.enable_slow_mode(dur)
                    }
                }
            }

            Notice(msg) => {
                let ch = self.channels.get_mut(msg.channel());
                match (msg.msg_id(), ch) {
                    // we should enable slow mode
                    (Some(MessageId::SlowOn), Some(ch)) => ch.enable_slow_mode(30),
                    // we should disable slow mode
                    (Some(MessageId::SlowOff), Some(ch)) => ch.disable_slow_mode(),
                    // we've been rate limited on the channel
                    (Some(MessageId::MsgRatelimit), Some(ch)) => ch.set_rate_limited(),
                    // we cannot join/send to the channel because we're banned
                    (Some(MessageId::MsgBanned), ..) => self.channels.remove(msg.channel()),
                    _ => {}
                }
            }

            Reconnect(_) => return Err(Error::ShouldReconnect),

            _ => {}
        }

        Ok(())
    }
}

type WaitFor<T> = Either<VecDeque<T>, Status<'static>>;

impl AsyncRunner {
    async fn wait_for_cmd<F, T, E>(&mut self, extract: F, cmp: E) -> Result<(), Error>
    where
        for<'a> F: Fn(&'a AllCommands<'static>) -> Option<&'a T>,
        E: Fn(&T, &Self) -> bool,

        // for clippy
        F: Send + Sync,
        T: Send + Sync,
        E: Send + Sync,
    {
        let messages = self
            .wait_for(|msg, this| extract(msg).map(|msg| cmp(msg, this)).unwrap_or(false))
            .await?;

        match messages {
            Either::Left(messages) => self.missed_messages.extend(messages),
            Either::Right(res) => match res {
                Status::Quit | Status::Eof => return Err(Error::UnexpectedEof),
                _ => unimplemented!(),
            },
        }

        Ok(())
    }

    async fn wait_for<F>(&mut self, func: F) -> Result<WaitFor<AllCommands<'static>>, Error>
    where
        F: Fn(&AllCommands<'static>, &Self) -> bool,

        // for clippy
        F: Send + Sync,
    {
        let mut missed_messages = VecDeque::new();
        loop {
            match self.step().await? {
                StepResult::Status(Status::Message(msg)) => {
                    if func(&msg, self) {
                        break Ok(Either::Left(missed_messages));
                    }
                    missed_messages.push_back(msg);
                }
                StepResult::Status(d) => return Ok(Either::Right(d)),
                StepResult::Nothing => futures_lite::future::yield_now().await,
            }
        }
    }

    fn available_queued_messages(&self) -> usize {
        self.channels
            .map
            .values()
            .map(|s| s.rate_limited.queue.len())
            .sum()
    }

    async fn drain_queued_messages(&mut self) -> std::io::Result<()> {
        let enc = &mut self.encoder;
        let limit = &mut self.global_rate_limit.get_available_tokens();

        let start = *limit;
        // log::error!("available tokens: {}", limit);

        // for each channel, try to take up to 'limit' tokens
        for channel in self.channels.map.values_mut() {
            if channel.rated_limited_at.map(|s| s.elapsed()) > Some(RATE_LIMIT_WINDOW) {
                channel.reset_rate_limit();
            }

            // drain until we're out of messages, or tokens
            channel
                .rate_limited
                .drain_until_blocked(&channel.name, limit, enc)
                .await?;

            let diff = start - *limit;
            // log::error!("'{}' took '{}' tokens", channel.name, diff);

            if *limit == 0 {
                log::warn!(target: "twitchchat::rate_limit", "global rate limit hit while draining '{}'", &channel.name);
                break;
            }

            // and throttle the global one
            match self.global_rate_limit.consume(diff) {
                // use the new remaining amount of tokens
                Ok(rem) => *limit = rem,

                // we're globally rate limited, so just return
                Err(..) => {
                    log::warn!(target: "twitchchat::rate_limit", "global rate limit hit while draining '{}'", &channel.name);
                    break;
                }
            }
        }

        Ok(())
    }

    async fn wait_for_ready<R, W>(
        decoder: &mut AsyncDecoder<R>,
        encoder: &mut AsyncEncoder<W>,
        user_config: &UserConfig,
    ) -> Result<Identity, Error>
    where
        R: AsyncRead + Send + Sync + Unpin,
        W: AsyncWrite + Send + Sync + Unpin,
    {
        let is_anonymous = user_config.is_anonymous();

        let mut looking_for: HashSet<_> = user_config.capabilities.iter().collect();
        let mut caps = Capabilities::default();
        let mut our_name = None;

        let identity = loop {
            let msg = decoder.read_message().await?;

            // this should always be infallible. its not marked infallible
            // because of the 'non-exhaustive' attribute
            use AllCommands::*;
            match AllCommands::from_irc(msg).unwrap() {
                Ready(msg) => {
                    our_name.replace(msg.username().to_string());

                    // if we aren't going to be receiving tags, then we
                    // won't be looking for any more messages

                    // if we're anonymous, we won't get GLOBALUSERSTATE even
                    // if we do send Tags
                    if is_anonymous {
                        break Identity::Anonymous { caps };
                    };

                    if !caps.tags {
                        break Identity::Basic {
                            name: our_name.take().unwrap(),
                            caps,
                        };
                    }
                }

                Cap(msg) => match msg.capability() {
                    Capability::Acknowledged(name) => {
                        use crate::Capability as Cap;

                        let cap = match Cap::maybe_from_str(name) {
                            Some(cap) => cap,
                            // Twitch sent us an unknown capability
                            None => {
                                caps.unknown.insert(name.to_string());
                                continue;
                            }
                        };

                        *match cap {
                            Cap::Tags => &mut caps.tags,
                            Cap::Membership => &mut caps.membership,
                            Cap::Commands => &mut caps.commands,
                        } = true;

                        looking_for.remove(&cap);
                    }

                    Capability::NotAcknowledged(name) => {
                        return Err(Error::InvalidCap {
                            cap: name.to_string(),
                        })
                    }
                },

                GlobalUserState(msg) => {
                    break Identity::Full {
                        name: our_name.unwrap(),
                        user_id: msg.user_id.parse().unwrap(),
                        display_name: msg.display_name.map(|s| s.to_string()),
                        color: msg.color,
                        caps,
                    }
                }

                // Reply to any PINGs while waiting. Although Twitch doesn't
                // currently send a PING for spoof detection on initial
                // handshake, one day they may. Most IRC servers do this
                // already
                Ping(msg) => encoder.encode(commands::pong(msg.token())).await?,

                _ => {}
            }
        };

        Ok(identity)
    }
}
