//! Consistency preference model.

use serde::{Deserialize, Serialize};

/// Controls whether the API uses a consistent or eventually-consistent read.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum ConsistencyPreference {
    /// Unspecified (server decides).
    #[default]
    #[serde(rename = "CONSISTENCY_PREFERENCE_UNSPECIFIED")]
    Unspecified,
    /// Minimise latency; may return stale data.
    #[serde(rename = "MINIMIZE_LATENCY")]
    MinimizeLatency,
    /// Always read from the latest consistent snapshot.
    #[serde(rename = "HIGHER_CONSISTENCY")]
    HigherConsistency,
}
