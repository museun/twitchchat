use crate::{
    channel::Receiver,
    commands,
    connector::Connector,
    decoder::{AsyncDecoder, DecodeError},
    encoder::{AsyncEncoder, Encodable},
    messages::{Capability, Commands, MessageId},
    rate_limit::{RateClass, RateLimit},
    twitch::UserConfig,
    util::{Notify, NotifyHandle},
    writer::{AsyncWriter, MpscWriter},
    FromIrcMessage,
};

use super::{
    channel::Channels,
    timeout::{TimeoutState, RATE_LIMIT_WINDOW, TIMEOUT, WINDOW},
    Capabilities, Channel, Error, Identity, Status, StepResult,
};

use futures_lite::{AsyncRead, AsyncWrite, AsyncWriteExt, Stream};
use std::{
    collections::{HashSet, VecDeque},
    pin::Pin,
    task::{Context, Poll},
};

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

    missed_messages: VecDeque<Commands<'static>>,
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
        log::debug!("connecting");
        let mut stream = { connector }.connect().await?;
        log::debug!("connection established");

        log::debug!("registering");
        let mut buf = vec![];
        commands::register(user_config).encode(&mut buf)?;
        stream.write_all(&buf).await?;
        log::debug!("registered");

        let read = async_dup::Arc::new(stream);
        let write = read.clone();

        let read: Box<dyn AsyncRead + Send + Sync + Unpin> = Box::new(read);
        let write: Box<dyn AsyncWrite + Send + Sync + Unpin> = Box::new(write);

        let mut decoder = AsyncDecoder::new(read);
        let mut encoder = AsyncEncoder::new(write);

        log::debug!("waiting for the connection to be ready");
        let mut missed_messages = VecDeque::new();
        let identity = Self::wait_for_ready(
            &mut decoder,
            &mut encoder,
            user_config,
            &mut missed_messages,
        )
        .await?;
        log::debug!("connection is ready: {:?}", identity);

        let (writer_tx, writer_rx) = crate::channel::bounded(64);
        let (notify, notify_handle) = Notify::new();
        let (activity_tx, activity_rx) = crate::channel::bounded(32);

        let writer = AsyncWriter::new(MpscWriter::new(writer_tx), activity_tx);

        let timeout_state = TimeoutState::Start;
        let channels = Channels::default();

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

        log::debug!("joining '{}'", channel);
        self.encoder.encode(commands::join(channel)).await?;

        let channel = crate::commands::Channel(channel).to_string();
        log::debug!("waiting for a response");

        let mut queue = VecDeque::new();

        let status = self
            .wait_for(&mut queue, |msg, this| match msg {
                // check to see if it was us that joined the channel
                Commands::Join(msg) => {
                    Ok(msg.channel() == channel && msg.name() == this.identity.username())
                }

                // check to see if we were banned
                Commands::Notice(msg) if matches!(msg.msg_id(), Some(MessageId::MsgBanned)) => {
                    Err(Error::BannedFromChannel {
                        channel: msg.channel().to_string(),
                    })
                }

                _ => Ok(false),
            })
            .await?;

        if let Some(status) = status {
            match status {
                Status::Quit | Status::Eof => return Err(Error::UnexpectedEof),
                _ => unimplemented!(),
            }
        }

        self.missed_messages.extend(queue);

        log::debug!("joined '{}'", channel);

        Ok(())
    }

    /// Part `channel` and wait for it to complete
    pub async fn part(&mut self, channel: &str) -> Result<(), Error> {
        if !self.is_on_channel(channel) {
            return Err(Error::NotOnChannel {
                channel: channel.to_string(),
            });
        }

        log::debug!("leaving '{}'", channel);
        self.encoder.encode(commands::part(channel)).await?;

        let channel = crate::commands::Channel(channel).to_string();
        log::debug!("waiting for a response");

        let mut queue = VecDeque::new();

        let status = self
            .wait_for(&mut queue, |msg, this| match msg {
                // check to see if it was us that left the channel
                Commands::Part(msg) => {
                    Ok(msg.channel() == channel && msg.name() == this.identity.username())
                }
                _ => Ok(false),
            })
            .await?;

        if let Some(status) = status {
            match status {
                Status::Quit | Status::Eof => return Err(Error::UnexpectedEof),
                _ => unimplemented!(),
            }
        }
        log::debug!("left '{}'", channel);

        self.missed_messages.extend(queue);

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

                let all = Commands::from_irc(msg) //
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
                let msg = crate::irc::IrcMessage::parse(crate::MaybeOwned::Borrowed(msg))
                    .expect("encoder should produce valid IRC messages");

                if let crate::irc::IrcMessage::PRIVMSG = msg.get_command() {
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

    async fn check_messages(&mut self, all: &Commands<'static>) -> Result<(), Error> {
        use {Commands::*, TimeoutState::*};

        log::trace!("< {}", all.raw().escape_debug());

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

impl AsyncRunner {
    async fn wait_for<F>(
        &mut self,
        missed: &mut VecDeque<Commands<'static>>,
        func: F,
    ) -> Result<Option<Status<'static>>, Error>
    where
        F: Fn(&Commands<'static>, &Self) -> Result<bool, Error> + Send + Sync,
    {
        loop {
            match self.step().await? {
                StepResult::Status(Status::Message(msg)) => {
                    if func(&msg, self)? {
                        break Ok(None);
                    }
                    missed.push_back(msg);
                }
                StepResult::Status(d) => return Ok(Some(d)),
                StepResult::Nothing => continue,
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

            let left = std::cmp::max(start, *limit);
            let right = std::cmp::min(start, *limit);

            let diff = left - right;

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
        missed_messages: &mut VecDeque<Commands<'static>>,
    ) -> Result<Identity, Error>
    where
        R: AsyncRead + Send + Sync + Unpin,
        W: AsyncWrite + Send + Sync + Unpin,
    {
        use crate::IntoOwned as _;

        let is_anonymous = user_config.is_anonymous();

        let mut looking_for: HashSet<_> = user_config.capabilities.iter().collect();
        let mut caps = Capabilities::default();
        let mut our_name = None;

        let identity = loop {
            let msg = decoder.read_message().await?;

            // this should always be infallible. its not marked infallible
            // because of the 'non-exhaustive' attribute
            use Commands::*;
            let commands = Commands::from_irc(msg)?;

            // this is the simpliest way. and this'll only clone like 9 messages
            missed_messages.push_back(commands.clone().into_owned());

            match commands {
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
                        use crate::twitch::Capability as Cap;

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
                        // these unwraps should be safe because we'll have all of the TAGs here
                        name: our_name.unwrap(),
                        user_id: msg.user_id.unwrap().parse().unwrap(),
                        display_name: msg.display_name.map(|s| s.to_string()),
                        color: msg.color,
                        caps,
                    };
                }

                // Reply to any PINGs while waiting. Although Twitch doesn't
                // currently send a PING for spoof detection on initial
                // handshake, one day they may. Most IRC servers do this
                // already
                Ping(msg) => encoder.encode(commands::pong(msg.token())).await?,

                _ => {}
            };
        };

        Ok(identity)
    }
}

impl Stream for AsyncRunner {
    type Item = Commands<'static>;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        use std::future::Future;
        let fut = self.get_mut().next_message();
        futures_lite::pin!(fut);

        match futures_lite::ready!(fut.poll(ctx)) {
            Ok(status) => match status {
                Status::Message(msg) => Poll::Ready(Some(msg)),
                Status::Quit | Status::Eof => Poll::Ready(None),
            },
            Err(..) => Poll::Ready(None),
        }
    }
}
