/// The status of the runner after it finished its main loop.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Status {
    /// The loop was cancelled
    Cancelled,
    /// The loop timed out
    TimedOut,
    /// The loop ran to completion
    Eof,
}
