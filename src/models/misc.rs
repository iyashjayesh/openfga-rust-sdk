//! Miscellaneous model types.

use serde::{Deserialize, Serialize};

// ────────────────────────────────────────────────────────────────────────────
// Status (used in streaming errors)
// ────────────────────────────────────────────────────────────────────────────

/// gRPC-style status used in streaming error envelopes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Status {
    /// Numeric status code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<i32>,
    /// Human-readable message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Additional error details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<serde_json::Value>>,
}

// ────────────────────────────────────────────────────────────────────────────
// StreamResult - envelope for NDJSON streaming responses
// ────────────────────────────────────────────────────────────────────────────

/// Wrapper for each line in a streaming (NDJSON) response.
///
/// Each line is either a successful `result` or an `error`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamResult<T> {
    /// A successful result.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,
    /// An error encountered mid-stream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Status>,
}
