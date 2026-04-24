//! `ApiExecutor` trait and its default implementation.
//!
//! The [`ApiExecutor`] trait is always available and can be implemented by
//! consumers who want to bring their own HTTP runtime.
//!
//! [`ApiExecutorImpl`] is the bundled tokio + reqwest implementation.  It is
//! only compiled when the `default-executor` feature is enabled (which is the
//! default).  Mirrors `api_executor.go` from the Go SDK.

use serde::de::DeserializeOwned;

use crate::error::Result;

// ────────────────────────────────────────────────────────────────────────────
// ApiExecutorRequest / Response
// ────────────────────────────────────────────────────────────────────────────

use std::collections::HashMap;

/// A fully-specified API request ready to execute.
#[derive(Debug, Clone, Default)]
pub struct ApiExecutorRequest {
    /// Human-readable operation name (used in logs and telemetry).
    pub operation_name: String,
    /// HTTP method.
    pub method: String,
    /// URL path template (e.g. `/stores/{store_id}/check`).
    pub path: String,
    /// Values to substitute into `path` template variables.
    pub path_parameters: HashMap<String, String>,
    /// URL query parameters.
    pub query_parameters: Vec<(String, String)>,
    /// Request body (will be JSON-serialised).
    pub body: Option<serde_json::Value>,
    /// Additional per-request headers.
    pub headers: HashMap<String, String>,
}

impl ApiExecutorRequest {
    /// Inserts a path parameter value.
    pub fn with_path_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.path_parameters.insert(key.into(), value.into());
        self
    }

    /// Appends a query parameter.
    pub fn with_query_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_parameters.push((key.into(), value.into()));
        self
    }

    /// Sets the JSON body.
    #[allow(dead_code)]
    pub fn with_body(mut self, body: serde_json::Value) -> Self {
        self.body = Some(body);
        self
    }

    /// Adds a custom header.
    #[allow(dead_code)]
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Resolves path template parameters (e.g. `{store_id}` → actual value).
    pub(crate) fn resolved_path(&self) -> Result<String> {
        use crate::error::OpenFgaError;
        let mut path = self.path.clone();
        for (k, v) in &self.path_parameters {
            path = path.replace(&format!("{{{}}}", k), v);
        }
        if path.contains('{') || path.contains('}') {
            return Err(OpenFgaError::Configuration(format!(
                "Not all path parameters were resolved in: {}",
                path
            )));
        }
        Ok(path)
    }

    /// Builds a `HeaderMap` from the per-request headers.
    #[cfg(feature = "default-executor")]
    pub(crate) fn header_map(&self) -> reqwest::header::HeaderMap {
        let mut map = reqwest::header::HeaderMap::new();
        for (k, v) in &self.headers {
            if let (Ok(name), Ok(val)) = (
                reqwest::header::HeaderName::from_bytes(k.as_bytes()),
                reqwest::header::HeaderValue::from_str(v),
            ) {
                map.insert(name, val);
            }
        }
        map
    }
}

// ────────────────────────────────────────────────────────────────────────────
// ApiExecutorResponse
// ────────────────────────────────────────────────────────────────────────────

/// Raw response from the executor.
#[derive(Debug)]
pub struct ApiExecutorResponse {
    /// HTTP status code.
    #[allow(dead_code)]
    pub status_code: u16,
    /// Response headers (only present with `default-executor`).
    #[cfg(feature = "default-executor")]
    #[allow(dead_code)]
    pub headers: reqwest::header::HeaderMap,
    /// Raw response body bytes.
    #[cfg(feature = "default-executor")]
    pub body: bytes::Bytes,
    /// Raw response body (fallback when `default-executor` is disabled).
    #[cfg(not(feature = "default-executor"))]
    pub body: Vec<u8>,
}

// ────────────────────────────────────────────────────────────────────────────
// ApiExecutor trait  (always compiled, no runtime dependency)
// ────────────────────────────────────────────────────────────────────────────

/// Executes API requests with automatic retry, error handling, and telemetry.
///
/// Implement this trait to provide a custom HTTP backend.  The bundled
/// implementation ([`ApiExecutorImpl`]) is available under the
/// `default-executor` feature.
pub trait ApiExecutor: Send + Sync {
    /// Executes a request and returns the raw response.
    fn execute(
        &self,
        req: ApiExecutorRequest,
    ) -> impl Future<Output = Result<ApiExecutorResponse>> + Send;

    /// Executes a request and decodes the JSON response body into `T`.
    fn execute_with_decode<T: DeserializeOwned + Send>(
        &self,
        req: ApiExecutorRequest,
    ) -> impl Future<Output = Result<(ApiExecutorResponse, T)>> + Send;
}

// Bring `Future` into scope for the trait definition above.
use std::future::Future;

// ────────────────────────────────────────────────────────────────────────────
// ApiExecutorImpl  (only with `default-executor`)
// ────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "default-executor")]
mod impl_executor {
    use reqwest::Method;
    use tokio::time::sleep;
    use url::Url;

    use crate::{
        api::api_client::ApiClient,
        error::{OpenFgaError, Result},
        internal::retry::RetryParams,
    };

    #[cfg(feature = "opentelemetry")]
    use crate::telemetry::attributes;
    #[cfg(feature = "opentelemetry")]
    use opentelemetry::KeyValue;
    #[cfg(feature = "opentelemetry")]
    use tokio::time::Instant;

    use super::{ApiExecutor, ApiExecutorRequest, ApiExecutorResponse};
    use serde::de::DeserializeOwned;

    /// Concrete [`ApiExecutor`] backed by an [`ApiClient`] (tokio + reqwest).
    #[derive(Clone, Debug)]
    pub struct ApiExecutorImpl {
        pub(crate) client: ApiClient,
    }

    impl ApiExecutorImpl {
        /// Creates a new `ApiExecutorImpl` from an `ApiClient`.
        pub fn new(client: ApiClient) -> Self {
            Self { client }
        }

        fn retry_params(&self) -> RetryParams {
            self.client.cfg.get_retry_params()
        }

        #[cfg(feature = "opentelemetry")]
        fn get_base_attributes(
            &self,
            req: &ApiExecutorRequest,
            store_id: &str,
            attempt: u32,
        ) -> Vec<KeyValue> {
            let mut attrs = vec![
                KeyValue::new(
                    attributes::FGA_CLIENT_REQUEST_METHOD,
                    req.operation_name.clone(),
                ),
                KeyValue::new(
                    attributes::FGA_CLIENT_REQUEST_STORE_ID,
                    store_id.to_string(),
                ),
                KeyValue::new(
                    attributes::HTTP_HOST,
                    Url::parse(&self.client.cfg.api_url)
                        .map(|u| u.host_str().unwrap_or("").to_string())
                        .unwrap_or_default(),
                ),
                KeyValue::new(attributes::HTTP_REQUEST_METHOD, req.method.to_uppercase()),
            ];

            if attempt > 1 {
                attrs.push(KeyValue::new(
                    attributes::HTTP_REQUEST_RESEND_COUNT,
                    (attempt - 1) as i64,
                ));
            }

            if let Some(body) = req.body.as_ref() {
                if let Some(tuple_key) = body.get("tuple_key") {
                    if let Some(user) = tuple_key.get("user").and_then(|u| u.as_str()) {
                        attrs.push(KeyValue::new(attributes::FGA_CLIENT_USER, user.to_string()));
                    }
                }
            }

            attrs
        }

        async fn execute_internal(&self, req: &ApiExecutorRequest) -> Result<ApiExecutorResponse> {
            #[cfg(feature = "opentelemetry")]
            let start_time = Instant::now();
            let retry = self.retry_params();
            let store_id = req
                .path_parameters
                .get("store_id")
                .map(String::as_str)
                .unwrap_or("");

            let method =
                Method::from_bytes(req.method.to_uppercase().as_bytes()).map_err(|_| {
                    OpenFgaError::Configuration(format!("Invalid HTTP method: {}", req.method))
                })?;
            let path = req.resolved_path()?;
            let headers = req.header_map();
            let query = &req.query_parameters;

            let mut last_err: Option<OpenFgaError> = None;

            for attempt in 0..=(retry.max_retry) {
                // Wait before retrying (not before the first attempt).
                if attempt > 0 {
                    if let Some(ref err) = last_err {
                        let wait = err.get_time_to_wait(attempt - 1, &retry);
                        if wait.is_zero() {
                            break;
                        }
                        sleep(wait).await;
                    }
                }

                #[cfg(feature = "opentelemetry")]
                let http_start = Instant::now();
                let resp = self
                    .client
                    .call(
                        method.clone(),
                        &path,
                        headers.clone(),
                        query,
                        req.body.clone(),
                    )
                    .await;

                #[cfg(feature = "opentelemetry")]
                let http_duration = http_start.elapsed().as_millis() as f64;

                match resp {
                    Ok(r) => {
                        if !r.status().is_success() {
                            let err =
                                ApiClient::handle_error_response(r, store_id, &req.operation_name)
                                    .await;
                            if !err.should_retry() {
                                return Err(err);
                            }
                            last_err = Some(err);
                            continue;
                        }

                        let status_code = r.status().as_u16();
                        let resp_headers = r.headers().clone();

                        #[cfg(feature = "opentelemetry")]
                        {
                            let mut attrs = self.get_base_attributes(req, store_id, attempt + 1);
                            attrs.push(KeyValue::new(
                                attributes::HTTP_RESPONSE_STATUS_CODE,
                                status_code as i64,
                            ));

                            self.client
                                .telemetry
                                .record_http_request_duration(http_duration, &attrs);

                            if let Some(val) = resp_headers.get("fga-query-duration-ms") {
                                if let Ok(ms_str) = val.to_str() {
                                    if let Ok(ms) = ms_str.parse::<f64>() {
                                        self.client.telemetry.record_query_duration(ms, &attrs);
                                    }
                                }
                            }

                            if let Some(val) = resp_headers.get("openfga-authorization-model-id") {
                                if let Ok(model_id) = val.to_str() {
                                    attrs.push(KeyValue::new(
                                        attributes::FGA_CLIENT_RESPONSE_MODEL_ID,
                                        model_id.to_string(),
                                    ));
                                }
                            }

                            let total_ms = start_time.elapsed().as_millis() as f64;
                            self.client
                                .telemetry
                                .record_request_duration(total_ms, &attrs);
                            self.client.telemetry.record_request_count(&attrs);
                        }

                        let body = r
                            .bytes()
                            .await
                            .map_err(|e| OpenFgaError::Http(e.to_string()))?;
                        return Ok(ApiExecutorResponse {
                            status_code,
                            headers: resp_headers,
                            body,
                        });
                    }
                    Err(e) => {
                        #[cfg(feature = "opentelemetry")]
                        {
                            let mut attrs = self.get_base_attributes(req, store_id, attempt + 1);
                            if let Some(status) = e.status_code() {
                                attrs.push(KeyValue::new(
                                    attributes::HTTP_RESPONSE_STATUS_CODE,
                                    status as i64,
                                ));
                            }
                            self.client
                                .telemetry
                                .record_http_request_duration(http_duration, &attrs);
                        }

                        if !e.should_retry() {
                            #[cfg(feature = "opentelemetry")]
                            {
                                let mut attrs =
                                    self.get_base_attributes(req, store_id, attempt + 1);
                                if let Some(status) = e.status_code() {
                                    attrs.push(KeyValue::new(
                                        attributes::HTTP_RESPONSE_STATUS_CODE,
                                        status as i64,
                                    ));
                                }
                                let total_ms = start_time.elapsed().as_millis() as f64;
                                self.client
                                    .telemetry
                                    .record_request_duration(total_ms, &attrs);
                                self.client.telemetry.record_request_count(&attrs);
                            }
                            return Err(e);
                        }
                        last_err = Some(e);
                    }
                }
            }

            let err = last_err.unwrap_or_else(|| {
                OpenFgaError::Configuration("Max retries exceeded with no response".to_string())
            });

            #[cfg(feature = "opentelemetry")]
            {
                let mut attrs = self.get_base_attributes(req, store_id, retry.max_retry + 1);
                if let Some(status) = err.status_code() {
                    attrs.push(KeyValue::new(
                        attributes::HTTP_RESPONSE_STATUS_CODE,
                        status as i64,
                    ));
                }
                let total_ms = start_time.elapsed().as_millis() as f64;
                self.client
                    .telemetry
                    .record_request_duration(total_ms, &attrs);
                self.client.telemetry.record_request_count(&attrs);
            }

            Err(err)
        }
    }

    impl ApiExecutor for ApiExecutorImpl {
        async fn execute(&self, req: ApiExecutorRequest) -> Result<ApiExecutorResponse> {
            self.execute_internal(&req).await
        }

        async fn execute_with_decode<T: DeserializeOwned + Send>(
            &self,
            req: ApiExecutorRequest,
        ) -> Result<(ApiExecutorResponse, T)> {
            let resp = self.execute_internal(&req).await?;
            let decoded: T = serde_json::from_slice(&resp.body).map_err(OpenFgaError::Json)?;
            Ok((resp, decoded))
        }
    }
}

#[cfg(feature = "default-executor")]
pub use impl_executor::ApiExecutorImpl;
