//! ListUsers models.

use serde::{Deserialize, Serialize};

use super::{consistency::ConsistencyPreference, contextual_tuples::ContextualTupleKeys};

// ────────────────────────────────────────────────────────────────────────────
// User / UserObject types
// ────────────────────────────────────────────────────────────────────────────

/// A typed wildcard user (all users of a type).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypedWildcardUser {
    /// The type.
    #[serde(rename = "type")]
    pub type_name: String,
}

/// A userset user (e.g. `document:budget#viewer`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsersetUser {
    /// The object.
    pub object: FgaObject,
    /// The relation.
    pub relation: String,
}

/// An FGA object reference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FgaObject {
    /// The type.
    #[serde(rename = "type")]
    pub type_name: String,
    /// The object ID.
    pub id: String,
}

/// A user - can be a typed wildcard, a userset user, or a plain object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    /// Typed wildcard.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wildcard: Option<TypedWildcardUser>,
    /// Userset user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub userset: Option<UsersetUser>,
    /// Plain object user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<FgaObject>,
}

// ────────────────────────────────────────────────────────────────────────────
// UserTypeFilter
// ────────────────────────────────────────────────────────────────────────────

/// Filter for restricting which user types are returned by `ListUsers`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserTypeFilter {
    /// The type to include.
    #[serde(rename = "type")]
    pub type_name: String,
    /// Optional relation within the type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relation: Option<String>,
}

// ────────────────────────────────────────────────────────────────────────────
// ListUsersRequest / ListUsersResponse
// ────────────────────────────────────────────────────────────────────────────

/// Request body for `ListUsers`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListUsersRequest {
    /// Authorization model ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_model_id: Option<String>,
    /// Object to check.
    pub object: FgaObject,
    /// Relation to check.
    pub relation: String,
    /// Filter which user types to include in the response.
    pub user_filters: Vec<UserTypeFilter>,
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

/// Response from `ListUsers`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListUsersResponse {
    /// The users that have the requested relation with the object.
    pub users: Vec<User>,
    /// Excluded users (typed wildcards that are excluded).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excluded_users: Option<Vec<User>>,
}
