//! Low-level API client — mirrors `api_client.go`.


use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client, Method, Response,
};
use url::Url;

use crate::{
    error::{
        ApiErrorContext, FgaApiAuthenticationError, FgaApiError, FgaApiInternalError,
        FgaApiNotFoundError, FgaApiRateLimitExceededError, FgaApiValidationError, OpenFgaError,
        Result,
    },
};

use super::configuration::Configuration;

/// Low-level HTTP client for the OpenFGA API.
///
/// You usually interact with the SDK via [`OpenFgaClient`](crate::client::OpenFgaClient)
/// rather than this type directly.
#[derive(Debug, Clone)]
pub struct ApiClient {
    pub(crate) cfg: Configuration,
    pub(crate) http: Client,
}

impl ApiClient {
    /// Creates a new `ApiClient` from a validated `Configuration`.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be built.
    pub fn new(cfg: Configuration) -> Result<Self> {
        let http = if let Some(client) = cfg.http_client.clone() {
            client
        } else {
            Self::build_http_client(&cfg)?
        };
        Ok(Self { cfg, http })
    }

    /// Builds a default `reqwest::Client` based on the configuration.
    fn build_http_client(cfg: &Configuration) -> Result<Client> {
        let mut builder = Client::builder().user_agent(&cfg.user_agent);

        // Apply credential-specific transport headers.
        if let Some(ref creds) = cfg.credentials {
            builder = creds.apply_to_client_builder(builder);
        }

        builder
            .build()
            .map_err(|e| OpenFgaError::Http(e.to_string()))
    }

    /// Returns the `Configuration` used by this client.
    pub fn config(&self) -> &Configuration {
        &self.cfg
    }

    /// Performs a single HTTP call with no retry logic.
    ///
    /// This is the innermost transport layer used by [`ApiExecutor`](super::executor::ApiExecutor).
    pub(crate) async fn call(
        &self,
        method: Method,
        path: &str,
        headers: HeaderMap,
        query: &[(String, String)],
        body: Option<serde_json::Value>,
    ) -> Result<Response> {
        let base = Url::parse(&self.cfg.api_url).map_err(OpenFgaError::Url)?;
        let mut url = base.join(path).map_err(OpenFgaError::Url)?;

        for (k, v) in query {
            url.query_pairs_mut().append_pair(k, v);
        }

        if self.cfg.debug {
            eprintln!(
                "[openfga-sdk] --> {} {} {:?}",
                method,
                url,
                body.as_ref().map(|b| b.to_string())
            );
        }

        let mut req = self.http.request(method, url);

        // Set default + custom headers.
        for (k, v) in &self.cfg.default_headers {
            if let (Ok(name), Ok(val)) = (
                HeaderName::from_bytes(k.as_bytes()),
                HeaderValue::from_str(v),
            ) {
                req = req.header(name, val);
            }
        }
        req = req.headers(headers);

        // Attach JSON body.
        if let Some(body) = body {
            req = req.json(&body);
        }

        let resp = req.send().await.map_err(|e| OpenFgaError::Http(e.to_string()))?;

        if self.cfg.debug {
            eprintln!("[openfga-sdk] <-- {}", resp.status());
        }

        Ok(resp)
    }

    /// Converts an HTTP error response into a typed [`OpenFgaError`].
    pub(crate) async fn handle_error_response(
        resp: Response,
        store_id: &str,
        endpoint: &str,
    ) -> OpenFgaError {
        let status = resp.status().as_u16();
        let headers = resp.headers().clone();
        let body = resp.bytes().await.unwrap_or_default().to_vec();

        let ctx = ApiErrorContext::from_response(
            store_id,
            endpoint,
            "UNKNOWN",
            "",
            status,
            &headers,
            body,
        );

        match status {
            400 | 422 => OpenFgaError::Validation(FgaApiValidationError::new(ctx)),
            401 | 403 => OpenFgaError::Authentication(FgaApiAuthenticationError::new(ctx)),
            404 => OpenFgaError::NotFound(FgaApiNotFoundError::new(ctx)),
            429 => OpenFgaError::RateLimitExceeded(FgaApiRateLimitExceededError::new(ctx)),
            s if s >= 500 => OpenFgaError::Internal(FgaApiInternalError::new(ctx)),
            _ => OpenFgaError::Api(FgaApiError::new(ctx)),
        }
    }
}
