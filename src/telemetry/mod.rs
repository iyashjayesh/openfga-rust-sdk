//! OpenTelemetry telemetry — mirrors `telemetry/` from the Go SDK.
//!
//! Telemetry is optional and gated behind the `opentelemetry` feature flag.

// ────────────────────────────────────────────────────────────────────────────
// TelemetryConfiguration
// ────────────────────────────────────────────────────────────────────────────

/// Top-level telemetry configuration.
#[derive(Debug, Clone, Default)]
pub struct TelemetryConfiguration {
    /// Metric-level configuration.
    pub metrics: MetricsConfiguration,
}

/// Configures which metrics and attributes are emitted.
#[derive(Debug, Clone)]
pub struct MetricsConfiguration {
    /// Enable `fga_client_request_duration` histogram.
    pub request_duration: bool,
    /// Enable `fga_client_query_duration` histogram.
    pub query_duration: bool,
    /// Enable `fga_client_request_count` counter.
    pub request_count: bool,
    /// Enable `http_client_request_duration` histogram.
    pub http_request_duration: bool,
    /// High-cardinality attributes to include (disabled by default).
    /// Example: `"url.full"`, `"fga_client.user"`.
    pub enabled_attributes: Vec<String>,
}

impl Default for MetricsConfiguration {
    fn default() -> Self {
        Self {
            request_duration: true,
            query_duration: true,
            request_count: true,
            http_request_duration: true,
            // High-cardinality attributes are disabled by default.
            enabled_attributes: vec![
                "http.host".to_string(),
                "http.request.method".to_string(),
                "http.response.status_code".to_string(),
                "user_agent.original".to_string(),
                "fga_client.request.client_id".to_string(),
                "fga_client.request.method".to_string(),
                "fga_client.request.model_id".to_string(),
                "fga_client.request.store_id".to_string(),
                "fga_client.response.model_id".to_string(),
            ],
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Attribute key constants (mirrors telemetry/attributes.go)
// ────────────────────────────────────────────────────────────────────────────

/// Attribute key constants for OpenTelemetry metric attributes.
pub mod attributes {
    pub const HTTP_HOST: &str = "http.host";
    pub const HTTP_REQUEST_METHOD: &str = "http.request.method";
    pub const HTTP_RESPONSE_STATUS_CODE: &str = "http.response.status_code";
    pub const URL_FULL: &str = "url.full";
    pub const URL_SCHEME: &str = "url.scheme";
    pub const USER_AGENT_ORIGINAL: &str = "user_agent.original";
    pub const FGA_CLIENT_REQUEST_CLIENT_ID: &str = "fga_client.request.client_id";
    pub const FGA_CLIENT_REQUEST_METHOD: &str = "fga_client.request.method";
    pub const FGA_CLIENT_REQUEST_MODEL_ID: &str = "fga_client.request.model_id";
    pub const FGA_CLIENT_REQUEST_STORE_ID: &str = "fga_client.request.store_id";
    pub const FGA_CLIENT_RESPONSE_MODEL_ID: &str = "fga_client.response.model_id";
    /// High-cardinality — disabled by default.
    pub const FGA_CLIENT_USER: &str = "fga_client.user";
}

// ────────────────────────────────────────────────────────────────────────────
// Metric name constants (mirrors telemetry/metrics.go)
// ────────────────────────────────────────────────────────────────────────────

/// Metric name constants.
pub mod metrics {
    pub const REQUEST_DURATION: &str = "fga_client.request.duration";
    pub const QUERY_DURATION: &str = "fga_client.query.duration";
    pub const REQUEST_COUNT: &str = "fga_client.request.count";
    pub const CREDENTIALS_REQUEST: &str = "fga_client.credentials.request";
    pub const HTTP_REQUEST_DURATION: &str = "http.client.request.duration";
}

// ────────────────────────────────────────────────────────────────────────────
// Noop implementation (always compiled in)
// ────────────────────────────────────────────────────────────────────────────

/// A no-op recorder used when the `opentelemetry` feature is disabled.
#[derive(Debug, Clone, Default)]
pub struct NoopTelemetry;

impl NoopTelemetry {
    /// Records a request count (no-op).
    pub fn record_request_count(&self, _operation: &str, _store_id: &str) {}
    /// Records a request duration (no-op).
    pub fn record_request_duration(&self, _ms: f64, _operation: &str, _store_id: &str) {}
    /// Records a query duration (no-op).
    pub fn record_query_duration(&self, _ms: f64, _operation: &str, _store_id: &str) {}
}
