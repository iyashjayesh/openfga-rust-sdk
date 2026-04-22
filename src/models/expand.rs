//! Expand-related models — userset tree types.

use serde::{Deserialize, Serialize};

use super::{consistency::ConsistencyPreference, tuple::ExpandRequestTupleKey};

// ────────────────────────────────────────────────────────────────────────────
// Expand request/response
// ────────────────────────────────────────────────────────────────────────────

/// Request body for `Expand`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpandRequest {
    /// The tuple key to expand.
    pub tuple_key: ExpandRequestTupleKey,
    /// Authorization model ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_model_id: Option<String>,
    /// Read consistency preference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consistency: Option<ConsistencyPreference>,
}

/// Response from `Expand`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpandResponse {
    /// The root node of the userset tree.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree: Option<UsersetTree>,
}

// ────────────────────────────────────────────────────────────────────────────
// Userset tree types
// ────────────────────────────────────────────────────────────────────────────

/// The root of a userset tree returned by `Expand`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsersetTree {
    /// The root node.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root: Option<Node>,
}

/// A node in a userset tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    /// Node name / description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Leaf node (direct users).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leaf: Option<Leaf>,
    /// Difference (A minus B).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub difference: Option<UsersetTreeDifference>,
    /// Union of child nodes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub union: Option<Nodes>,
    /// Intersection of child nodes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intersection: Option<Nodes>,
}

/// A collection of nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Nodes {
    /// The child nodes.
    pub nodes: Vec<Node>,
}

/// A leaf node in a userset tree — contains actual users.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Leaf {
    /// Direct users.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users: Option<Users>,
    /// Computed userset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub computed: Option<Computed>,
    /// Tuple-to-userset expansion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tuple_to_userset: Option<UsersetTreeTupleToUserset>,
}

/// The set of users at a leaf node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Users {
    /// The user string IDs.
    pub users: Vec<String>,
}

/// A computed userset in an expand result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Computed {
    /// The userset string.
    pub userset: String,
}

/// A tuple-to-userset expansion in an expand result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsersetTreeTupleToUserset {
    /// The tupleset string.
    pub tupleset: String,
    /// Computed from each matching tuple.
    pub computed: Vec<Computed>,
}

/// A difference between two subtrees.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsersetTreeDifference {
    /// The base subtree.
    pub base: Box<Node>,
    /// The subtracted subtree.
    pub subtract: Box<Node>,
}
