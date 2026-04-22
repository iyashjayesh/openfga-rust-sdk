//! Read-related models.

use serde::{Deserialize, Serialize};

use super::{
    consistency::ConsistencyPreference,
    tuple::{ReadRequestTupleKey, Tuple, TupleChange},
};

// ────────────────────────────────────────────────────────────────────────────
// ReadRequest / ReadResponse
// ────────────────────────────────────────────────────────────────────────────

/// Request body for `Read`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReadRequest {
    /// Optional filter tuple key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tuple_key: Option<ReadRequestTupleKey>,
    /// Authorization model ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_model_id: Option<String>,
    /// Pagination - maximum number of results to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<i32>,
    /// Pagination continuation token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation_token: Option<String>,
    /// Read consistency preference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consistency: Option<ConsistencyPreference>,
}

/// Response from `Read`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadResponse {
    /// The tuples returned.
    pub tuples: Vec<Tuple>,
    /// Pagination token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation_token: Option<String>,
}

// ────────────────────────────────────────────────────────────────────────────
// ReadChanges
// ────────────────────────────────────────────────────────────────────────────

/// Request body for `ReadChanges`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReadChangesRequest {
    /// Filter changes to a specific object type.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub type_filter: Option<String>,
    /// Filter changes that occurred at or after this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// Response from `ReadChanges`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadChangesResponse {
    /// The tuple changes.
    pub changes: Vec<TupleChange>,
    /// Pagination token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation_token: Option<String>,
}

// ────────────────────────────────────────────────────────────────────────────
// ReadAssertions / WriteAssertions
// ────────────────────────────────────────────────────────────────────────────

/// An assertion in the authorization model test suite.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assertion {
    /// The tuple key to assert on.
    pub tuple_key: AssertionTupleKey,
    /// Expected result.
    pub expectation: bool,
    /// Optional contextual tuples for the assertion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contextual_tuples: Option<Vec<super::tuple::TupleKey>>,
    /// Optional ABAC context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

/// The tuple key inside an `Assertion`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssertionTupleKey {
    /// The user.
    pub user: String,
    /// The relation.
    pub relation: String,
    /// The object.
    pub object: String,
}

/// Response from `ReadAssertions`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadAssertionsResponse {
    /// The authorization model ID these assertions belong to.
    pub authorization_model_id: String,
    /// The assertions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assertions: Option<Vec<Assertion>>,
}

/// Request body for `WriteAssertions`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WriteAssertionsRequest {
    /// The assertions to write.
    pub assertions: Vec<Assertion>,
}
