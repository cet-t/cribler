#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Verdict {
    /// The observed z-score was within the configured limit.
    Pass,
    /// The observed z-score exceeded the configured limit.
    Fail,
}
