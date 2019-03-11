/// When a channel starts host mode.
#[derive(Debug, PartialEq, Clone)]
pub struct HostTargetStart {
    /// The hosting channel
    pub source: String,
    /// The hosted channel
    pub target: String,
    /// Optional number of viewers watching
    pub viewers: Option<usize>,
}
