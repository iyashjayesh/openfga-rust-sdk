//! `ApiExecutor` — retry-and-telemetry wrapper around `ApiClient`.
//!
//! Mirrors `api_executor.go` from the Go SDK.

use std::collections::HashMap;

use async_trait::async_trait;
use reqwest::{header::HeaderMap, Method};
use serde::de::DeserializeOwned;
use tokio::time::sleep;

use crate::{
    error::{OpenFgaError, Result},
    internal::retry::RetryParams,
};

use super::api_client::ApiClient;

// ────────────────────────────────────────────────────────────────────────────
// ApiExecutorRequest / Response
// ────────────────────────────────────────────────────────────────────────────

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
    pub fn with_body(mut self, body: serde_json::Value) -> Self {
        self.body = Some(body);
        self
    }

    /// Adds a custom header.
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Resolves path template parameters (e.g. `{store_id}` → actual value).
    pub(crate) fn resolved_path(&self) -> Result<String> {
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
    pub(crate) fn header_map(&self) -> HeaderMap {
        let mut map = HeaderMap::new();
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

/// Raw response from the executor.
#[derive(Debug)]
pub struct ApiExecutorResponse {
    /// HTTP status code.
    pub status_code: u16,
    /// Response headers.
    pub headers: reqwest::header::HeaderMap,
    /// Raw response body bytes.
    pub body: bytes::Bytes,
}

// ────────────────────────────────────────────────────────────────────────────
// ApiExecutor trait
// ────────────────────────────────────────────────────────────────────────────

/// Executes API requests with automatic retry, error handling, and telemetry.
#[async_trait]
pub trait ApiExecutor: Send + Sync {
    /// Executes a request and returns the raw response.
    async fn execute(&self, req: ApiExecutorRequest) -> Result<ApiExecutorResponse>;

    /// Executes a request and decodes the JSON response body into `T`.
    async fn execute_with_decode<T: DeserializeOwned + Send>(
        &self,
        req: ApiExecutorRequest,
    ) -> Result<(ApiExecutorResponse, T)>;
}

// ────────────────────────────────────────────────────────────────────────────
// ApiExecutorImpl
// ────────────────────────────────────────────────────────────────────────────

/// Concrete `ApiExecutor` backed by an [`ApiClient`].
#[derive(Clone)]
pub struct ApiExecutorImpl {
    client: ApiClient,
}

impl ApiExecutorImpl {
    /// Creates a new `ApiExecutorImpl` from an `ApiClient`.
    pub fn new(client: ApiClient) -> Self {
        Self { client }
    }

    fn retry_params(&self) -> RetryParams {
        self.client.cfg.get_retry_params()
    }

    async fn execute_internal(&self, req: &ApiExecutorRequest) -> Result<ApiExecutorResponse> {
        let retry = self.retry_params();
        let store_id = req
            .path_parameters
            .get("store_id")
            .map(String::as_str)
            .unwrap_or("");

        let method = Method::from_bytes(req.method.to_uppercase().as_bytes()).map_err(|_| {
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

            let resp = self
                .client
                .call(method.clone(), &path, headers.clone(), query, req.body.clone())
                .await;

            match resp {
                Ok(r) => {
                    if !r.status().is_success() {
                        let err = ApiClient::handle_error_response(r, store_id, &req.operation_name)
                            .await;
                        if !err.should_retry() {
                            return Err(err);
                        }
                        last_err = Some(err);
                        continue;
                    }

                    let status_code = r.status().as_u16();
                    let resp_headers = r.headers().clone();
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
                    if !e.should_retry() {
                        return Err(e);
                    }
                    last_err = Some(e);
                }
            }
        }

        Err(last_err.unwrap_or_else(|| {
            OpenFgaError::Configuration("Max retries exceeded with no response".to_string())
        }))
    }
}

#[async_trait]
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
