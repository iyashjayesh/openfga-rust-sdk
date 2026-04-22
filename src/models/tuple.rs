//! Tuple-related data models.

use serde::{Deserialize, Serialize};

// ────────────────────────────────────────────────────────────────────────────
// RelationshipCondition
// ────────────────────────────────────────────────────────────────────────────

/// A condition attached to a relationship tuple for ABAC evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationshipCondition {
    /// Condition name as defined in the authorization model.
    pub name: String,
    /// Runtime context values to evaluate the condition against.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

// ────────────────────────────────────────────────────────────────────────────
// TupleKey
// ────────────────────────────────────────────────────────────────────────────

/// A relationship tuple key (user + relation + object), optionally with a condition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TupleKey {
    /// Subject of the relationship — e.g. `"user:anne"`.
    pub user: String,
    /// Relation name — e.g. `"viewer"`.
    pub relation: String,
    /// Object — e.g. `"document:roadmap"`.
    pub object: String,
    /// Optional ABAC condition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<RelationshipCondition>,
}

impl TupleKey {
    /// Creates a new `TupleKey`.
    pub fn new(user: impl Into<String>, relation: impl Into<String>, object: impl Into<String>) -> Self {
        Self {
            user: user.into(),
            relation: relation.into(),
            object: object.into(),
            condition: None,
        }
    }

    /// Attaches a condition to this tuple key.
    pub fn with_condition(mut self, condition: RelationshipCondition) -> Self {
        self.condition = Some(condition);
        self
    }
}

// ────────────────────────────────────────────────────────────────────────────
// TupleKeyWithoutCondition
// ────────────────────────────────────────────────────────────────────────────

/// A relationship tuple key without a condition (used for delete operations).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TupleKeyWithoutCondition {
    /// Subject of the relationship.
    pub user: String,
    /// Relation name.
    pub relation: String,
    /// Object.
    pub object: String,
}

impl TupleKeyWithoutCondition {
    /// Creates a new `TupleKeyWithoutCondition`.
    pub fn new(user: impl Into<String>, relation: impl Into<String>, object: impl Into<String>) -> Self {
        Self {
            user: user.into(),
            relation: relation.into(),
            object: object.into(),
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// CheckRequestTupleKey / ReadRequestTupleKey / ExpandRequestTupleKey
// ────────────────────────────────────────────────────────────────────────────

/// Tuple key used in `CheckRequest` — user and relation are required.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckRequestTupleKey {
    /// Subject.
    pub user: String,
    /// Relation.
    pub relation: String,
    /// Object.
    pub object: String,
}

impl CheckRequestTupleKey {
    /// Creates a new `CheckRequestTupleKey`.
    pub fn new(user: impl Into<String>, relation: impl Into<String>, object: impl Into<String>) -> Self {
        Self { user: user.into(), relation: relation.into(), object: object.into() }
    }
}

/// Tuple key used in `ReadRequest` — all fields are optional for filtering.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReadRequestTupleKey {
    /// Optional user filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Optional relation filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relation: Option<String>,
    /// Optional object filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
}

/// Tuple key used in `ExpandRequest`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpandRequestTupleKey {
    /// Relation to expand.
    pub relation: String,
    /// Object to expand from.
    pub object: String,
}

impl ExpandRequestTupleKey {
    /// Creates a new `ExpandRequestTupleKey`.
    pub fn new(relation: impl Into<String>, object: impl Into<String>) -> Self {
        Self { relation: relation.into(), object: object.into() }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Tuple / TupleChange / TupleOperation
// ────────────────────────────────────────────────────────────────────────────

/// A full relationship tuple (key + timestamp).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tuple {
    /// The tuple key.
    pub key: TupleKey,
    /// When the tuple was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

/// An operation on a relationship tuple (write or delete).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TupleOperation {
    /// A write (creation) operation.
    #[serde(rename = "TUPLE_OPERATION_WRITE")]
    Write,
    /// A delete operation.
    #[serde(rename = "TUPLE_OPERATION_DELETE")]
    Delete,
}

/// A historical record of a change to a relationship tuple.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TupleChange {
    /// The tuple key.
    pub tuple_key: TupleKeyWithoutCondition,
    /// The operation performed.
    pub operation: TupleOperation,
    /// When the change occurred.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}
