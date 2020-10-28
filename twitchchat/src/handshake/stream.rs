use crate::{
    commands,
    irc::IrcMessage,
    stream::{self, BoxedSink, BoxedStream, HandshakeError, Identity, ReadMessage},
    twitch::UserConfig,
    IntoOwned as _,
};

use futures::{Sink, Stream, StreamExt as _};

/// Type alias for a _split_ [`StreamDecoder`](stream::StreamDecoder)
pub type Decoder<IO> = stream::StreamDecoder<futures::stream::SplitStream<IO>>;
/// Type alias for a _split_ [`SinkEncoder`](stream::SinkEncoder)
pub type Encoder<IO, M> = stream::SinkEncoder<futures::stream::SplitSink<IO, M>, M>;

/// Boxed, _split_ [`StreamDecoder`](stream::StreamDecoder) -- this erases some of the more tedious types
pub type BoxedDecoder<IO> = stream::StreamDecoder<BoxedStream<<IO as Stream>::Item>>;
/// Boxed, _split_ [`SinkEncoder`](stream::SinkEncoder) -- this erases some of the more tedious types
pub type BoxedEncoder<IO, M> = stream::SinkEncoder<BoxedSink<M, <IO as Sink<M>>::Error>, M>;

/// The initial connection handshake.
///
/// ### Required features
/// This is available with `features = ["sink_stream"]`
///
/// ### Usage
///
/// * Open/create your IO object (usually your tcp/ws connection (or in-memory buffer), etc.).
/// * Make this type from the `Stream + Sink` representation for that.
///     * If you only have an `AsyncRead + AsyncWrite` look into this [`Handshake`][crate::asynchronous::Handshake] instead.
///     * If you only have a `Read + Write` look into this [`Handshake`][crate::sync::Handshake] instead.
/// * When ready, call [`Handshake::wait_until_ready()`]
/// * When it returns `Ok`, use one of:
///     * [`Handshake::into_inner()`]
///     * [`Handshake::split()`]
///     * [`Handshake::split_boxed()`]
///
pub struct Handshake<IO, M> {
    inner: IO,
    _marker: std::marker::PhantomData<M>,
}

impl<IO, M> Handshake<IO, M>
where
    IO: Stream + Sink<M> + Send + Unpin + 'static,
    <IO as Stream>::Item: ReadMessage + Send + Sync + 'static,
    <IO as Sink<M>>::Error: std::error::Error + Send + Sync + 'static,
    M: From<String> + Send + Sync + 'static,
{
    /// Create a new Handshake from the provided `Sink + Stream`
    ///
    /// ---
    ///
    /// The type soup above looks a bit complex but to simply explain it:
    /// * the `IO` type must implement both `Sink` and `Stream`
    /// * the Stream's `Item` must implement the [`ReadMessage`] trait. generally you can do:
    ///     * [`StreamExt::map`][map] to produce a `String` to satisfy this.
    ///     * [`TryStream::map_ok`][map_ok] to produce a `std::io::Result<String>` to satisfy this.
    /// * the Sink's `Error` must implement the `std::error::Error` convention
    /// * the Sink's generic parameter has to be convertable to a `String` via `Into`
    ///
    /// [map]: https://docs.rs/futures/0.3.7/futures/stream/trait.StreamExt.html#method.map
    /// [map_ok]: https://docs.rs/futures/0.3.7/futures/stream/trait.TryStreamExt.html#method.map_ok
    pub fn new(io: IO) -> Self {
        Self {
            inner: io,
            _marker: std::marker::PhantomData,
        }
    }

    /// This ***registers*** the provided user configuration with the connection.
    ///
    /// This blocks until Twitch is ready or an error is produced.
    ///
    /// Once ready, it returns a tuple of your determined [`Identity`] and any messages that were 'consumed' up until the identity was determined.
    ///
    /// Now, you can do one of three things:
    /// * get the inner `Sink+Stream` out with [`Handshake::into_inner()`]
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
    /// [extend]: crate::stream::StreamDecoder::extend()
    pub async fn wait_until_ready(
        &mut self,
        user_config: &UserConfig,
    ) -> Result<(Identity, Vec<IrcMessage<'static>>), HandshakeError> {
        use crate::wait_for;

        let (write, read) = (&mut self.inner).split();
        let (mut read, mut write) = (stream::Decoder::new(read), stream::Encoder::new(write));

        write.encode(commands::register(user_config)).await?;

        let mut missed_messages = Vec::new();
        let mut state = wait_for::ReadyState::new(user_config);
        loop {
            let msg = read.read_message().await?;

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
                Identity(identity) => break Ok((identity, missed_messages)),
            }
        }
    }

    /// Consume the Handshake returning a `Decoder`/`Encoder` pair
    ///
    /// You should generally only call this after [`Handshake::wait_until_ready()`] has returned Ok
    pub fn split(self) -> (Decoder<IO>, Encoder<IO, M>) {
        let (write, read) = self.inner.split();
        (stream::Decoder::new(read), stream::Encoder::new(write))
    }

    /// Consume the Handshake returning a type erased `Decoder`/`Encoder` pair
    ///
    /// You should generally only call this after [`Handshake::wait_until_ready()`] has returned Ok
    pub fn split_boxed(self) -> (BoxedDecoder<IO>, BoxedEncoder<IO, M>) {
        let (write, read) = self.inner.split();
        let read: BoxedStream<<IO as Stream>::Item> = Box::new(read);
        let write: BoxedSink<M, <IO as Sink<M>>::Error> = Box::new(write);
        (
            stream::StreamDecoder::new(read),
            stream::SinkEncoder::new(write),
        )
    }

    /// Consume the Handshake returning the inner IO
    ///
    /// You should generally only call this after [`Handshake::wait_until_ready()`] has returned Ok
    pub fn into_inner(self) -> IO {
        self.inner
    }
}

impl<IO, M> std::fmt::Debug for Handshake<IO, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HandshakeStream").finish()
    }
}
