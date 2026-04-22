//! Batch check models.

use serde::{Deserialize, Serialize};

use super::{check::CheckError, consistency::ConsistencyPreference, contextual_tuples::ContextualTupleKeys};

// ────────────────────────────────────────────────────────────────────────────
// BatchCheckItem
// ────────────────────────────────────────────────────────────────────────────

/// A single check item within a `BatchCheckRequest`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchCheckItem {
    /// Subject.
    pub user: String,
    /// Relation.
    pub relation: String,
    /// Object.
    pub object: String,
    /// Caller-provided correlation ID used to match responses to requests.
    pub correlation_id: String,
    /// Contextual tuples for this specific check item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contextual_tuples: Option<ContextualTupleKeys>,
    /// ABAC context for this specific check item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

// ────────────────────────────────────────────────────────────────────────────
// BatchCheckRequest
// ────────────────────────────────────────────────────────────────────────────

/// Request body for the server-side `BatchCheck` API (requires FGA ≥ 1.8.0).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchCheckRequest {
    /// The list of checks to perform.
    pub checks: Vec<BatchCheckItem>,
    /// Authorization model ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_model_id: Option<String>,
    /// Read consistency preference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consistency: Option<ConsistencyPreference>,
}

// ────────────────────────────────────────────────────────────────────────────
// BatchCheckSingleResult
// ────────────────────────────────────────────────────────────────────────────

/// The result for a single item in a `BatchCheckResponse`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchCheckSingleResult {
    /// Whether the check succeeded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed: Option<bool>,
    /// Error details if this individual check failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<CheckError>,
}

// ────────────────────────────────────────────────────────────────────────────
// BatchCheckResponse
// ────────────────────────────────────────────────────────────────────────────

/// Response from the server-side `BatchCheck` API.
///
/// The response map key is the `correlation_id` from the request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchCheckResponse {
    /// Map from `correlation_id` → result.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<std::collections::HashMap<String, BatchCheckSingleResult>>,
}
