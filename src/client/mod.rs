//! High-level OpenFGA client — mirrors `client/client.go`.
//!
//! [`OpenFgaClient`] is the recommended entry point for users. It wraps the
//! lower-level [`ApiClient`] and [`ApiExecutor`] with:
//!
//! - A fluent builder API: `.body(…).options(…).execute()`
//! - Store ID / authorization model ID management
//! - Batch write chunking (non-transaction mode)
//! - Parallel `ClientBatchCheck`
//! - Convenience helpers: `write_tuples`, `delete_tuples`, `read_latest_authorization_model`

use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use crate::{
    api::{
        api_client::ApiClient,
        configuration::Configuration,
        executor::{ApiExecutor, ApiExecutorImpl, ApiExecutorRequest, ApiExecutorResponse},
    },
    credentials::Credentials,
    error::{OpenFgaError, Result},
    internal::{
        constants::CLIENT_MAX_METHOD_PARALLEL_REQUESTS,
        ulid::is_well_formed_ulid,
    },
    models::{
        AuthorizationModel, BatchCheckRequest, BatchCheckResponse, CheckRequest, CheckResponse,
        ContextualTupleKeys, CreateStoreRequest, CreateStoreResponse,
        ExpandRequest, ExpandResponse, GetStoreResponse, ListObjectsRequest, ListObjectsResponse,
        ListStoresResponse, ListUsersRequest, ListUsersResponse, ReadAssertionsResponse,
        ReadAuthorizationModelResponse, ReadAuthorizationModelsResponse, ReadChangesResponse,
        ReadRequest, ReadResponse, WriteAssertionsRequest,
        WriteAuthorizationModelRequest, WriteAuthorizationModelResponse, WriteRequest,
        WriteRequestDeletes, WriteRequestWrites,
    },
};

// ────────────────────────────────────────────────────────────────────────────
// ClientConfiguration
// ────────────────────────────────────────────────────────────────────────────

/// Configuration for [`OpenFgaClient`].
///
/// Extends [`Configuration`] with `store_id` and `authorization_model_id`.
#[derive(Debug, Clone, Default)]
pub struct ClientConfiguration {
    /// Full API base URL, e.g. `https://api.fga.example`.
    pub api_url: String,
    /// Default store ID (ULID). Can be overridden per request.
    pub store_id: Option<String>,
    /// Default authorization model ID (ULID). Can be overridden per request.
    pub authorization_model_id: Option<String>,
    /// Authentication credentials.
    pub credentials: Option<Credentials>,
    /// Default HTTP headers sent with every request.
    pub default_headers: HashMap<String, String>,
    /// If `true`, log request/response details.
    pub debug: bool,
    /// Retry configuration.
    pub retry_params: Option<crate::internal::retry::RetryParams>,
    /// OpenTelemetry configuration.
    pub telemetry: Option<crate::telemetry::TelemetryConfiguration>,
}

// ────────────────────────────────────────────────────────────────────────────
// Per-request Options
// ────────────────────────────────────────────────────────────────────────────

/// Per-request options shared by most methods.
#[derive(Debug, Clone, Default)]
pub struct ClientRequestOptions {
    /// Override the default store ID for this request.
    pub store_id: Option<String>,
    /// Override the default authorization model ID for this request.
    pub authorization_model_id: Option<String>,
    /// Additional per-request HTTP headers.
    pub headers: Option<HashMap<String, String>>,
}

/// Options specific to `Write`.
#[derive(Debug, Clone, Default)]
pub struct ClientWriteOptions {
    /// Per-request overrides.
    pub base: ClientRequestOptions,
    /// Non-transaction write settings.
    pub transaction: Option<TransactionOptions>,
}

/// Controls non-transaction (chunked) write behaviour.
#[derive(Debug, Clone)]
pub struct TransactionOptions {
    /// If `true`, disable single-transaction writes and use chunked mode.
    pub disable: bool,
    /// Maximum items per write chunk (default: 100).
    pub max_per_chunk: usize,
    /// Maximum parallel chunk requests (default: `CLIENT_MAX_METHOD_PARALLEL_REQUESTS`).
    pub max_parallel_requests: usize,
}

impl Default for TransactionOptions {
    fn default() -> Self {
        Self {
            disable: false,
            max_per_chunk: 100,
            max_parallel_requests: CLIENT_MAX_METHOD_PARALLEL_REQUESTS,
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Client-batch check types
// ────────────────────────────────────────────────────────────────────────────

/// A single item in a client-side batch check request.
#[derive(Debug, Clone)]
pub struct ClientBatchCheckItem {
    /// Subject.
    pub user: String,
    /// Relation.
    pub relation: String,
    /// Object.
    pub object: String,
    /// Caller-provided correlation ID.
    pub correlation_id: String,
    /// Contextual tuples.
    pub contextual_tuples: Option<ContextualTupleKeys>,
    /// ABAC context.
    pub context: Option<serde_json::Value>,
}

/// The result of a single client-side batch check item.
#[derive(Debug)]
pub struct ClientBatchCheckSingleResult {
    /// Whether the check passed. Always `false` on error.
    pub allowed: bool,
    /// The original request item.
    pub request: ClientBatchCheckItem,
    /// Error, if the individual check failed.
    pub error: Option<OpenFgaError>,
}

/// Response from the client-side `batch_check` method.
#[derive(Debug)]
pub struct ClientBatchCheckResponse {
    /// Per-item results keyed by `correlation_id`.
    pub responses: HashMap<String, ClientBatchCheckSingleResult>,
}

// ────────────────────────────────────────────────────────────────────────────
// Write response types
// ────────────────────────────────────────────────────────────────────────────

/// Status of a single write tuple in a non-transaction write.
#[derive(Debug)]
pub enum ClientWriteStatus {
    /// Successfully written.
    Success,
    /// Failed with an error.
    Failure(OpenFgaError),
}

/// Result of a single write chunk.
#[derive(Debug)]
pub struct ClientWriteTupleResult {
    /// The tuple key that was attempted.
    pub tuple_key: crate::models::TupleKey,
    /// Whether this tuple write succeeded.
    pub status: ClientWriteStatus,
}

/// Result of a single delete chunk.
#[derive(Debug)]
pub struct ClientDeleteTupleResult {
    /// The tuple key that was attempted.
    pub tuple_key: crate::models::TupleKeyWithoutCondition,
    /// Whether this tuple delete succeeded.
    pub status: ClientWriteStatus,
}

/// Response from the `write` method (non-transaction mode).
#[derive(Debug)]
pub struct ClientWriteResponse {
    /// Results for all written tuples.
    pub writes: Vec<ClientWriteTupleResult>,
    /// Results for all deleted tuples.
    pub deletes: Vec<ClientDeleteTupleResult>,
}

// ────────────────────────────────────────────────────────────────────────────
// OpenFgaClient
// ────────────────────────────────────────────────────────────────────────────

/// The high-level OpenFGA client.
///
/// This is the recommended entry point for API consumers. Construct it once
/// and reuse it across your application to benefit from HTTP connection pooling
/// and OAuth2 token caching.
///
/// # Example
///
/// ```rust,no_run
/// use openfga_sdk::client::{ClientConfiguration, OpenFgaClient};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = OpenFgaClient::new(&ClientConfiguration {
///         api_url: "https://api.fga.example".to_string(),
///         store_id: Some("01FQH7V8BEG3GPQW93KTRFR8JB".to_string()),
///         ..Default::default()
///     })?;
///
///     // Check a permission
///     use openfga_sdk::models::{CheckRequest, CheckRequestTupleKey};
///     let resp = client
///         .check(CheckRequest::new(CheckRequestTupleKey::new(
///             "user:alice",
///             "viewer",
///             "document:budget",
///         )), None)
///         .await?;
///     println!("allowed = {}", resp.is_allowed());
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct OpenFgaClient {
    executor: ApiExecutorImpl,
    store_id: Arc<RwLock<Option<String>>>,
    authorization_model_id: Arc<RwLock<Option<String>>>,
}

impl OpenFgaClient {
    /// Creates a new `OpenFgaClient` from a [`ClientConfiguration`].
    ///
    /// # Errors
    ///
    /// Returns an error if `api_url` is missing or invalid, credentials fail
    /// validation, or the HTTP client cannot be built.
    pub fn new(cfg: &ClientConfiguration) -> Result<Self> {
        // Validate store_id / model_id are well-formed ULIDs if provided.
        if let Some(ref sid) = cfg.store_id {
            if !sid.is_empty() && !is_well_formed_ulid(sid) {
                return Err(OpenFgaError::InvalidParam {
                    param: "store_id".to_string(),
                    description: format!("'{}' is not a valid ULID", sid),
                });
            }
        }
        if let Some(ref mid) = cfg.authorization_model_id {
            if !mid.is_empty() && !is_well_formed_ulid(mid) {
                return Err(OpenFgaError::InvalidParam {
                    param: "authorization_model_id".to_string(),
                    description: format!("'{}' is not a valid ULID", mid),
                });
            }
        }

        let low_cfg = Configuration::new(Configuration {
            api_url: cfg.api_url.clone(),
            credentials: cfg.credentials.clone(),
            default_headers: cfg.default_headers.clone(),
            user_agent: crate::internal::constants::DEFAULT_USER_AGENT.to_string(),
            debug: cfg.debug,
            http_client: None,
            retry_params: cfg.retry_params.clone(),
            telemetry: cfg.telemetry.clone(),
        })?;

        let api_client = ApiClient::new(low_cfg)?;
        let executor = ApiExecutorImpl::new(api_client);

        Ok(Self {
            executor,
            store_id: Arc::new(RwLock::new(cfg.store_id.clone())),
            authorization_model_id: Arc::new(RwLock::new(cfg.authorization_model_id.clone())),
        })
    }

    // ────────────────────────────────────────────────────────────────────────
    // Store ID / Model ID management
    // ────────────────────────────────────────────────────────────────────────

    /// Returns the current store ID.
    pub async fn store_id(&self) -> Result<String> {
        let guard = self.store_id.read().await;
        guard.clone().ok_or_else(|| {
            OpenFgaError::Configuration("No store_id configured".to_string())
        })
    }

    /// Sets the store ID. Must be a valid ULID.
    pub async fn set_store_id(&self, id: impl Into<String>) -> Result<()> {
        let id = id.into();
        if !is_well_formed_ulid(&id) {
            return Err(OpenFgaError::InvalidParam {
                param: "store_id".to_string(),
                description: format!("'{}' is not a valid ULID", id),
            });
        }
        let mut guard = self.store_id.write().await;
        *guard = Some(id);
        Ok(())
    }

    /// Returns the current authorization model ID.
    pub async fn authorization_model_id(&self) -> Option<String> {
        self.authorization_model_id.read().await.clone()
    }

    /// Sets the authorization model ID. Must be a valid ULID.
    pub async fn set_authorization_model_id(&self, id: impl Into<String>) -> Result<()> {
        let id = id.into();
        if !is_well_formed_ulid(&id) {
            return Err(OpenFgaError::InvalidParam {
                param: "authorization_model_id".to_string(),
                description: format!("'{}' is not a valid ULID", id),
            });
        }
        let mut guard = self.authorization_model_id.write().await;
        *guard = Some(id);
        Ok(())
    }

    // ────────────────────────────────────────────────────────────────────────
    // Helpers
    // ────────────────────────────────────────────────────────────────────────

    async fn effective_store_id(&self, opts: Option<&ClientRequestOptions>) -> Result<String> {
        if let Some(sid) = opts.and_then(|o| o.store_id.as_deref()) {
            return Ok(sid.to_string());
        }
        self.store_id().await
    }

    async fn effective_model_id(&self, opts: Option<&ClientRequestOptions>) -> Option<String> {
        if let Some(mid) = opts.and_then(|o| o.authorization_model_id.as_deref()) {
            return Some(mid.to_string());
        }
        self.authorization_model_id().await
    }

    // ────────────────────────────────────────────────────────────────────────
    // Stores API
    // ────────────────────────────────────────────────────────────────────────

    /// Lists all stores.
    pub async fn list_stores(
        &self,
        page_size: Option<i32>,
        continuation_token: Option<String>,
    ) -> Result<ListStoresResponse> {
        let mut req = ApiExecutorRequest {
            operation_name: "ListStores".to_string(),
            method: "GET".to_string(),
            path: "/stores".to_string(),
            ..Default::default()
        };
        if let Some(ps) = page_size {
            req = req.with_query_param("page_size", ps.to_string());
        }
        if let Some(ct) = continuation_token {
            req = req.with_query_param("continuation_token", ct);
        }
        let (_, resp): (_, ListStoresResponse) = self.executor.execute_with_decode(req).await?;
        Ok(resp)
    }

    /// Creates a new store.
    pub async fn create_store(&self, body: CreateStoreRequest) -> Result<CreateStoreResponse> {
        let req = ApiExecutorRequest {
            operation_name: "CreateStore".to_string(),
            method: "POST".to_string(),
            path: "/stores".to_string(),
            body: Some(serde_json::to_value(&body).map_err(OpenFgaError::Json)?),
            ..Default::default()
        };
        let (_, resp): (_, CreateStoreResponse) = self.executor.execute_with_decode(req).await?;
        Ok(resp)
    }

    /// Gets the current store.
    pub async fn get_store(&self, opts: Option<&ClientRequestOptions>) -> Result<GetStoreResponse> {
        let store_id = self.effective_store_id(opts).await?;
        let req = ApiExecutorRequest {
            operation_name: "GetStore".to_string(),
            method: "GET".to_string(),
            path: "/stores/{store_id}".to_string(),
            ..Default::default()
        }
        .with_path_param("store_id", store_id);
        let (_, resp): (_, GetStoreResponse) = self.executor.execute_with_decode(req).await?;
        Ok(resp)
    }

    /// Deletes the current store.
    pub async fn delete_store(&self, opts: Option<&ClientRequestOptions>) -> Result<()> {
        let store_id = self.effective_store_id(opts).await?;
        let req = ApiExecutorRequest {
            operation_name: "DeleteStore".to_string(),
            method: "DELETE".to_string(),
            path: "/stores/{store_id}".to_string(),
            ..Default::default()
        }
        .with_path_param("store_id", store_id);
        self.executor.execute(req).await?;
        Ok(())
    }

    // ────────────────────────────────────────────────────────────────────────
    // Authorization Models API
    // ────────────────────────────────────────────────────────────────────────

    /// Lists authorization models for the store.
    pub async fn read_authorization_models(
        &self,
        page_size: Option<i32>,
        continuation_token: Option<String>,
        opts: Option<&ClientRequestOptions>,
    ) -> Result<ReadAuthorizationModelsResponse> {
        let store_id = self.effective_store_id(opts).await?;
        let mut req = ApiExecutorRequest {
            operation_name: "ReadAuthorizationModels".to_string(),
            method: "GET".to_string(),
            path: "/stores/{store_id}/authorization-models".to_string(),
            ..Default::default()
        }
        .with_path_param("store_id", store_id);
        if let Some(ps) = page_size {
            req = req.with_query_param("page_size", ps.to_string());
        }
        if let Some(ct) = continuation_token {
            req = req.with_query_param("continuation_token", ct);
        }
        let (_, resp) = self.executor.execute_with_decode(req).await?;
        Ok(resp)
    }

    /// Writes a new authorization model.
    pub async fn write_authorization_model(
        &self,
        body: WriteAuthorizationModelRequest,
        opts: Option<&ClientRequestOptions>,
    ) -> Result<WriteAuthorizationModelResponse> {
        let store_id = self.effective_store_id(opts).await?;
        let req = ApiExecutorRequest {
            operation_name: "WriteAuthorizationModel".to_string(),
            method: "POST".to_string(),
            path: "/stores/{store_id}/authorization-models".to_string(),
            body: Some(serde_json::to_value(&body).map_err(OpenFgaError::Json)?),
            ..Default::default()
        }
        .with_path_param("store_id", store_id);
        let (_, resp) = self.executor.execute_with_decode(req).await?;
        Ok(resp)
    }

    /// Reads a specific authorization model. If `model_id` is `None`, uses the configured default.
    pub async fn read_authorization_model(
        &self,
        opts: Option<&ClientRequestOptions>,
    ) -> Result<ReadAuthorizationModelResponse> {
        let store_id = self.effective_store_id(opts).await?;
        let model_id = self
            .effective_model_id(opts)
            .await
            .ok_or_else(|| OpenFgaError::Configuration("No authorization_model_id configured".to_string()))?;
        let req = ApiExecutorRequest {
            operation_name: "ReadAuthorizationModel".to_string(),
            method: "GET".to_string(),
            path: "/stores/{store_id}/authorization-models/{model_id}".to_string(),
            ..Default::default()
        }
        .with_path_param("store_id", store_id)
        .with_path_param("model_id", model_id);
        let (_, resp) = self.executor.execute_with_decode(req).await?;
        Ok(resp)
    }

    /// Reads the latest authorization model (ignores any configured model ID).
    pub async fn read_latest_authorization_model(
        &self,
        opts: Option<&ClientRequestOptions>,
    ) -> Result<Option<AuthorizationModel>> {
        let store_id = self.effective_store_id(opts).await?;
        let req = ApiExecutorRequest {
            operation_name: "ReadAuthorizationModels".to_string(),
            method: "GET".to_string(),
            path: "/stores/{store_id}/authorization-models".to_string(),
            ..Default::default()
        }
        .with_path_param("store_id", store_id)
        .with_query_param("page_size", "1");
        let (_, resp): (_, ReadAuthorizationModelsResponse) =
            self.executor.execute_with_decode(req).await?;
        Ok(resp.authorization_models.into_iter().next())
    }

    // ────────────────────────────────────────────────────────────────────────
    // Relationship Tuples API
    // ────────────────────────────────────────────────────────────────────────

    /// Reads relationship tuples matching the filter.
    pub async fn read(
        &self,
        body: ReadRequest,
        opts: Option<&ClientRequestOptions>,
    ) -> Result<ReadResponse> {
        let store_id = self.effective_store_id(opts).await?;
        let req = ApiExecutorRequest {
            operation_name: "Read".to_string(),
            method: "POST".to_string(),
            path: "/stores/{store_id}/read".to_string(),
            body: Some(serde_json::to_value(&body).map_err(OpenFgaError::Json)?),
            ..Default::default()
        }
        .with_path_param("store_id", store_id);
        let (_, resp) = self.executor.execute_with_decode(req).await?;
        Ok(resp)
    }

    /// Writes (and/or deletes) relationship tuples.
    ///
    /// In non-transaction mode (`opts.transaction.disable = true`), tuples are split into
    /// chunks and sent in parallel. Each chunk result is tracked individually.
    pub async fn write(
        &self,
        body: WriteRequest,
        opts: Option<&ClientWriteOptions>,
    ) -> Result<()> {
        let base_opts = opts.map(|o| &o.base);
        let store_id = self.effective_store_id(base_opts).await?;
        let model_id = self.effective_model_id(base_opts).await;

        let non_tx = opts
            .and_then(|o| o.transaction.as_ref())
            .map(|t| t.disable)
            .unwrap_or(false);

        if non_tx {
            // Non-transaction mode: chunk and parallelise.
            self.write_non_transactional(body, &store_id, model_id, opts).await
        } else {
            // Single transaction write.
            let req = ApiExecutorRequest {
                operation_name: "Write".to_string(),
                method: "POST".to_string(),
                path: "/stores/{store_id}/write".to_string(),
                body: Some(serde_json::to_value(&body).map_err(OpenFgaError::Json)?),
                ..Default::default()
            }
            .with_path_param("store_id", store_id);
            self.executor.execute(req).await?;
            Ok(())
        }
    }

    async fn write_non_transactional(
        &self,
        body: WriteRequest,
        store_id: &str,
        model_id: Option<String>,
        opts: Option<&ClientWriteOptions>,
    ) -> Result<()> {
        let max_per_chunk = opts
            .and_then(|o| o.transaction.as_ref())
            .map(|t| t.max_per_chunk)
            .unwrap_or(100);

        let writes = body.writes.unwrap_or_default().tuple_keys;
        let deletes = body.deletes.unwrap_or_default().tuple_keys;

        // Chunk writes.
        let write_chunks: Vec<_> = writes.chunks(max_per_chunk).map(|c| c.to_vec()).collect();
        // Chunk deletes.
        let delete_chunks: Vec<_> = deletes.chunks(max_per_chunk).map(|c| c.to_vec()).collect();

        let mut handles = vec![];

        for chunk in write_chunks {
            let executor = self.executor.clone();
            let store_id = store_id.to_string();
            let model_id = model_id.clone();
            handles.push(tokio::spawn(async move {
                let req_body = WriteRequest {
                    writes: Some(WriteRequestWrites { tuple_keys: chunk }),
                    deletes: None,
                    authorization_model_id: model_id,
                };
                let req = ApiExecutorRequest {
                    operation_name: "Write".to_string(),
                    method: "POST".to_string(),
                    path: "/stores/{store_id}/write".to_string(),
                    body: Some(serde_json::to_value(&req_body).unwrap()),
                    ..Default::default()
                }
                .with_path_param("store_id", store_id);
                executor.execute(req).await
            }));
        }

        for chunk in delete_chunks {
            let executor = self.executor.clone();
            let store_id = store_id.to_string();
            let model_id = model_id.clone();
            handles.push(tokio::spawn(async move {
                let req_body = WriteRequest {
                    writes: None,
                    deletes: Some(WriteRequestDeletes { tuple_keys: chunk }),
                    authorization_model_id: model_id,
                };
                let req = ApiExecutorRequest {
                    operation_name: "Write".to_string(),
                    method: "POST".to_string(),
                    path: "/stores/{store_id}/write".to_string(),
                    body: Some(serde_json::to_value(&req_body).unwrap()),
                    ..Default::default()
                }
                .with_path_param("store_id", store_id);
                executor.execute(req).await
            }));
        }

        // Collect results; return first error encountered.
        for handle in handles {
            handle
                .await
                .map_err(|e| OpenFgaError::Configuration(format!("Task join error: {}", e)))??;
        }

        Ok(())
    }

    /// Convenience: write tuples only.
    pub async fn write_tuples(
        &self,
        tuples: Vec<crate::models::TupleKey>,
        opts: Option<&ClientWriteOptions>,
    ) -> Result<()> {
        self.write(
            WriteRequest {
                writes: Some(WriteRequestWrites { tuple_keys: tuples }),
                deletes: None,
                authorization_model_id: None,
            },
            opts,
        )
        .await
    }

    /// Convenience: delete tuples only.
    pub async fn delete_tuples(
        &self,
        tuples: Vec<crate::models::TupleKeyWithoutCondition>,
        opts: Option<&ClientWriteOptions>,
    ) -> Result<()> {
        self.write(
            WriteRequest {
                writes: None,
                deletes: Some(WriteRequestDeletes { tuple_keys: tuples }),
                authorization_model_id: None,
            },
            opts,
        )
        .await
    }

    /// Reads tuple changes.
    pub async fn read_changes(
        &self,
        type_filter: Option<String>,
        page_size: Option<i32>,
        continuation_token: Option<String>,
        opts: Option<&ClientRequestOptions>,
    ) -> Result<ReadChangesResponse> {
        let store_id = self.effective_store_id(opts).await?;
        let mut req = ApiExecutorRequest {
            operation_name: "ReadChanges".to_string(),
            method: "GET".to_string(),
            path: "/stores/{store_id}/changes".to_string(),
            ..Default::default()
        }
        .with_path_param("store_id", store_id);
        if let Some(t) = type_filter {
            req = req.with_query_param("type", t);
        }
        if let Some(ps) = page_size {
            req = req.with_query_param("page_size", ps.to_string());
        }
        if let Some(ct) = continuation_token {
            req = req.with_query_param("continuation_token", ct);
        }
        let (_, resp) = self.executor.execute_with_decode(req).await?;
        Ok(resp)
    }

    // ────────────────────────────────────────────────────────────────────────
    // Relationship Queries
    // ────────────────────────────────────────────────────────────────────────

    /// Checks whether a user has a specific relation with an object.
    pub async fn check(
        &self,
        mut body: CheckRequest,
        opts: Option<&ClientRequestOptions>,
    ) -> Result<CheckResponse> {
        let store_id = self.effective_store_id(opts).await?;
        if body.authorization_model_id.is_none() {
            body.authorization_model_id = self.effective_model_id(opts).await;
        }
        let req = ApiExecutorRequest {
            operation_name: "Check".to_string(),
            method: "POST".to_string(),
            path: "/stores/{store_id}/check".to_string(),
            body: Some(serde_json::to_value(&body).map_err(OpenFgaError::Json)?),
            ..Default::default()
        }
        .with_path_param("store_id", store_id);
        let (_, resp) = self.executor.execute_with_decode(req).await?;
        Ok(resp)
    }

    /// Server-side batch check (requires FGA ≥ 1.8.0).
    pub async fn batch_check(
        &self,
        mut body: BatchCheckRequest,
        opts: Option<&ClientRequestOptions>,
    ) -> Result<BatchCheckResponse> {
        let store_id = self.effective_store_id(opts).await?;
        if body.authorization_model_id.is_none() {
            body.authorization_model_id = self.effective_model_id(opts).await;
        }
        let req = ApiExecutorRequest {
            operation_name: "BatchCheck".to_string(),
            method: "POST".to_string(),
            path: "/stores/{store_id}/batch-check".to_string(),
            body: Some(serde_json::to_value(&body).map_err(OpenFgaError::Json)?),
            ..Default::default()
        }
        .with_path_param("store_id", store_id);
        let (_, resp) = self.executor.execute_with_decode(req).await?;
        Ok(resp)
    }

    /// Client-side batch check — runs multiple `check` calls in parallel.
    ///
    /// Suitable for FGA < 1.8.0 where server-side `BatchCheck` is unavailable.
    /// Errors on individual checks set `allowed = false` and attach the error.
    pub async fn client_batch_check(
        &self,
        items: Vec<ClientBatchCheckItem>,
        opts: Option<&ClientRequestOptions>,
    ) -> Result<ClientBatchCheckResponse> {
        let _max_parallel = CLIENT_MAX_METHOD_PARALLEL_REQUESTS;
        let mut handles: Vec<(String, ClientBatchCheckItem, tokio::task::JoinHandle<Result<(ApiExecutorResponse, CheckResponse)>>)> = Vec::with_capacity(items.len());

        for item in items {
            let executor = self.executor.clone();
            let store_id = self.effective_store_id(opts).await?;
            let model_id = self.effective_model_id(opts).await;
            let body = CheckRequest {
                tuple_key: crate::models::CheckRequestTupleKey::new(
                    item.user.clone(),
                    item.relation.clone(),
                    item.object.clone(),
                ),
                contextual_tuples: item.contextual_tuples.clone(),
                authorization_model_id: model_id,
                trace: None,
                context: item.context.clone(),
                consistency: None,
            };
            let corr_id = item.correlation_id.clone();
            handles.push((
                corr_id,
                item,
                tokio::spawn(async move {
                    let req = ApiExecutorRequest {
                        operation_name: "Check".to_string(),
                        method: "POST".to_string(),
                        path: "/stores/{store_id}/check".to_string(),
                        body: Some(serde_json::to_value(&body).unwrap()),
                        ..Default::default()
                    }
                    .with_path_param("store_id", store_id);
                    executor.execute_with_decode::<CheckResponse>(req).await
                }),
            ));
        }

        let mut responses = HashMap::new();
        for (corr_id, item, handle) in handles {
            let result = match handle.await {
                Ok(Ok((_, check_resp))) => ClientBatchCheckSingleResult {
                    allowed: check_resp.is_allowed(),
                    request: item,
                    error: None,
                },
                Ok(Err(e)) => ClientBatchCheckSingleResult {
                    allowed: false,
                    request: item,
                    error: Some(e),
                },
                Err(e) => ClientBatchCheckSingleResult {
                    allowed: false,
                    request: item,
                    error: Some(OpenFgaError::Configuration(e.to_string())),
                },
            };
            responses.insert(corr_id, result);
        }

        Ok(ClientBatchCheckResponse { responses })
    }

    /// Expands a userset.
    pub async fn expand(
        &self,
        mut body: ExpandRequest,
        opts: Option<&ClientRequestOptions>,
    ) -> Result<ExpandResponse> {
        let store_id = self.effective_store_id(opts).await?;
        if body.authorization_model_id.is_none() {
            body.authorization_model_id = self.effective_model_id(opts).await;
        }
        let req = ApiExecutorRequest {
            operation_name: "Expand".to_string(),
            method: "POST".to_string(),
            path: "/stores/{store_id}/expand".to_string(),
            body: Some(serde_json::to_value(&body).map_err(OpenFgaError::Json)?),
            ..Default::default()
        }
        .with_path_param("store_id", store_id);
        let (_, resp) = self.executor.execute_with_decode(req).await?;
        Ok(resp)
    }

    /// Lists objects a user has access to.
    pub async fn list_objects(
        &self,
        mut body: ListObjectsRequest,
        opts: Option<&ClientRequestOptions>,
    ) -> Result<ListObjectsResponse> {
        let store_id = self.effective_store_id(opts).await?;
        if body.authorization_model_id.is_none() {
            body.authorization_model_id = self.effective_model_id(opts).await;
        }
        let req = ApiExecutorRequest {
            operation_name: "ListObjects".to_string(),
            method: "POST".to_string(),
            path: "/stores/{store_id}/list-objects".to_string(),
            body: Some(serde_json::to_value(&body).map_err(OpenFgaError::Json)?),
            ..Default::default()
        }
        .with_path_param("store_id", store_id);
        let (_, resp) = self.executor.execute_with_decode(req).await?;
        Ok(resp)
    }

    /// Streams objects a user has access to using the NDJSON `StreamedListObjects` endpoint.
    ///
    /// Returns a [`crate::streaming::NdJsonStream`] that yields each object as
    /// it arrives.  Use `futures::StreamExt::next` to iterate.
    ///
    /// ```rust,no_run
    /// use futures::StreamExt;
    /// # async fn example(client: openfga_sdk::client::OpenFgaClient, body: openfga_sdk::models::ListObjectsRequest) -> openfga_sdk::error::Result<()> {
    /// let mut stream = client.stream_list_objects(body, None).await?;
    /// while let Some(item) = stream.next().await {
    ///     println!("{}", item?.object);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn stream_list_objects(
        &self,
        mut body: ListObjectsRequest,
        opts: Option<&ClientRequestOptions>,
    ) -> Result<crate::streaming::NdJsonStream> {
        use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
        use url::Url;

        let store_id = self.effective_store_id(opts).await?;
        if body.authorization_model_id.is_none() {
            body.authorization_model_id = self.effective_model_id(opts).await;
        }

        let api_url = &self.executor.client.cfg.api_url;
        let base = Url::parse(api_url).map_err(OpenFgaError::Url)?;
        let url = base
            .join(&format!("/stores/{}/streamed-list-objects", store_id))
            .map_err(OpenFgaError::Url)?;

        let body_val = serde_json::to_value(&body).map_err(OpenFgaError::Json)?;

        // Build default headers from config.
        let mut headers = HeaderMap::new();
        for (k, v) in &self.executor.client.cfg.default_headers {
            if let (Ok(name), Ok(val)) = (
                HeaderName::from_bytes(k.as_bytes()),
                HeaderValue::from_str(v),
            ) {
                headers.insert(name, val);
            }
        }

        let resp = self
            .executor
            .client
            .http
            .post(url)
            .headers(headers)
            .json(&body_val)
            .send()
            .await
            .map_err(|e| OpenFgaError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            let err =
                crate::api::api_client::ApiClient::handle_error_response(resp, &store_id, "StreamedListObjects")
                    .await;
            return Err(err);
        }

        Ok(crate::streaming::NdJsonStream::new(resp))
    }

    /// Lists users that have a relation with an object.
    pub async fn list_users(
        &self,
        mut body: ListUsersRequest,
        opts: Option<&ClientRequestOptions>,
    ) -> Result<ListUsersResponse> {
        let store_id = self.effective_store_id(opts).await?;
        if body.authorization_model_id.is_none() {
            body.authorization_model_id = self.effective_model_id(opts).await;
        }
        let req = ApiExecutorRequest {
            operation_name: "ListUsers".to_string(),
            method: "POST".to_string(),
            path: "/stores/{store_id}/list-users".to_string(),
            body: Some(serde_json::to_value(&body).map_err(OpenFgaError::Json)?),
            ..Default::default()
        }
        .with_path_param("store_id", store_id);
        let (_, resp) = self.executor.execute_with_decode(req).await?;
        Ok(resp)
    }

    // ────────────────────────────────────────────────────────────────────────
    // Assertions
    // ────────────────────────────────────────────────────────────────────────

    /// Reads assertions for the authorization model.
    pub async fn read_assertions(
        &self,
        opts: Option<&ClientRequestOptions>,
    ) -> Result<ReadAssertionsResponse> {
        let store_id = self.effective_store_id(opts).await?;
        let model_id = self
            .effective_model_id(opts)
            .await
            .ok_or_else(|| OpenFgaError::Configuration("No authorization_model_id configured".to_string()))?;
        let req = ApiExecutorRequest {
            operation_name: "ReadAssertions".to_string(),
            method: "GET".to_string(),
            path: "/stores/{store_id}/assertions/{model_id}".to_string(),
            ..Default::default()
        }
        .with_path_param("store_id", store_id)
        .with_path_param("model_id", model_id);
        let (_, resp) = self.executor.execute_with_decode(req).await?;
        Ok(resp)
    }

    /// Writes assertions for the authorization model.
    pub async fn write_assertions(
        &self,
        body: WriteAssertionsRequest,
        opts: Option<&ClientRequestOptions>,
    ) -> Result<()> {
        let store_id = self.effective_store_id(opts).await?;
        let model_id = self
            .effective_model_id(opts)
            .await
            .ok_or_else(|| OpenFgaError::Configuration("No authorization_model_id configured".to_string()))?;
        let req = ApiExecutorRequest {
            operation_name: "WriteAssertions".to_string(),
            method: "PUT".to_string(),
            path: "/stores/{store_id}/assertions/{model_id}".to_string(),
            body: Some(serde_json::to_value(&body).map_err(OpenFgaError::Json)?),
            ..Default::default()
        }
        .with_path_param("store_id", store_id)
        .with_path_param("model_id", model_id);
        self.executor.execute(req).await?;
        Ok(())
    }
}

