/// Status of the client after running
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Status {
    /// The connection timedout
    Timeout,
    /// It ran to completion
    Eof,
    /// It was canceled
    Canceled,
}
