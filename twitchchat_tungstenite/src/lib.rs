use std::{
    future::Future,
    io::{Error, ErrorKind},
    pin::Pin,
    task::{Context, Poll},
};

use async_tungstenite::tungstenite::Message;
use futures_core::Stream;
use futures_io::{AsyncRead, AsyncWrite};
use futures_sink::Sink;

// this type is re-exported in 'sync' 'asynchronous' and 'stream'
// but without any default features, we can grab it from 'sync'
use twitchchat::sync::DecodeError;

pub struct WebSocketStream<IO> {
    conn: async_tungstenite::WebSocketStream<IO>,
    errored: bool,
}

impl<IO> std::fmt::Debug for WebSocketStream<IO>
where
    IO: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.conn.fmt(f)
    }
}

impl<IO> Stream for WebSocketStream<IO>
where
    IO: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    type Item = Result<String, DecodeError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.errored {
            return Poll::Ready(None);
        }

        let this = self.get_mut();

        match futures_core::ready!(Pin::new(&mut this.conn).poll_next(cx)) {
            Some(Ok(Message::Text(data))) => Poll::Ready(Some(Ok(data))),

            Some(Ok(Message::Close(..))) | None => Poll::Ready(None),

            Some(Err(err)) => {
                this.errored = true;
                Poll::Ready(Some(Err(map_to_decode_err(err))))
            }

            _ => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

impl<IO> Sink<Message> for WebSocketStream<IO>
where
    IO: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    type Error = std::io::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = &mut self.get_mut().conn;
        Pin::new(this).poll_ready(cx).map_err(map_to_io_err)
    }

    fn start_send(self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        let this = &mut self.get_mut().conn;
        Pin::new(this).start_send(item).map_err(map_to_io_err)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = &mut self.get_mut().conn;
        Pin::new(this).poll_flush(cx).map_err(map_to_io_err)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = &mut self.get_mut().conn;
        Pin::new(this).poll_close(cx).map_err(map_to_io_err)
    }
}

fn map_to_io_err<E>(err: E) -> std::io::Error
where
    E: std::error::Error + Send + Sync + 'static,
{
    Error::new(ErrorKind::Other, err)
}

fn map_to_decode_err<E>(err: E) -> DecodeError
where
    E: std::error::Error + Send + Sync + 'static,
{
    DecodeError::Io(map_to_io_err(err))
}

/// Connect to the given address with the provided 'connect' function
///
/// This is intentionally vague so you can provide whatever runtime/tls configuration you may need.
///
/// For example, say you want to use `async-std` (or one if its compatible sibling crates) with `async-tls`:
/// ```no_compile,toml
/// # in your toml
/// twitchchat_tungstenite = { version = "0.1", features = ["async_tungstenite/async-std-runtime,async-tls"]}
/// ```
///
/// ```no_compile,rust
/// let stream = twitchchat_tungstenite::connect("irc-ws.chat.twitch.tv:443", |addr| async move {
///     let stream = async_std::net::TcpStream::connect(&addr).await?;
///     async_tls::TlsConnector::new().connect("irc-ws.chat.twitch.tv", stream).await
/// }).await.unwrap();
/// ```
pub async fn connect<F, Fut, IO>(address: &str, connect: F) -> Result<WebSocketStream<IO>, Error>
where
    F: Fn(String) -> Fut + Send + 'static,
    Fut: Future<Output = std::io::Result<IO>> + Send + 'static,
    IO: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    let mut iter = address.splitn(2, "://");

    let (tcp_addr, address) = match (iter.next(), iter.next()) {
        (Some(addr), None) => (addr, format!("wss://{}", addr)),
        (Some(..), Some(addr)) => (addr, address.to_string()),
        _ => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("invalid address: '{}'", address.escape_debug()),
            ))
        }
    };

    let stream = connect(tcp_addr.to_string()).await?;
    let (conn, _resp) = async_tungstenite::client_async(address, stream)
        .await
        .map_err(|err| Error::new(ErrorKind::Other, err))?;

    Ok(WebSocketStream {
        conn,
        errored: false,
    })
}
