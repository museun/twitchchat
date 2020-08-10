use crate::{
    connector::Connector,
    messages::{Ping, Pong},
    rate_limit::AsyncBlocker,
    util::timestamp,
    util::Either::{self, Left, Right},
    *,
};

use async_writer::{AsyncWriter, MpscWriter};
use futures_lite::{pin, AsyncRead, AsyncWrite, StreamExt};
use futures_timer::Delay;

use std::{
    future::Future,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub enum RunnerError {
    Dispatch(DispatchError),
    Decode(DecodeError),
    Io(std::io::Error),
}

impl From<DispatchError> for RunnerError {
    fn from(err: DispatchError) -> Self {
        Self::Dispatch(err)
    }
}

impl From<DecodeError> for RunnerError {
    fn from(err: DecodeError) -> Self {
        Self::Decode(err)
    }
}

impl From<std::io::Error> for RunnerError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

/// Some common retry strategies.
///
/// These are used with [`Runner::run_with_retry`][retry].
///
/// You can provide your own by simplying having an async function with the same
/// signature.
///
/// That is `async fn(result: Result<Status, Error>) -> Result<bool, Error>`.
///
/// Return one of:
/// * `Ok(true)` to cause it to reconnect.
/// * `Ok(false)` will gracefully exit with `Ok(Status::Eof)`
/// * `Err(err)` will return that error
#[derive(Copy, Clone, Debug, Default)]
pub struct RetryStrategy;

impl RetryStrategy {
    /// Reconnect immediately unless the `Status` was `Cancelled`
    pub async fn immediately(result: Result<Status, RunnerError>) -> Result<bool, RunnerError> {
        if let Ok(Status::Cancelled) = result {
            return Ok(false);
        }
        Ok(true)
    }

    /// Retries if `Status` was a **TimedOut**, otherwise return the `Err` or `false` (to stop the connection loop).
    pub async fn on_timeout(result: Result<Status, RunnerError>) -> Result<bool, RunnerError> {
        let status = if let Status::TimedOut = result? {
            true
        } else {
            false
        };

        Ok(status)
    }

    /// Retries if the `Result` was an error
    pub async fn on_error(result: Result<Status, RunnerError>) -> Result<bool, RunnerError> {
        Ok(result.is_err())
    }
}

const WINDOW: Duration = Duration::from_secs(45);
const TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Status {
    TimedOut,
    Cancelled,
    Eof,
}

pub struct AsyncRunner {
    dispatcher: Dispatcher,
    writer: (Sender<Vec<u8>>, Receiver<Vec<u8>>),
    activity: (Sender<()>, Receiver<()>),
    quit: (Sender<()>, Receiver<()>),
}

// where for<'a> &'a IO: AsyncRead + AsyncWrite + Unpin + Send + Sync,
impl AsyncRunner {
    pub fn create(dispatcher: Dispatcher) -> Self {
        Self {
            dispatcher,
            writer: channel::bounded(64),
            activity: channel::bounded(32),
            quit: channel::bounded(1),
        }
    }

    pub fn writer(
        &self,
        rate_limit: RateLimit,
        blocker: impl AsyncBlocker,
    ) -> AsyncWriter<MpscWriter> {
        let (tx, rx) = (self.activity.0.clone(), self.quit.1.clone());
        let writer = MpscWriter::new(self.writer.0.clone());
        AsyncWriter::new(writer, tx, rx, rate_limit, blocker)
    }

    pub fn dispatcher(&mut self) -> &mut Dispatcher {
        &mut self.dispatcher
    }

    pub fn quit_signal(&self) -> Receiver<()> {
        let (_, rx) = &self.quit;
        rx.clone()
    }

    pub async fn run_with_retry<C, F, R>(
        &mut self,
        connector: C,
        retry: F,
    ) -> Result<(), RunnerError>
    where
        C: Connector + Send + Sync,
        for<'a> &'a C::Output: AsyncRead + AsyncWrite + Unpin + Send + Sync,
        F: Fn(Result<Status, RunnerError>) -> R,
        R: Future<Output = Result<bool, RunnerError>> + Send + Sync,
        R::Output: Send + Sync,
    {
        loop {
            let status = self.run_to_completion(connector.clone()).await;
            match retry(status).await {
                Err(err) => break Err(err),
                Ok(false) => break Ok(()),
                Ok(true) => {}
            }

            // TODO do we reset here?
        }
    }

    pub async fn run_to_completion<C>(&mut self, mut connector: C) -> Result<Status, RunnerError>
    where
        C: Connector + Send + Sync,
        for<'a> &'a C::Output: AsyncRead + AsyncWrite + Unpin + Send + Sync,
    {
        let stream = connector.connect().await?;
        let stream = async_dup::Arc::new(stream);

        let (mut ping, mut pong) = (
            self.dispatcher.subscribe_internal::<Ping>(),
            self.dispatcher.subscribe_internal::<Pong>(),
        );
        let (mut reader, mut writer) = (
            AsyncDecoder::new(stream.clone()), //
            AsyncEncoder::new(stream),
        );

        let mut state = TimeoutState::Start;

        let (_, rx) = &self.activity;
        let (_, write) = &self.writer;

        // this is awful. but look. no select!{}
        let status = loop {
            let (read, activity) = (reader.read_message(), rx.recv());
            let (ping, pong) = (ping.next(), pong.next());
            pin!(read);
            pin!(ping);
            pin!(activity);
            pin!(pong);

            // Bind all 4 interesting events together
            let (left, right) = (Either::select(read, ping), Either::select(activity, pong));
            pin!(left);
            pin!(right);

            let notification = Either::select(left, right);
            pin!(notification);

            let write = write.recv();
            pin!(write);

            let notification = Either::select(notification, write);
            pin!(notification);

            // and bind them with the timeout
            let timeout = Delay::new(WINDOW);
            pin!(timeout);

            // and select the first one
            match Either::select(notification, timeout).await {
                // we read a message
                Left(Left(Left(Left(read)))) => {
                    let msg = match read {
                        Err(DecodeError::Eof) => {
                            log::info!("got an EOF, exiting main loop");
                            break Status::Eof;
                        }
                        Err(err) => return Err(err.into()),
                        Ok(msg) => msg,
                    };
                    log::trace!("dispatching: {:#?}", msg);
                    self.dispatcher.dispatch(msg)?;
                    state = TimeoutState::Activity(Instant::now())
                }

                // we get a ping
                Left(Left(Left(Right(Some(ping))))) => {
                    let token = ping.token();
                    log::debug!(
                        "got a ping from the server. responding with token '{}'",
                        token
                    );
                    let pong = crate::commands::pong(token);
                    writer.encode(pong).await?;
                    state = TimeoutState::activity();
                }

                // they wrote a message
                Left(Left(Right(Left(_write)))) => {
                    state = TimeoutState::activity();
                }

                // we got a pong
                Left(Left(Right(Right(Some(_pong))))) => {
                    if let TimeoutState::WaitingForPong(_ts) = state {
                        state = TimeoutState::activity();
                    }
                }

                // our future timed out, send a ping
                Left(Right(write)) => {
                    if let Some(write) = write {
                        writer.encode(write).await?;
                    } else {
                        log::warn!("no more writers detected");
                    }
                }

                Right(_timeout) => {
                    log::info!("idle connectiond detected, sending a ping");
                    let ts = timestamp().to_string();
                    writer.encode(crate::commands::ping(&ts)).await?;
                    state = TimeoutState::waiting_for_pong();
                }

                // we have a dead future -- they should all be alive unless we're shutting down
                _ => break Status::Eof,
            };

            match state {
                TimeoutState::WaitingForPong(dt) => {
                    if dt.elapsed() > TIMEOUT {
                        log::warn!("PING timeout detected, exiting");
                        break Status::TimedOut;
                    }
                }
                TimeoutState::Activity(dt) => {
                    if dt.elapsed() > WINDOW {
                        log::warn!("idle connectiond detected, sending a PING");
                        let ts = timestamp().to_string();
                        writer.encode(crate::commands::ping(&ts)).await?;
                        state = TimeoutState::waiting_for_pong();
                    }
                }
                TimeoutState::Start => {}
            }
        };

        let (tx, _) = &self.quit;
        // send the quit signal
        let _ = tx.send(()).await;

        // TODO: determine if we want to wait for all writers to finish
        // it wouldn't make much sense, twitch closes the connection as soon as
        // it reads the QUIT message
        //
        // but this could 'spin' on writers (or if we don't give quit_tx to
        // writers), or some other 'spawned' task
        // while !self.quit_tx.is_closed() {
        //     futures_lite::future::yield_now().await;
        // }

        Ok(status)
    }
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

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn does_it_loop() {
//         use std::net::ToSocketAddrs;

//         std::env::set_var("RUST_LOG", "twitchchat=trace");

//         let _ = alto_logger::init_term_logger();

//         let addr = crate::TWITCH_IRC_ADDRESS
//             .to_socket_addrs()
//             .unwrap()
//             .next()
//             .unwrap();

//         log::trace!("hello?");

//         async_executor::Executor::new().run(async move {
//             log::info!("connecting");
//             let conn = async_io::Async::<std::net::TcpStream>::connect(addr)
//                 .await
//                 .unwrap();
//             log::info!("connected");

//             let mut dispatcher = Dispatcher::new();
//             let mut all = dispatcher.subscribe::<crate::messages::AllCommands>();

//             log::info!("spawning");
//             async_executor::Spawner::current()
//                 .spawn(async move {
//                     while let Some(msg) = <_ as StreamExt>::next(&mut all).await {
//                         log::debug!("{:#?}", msg)
//                     }
//                 })
//                 .detach();

//             log::info!("creating thing");
//             let runner = AsyncRunner::create(dispatcher, conn);

//             let mut writer = runner.writer(RateLimit::default(), super::rate_limit::NullBlocker {});

//             let task = async_executor::Spawner::current().spawn({
//                 let writer = writer.clone();
//                 async move {
//                     log::info!("sending quit in 5 seconds");
//                     futures_timer::Delay::new(std::time::Duration::from_secs(5)).await;
//                     log::info!("sending quit");
//                     writer.quit().await.unwrap();
//                     log::info!("exiting task");
//                 }
//             });

//             log::info!("registering");
//             writer
//                 .encode("PASS justinfan1234\r\nNICK justinfan1234\r\n")
//                 .await
//                 .unwrap();

//             log::info!("joining");
//             writer
//                 .encode(crate::commands::join("#museun"))
//                 .await
//                 .unwrap();

//             log::info!("running to completion");
//             let t = runner.run_to_completion().await.unwrap();
//             log::error!("{:?}", t);

//             log::info!("waiting quit task");
//             task.await;

//             log::info!("done running?");
//         });

//         log::error!("end of test");
//     }
// }
