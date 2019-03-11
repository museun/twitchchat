/// When a channel stops host mode.
#[derive(Debug, PartialEq, Clone)]
pub struct HostTargetEnd {
    // The hosting channel
    pub source: String,
    /// Optional number of viewers watch the host
    pub viewers: Option<usize>,
}
