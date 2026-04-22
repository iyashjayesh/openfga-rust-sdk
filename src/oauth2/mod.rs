//! OAuth2 Client Credentials flow — mirrors `oauth2/` from the Go SDK.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use reqwest::Client;
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::{
    error::{OpenFgaError, Result},
    internal::constants::TOKEN_EXPIRY_JITTER_SECS,
};

// ────────────────────────────────────────────────────────────────────────────
// Token response from the issuer
// ────────────────────────────────────────────────────────────────────────────

/// The JSON body returned by an OAuth2 token endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
    /// The bearer token.
    pub access_token: String,
    /// Seconds until the token expires.
    pub expires_in: Option<u64>,
    /// Token type (should be "Bearer").
    pub token_type: Option<String>,
}

// ────────────────────────────────────────────────────────────────────────────
// Cached token
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct CachedToken {
    access_token: String,
    expires_at: Instant,
}

impl CachedToken {
    fn new(token: &str, expires_in_secs: u64) -> Self {
        // Subtract jitter to refresh before the token actually expires.
        let effective_secs = expires_in_secs.saturating_sub(TOKEN_EXPIRY_JITTER_SECS);
        Self {
            access_token: token.to_string(),
            expires_at: Instant::now() + Duration::from_secs(effective_secs),
        }
    }

    fn is_valid(&self) -> bool {
        Instant::now() < self.expires_at
    }
}

// ────────────────────────────────────────────────────────────────────────────
// ClientCredentialsProvider
// ────────────────────────────────────────────────────────────────────────────

/// Parameters for the Client Credentials OAuth2 grant.
#[derive(Debug, Clone)]
pub struct ClientCredentialsParams {
    /// OAuth2 Client ID.
    pub client_id: String,
    /// OAuth2 Client Secret.
    pub client_secret: String,
    /// Token endpoint URL.
    pub token_url: String,
    /// Audience claim.
    pub audience: String,
    /// Space-separated scopes.
    pub scopes: Option<String>,
}

/// Fetches and caches OAuth2 client credential access tokens.
///
/// Thread-safe — can be cloned and shared across tasks.
#[derive(Debug, Clone)]
pub struct ClientCredentialsProvider {
    params: ClientCredentialsParams,
    http: Client,
    cache: Arc<Mutex<Option<CachedToken>>>,
}

impl ClientCredentialsProvider {
    /// Creates a new provider.
    pub fn new(params: ClientCredentialsParams, http: Client) -> Self {
        Self {
            params,
            http,
            cache: Arc::new(Mutex::new(None)),
        }
    }

    /// Returns a valid access token, fetching a new one if expired or absent.
    pub async fn get_token(&self) -> Result<String> {
        let mut cache = self.cache.lock().await;
        if let Some(ref cached) = *cache {
            if cached.is_valid() {
                return Ok(cached.access_token.clone());
            }
        }
        // Fetch a new token.
        let token_resp = self.fetch_token().await?;
        let expires_in = token_resp.expires_in.unwrap_or(3600);
        let cached = CachedToken::new(&token_resp.access_token, expires_in);
        let token = cached.access_token.clone();
        *cache = Some(cached);
        Ok(token)
    }

    async fn fetch_token(&self) -> Result<TokenResponse> {
        let mut form = vec![
            ("grant_type", "client_credentials".to_string()),
            ("client_id", self.params.client_id.clone()),
            ("client_secret", self.params.client_secret.clone()),
            ("audience", self.params.audience.clone()),
        ];
        if let Some(ref scopes) = self.params.scopes {
            form.push(("scope", scopes.clone()));
        }

        let resp = self
            .http
            .post(&self.params.token_url)
            .form(&form)
            .send()
            .await
            .map_err(|e| OpenFgaError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(OpenFgaError::Configuration(format!(
                "Token fetch failed (HTTP {}): {}",
                status, body
            )));
        }

        resp.json::<TokenResponse>()
            .await
            .map_err(|e| OpenFgaError::Http(e.to_string()))
    }
}
