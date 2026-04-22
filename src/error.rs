//! Error types for the OpenFGA SDK.
//!
//! Mirrors `errors.go` from the Go SDK. Every error carries structured
//! context (store ID, endpoint, HTTP status, request ID) so callers can
//! act on errors without parsing strings.

use reqwest::header::HeaderMap;
use std::time::Duration;
use thiserror::Error;

use crate::internal::retry::{get_time_to_wait, parse_retry_after_header, RetryParams};

// ────────────────────────────────────────────────────────────────────────────
// Top-level error enum
// ────────────────────────────────────────────────────────────────────────────

/// Top-level SDK error type.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum OpenFgaError {
    /// Authentication / authorisation failure (401, 403).
    #[error("OpenFGA authentication error: {0}")]
    Authentication(#[from] FgaApiAuthenticationError),

    /// Request validation failure (400, 422).
    #[error("OpenFGA validation error: {0}")]
    Validation(#[from] FgaApiValidationError),

    /// Resource not found (404).
    #[error("OpenFGA not found: {0}")]
    NotFound(#[from] FgaApiNotFoundError),

    /// Rate limit exceeded (429).
    #[error("OpenFGA rate limit exceeded: {0}")]
    RateLimitExceeded(#[from] FgaApiRateLimitExceededError),

    /// Internal server error (5xx).
    #[error("OpenFGA internal error: {0}")]
    Internal(#[from] FgaApiInternalError),

    /// Generic / unclassified API error.
    #[error("OpenFGA API error: {0}")]
    Api(#[from] FgaApiError),

    /// Invalid SDK parameter (e.g. malformed ULID).
    #[error("Invalid parameter '{param}': {description}")]
    InvalidParam {
        /// Parameter name.
        param: String,
        /// Description of what is wrong.
        description: String,
    },

    /// Configuration is invalid.
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Underlying HTTP transport error.
    #[error("HTTP error: {0}")]
    Http(String),

    /// JSON serialisation / deserialisation error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// URL parsing error.
    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),

    /// Wrapped reqwest error (display only).
    #[error("HTTP request error: {0}")]
    Request(String),
}

impl OpenFgaError {
    /// Returns `true` if this error can be retried.
    pub fn should_retry(&self) -> bool {
        match self {
            Self::RateLimitExceeded(e) => e.should_retry(),
            Self::Internal(e) => e.should_retry(),
            // Network-level errors stored as strings — always retry.
            Self::Http(_) => true,
            _ => false,
        }
    }

    /// Returns how long to wait before retrying, or `Duration::ZERO` if the
    /// error is not retryable or the retry budget is exhausted.
    pub fn get_time_to_wait(&self, attempt: u32, retry_params: &RetryParams) -> Duration {
        match self {
            Self::RateLimitExceeded(e) => e.get_time_to_wait(attempt, retry_params),
            Self::Internal(e) => e.get_time_to_wait(attempt, retry_params),
            Self::Http(_) => get_time_to_wait(
                attempt,
                retry_params.max_retry,
                retry_params.min_wait_ms,
                &HeaderMap::new(),
                "http",
            ),
            _ => Duration::ZERO,
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Shared context carried by all API errors
// ────────────────────────────────────────────────────────────────────────────

/// Context attached to every API-level error.
#[derive(Debug, Clone, Default)]
pub struct ApiErrorContext {
    /// Store ID at the time of the request.
    pub store_id: String,
    /// Human-readable API operation name, e.g. `"Check"`.
    pub endpoint_category: String,
    /// HTTP method (GET, POST, …).
    pub request_method: String,
    /// Host portion of the request URL.
    pub request_host: String,
    /// HTTP status code returned by the server.
    pub response_status_code: u16,
    /// Full response headers.
    pub response_headers: HeaderMap,
    /// `Fga-Request-Id` or `X-Request-Id` from the response headers.
    pub request_id: String,
    /// Machine-readable error code string from the response body.
    pub response_code: String,
    /// Raw response body bytes.
    pub body: Vec<u8>,
}

impl ApiErrorContext {
    pub(crate) fn from_response(
        store_id: &str,
        endpoint_category: &str,
        request_method: &str,
        request_host: &str,
        status: u16,
        headers: &HeaderMap,
        body: Vec<u8>,
    ) -> Self {
        let request_id = headers
            .get("Fga-Request-Id")
            .or_else(|| headers.get("X-Request-Id"))
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        Self {
            store_id: store_id.to_string(),
            endpoint_category: endpoint_category.to_string(),
            request_method: request_method.to_string(),
            request_host: request_host.to_string(),
            response_status_code: status,
            response_headers: headers.clone(),
            request_id,
            response_code: String::new(),
            body,
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// FgaApiAuthenticationError (401 / 403)
// ────────────────────────────────────────────────────────────────────────────

/// Returned when the API responds with 401 or 403.
#[derive(Debug, Clone, Error)]
#[error("{message}")]
pub struct FgaApiAuthenticationError {
    /// Human-readable message.
    pub message: String,
    /// Structured request/response context.
    pub context: ApiErrorContext,
}

impl FgaApiAuthenticationError {
    pub(crate) fn new(ctx: ApiErrorContext) -> Self {
        let message = format!(
            "{} auth error for {} {}",
            ctx.request_method, ctx.endpoint_category, ctx.request_method
        );
        Self { message, context: ctx }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// FgaApiValidationError (400 / 422)
// ────────────────────────────────────────────────────────────────────────────

/// Returned when the API responds with 400 or 422.
#[derive(Debug, Clone, Error)]
#[error("{message}")]
pub struct FgaApiValidationError {
    /// Human-readable message.
    pub message: String,
    /// Structured request/response context.
    pub context: ApiErrorContext,
}

impl FgaApiValidationError {
    pub(crate) fn new(mut ctx: ApiErrorContext) -> Self {
        let message = format!(
            "{} validation error for {} {}",
            ctx.request_method, ctx.endpoint_category, ctx.request_method
        );
        // Try to parse a response_code from the body.
        if let Ok(body_str) = std::str::from_utf8(&ctx.body) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(body_str) {
                if let Some(code) = v.get("code").and_then(|c| c.as_str()) {
                    ctx.response_code = code.to_string();
                }
            }
        }
        Self { message, context: ctx }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// FgaApiNotFoundError (404)
// ────────────────────────────────────────────────────────────────────────────

/// Returned when the API responds with 404.
#[derive(Debug, Clone, Error)]
#[error("{message}")]
pub struct FgaApiNotFoundError {
    /// Human-readable message.
    pub message: String,
    /// Structured request/response context.
    pub context: ApiErrorContext,
}

impl FgaApiNotFoundError {
    pub(crate) fn new(ctx: ApiErrorContext) -> Self {
        let message = format!(
            "{} not found error for {}",
            ctx.request_method, ctx.endpoint_category
        );
        Self { message, context: ctx }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// FgaApiRateLimitExceededError (429)
// ────────────────────────────────────────────────────────────────────────────

/// Returned when the API responds with 429 (rate limit exceeded).
#[derive(Debug, Clone, Error)]
#[error("{message}")]
pub struct FgaApiRateLimitExceededError {
    /// Human-readable message.
    pub message: String,
    /// Structured request/response context.
    pub context: ApiErrorContext,
    /// Milliseconds to wait before retrying (from `Retry-After`).
    pub retry_after_ms: Option<u64>,
    /// Epoch timestamp when rate limit resets (from `X-RateLimit-Reset`).
    pub rate_limit_reset_epoch: Option<String>,
    /// The rate limit value (from `X-RateLimit-Limit`).
    pub rate_limit: Option<u32>,
    /// The rate unit (from `X-RateLimit-Unit`).
    pub rate_unit: Option<String>,
}

impl FgaApiRateLimitExceededError {
    pub(crate) fn new(ctx: ApiErrorContext) -> Self {
        let message = format!(
            "{} rate limit error for {}",
            ctx.request_method, ctx.endpoint_category
        );
        let retry_after_ms = parse_retry_after_header(&ctx.response_headers)
            .map(|d| d.as_millis() as u64);
        let rate_limit_reset_epoch = ctx
            .response_headers
            .get(crate::internal::retry::RATE_LIMIT_RESET_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let rate_limit = ctx
            .response_headers
            .get("X-RateLimit-Limit")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok());
        let rate_unit = ctx
            .response_headers
            .get("X-RateLimit-Unit")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        Self {
            message,
            context: ctx,
            retry_after_ms,
            rate_limit_reset_epoch,
            rate_limit,
            rate_unit,
        }
    }

    /// Always returns `true` — 429 errors are always retryable.
    pub fn should_retry(&self) -> bool {
        true
    }

    /// Returns how long to wait before retrying.
    pub fn get_time_to_wait(&self, attempt: u32, retry_params: &RetryParams) -> Duration {
        get_time_to_wait(
            attempt,
            retry_params.max_retry,
            retry_params.min_wait_ms,
            &self.context.response_headers,
            &self.context.endpoint_category,
        )
    }
}

// ────────────────────────────────────────────────────────────────────────────
// FgaApiInternalError (5xx)
// ────────────────────────────────────────────────────────────────────────────

/// Returned when the API responds with a 5xx status code.
#[derive(Debug, Clone, Error)]
#[error("{message}")]
pub struct FgaApiInternalError {
    /// Human-readable message.
    pub message: String,
    /// Structured request/response context.
    pub context: ApiErrorContext,
    /// Milliseconds to wait before retrying (from `Retry-After`).
    pub retry_after_ms: Option<u64>,
}

impl FgaApiInternalError {
    pub(crate) fn new(ctx: ApiErrorContext) -> Self {
        let message = format!(
            "{} internal error for {}",
            ctx.request_method, ctx.endpoint_category
        );
        let retry_after_ms = parse_retry_after_header(&ctx.response_headers)
            .map(|d| d.as_millis() as u64);
        Self { message, context: ctx, retry_after_ms }
    }

    /// Returns `false` for 501 Not Implemented (cannot be retried).
    pub fn should_retry(&self) -> bool {
        self.context.response_status_code != 501
    }

    /// Returns how long to wait before retrying.
    pub fn get_time_to_wait(&self, attempt: u32, retry_params: &RetryParams) -> Duration {
        if !self.should_retry() {
            return Duration::ZERO;
        }
        get_time_to_wait(
            attempt,
            retry_params.max_retry,
            retry_params.min_wait_ms,
            &self.context.response_headers,
            &self.context.endpoint_category,
        )
    }
}

// ────────────────────────────────────────────────────────────────────────────
// FgaApiError (generic / unclassified)
// ────────────────────────────────────────────────────────────────────────────

/// Generic API error for status codes not handled by the specific variants.
#[derive(Debug, Clone, Error)]
#[error("{message}")]
pub struct FgaApiError {
    /// Human-readable message.
    pub message: String,
    /// Structured request/response context.
    pub context: ApiErrorContext,
}

impl FgaApiError {
    pub(crate) fn new(ctx: ApiErrorContext) -> Self {
        let message = format!(
            "{} error for {} with status {}",
            ctx.request_method, ctx.endpoint_category, ctx.response_status_code
        );
        Self { message, context: ctx }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Result alias
// ────────────────────────────────────────────────────────────────────────────

/// Convenience `Result` type defaulting to [`OpenFgaError`].
pub type Result<T> = std::result::Result<T, OpenFgaError>;
