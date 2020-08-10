#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Status {
    TimedOut,
    Cancelled,
    Eof,
}
