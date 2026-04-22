//! Check-related models.

use serde::{Deserialize, Serialize};

use super::{
    consistency::ConsistencyPreference,
    contextual_tuples::ContextualTupleKeys,
    tuple::CheckRequestTupleKey,
};

// ────────────────────────────────────────────────────────────────────────────
// CheckRequest
// ────────────────────────────────────────────────────────────────────────────

/// Request body for the `Check` API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckRequest {
    /// The tuple to check.
    pub tuple_key: CheckRequestTupleKey,
    /// Contextual tuples to use during evaluation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contextual_tuples: Option<ContextualTupleKeys>,
    /// Authorization model ID to use (overrides the default).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_model_id: Option<String>,
    /// If `true`, include a resolution trace in the response (performance cost).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<bool>,
    /// Additional ABAC context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
    /// Read consistency preference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consistency: Option<ConsistencyPreference>,
}

impl CheckRequest {
    /// Creates a minimal `CheckRequest`.
    pub fn new(tuple_key: CheckRequestTupleKey) -> Self {
        Self {
            tuple_key,
            contextual_tuples: None,
            authorization_model_id: None,
            trace: None,
            context: None,
            consistency: Some(ConsistencyPreference::Unspecified),
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// CheckResponse
// ────────────────────────────────────────────────────────────────────────────

/// Response from the `Check` API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckResponse {
    /// Whether the user has the requested relation with the object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed: Option<bool>,
    /// Resolution trace (only present when `trace = true`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
}

impl CheckResponse {
    /// Returns `true` if the check passed.
    pub fn is_allowed(&self) -> bool {
        self.allowed.unwrap_or(false)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// CheckError
// ────────────────────────────────────────────────────────────────────────────

/// An error returned inside a batch check result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckError {
    /// Machine-readable error code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Human-readable error message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}
