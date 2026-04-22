//! SDK configuration - mirrors `configuration.go`.

use std::collections::HashMap;

use reqwest::Client;
use url::Url;

use crate::{
    credentials::Credentials,
    error::{OpenFgaError, Result},
    internal::{
        constants::{DEFAULT_USER_AGENT, SDK_VERSION},
        retry::RetryParams,
    },
    telemetry::TelemetryConfiguration,
};

/// Configuration for the low-level [`super::api_client::ApiClient`].
///
/// Prefer constructing via [`ClientConfiguration`](crate::client::ClientConfiguration)
/// which adds `store_id` and `authorization_model_id`.
#[derive(Debug, Clone)]
pub struct Configuration {
    /// Full API base URL, e.g. `https://api.fga.example`.
    pub api_url: String,
    /// Authentication credentials.
    pub credentials: Option<Credentials>,
    /// Headers sent with every outgoing request.
    pub default_headers: HashMap<String, String>,
    /// `User-Agent` header value.
    pub user_agent: String,
    /// If `true`, log request/response details to stderr.
    pub debug: bool,
    /// Custom `reqwest::Client` (optional - one is created automatically).
    pub http_client: Option<Client>,
    /// Retry configuration.
    pub retry_params: Option<RetryParams>,
    /// OpenTelemetry configuration.
    pub telemetry: Option<TelemetryConfiguration>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            api_url: String::new(),
            credentials: None,
            default_headers: HashMap::new(),
            user_agent: DEFAULT_USER_AGENT.to_string(),
            debug: false,
            http_client: None,
            retry_params: None,
            telemetry: None,
        }
    }
}

impl Configuration {
    /// Creates and validates a new `Configuration`.
    ///
    /// # Errors
    ///
    /// Returns [`OpenFgaError::Configuration`] if `api_url` is empty or not a valid URL,
    /// or if credentials / retry params fail validation.
    pub fn new(mut cfg: Configuration) -> Result<Self> {
        if cfg.api_url.is_empty() {
            return Err(OpenFgaError::Configuration(
                "api_url is required".to_string(),
            ));
        }

        // Validate URL format.
        Url::parse(&cfg.api_url).map_err(|e| {
            OpenFgaError::Configuration(format!(
                "api_url '{}' is not a valid URL: {}",
                cfg.api_url, e
            ))
        })?;

        // Default user agent.
        if cfg.user_agent.is_empty() {
            cfg.user_agent = DEFAULT_USER_AGENT.to_string();
        }

        // Default headers map.
        if cfg.default_headers.is_empty() {
            cfg.default_headers = HashMap::new();
        }

        // Validate retry params.
        if let Some(ref rp) = cfg.retry_params {
            rp.validate().map_err(|e| OpenFgaError::Configuration(e))?;
        } else {
            cfg.retry_params = Some(RetryParams::default());
        }

        // Validate credentials.
        if let Some(ref creds) = cfg.credentials {
            creds
                .validate()
                .map_err(|e| OpenFgaError::Configuration(e))?;
        }

        // Set default telemetry.
        if cfg.telemetry.is_none() {
            cfg.telemetry = Some(TelemetryConfiguration::default());
        }

        Ok(cfg)
    }

    /// Returns the effective retry params (or defaults).
    pub fn get_retry_params(&self) -> RetryParams {
        self.retry_params.clone().unwrap_or_default()
    }

    /// Adds a default header to be sent with all requests.
    #[allow(dead_code)]
    pub fn add_default_header(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.default_headers.insert(key.into(), value.into());
    }

    /// Returns the SDK version string.
    #[allow(dead_code)]
    pub fn sdk_version() -> &'static str {
        SDK_VERSION
    }
}
