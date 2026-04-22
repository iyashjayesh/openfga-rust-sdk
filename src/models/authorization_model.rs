//! Authorization model types.

use serde::{Deserialize, Serialize};

// ────────────────────────────────────────────────────────────────────────────
// TypeName
// ────────────────────────────────────────────────────────────────────────────

/// The type name for a condition parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(missing_docs)]
pub enum TypeName {
    #[serde(rename = "TYPE_NAME_UNSPECIFIED")]
    Unspecified,
    #[serde(rename = "TYPE_NAME_ANY")]
    Any,
    #[serde(rename = "TYPE_NAME_BOOL")]
    Bool,
    #[serde(rename = "TYPE_NAME_STRING")]
    String,
    #[serde(rename = "TYPE_NAME_INT")]
    Int,
    #[serde(rename = "TYPE_NAME_UINT")]
    Uint,
    #[serde(rename = "TYPE_NAME_DOUBLE")]
    Double,
    #[serde(rename = "TYPE_NAME_DURATION")]
    Duration,
    #[serde(rename = "TYPE_NAME_TIMESTAMP")]
    Timestamp,
    #[serde(rename = "TYPE_NAME_MAP")]
    Map,
    #[serde(rename = "TYPE_NAME_LIST")]
    List,
    #[serde(rename = "TYPE_NAME_IPADDRESS")]
    IpAddress,
}

// ────────────────────────────────────────────────────────────────────────────
// ConditionParamTypeRef
// ────────────────────────────────────────────────────────────────────────────

/// Reference to a condition parameter type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConditionParamTypeRef {
    /// The type name.
    pub type_name: TypeName,
    /// Generic type arguments, e.g. the value type of a map.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generic_types: Option<Vec<ConditionParamTypeRef>>,
}

// ────────────────────────────────────────────────────────────────────────────
// ConditionMetadata
// ────────────────────────────────────────────────────────────────────────────

/// Metadata about a condition's parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConditionMetadata {
    /// Map from parameter name → type reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    /// Source info for the condition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_info: Option<SourceInfo>,
}

/// Source information for a model element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceInfo {
    /// The file the element was defined in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}

// ────────────────────────────────────────────────────────────────────────────
// Condition
// ────────────────────────────────────────────────────────────────────────────

/// A condition definition in an authorization model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Condition {
    /// Condition name.
    pub name: String,
    /// CEL expression evaluated at runtime.
    pub expression: String,
    /// Parameter type references.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<std::collections::HashMap<String, ConditionParamTypeRef>>,
    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ConditionMetadata>,
}

// ────────────────────────────────────────────────────────────────────────────
// Userset types
// ────────────────────────────────────────────────────────────────────────────

/// An empty object representing the "this" userset (direct relations).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct This {}

/// A computed userset reference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectRelation {
    /// Object (can be empty for computed usersets relative to the current object).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// Relation name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relation: Option<String>,
}

/// A tuple-to-userset definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TupleToUserset {
    /// The tuple set from which to compute the userset.
    pub tupleset: ObjectRelation,
    /// The computed userset derived from the tuple set.
    pub computed_userset: ObjectRelation,
}

/// A typed wildcard (all users of a type).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypedWildcard {
    /// The type name.
    #[serde(rename = "type")]
    pub type_name: String,
}

/// A difference of two usersets (A minus B).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Difference {
    /// The base userset.
    pub base: Box<Userset>,
    /// The subtract userset.
    pub subtract: Box<Userset>,
}

/// A computed userset definition inside an authorization model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComputedUserset {
    /// The userset relation.
    pub userset: String,
}

/// A union / intersection of multiple usersets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Usersets {
    /// The child usersets.
    pub child: Vec<Userset>,
}

/// A userset definition - can be `this`, a union, intersection, difference,
/// a computed userset, or a tuple-to-userset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Userset {
    /// Direct relation (this).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub this: Option<This>,
    /// Computed userset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub computed_userset: Option<ObjectRelation>,
    /// Tuple-to-userset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tuple_to_userset: Option<TupleToUserset>,
    /// Union of usersets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub union: Option<Usersets>,
    /// Intersection of usersets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intersection: Option<Usersets>,
    /// Difference of usersets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub difference: Option<Difference>,
}

// ────────────────────────────────────────────────────────────────────────────
// RelationReference / RelationMetadata / Metadata
// ────────────────────────────────────────────────────────────────────────────

/// A reference to a type and optional relation or wildcard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationReference {
    /// The type name.
    #[serde(rename = "type")]
    pub type_name: String,
    /// Optional relation within the type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relation: Option<String>,
    /// Wildcard - represents all users of the type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wildcard: Option<TypedWildcard>,
    /// Optional condition name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
}

/// Metadata for a single relation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RelationMetadata {
    /// The user types that can be directly related via this relation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directly_related_user_types: Option<Vec<RelationReference>>,
    /// Module information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    /// Source info.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_info: Option<SourceInfo>,
}

/// Metadata for a `TypeDefinition`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Metadata {
    /// Per-relation metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relations: Option<std::collections::HashMap<String, RelationMetadata>>,
    /// Module information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    /// Source info.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_info: Option<SourceInfo>,
}

// ────────────────────────────────────────────────────────────────────────────
// TypeDefinition / AuthorizationModel
// ────────────────────────────────────────────────────────────────────────────

/// A type definition within an authorization model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeDefinition {
    /// The type name.
    #[serde(rename = "type")]
    pub type_name: String,
    /// Relation definitions keyed by relation name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relations: Option<std::collections::HashMap<String, Userset>>,
    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

/// A full authorization model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthorizationModel {
    /// The model ID (ULID).
    pub id: String,
    /// Schema version string (e.g. `"1.1"`).
    pub schema_version: String,
    /// Type definitions.
    pub type_definitions: Vec<TypeDefinition>,
    /// Optional condition definitions keyed by condition name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<std::collections::HashMap<String, Condition>>,
}

// ────────────────────────────────────────────────────────────────────────────
// Write / Read model requests & responses
// ────────────────────────────────────────────────────────────────────────────

/// Request body for `WriteAuthorizationModel`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WriteAuthorizationModelRequest {
    /// Schema version.
    pub schema_version: String,
    /// Type definitions.
    pub type_definitions: Vec<TypeDefinition>,
    /// Optional conditions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<std::collections::HashMap<String, Condition>>,
}

/// Response from `WriteAuthorizationModel`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WriteAuthorizationModelResponse {
    /// The ID of the newly created model.
    pub authorization_model_id: String,
}

/// Response from `ReadAuthorizationModel`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadAuthorizationModelResponse {
    /// The authorization model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_model: Option<AuthorizationModel>,
}

/// Response from `ReadAuthorizationModels`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadAuthorizationModelsResponse {
    /// The list of authorization models.
    pub authorization_models: Vec<AuthorizationModel>,
    /// Pagination token for the next page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation_token: Option<String>,
}
