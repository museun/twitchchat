use std::{
    sync::mpsc::{sync_channel, Receiver, RecvTimeoutError, SyncSender},
    time::{Duration, Instant},
};

use crate::{
    messages::Commands::{self, *},
    util::timestamp,
    FromIrcMessage as _, IrcMessage,
};

use super::Error;

/// A handle for notifying the detection logic that it should delay
///
/// If you drop this, the other end will hang up
pub struct ActivitySender(SyncSender<Activity>);

impl std::fmt::Debug for ActivitySender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActivitySender").finish()
    }
}

impl ActivitySender {
    /// Send this activitiy
    pub fn message(&self, activity: Activity) {
        let _ = self.0.send(activity);
    }
}

/// A handle that you give to the timeout detection logic
pub struct ActivityReceiver(Receiver<Activity>);

impl std::fmt::Debug for ActivityReceiver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActivityReceiver").finish()
    }
}

#[derive(Debug)]
/// The kind of activity
pub enum Activity {
    /// A message was read
    ///
    /// When you read a messsage, you should give the logic a copy of it, via
    /// this type
    Message(IrcMessage<'static>),
    /// Delay the logic until the next deadline
    ///
    /// When you write a message, you should send this.
    Tick,

    /// A signal that the loop should exit
    Quit,
}

impl Activity {
    /// Create a pair of activity handles
    pub fn pair() -> (ActivitySender, ActivityReceiver) {
        let (tx, rx) = sync_channel(64);
        (ActivitySender(tx), ActivityReceiver(rx))
    }
}

cfg_async! {
cfg_writer!{
/// A helper function that'll respond to the tokens produced by [`idle_detection_loop`]
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
}
}

cfg_async! {
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
/// # use twitchchat::{runner::*, *};
/// # use futures_lite::StreamExt as _;
/// # let stream = std::io::Cursor::new(Vec::new());
/// # let decoder = twitchchat::Decoder::new(stream);
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
/// if let Err(twitchchat::Error::TimedOut) = res {
///     // the connection timed out
/// }
/// ```
///
/// [window]: crate::runner::WINDOW
/// [timeout]: crate::runner::TIMEOUT
/// [ping]: crate::commands::ping
pub async fn idle_detection_loop(
    input: ActivityReceiver,
    output: flume::Sender<String>,
) -> Result<(), Error> {
    let (tx, rx) = flume::bounded(1);
    std::thread::spawn(move || {
        let res = actual_loop(input, output);
        let _ = tx.send(res);
    });
    rx.into_recv_async().await.map_err(|_| Error::TimedOut)?
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
/// # use twitchchat::{runner::*, *};
/// # let stream = std::io::Cursor::new(Vec::new());
/// # let decoder = twitchchat::Decoder::new(stream);
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
/// let handle = std::thread::spawn(move || idle_detection_loop_sync(receiver, tx));
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
/// if let Err(twitchchat::Error::TimedOut) = handle.join().unwrap() {
///     // the connection timed out
/// }
/// ```
///
/// [window]: crate::runner::WINDOW
/// [timeout]: crate::runner::TIMEOUT
/// [ping]: crate::commands::ping
pub fn idle_detection_loop_sync(
    input: ActivityReceiver,
    output: SyncSender<String>,
) -> Result<(), Error> {
    actual_loop(input, output)
}

fn actual_loop(input: ActivityReceiver, output: impl SendIt<String>) -> Result<(), Error> {
    let mut state = TimeoutState::Start;

    loop {
        match input.0.recv_timeout(WINDOW) {
            Err(RecvTimeoutError::Disconnected) => break,
            Ok(Activity::Message(msg)) => {
                let msg = Commands::from_irc(msg) //
                    .expect("msg identity conversion should be upheld");
                match msg {
                    Ping(msg) => {
                        if !output.send(msg.token().to_string()) {
                            break;
                        }
                        state = TimeoutState::activity()
                    }
                    Pong(..) if matches!(state, TimeoutState::WaitingForPong{..}) => {
                        state = TimeoutState::activity()
                    }
                    _ => {}
                }
            }

            Ok(Activity::Quit) => break,

            Ok(Activity::Tick) => state = TimeoutState::activity(),

            Err(..) if !matches!(state, TimeoutState::WaitingForPong{..}) => {
                if !output.send(timestamp().to_string()) {
                    break;
                }

                state = TimeoutState::waiting_for_pong();
            }

            // we're already waiting for a PONG
            _ => {}
        }

        match state {
            TimeoutState::WaitingForPong(dt) => {
                if dt.elapsed() > TIMEOUT {
                    log::warn!(target: "twitchchat::timeout", "timeout detected after {:.0?}", dt.elapsed());
                    return Err(Error::TimedOut);
                }
            }

            TimeoutState::Activity(dt) => {
                if dt.elapsed() > TIMEOUT {
                    if !output.send(timestamp().to_string()) {
                        break;
                    }
                    log::warn!(target: "twitchchat::timeout", "sending a PING");
                    state = TimeoutState::waiting_for_pong();
                }
            }

            TimeoutState::Start => {}
        }
    }

    Ok(())
}

/// The Timeout window. Its hardcoded to ***45 seconds***.
///
/// This is the amount of time that has to pass before the loop tries to send a heartbeat
pub const WINDOW: Duration = Duration::from_secs(45);

/// The Timeout timeout. Its hardcoded to ***10 seconds***.
///
/// This is the amount of time the server has to respond to the heartbeat
pub const TIMEOUT: Duration = Duration::from_secs(10);

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

trait SendIt<T> {
    fn send(&self, item: T) -> bool;
}

impl<T> SendIt<T> for SyncSender<T> {
    fn send(&self, item: T) -> bool {
        self.try_send(item).is_ok()
    }
}

#[cfg(feature = "async")]
impl<T> SendIt<T> for flume::Sender<T> {
    fn send(&self, item: T) -> bool {
        self.try_send(item).is_ok()
    }
}
