#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum Capability {
    Generic,
    Membership,
    Commands,
    Tags,
}

impl Default for Capability {
    fn default() -> Self {
        Capability::Generic
    }
}
