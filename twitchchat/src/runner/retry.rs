use super::{error::Error, Status};

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
    pub async fn immediately(result: Result<Status, Error>) -> Result<bool, Error> {
        Ok(!matches!(result, Ok(Status::Cancelled)))
    }

    /// Retries if `Status` was a **TimedOut**, otherwise return the `Err` or `false` (to stop the connection loop).
    pub async fn on_timeout(result: Result<Status, Error>) -> Result<bool, Error> {
        Ok(matches!(result?, Status::TimedOut))
    }

    /// Retries if the `Result` was an error
    pub async fn on_error(result: Result<Status, Error>) -> Result<bool, Error> {
        Ok(result.is_err())
    }
}
