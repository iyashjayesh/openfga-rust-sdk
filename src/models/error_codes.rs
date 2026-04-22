//! Error code enums.

use serde::{Deserialize, Serialize};

// ────────────────────────────────────────────────────────────────────────────
// ErrorCode
// ────────────────────────────────────────────────────────────────────────────

/// Machine-readable error codes for validation / request errors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(missing_docs)]
pub enum ErrorCode {
    #[serde(rename = "no_error")]
    NoError,
    #[serde(rename = "validation_error")]
    ValidationError,
    #[serde(rename = "authorization_model_not_found")]
    AuthorizationModelNotFound,
    #[serde(rename = "authorization_model_resolution_too_complex")]
    AuthorizationModelResolutionTooComplex,
    #[serde(rename = "invalid_write_input")]
    InvalidWriteInput,
    #[serde(rename = "cannot_allow_duplicate_tuples_in_one_request")]
    CannotAllowDuplicateTuplesInOneRequest,
    #[serde(rename = "cannot_allow_duplicate_types_in_one_request")]
    CannotAllowDuplicateTypesInOneRequest,
    #[serde(rename = "cannot_allow_multiple_references_to_one_relation")]
    CannotAllowMultipleReferencesToOneRelation,
    #[serde(rename = "invalid_continuation_token")]
    InvalidContinuationToken,
    #[serde(rename = "invalid_tuple_set")]
    InvalidTupleSet,
    #[serde(rename = "invalid_check_input")]
    InvalidCheckInput,
    #[serde(rename = "invalid_expand_input")]
    InvalidExpandInput,
    #[serde(rename = "unsupported_user_set")]
    UnsupportedUserSet,
    #[serde(rename = "invalid_object_format")]
    InvalidObjectFormat,
    #[serde(rename = "write_failed_due_to_invalid_input")]
    WriteFailedDueToInvalidInput,
    #[serde(rename = "authorization_model_assertions_not_found")]
    AuthorizationModelAssertionsNotFound,
    #[serde(rename = "latest_authorization_model_not_found")]
    LatestAuthorizationModelNotFound,
    #[serde(rename = "type_not_found")]
    TypeNotFound,
    #[serde(rename = "relation_not_found")]
    RelationNotFound,
    #[serde(rename = "empty_relations_for_a_type_in_authority_model")]
    EmptyRelationsForATypeInAuthorityModel,
    #[serde(rename = "store_id_invalid_length")]
    StoreIdInvalidLength,
    #[serde(rename = "assertions_too_many_items")]
    AssertionsTooManyItems,
    #[serde(rename = "id_too_long")]
    IdTooLong,
    #[serde(rename = "authorization_model_id_too_long")]
    AuthorizationModelIdTooLong,
    #[serde(rename = "tuple_key_value_not_specified")]
    TupleKeyValueNotSpecified,
    #[serde(rename = "tuple_keys_too_many_or_too_few_items")]
    TupleKeysTooManyOrTooFewItems,
    #[serde(rename = "page_size_invalid")]
    PageSizeInvalid,
    #[serde(rename = "param_missing_value")]
    ParamMissingValue,
    #[serde(rename = "difference_base_missing_value")]
    DifferenceBaseMissingValue,
    #[serde(rename = "at_least_one_condition_must_be_specified")]
    AtLeastOneConditionMustBeSpecified,
    #[serde(rename = "condition_not_found")]
    ConditionNotFound,
    #[serde(rename = "invalid_syntax_type")]
    InvalidSyntaxType,
    #[serde(rename = "invalid_schema_version")]
    InvalidSchemaVersion,
    #[serde(rename = "invalid_authorization_model")]
    InvalidAuthorizationModel,
    #[serde(rename = "exceeded_entity_limit")]
    ExceededEntityLimit,
    #[serde(rename = "invalid_contextual_tuple")]
    InvalidContextualTuple,
    #[serde(rename = "no_writes_or_deletes_provided")]
    NoWritesOrDeletesProvided,
    #[serde(rename = "duplicate_contextual_tuple")]
    DuplicateContextualTuple,
    #[serde(rename = "unknown")]
    Unknown,
}

// ────────────────────────────────────────────────────────────────────────────
// AuthErrorCode
// ────────────────────────────────────────────────────────────────────────────

/// Error codes for authentication failures.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(missing_docs)]
pub enum AuthErrorCode {
    #[serde(rename = "auth_failed_invalid_subject")]
    AuthFailedInvalidSubject,
    #[serde(rename = "auth_failed_invalid_audience")]
    AuthFailedInvalidAudience,
    #[serde(rename = "auth_failed_invalid_issuer")]
    AuthFailedInvalidIssuer,
    #[serde(rename = "invalid_claims")]
    InvalidClaims,
    #[serde(rename = "auth_failed_invalid_bearer_token")]
    AuthFailedInvalidBearerToken,
    #[serde(rename = "bearer_token_missing")]
    BearerTokenMissing,
    #[serde(rename = "unauthenticated")]
    Unauthenticated,
}

// ────────────────────────────────────────────────────────────────────────────
// NotFoundErrorCode
// ────────────────────────────────────────────────────────────────────────────

/// Error codes for 404 not found responses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(missing_docs)]
pub enum NotFoundErrorCode {
    #[serde(rename = "no_not_found_error")]
    NoNotFoundError,
    #[serde(rename = "undefined_endpoint")]
    UndefinedEndpoint,
    #[serde(rename = "store_id_not_found")]
    StoreIdNotFound,
    #[serde(rename = "unimplemented")]
    Unimplemented,
}

// ────────────────────────────────────────────────────────────────────────────
// InternalErrorCode
// ────────────────────────────────────────────────────────────────────────────

/// Error codes for internal server errors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(missing_docs)]
pub enum InternalErrorCode {
    #[serde(rename = "no_internal_error")]
    NoInternalError,
    #[serde(rename = "internal_error")]
    InternalError,
    #[serde(rename = "cancelled")]
    Cancelled,
    #[serde(rename = "deadline_exceeded")]
    DeadlineExceeded,
    #[serde(rename = "already_exists")]
    AlreadyExists,
    #[serde(rename = "resource_exhausted")]
    ResourceExhausted,
    #[serde(rename = "failed_precondition")]
    FailedPrecondition,
    #[serde(rename = "aborted")]
    Aborted,
    #[serde(rename = "out_of_range")]
    OutOfRange,
    #[serde(rename = "unavailable")]
    Unavailable,
    #[serde(rename = "data_loss")]
    DataLoss,
}

// ────────────────────────────────────────────────────────────────────────────
// API Error response bodies
// ────────────────────────────────────────────────────────────────────────────

/// Body of a 400/422 validation error response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationErrorMessageResponse {
    /// Machine-readable code.
    pub code: ErrorCode,
    /// Human-readable message.
    pub message: String,
}

/// Body of a 5xx internal error response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InternalErrorMessageResponse {
    /// Machine-readable code.
    pub code: InternalErrorCode,
    /// Human-readable message.
    pub message: String,
}

/// Body of a 404 not found error response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PathUnknownErrorMessageResponse {
    /// Machine-readable code.
    pub code: NotFoundErrorCode,
    /// Human-readable message.
    pub message: String,
}

/// Body of a 401 unauthenticated error response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnauthenticatedResponse {
    /// Machine-readable code.
    pub code: AuthErrorCode,
    /// Human-readable message.
    pub message: String,
}

/// Body of a 403 forbidden response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForbiddenResponse {
    /// Machine-readable code.
    pub code: AuthErrorCode,
    /// Human-readable message.
    pub message: String,
}
