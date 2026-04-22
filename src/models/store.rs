//! Store-related models.

use serde::{Deserialize, Serialize};

// ────────────────────────────────────────────────────────────────────────────
// Store
// ────────────────────────────────────────────────────────────────────────────

/// An OpenFGA store.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Store {
    /// Store ID (ULID).
    pub id: String,
    /// Store name.
    pub name: String,
    /// Creation timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Last update timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Soft-delete timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ────────────────────────────────────────────────────────────────────────────
// CreateStore
// ────────────────────────────────────────────────────────────────────────────

/// Request body for `CreateStore`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateStoreRequest {
    /// The name for the new store.
    pub name: String,
}

/// Response from `CreateStore`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateStoreResponse {
    /// The created store ID.
    pub id: String,
    /// The store name.
    pub name: String,
    /// Creation timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Last update timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ────────────────────────────────────────────────────────────────────────────
// GetStore
// ────────────────────────────────────────────────────────────────────────────

/// Response from `GetStore`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetStoreResponse {
    /// The store ID.
    pub id: String,
    /// The store name.
    pub name: String,
    /// Creation timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Last update timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ────────────────────────────────────────────────────────────────────────────
// ListStores
// ────────────────────────────────────────────────────────────────────────────

/// Response from `ListStores`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListStoresResponse {
    /// The list of stores.
    pub stores: Vec<Store>,
    /// Pagination token for the next page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation_token: Option<String>,
}
