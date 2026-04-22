//! OpenTelemetry telemetry — mirrors `telemetry/` from the Go SDK.
//!
//! Telemetry is optional and gated behind the `opentelemetry` feature flag.

#[cfg(feature = "opentelemetry")]
use opentelemetry::{
    metrics::{Counter, Histogram, Meter},
    KeyValue,
};

#[cfg(not(feature = "opentelemetry"))]
/// Stub for `KeyValue` when OpenTelemetry is disabled.
#[derive(Debug, Clone)]
pub struct KeyValue {
    _priv: (),
}

#[cfg(not(feature = "opentelemetry"))]
impl KeyValue {
    /// No-op constructor.
    pub fn new(_k: impl Into<String>, _v: impl Into<String>) -> Self {
        Self { _priv: () }
    }
}

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
    /// Enable `fga_client.request.duration` histogram.
    pub request_duration: bool,
    /// Enable `fga_client.query.duration` histogram.
    pub query_duration: bool,
    /// Enable `fga_client.request.count` counter.
    pub request_count: bool,
    /// Enable `http.client.request.duration` histogram.
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
    /// The host portion of the request URL.
    pub const HTTP_HOST: &str = "http.host";
    /// The HTTP request method (GET, POST, etc.).
    pub const HTTP_REQUEST_METHOD: &str = "http.request.method";
    /// The HTTP response status code.
    pub const HTTP_RESPONSE_STATUS_CODE: &str = "http.response.status_code";
    /// The full request URL.
    pub const URL_FULL: &str = "url.full";
    /// The URL scheme (http, https).
    pub const URL_SCHEME: &str = "url.scheme";
    /// The user agent string.
    pub const USER_AGENT_ORIGINAL: &str = "user_agent.original";
    /// The client ID used for the request.
    pub const FGA_CLIENT_REQUEST_CLIENT_ID: &str = "fga_client.request.client_id";
    /// The FGA operation name (e.g. Check).
    pub const FGA_CLIENT_REQUEST_METHOD: &str = "fga_client.request.method";
    /// The authorization model ID from the request.
    pub const FGA_CLIENT_REQUEST_MODEL_ID: &str = "fga_client.request.model_id";
    /// The store ID from the request.
    pub const FGA_CLIENT_REQUEST_STORE_ID: &str = "fga_client.request.store_id";
    /// The authorization model ID from the response.
    pub const FGA_CLIENT_RESPONSE_MODEL_ID: &str = "fga_client.response.model_id";
    /// High-cardinality user ID (disabled by default).
    pub const FGA_CLIENT_USER: &str = "fga_client.user";
}

// ────────────────────────────────────────────────────────────────────────────
// Metric name constants (mirrors telemetry/metrics.go)
// ────────────────────────────────────────────────────────────────────────────

/// Metric name constants.
pub mod metrics {
    /// Total SDK request duration (including retries).
    pub const REQUEST_DURATION: &str = "fga_client.request.duration";
    /// Server-side processing time.
    pub const QUERY_DURATION: &str = "fga_client.query.duration";
    /// Total number of SDK requests.
    pub const REQUEST_COUNT: &str = "fga_client.request.count";
    /// Number of credential refresh requests.
    pub const CREDENTIALS_REQUEST: &str = "fga_client.credentials.request";
    /// HTTP round-trip duration.
    pub const HTTP_REQUEST_DURATION: &str = "http.client.request.duration";
}

// ────────────────────────────────────────────────────────────────────────────
// FgaTelemetry implementation
// ────────────────────────────────────────────────────────────────────────────

/// Recorder that emits OpenTelemetry metrics if the feature is enabled.
#[derive(Debug, Clone, Default)]
pub struct FgaTelemetry {
    pub(crate) config: TelemetryConfiguration,
    #[cfg(feature = "opentelemetry")]
    pub(crate) instruments: Option<Instruments>,
}

#[cfg(feature = "opentelemetry")]
#[derive(Debug, Clone)]
pub(crate) struct Instruments {
    pub request_duration: Histogram<f64>,
    pub query_duration: Histogram<f64>,
    pub request_count: Counter<u64>,
    pub http_request_duration: Histogram<f64>,
}

impl FgaTelemetry {
    /// Creates a new `FgaTelemetry` from configuration.
    pub fn new(config: TelemetryConfiguration) -> Self {
        #[cfg(feature = "opentelemetry")]
        {
            let meter = opentelemetry::global::meter("openfga-sdk");
            Self {
                instruments: Some(Instruments::new(&meter)),
                config,
            }
        }
        #[cfg(not(feature = "opentelemetry"))]
        {
            Self { config }
        }
    }

    /// Records an HTTP request duration.
    pub fn record_http_request_duration(&self, ms: f64, attributes: &[KeyValue]) {
        #[cfg(feature = "opentelemetry")]
        if self.config.metrics.http_request_duration {
            if let Some(ref inst) = self.instruments {
                inst.http_request_duration.record(ms, attributes);
            }
        }
        let _ = (ms, attributes);
    }

    /// Records a total FGA request duration.
    pub fn record_request_duration(&self, ms: f64, attributes: &[KeyValue]) {
        #[cfg(feature = "opentelemetry")]
        if self.config.metrics.request_duration {
            if let Some(ref inst) = self.instruments {
                inst.request_duration.record(ms, attributes);
            }
        }
        let _ = (ms, attributes);
    }

    /// Records server-side query duration.
    pub fn record_query_duration(&self, ms: f64, attributes: &[KeyValue]) {
        #[cfg(feature = "opentelemetry")]
        if self.config.metrics.query_duration {
            if let Some(ref inst) = self.instruments {
                inst.query_duration.record(ms, attributes);
            }
        }
        let _ = (ms, attributes);
    }

    /// Increments the request count.
    pub fn record_request_count(&self, attributes: &[KeyValue]) {
        #[cfg(feature = "opentelemetry")]
        if self.config.metrics.request_count {
            if let Some(ref inst) = self.instruments {
                inst.request_count.add(1, attributes);
            }
        }
        let _ = attributes;
    }
}

#[cfg(feature = "opentelemetry")]
impl Instruments {
    fn new(meter: &Meter) -> Self {
        Self {
            request_duration: meter
                .f64_histogram(metrics::REQUEST_DURATION)
                .with_description("Total SDK request duration")
                .with_unit("ms")
                .build(),
            query_duration: meter
                .f64_histogram(metrics::QUERY_DURATION)
                .with_description("Server-side query duration")
                .with_unit("ms")
                .build(),
            request_count: meter
                .u64_counter(metrics::REQUEST_COUNT)
                .with_description("Total number of SDK requests")
                .build(),
            http_request_duration: meter
                .f64_histogram(metrics::HTTP_REQUEST_DURATION)
                .with_description("HTTP round-trip duration")
                .with_unit("ms")
                .build(),
        }
    }
}
