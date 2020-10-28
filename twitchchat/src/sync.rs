//! Synchronous ([`std::io::Read`] + [`std::io::Write`]) types
//!
//! TODO write a demo here, and more of a description

use crate::timeout::timeout_detection_inner;
use std::sync::mpsc::SyncSender;

// re-exports
pub use crate::decoder::{DecodeError, Decoder};
pub use crate::encoder::Encoder;

pub use crate::io::{BoxedRead, BoxedWrite, ReadHalf, WriteHalf};

pub use crate::handshake::{sync::Handshake, HandshakeError};
pub use crate::timeout::{
    Activity, ActivityReceiver, ActivitySender, TimedOutError, TIMEOUT, WINDOW,
};

pub use crate::identity::{Identity, YourCapabilities};

/// A helper function that loops over the 'token' Receiver and responds with `PONGs`
///
/// This blocks the current thread.
///
/// This is only available when you have `features = ["writer"]` enabled
#[cfg(feature = "writer")]
#[cfg_attr(docsrs, doc(cfg(feature = "writer")))]
pub fn respond_to_idle_events(
    writer: crate::writer::MpscWriter,
    tokens: std::sync::mpsc::Receiver<String>,
) {
    for token in tokens {
        if writer.send(crate::commands::pong(&token)).is_err() {
            break;
        }
    }
}

/// A synchronous idle detection loop -- this will block until a timeout is detected, or you send a Quit signal
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
/// * start the loop (you'll probably want to spawn a thread so you don't block the current one)
///     * see [`idle_detection_loop()`] for an async version
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
/// # use twitchchat::{sync::*, *, irc::*};
/// # let stream = std::io::Cursor::new(Vec::new());
/// # let decoder = Decoder::new(stream);
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
/// let (tx, rx) = std::sync::mpsc::sync_channel(1);
///
/// // the loop will block the current thread
/// let handle = std::thread::spawn(move || idle_detection_loop(receiver, tx));
///
/// // you can either spawn the receiver loop off, or use a method like
/// //  `Receiver::try_recv` for non-blocking receives
/// # let mut writer = Encoder::new(vec![]);
/// std::thread::spawn(move || {
///     // this receiver return None when the loop exits
///     // this happens on success (Quit, you drop the ActivitySender)
///     //  or a timeout error occurs
///     for token in rx {
///         // send a ping to the server, wtih this token.
///         commands::ping(&token).encode(&mut writer).unwrap()
///         // if the server does not reply within the `TIMEOUT` an error
///         //  will be produced on the other end
///     }
/// });
///
/// // when the thread joins, it should return a Result, if a timeout occured,
/// //  it'll return an `Error::Timeout`
/// if let Err(TimedOutError::TimedOut) = handle.join().unwrap() {
///     // the connection timed out
/// }
/// ```
///
/// [window]: WINDOW
/// [timeout]: TIMEOUT
/// [ping]: crate::commands::ping
pub fn idle_detection_loop(
    input: ActivityReceiver,
    output: SyncSender<String>,
) -> Result<(), TimedOutError> {
    timeout_detection_inner(input, output)
}
