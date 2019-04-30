/// When a channel stops host mode.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HostTargetEnd {
    pub(super) source: String,
    pub(super) viewers: Option<usize>,
}

impl HostTargetEnd {
    /// The hosting channel
    pub fn source(&self) -> &str {
        &self.source
    }
    /// Optional number of viewers watch the host
    pub fn viewers(&self) -> Option<usize> {
        self.viewers
    }
}
