//! A Websocket 'Encoder' and 'Decoder'
//!
//! This requires the `ws` and `async` features

use futures_lite::{AsyncRead, AsyncWrite};
use std::collections::VecDeque;
use std::future::Future;

use crate::{
    commands,
    runner::{
        wait_until_ready::{check_message, ReadyState, StepState},
        Error, Identity,
    },
    util::is_blocking_error,
    DecodeError, Encodable, IntoOwned, IrcMessage, UserConfig,
};

pub use crate::runner::Event;

// TODO move this into split
/// Make a WebSocket decoder/encoder pair
pub fn make_pair<IO>(io: IO) -> (WsDecoder<IO>, WsEncoder<IO>)
where
    IO: AsyncRead + AsyncWrite + Send + Unpin,
{
    use soketto::connection::{Builder, Mode};
    let (write, read) = Builder::new(io, Mode::Client).finish();
    (WsDecoder::new(read), WsEncoder::new(write))
}

/// Wait for this specific [`Event`], using the provided [`WsDecoder`]
///
/// This'll block until it finds a matching [`Event`]
///
/// On success it'll return the [`IrcMessage`] and any messages you missed
///
/// You can use [`std::iter::Extend`] to feed these messages back into the [`WsDecoder`], if you want to replay things.
pub async fn wait_for<IO>(
    event: Event,
    dec: &mut WsDecoder<IO>,
) -> Result<(IrcMessage<'static>, Vec<IrcMessage<'static>>), Error>
where
    IO: AsyncRead + AsyncWrite + Send + Unpin,
{
    let mut missed = vec![];
    loop {
        let msg = match dec.read_message().await {
            Err(Error::Io(err)) if is_blocking_error(&err) => {
                futures_lite::future::yield_now().await;
                continue;
            }
            Err(err) => break Err(err),
            Ok(msg) => msg,
        };

        if Event::from_raw(msg.get_command()) == event {
            break Ok((msg.into_owned(), missed));
        } else {
            missed.push(msg.into_owned());
        }
    }
}

/// Register and wait until Twitch finishes the handshake
///
/// This returns your [`Identity`] and any _missed_ messages
///
/// On failure, it'll return:
///
/// | error                                 | cause                                           |
/// | ------------------------------------- | ----------------------------------------------- |
/// | [`InvalidCap`][invalid_cap]           | you provided an invalid capability              |
/// | [`BadPass`][bad_pass]                 | you provided an invalid OAuth token             |
/// | [`Io`][io]                            | an i/o error occured                            |
/// | [`ShouldReconnect`][should_reconnect] | the server was restarting, you should reconnect |
/// | [`UnexpectedEof`][unexpected_eof]     | the server closed the connection abruptly       |
///
/// [invalid_cap]: crate::runner::Error::InvalidCap
/// [bad_pass]: crate::runner::Error::BadPass
/// [io]: crate::runner::Error::Io
/// [timed_out]: crate::runner::Error::TimedOut
/// [should_reconnect]: crate::runner::Error::ShouldReconnect
/// [unexpected_eof]: crate::runner::Error::UnexpectedEof
pub async fn wait_until_ready<IO>(
    io: &mut IO,
    user_config: &UserConfig,
) -> Result<(Identity, Vec<IrcMessage<'static>>), Error>
where
    IO: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    let (mut read, mut write) = make_pair(io);
    write.encode(commands::register(user_config)).await?;

    let mut missed_messages = Vec::new();
    let mut state = ReadyState::new(user_config);
    loop {
        let msg = match read.read_message().await {
            Err(Error::Io(err)) if is_blocking_error(&err) => {
                futures_lite::future::yield_now().await;
                continue;
            }
            Err(err) => Err(err),
            Ok(ok) => Ok(ok),
        }?;

        match check_message(&msg, &mut state)? {
            StepState::Skip => continue,
            StepState::Continue => {
                missed_messages.push(msg.into_owned());
                continue;
            }
            StepState::ShouldPong(token) => write.encode(commands::pong(&token)).await?,
            StepState::Identity(identity) => break Ok((identity, missed_messages)),
        }
    }
}

/// Attempts to connect to the websocket server
///
/// This takes an "initial" address and a connection function. If a redirect is found, it'll follow it.
///
/// The connection function should take a `String` and return a future to an `std::io::Result<impl AsyncRead+AsyncWrite>`
pub async fn connect<F, Fut, IO>(address: &str, connect: F) -> Result<IO, Error>
where
    F: Fn(String) -> Fut + Send + 'static,
    Fut: Future<Output = std::io::Result<IO>> + Send + 'static,
    IO: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    use soketto::handshake::{Client, ServerResponse::*};

    let mut addr = address.to_string();
    let socket = loop {
        let connect = connect(addr.to_string());
        let stream = connect.await?;
        let mut socket = Client::new(stream, &addr, "/");
        match socket.handshake().await? {
            Accepted { .. } => break socket,
            Redirect { location, .. } => addr = location,
            Rejected { status_code } => return Err(Error::CannotConnect { status_code }),
        }
    };

    Ok(socket.into_inner())
}

/// A decoder over a [`soketto::Sender`]
pub struct WsEncoder<T> {
    buf: Vec<u8>,
    sender: soketto::Sender<T>,
}

impl<T> std::fmt::Debug for WsEncoder<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WsEncoder").finish()
    }
}

impl<T> WsEncoder<T>
where
    T: AsyncRead + AsyncWrite + Send + Unpin,
{
    /// Create a new [`WsEncoder`] from a [`soketto::Sender`]
    pub fn new(sender: soketto::Sender<T>) -> Self {
        Self {
            buf: Vec::new(),
            sender,
        }
    }

    /// Pull the [`soketto::Sender`] out of this type
    pub fn into_inner(self) -> soketto::Sender<T> {
        self.sender
    }

    /// Encode this message to the inner websocket.
    pub async fn encode(&mut self, msg: impl Encodable + Send) -> Result<(), Error> {
        self.buf.clear();
        msg.encode(&mut self.buf)?;

        if !self.buf.ends_with(b"\n") {
            let s = std::str::from_utf8(&self.buf).map_err(Error::InvalidUtf8)?;
            self.sender.send_text(s).await?;
            return self.sender.send_text("\n").await.map_err(Into::into);
        }

        let mut msg = &*self.buf;
        while let Some(p) = msg
            .iter()
            .position(|&c| c == b'\n')
            .filter(|&c| c < msg.len() && c != 0)
        {
            let (left, right) = msg.split_at(p + 1);
            msg = right;
            let s = std::str::from_utf8(left).map_err(Error::InvalidUtf8)?;
            self.sender.send_text(&s).await?;
        }

        Ok(())
    }
}

/// A decoder over a [`soketto::Receiver`]
pub struct WsDecoder<T> {
    pos: usize,
    buf: Vec<u8>,
    receiver: soketto::Receiver<T>,
    back_queue: VecDeque<IrcMessage<'static>>,
}

impl<R> Extend<IrcMessage<'static>> for WsDecoder<R> {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = IrcMessage<'static>>,
    {
        self.back_queue.extend(iter)
    }
}

impl<T> std::fmt::Debug for WsDecoder<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WsDecoder").finish()
    }
}

impl<T> WsDecoder<T>
where
    T: AsyncRead + AsyncWrite + Send + Unpin,
{
    /// Create a new [`WsDecoder`] from a [`soketto::Receiver`]
    pub fn new(receiver: soketto::Receiver<T>) -> Self {
        Self {
            pos: 0,
            buf: Vec::with_capacity(1 << 10),
            receiver,
            back_queue: VecDeque::new(),
        }
    }

    /// Pull the [`soketto::Receiver`] out of this type
    pub fn into_inner(self) -> soketto::Receiver<T> {
        self.receiver
    }

    /// Read a [`IrcMessage`], blocking until one is ready
    pub async fn read_message(&mut self) -> Result<IrcMessage<'_>, Error> {
        // if we have some buffered messages, do them first
        if self.pos != 0 {
            return self.parse_message().map_err(Into::into);
        }

        // we've already yielded all of the messages above so we should clear the buffer
        self.buf.clear();

        self.receiver.receive_data(&mut self.buf).await?;
        self.parse_message().map_err(Into::into)
    }

    fn parse_message(&mut self) -> Result<IrcMessage<'_>, DecodeError> {
        use crate::irc::parse_one;

        // ensure there is a \r\n at the end
        match &self.buf[self.pos..] {
            [.., b'\r', b'\n'] => {}
            [.., b'\n'] => {
                self.buf.pop();
                self.buf.extend_from_slice(b"\r\n");
            }
            [..] => self.buf.extend_from_slice(b"\r\n"),
        }

        let (p, msg) = std::str::from_utf8(&self.buf[self.pos..])
            .map_err(DecodeError::InvalidUtf8)
            .and_then(|s| parse_one(s).map_err(DecodeError::ParseError))?;

        self.pos = if p > 0 { self.pos + p } else { 0 };

        Ok(msg)
    }
}
