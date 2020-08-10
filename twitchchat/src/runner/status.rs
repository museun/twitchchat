/// Runner status
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Status {
    /// The loop timed out
    TimedOut,
    /// The loop was Cancelled
    Cancelled,
    /// The loop ran to completion
    Eof,
}
