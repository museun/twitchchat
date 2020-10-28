use std::{
    sync::mpsc::{sync_channel, Receiver, RecvTimeoutError, SyncSender},
    time::{Duration, Instant},
};

use crate::{
    irc::IrcMessage,
    messages::Commands::{self, *},
    util::timestamp,
    FromIrcMessage as _,
};

/// The Timeout window. Its hardcoded to ***45 seconds***.
///
/// This is the amount of time that has to pass before the loop tries to send a heartbeat
pub const WINDOW: Duration = Duration::from_secs(45);

/// The Timeout timeout. Its hardcoded to ***10 seconds***.
///
/// This is the amount of time the server has to respond to the heartbeat
pub const TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
#[non_exhaustive]
#[allow(missing_copy_implementations)]
/// An error returned by the timeout detection logic
pub enum TimedOutError {
    /// A timeout was detected
    TimedOut,
}

impl std::fmt::Display for TimedOutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TimedOut => f.write_str("connection timed out"),
        }
    }
}

impl std::error::Error for TimedOutError {}

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

/// The kind of activity
#[derive(Debug)]
#[non_exhaustive]
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

pub(crate) fn timeout_detection_inner(
    input: ActivityReceiver,
    output: impl SendIt<String>,
) -> Result<(), TimedOutError> {
    let mut state = TimeoutState::Start;

    loop {
        match input.0.recv_timeout(WINDOW) {
            Ok(Activity::Message(msg)) => match Commands::from_irc(msg) {
                Ok(Ping(msg)) => {
                    if !output.send(msg.token().to_string()) {
                        break;
                    }
                    state = TimeoutState::activity()
                }
                Ok(Pong(..)) if matches!(state, TimeoutState::WaitingForPong{..}) => {
                    state = TimeoutState::activity()
                }
                _ => {}
            },

            Ok(Activity::Tick) => state = TimeoutState::activity(),

            Ok(Activity::Quit) | Err(RecvTimeoutError::Disconnected) => break,

            Err(..) if matches!(state, TimeoutState::Activity{..} | TimeoutState::Start) => {
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
                    return Err(TimedOutError::TimedOut);
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

pub(crate) trait SendIt<T> {
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
