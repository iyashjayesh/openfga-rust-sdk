//! Write-related models.

use serde::{Deserialize, Serialize};

use super::tuple::{TupleKey, TupleKeyWithoutCondition};

// ────────────────────────────────────────────────────────────────────────────
// WriteRequest / sub-structs
// ────────────────────────────────────────────────────────────────────────────

/// The write portion of a `WriteRequest`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WriteRequestWrites {
    /// Tuple keys to write.
    pub tuple_keys: Vec<TupleKey>,
}

/// The delete portion of a `WriteRequest`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WriteRequestDeletes {
    /// Tuple keys to delete.
    pub tuple_keys: Vec<TupleKeyWithoutCondition>,
}

/// Request body for `Write`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WriteRequest {
    /// Tuples to write (create).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub writes: Option<WriteRequestWrites>,
    /// Tuples to delete.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deletes: Option<WriteRequestDeletes>,
    /// Authorization model ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_model_id: Option<String>,
}
