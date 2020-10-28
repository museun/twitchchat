use crate::{
    asynchronous::{
        BoxedRead, BoxedWrite, DecodeError, Decoder, Encoder, HandshakeError, Identity,
    },
    commands,
    irc::IrcMessage,
    twitch::UserConfig,
    util::is_blocking_error,
    IntoOwned as _,
};

use futures_lite::{
    io::{ReadHalf, WriteHalf},
    AsyncRead, AsyncWrite,
};

/// The initial connection handshake.
///
/// ### Required features
/// This is available with `features = ["async"]`
///
/// ### Usage
///
/// * Open/create your IO object (usually your tcp/ws connection (or in-memory buffer), etc.).
/// * Make this type from the `AsyncRead + AsyncWrite` representation for that.
///     * If you only have an `Stream + Sink` look into this [`Handshake`][crate::stream::Handshake] instead.
///     * If you only have a `Read + Write` look into this [`Handshake`][crate::sync::Handshake] instead.
/// * When ready, call [`Handshake::wait_until_ready()`]
/// * When it returns `Ok`, use one of:
///     * [`Handshake::into_inner()`]
///     * [`Handshake::split()`]
///     * [`Handshake::split_boxed()`]
///
pub struct Handshake<IO> {
    inner: IO,
}

impl<IO> Handshake<IO>
where
    IO: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    /// Create a new Handshake from the provided `AsyncRead + AsyncWrite`
    pub fn new(io: IO) -> Self {
        Self { inner: io }
    }

    /// This ***registers*** the provided user configuration with the connection.
    ///
    /// This blocks until Twitch is ready or an error is produced.
    ///
    /// Once ready, it returns a tuple of your determined [`Identity`] and any messages that were 'consumed' up until the identity was determined.
    ///
    /// Now, you can do one of three things:
    /// * get the inner `AsyncRead+AsyncWrite` out with [`Handshake::into_inner()`]
    /// * get a `Decoder`/`Encoder` pair with [`Handshake::split()`]
    /// * get a typed-erased `Decoder`/`Encoder` pair with [`Handshake::split_boxed()`]
    ///
    /// You can use [`Decoder::extend()`][extend] with the _missed messages_ if you want to receive them after this call, but generally you can ignore them.
    ///
    /// # Errors
    /// * [`HandshakeError::BadPass`]
    ///     * occurs when you provide an invalid OAuth token
    /// * [`HandshakeError::ShouldReconnect`]
    ///     * occurs when the server restarted while you were connecting
    /// * [`HandshakeError::InvalidCapability`]
    ///     * occurs when you provide an invalid capability
    /// * [`HandshakeError::Encode`]
    ///     * occurs when one of the required commands could not be encoded to the provided writer
    /// * [`HandshakeError::Decode`]
    ///     * occurs when a message could not be parsed (or read) from the provided reader
    ///
    /// [extend]: crate::asynchronous::Decoder::extend()
    pub async fn wait_until_ready(
        &mut self,
        user_config: &UserConfig,
    ) -> Result<(Identity, Vec<IrcMessage<'static>>), HandshakeError> {
        use crate::wait_for;

        let (read, write) = futures_lite::io::split(&mut self.inner);
        let (mut read, mut write) = (Decoder::new(read), Encoder::new(write));

        write.encode(commands::register(user_config)).await?;

        let mut missed_messages = Vec::new();
        let mut state = wait_for::ReadyState::new(user_config);
        loop {
            let msg = match read.read_message().await {
                Err(DecodeError::Io(err)) if is_blocking_error(&err) => {
                    futures_lite::future::yield_now().await;
                    continue;
                }
                Err(err) => Err(err),
                Ok(ok) => Ok(ok),
            }?;

            use wait_for::StepState::*;
            match wait_for::check_message(&msg, &mut state)
                .map_err(HandshakeError::from_check_err)?
            {
                Skip => continue,
                Continue => {
                    missed_messages.push(msg.into_owned());
                    continue;
                }
                ShouldPong(token) => write.encode(commands::pong(&token)).await?,
                Identity(identity) => return Ok((identity, missed_messages)),
            }
        }
    }

    /// Consume the Handshake returning a `Decoder`/`Encoder` pair
    ///
    /// You should generally only call this after [`Handshake::wait_until_ready()`] has returned Ok
    pub fn split(self) -> (Decoder<ReadHalf<IO>>, Encoder<WriteHalf<IO>>) {
        let (read, write) = futures_lite::io::split(self.inner);
        (Decoder::new(read), Encoder::new(write))
    }

    /// Consume the Handshake returning a type erased `Decoder`/`Encoder` pair
    ///
    /// You should generally only call this after [`Handshake::wait_until_ready()`] has returned Ok
    pub fn split_boxed(self) -> (Decoder<BoxedRead>, Encoder<BoxedWrite>) {
        let (read, write) = futures_lite::io::split(self.inner);
        let read: BoxedRead = Box::new(read);
        let write: BoxedWrite = Box::new(write);
        (Decoder::new(read), Encoder::new(write))
    }

    /// Consume the Handshake returning the inner IO
    ///
    /// You should generally only call this after [`Handshake::wait_until_ready()`] has returned Ok
    pub fn into_inner(self) -> IO {
        self.inner
    }
}

impl<IO> std::fmt::Debug for Handshake<IO> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncHandshake").finish()
    }
}
