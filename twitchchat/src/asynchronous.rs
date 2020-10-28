//! AsyncRead/AsynWrite ([`futures::AsyncRead`] + [`futures::AsyncWrite`]) types
//!
//! TODO write a demo here, and more of a description
//!
use crate::timeout::timeout_detection_inner;

/// A boxed [`futures::AsyncRead`] trait object
pub type BoxedRead = Box<dyn futures_lite::AsyncRead + Send + Sync + Unpin>;

/// A boxed [`futures::AsyncWrite`] trait object
pub type BoxedWrite = Box<dyn futures_lite::AsyncWrite + Send + Sync + Unpin>;

// re-exports
pub use crate::{
    decoder::{AsyncDecoder as Decoder, DecodeError},
    encoder::AsyncEncoder as Encoder,
};

pub use crate::handshake::{asynchronous::Handshake, HandshakeError};
pub use crate::timeout::{
    Activity, ActivityReceiver, ActivitySender, TimedOutError, TIMEOUT, WINDOW,
};

pub use crate::identity::{Identity, YourCapabilities};

/// A helper function that loops over the 'token' Receiver and responds with `PONGs`
///
/// This blocks the current future.
///
/// This is only available when you have `features = ["writer"]` enabled
#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub async fn respond_to_idle_events(
    writer: crate::writer::MpscWriter,
    tokens: flume::Receiver<String>,
) {
    while let Ok(token) = tokens.recv_async().await {
        if writer.send(crate::commands::pong(&token)).is_err() {
            break;
        }
    }
}

/// An asynchronous idle detection loop -- this will block until a timeout is detected, or you send a Quit signal
///
/// This allows you keep a connection alive with an out-of-band loop.
///
/// Normally, Twitch will send a `PING` every so often and you must reply with a `PONG` or you'll be disconnected.
///
/// But sometimes you want to detect an idle connection and force the 'heartbeat' to happen sooner. This function gives you that ability.
///
/// Usage:
/// * create [`ActivitySender`] and [`ActivityReceiver`] with the [`Activity::pair()`] function.
/// * create a [`std::sync::mpsc::sync_channel`] that is used for its responses (the **PING tokens**).
/// * start the loop (you'll probably want to spawn a task so you don't block the current future)
///     * see [`idle_detection_loop_sync()`] for an sync version
/// * when you want to delay the timeout (e.g. you wrote something), send a [`Activity::Tick`] via [`ActivitySender::message()`].
/// * when you read a message, you should give a copy of it to the loop, send a [`Activity::Message`] via [`ActivitySender::message()`].
/// * when you want to signal a shutdown,  send a [`Activity::Quit`] via [`ActivitySender::message()`].
/// * you'll periodically get a message via the channel you passed to it.
/// * you should encode this string via [`twitchchat::commands::ping()`][ping] to your Encoder.
///
/// When [`WINDOW`][window] has passed without any activity from you (`Quit`, `Message`), it will produce a **token** via the channel.
///
/// This is the type you should encode as `ping`.
///
/// If the server does not reply within [`TIMEOUT`][timeout] then the loop will exit, producing an [`Error::TimedOut`]
///
/// # Example
///
/// ```no_run
/// # use twitchchat::{asynchronous::*, *, irc::*};
/// # use futures_lite::StreamExt as _;
/// # let stream = std::io::Cursor::new(Vec::new());
/// # let decoder = twitchchat::sync::Decoder::new(stream);
/// # let executor = async_executor::LocalExecutor::new();
/// // create an activity pair -- this lets you reschedule the timeout
/// let (sender, receiver) = Activity::pair();
///
/// // interacting with the loop:
/// // this is a generic event to push the timeout forward
/// // you'll want to do this anytime you write
/// sender.message(Activity::Tick);
///
/// # let msg: IrcMessage = irc::parse(":PING 123456789\r\n").next().unwrap().unwrap();
/// // when you read a message, you should send a copy of it to the timeout
/// //  detector
/// sender.message(Activity::Message(msg));
///
/// // you can also signal for it to quit
/// // when you send this message, the loop will end and return `Ok(())`
/// sender.message(Activity::Quit);
///
/// // you can otherwise shut it down by dropping the `ActivitySender` the loop
/// //  will end and return `Ok(())`
///
/// // reading from suggested responses the loop:
/// // you'll need a channel that the detector will respond with.
/// // it'll send you a 'token' that you should send to the connection via
/// //  `commands::ping(&token).encode(&mut writer)`;
/// let (tx, rx) = flume::bounded(1);
///
/// // the loop will block the current future
/// let fut = executor.spawn(idle_detection_loop(receiver, tx));
///
/// // you can either spawn the receiver loop off, or use a method like
/// //  `Receiver::try_recv` for non-blocking receives
/// # let mut writer = Encoder::new(vec![]);
/// executor.spawn(async move {
///     // this receiver return None when the loop exits
///     // this happens on success (Quit, you drop the ActivitySender)
///     //  or a timeout error occurs
///     let mut stream = rx.into_stream();
///     while let Some(token) = stream.next().await {
///         // send a ping to the server, wtih this token.
///         commands::ping(&token).encode(&mut writer).unwrap()
///         // if the server does not reply within the `TIMEOUT` an error
///         //  will be produced on the other end
///     }
/// });
///
/// // block the future until the loop returns
/// let res = futures_lite::future::block_on(executor.run(fut));
///
/// // when the future ends, it should return a Result, if a timeout occured,
/// //  it'll return an `Error::Timeout`
/// if let Err(TimedOutError::TimedOut) = res {
///     // the connection timed out
/// }
/// ```
///
/// [window]: crate::runner::WINDOW
/// [timeout]: crate::runner::TIMEOUT
/// [ping]: crate::commands::ping
#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub async fn idle_detection_loop(
    input: ActivityReceiver,
    output: flume::Sender<String>,
) -> Result<(), TimedOutError> {
    let (tx, rx) = flume::bounded(1);
    std::thread::spawn(move || {
        let res = timeout_detection_inner(input, output);
        let _ = tx.send(res);
    });
    rx.into_recv_async()
        .await
        .map_err(|_| TimedOutError::TimedOut)?
}
