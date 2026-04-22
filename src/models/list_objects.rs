//! ListObjects and StreamedListObjects models.

use serde::{Deserialize, Serialize};

use super::{consistency::ConsistencyPreference, contextual_tuples::ContextualTupleKeys};

// ────────────────────────────────────────────────────────────────────────────
// ListObjectsRequest
// ────────────────────────────────────────────────────────────────────────────

/// Request body for `ListObjects`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListObjectsRequest {
    /// Authorization model ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_model_id: Option<String>,
    /// The object type to list.
    #[serde(rename = "type")]
    pub object_type: String,
    /// The relation to check.
    pub relation: String,
    /// The user for which to list objects.
    pub user: String,
    /// Contextual tuples.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contextual_tuples: Option<ContextualTupleKeys>,
    /// ABAC context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
    /// Read consistency preference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consistency: Option<ConsistencyPreference>,
}

/// Response from `ListObjects`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListObjectsResponse {
    /// The list of object IDs the user has access to.
    pub objects: Vec<String>,
}

/// A single object streamed from `StreamedListObjects`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamedListObjectsResponse {
    /// A single object ID.
    pub object: String,
}
