/// When a channel starts host mode.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HostTargetStart {
    pub(super) source: String,
    pub(super) target: String,
    pub(super) viewers: Option<usize>,
}

impl HostTargetStart {
    /// The hosting channel
    pub fn source(&self) -> &str {
        &self.source
    }
    /// The hosted channel
    pub fn target(&self) -> &str {
        &self.target
    }
    /// Optional number of viewers watching
    pub fn viewers(&self) -> Option<usize> {
        self.viewers
    }
}
