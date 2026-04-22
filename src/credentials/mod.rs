//! Credentials - mirrors `credentials/credentials.go`.

use reqwest::ClientBuilder;
use serde::{Deserialize, Serialize};

// ────────────────────────────────────────────────────────────────────────────
// CredentialsMethod
// ────────────────────────────────────────────────────────────────────────────

/// Which authentication mechanism to use.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialsMethod {
    /// No authentication (default).
    #[default]
    None,
    /// Static API token sent as `Authorization: Bearer <token>`.
    ApiToken,
    /// Client Credentials OAuth2 flow.
    ClientCredentials,
}

// ────────────────────────────────────────────────────────────────────────────
// CredentialsConfig
// ────────────────────────────────────────────────────────────────────────────

/// Concrete credential configuration details.
#[derive(Debug, Clone, Default)]
pub struct CredentialsConfig {
    /// Static API token (used with [`CredentialsMethod::ApiToken`]).
    pub api_token: Option<String>,

    // Client-Credentials fields:
    /// OAuth2 Client ID.
    pub client_credentials_client_id: Option<String>,
    /// OAuth2 Client Secret.
    pub client_credentials_client_secret: Option<String>,
    /// Token issuer URL (e.g. `https://accounts.example.com`).
    pub client_credentials_api_token_issuer: Option<String>,
    /// Audience claim required by the token issuer.
    pub client_credentials_api_audience: Option<String>,
    /// Scopes to request (space-separated).
    pub client_credentials_scopes: Option<String>,
}

// ────────────────────────────────────────────────────────────────────────────
// Credentials
// ────────────────────────────────────────────────────────────────────────────

/// Holds the chosen credential method and its configuration.
#[derive(Debug, Clone, Default)]
pub struct Credentials {
    /// The credential method.
    pub method: CredentialsMethod,
    /// Method-specific configuration.
    pub config: CredentialsConfig,
}

impl Credentials {
    /// Creates a `Credentials` instance for no authentication.
    pub fn none() -> Self {
        Self::default()
    }

    /// Creates a `Credentials` instance for a static API token.
    pub fn api_token(token: impl Into<String>) -> Self {
        Self {
            method: CredentialsMethod::ApiToken,
            config: CredentialsConfig {
                api_token: Some(token.into()),
                ..Default::default()
            },
        }
    }

    /// Creates a `Credentials` instance for the Client Credentials OAuth2 flow.
    pub fn client_credentials(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        token_issuer: impl Into<String>,
        audience: impl Into<String>,
    ) -> Self {
        Self {
            method: CredentialsMethod::ClientCredentials,
            config: CredentialsConfig {
                client_credentials_client_id: Some(client_id.into()),
                client_credentials_client_secret: Some(client_secret.into()),
                client_credentials_api_token_issuer: Some(token_issuer.into()),
                client_credentials_api_audience: Some(audience.into()),
                ..Default::default()
            },
        }
    }

    /// Validates the credential configuration.
    ///
    /// # Errors
    ///
    /// Returns an error string if required fields are missing.
    pub fn validate(&self) -> Result<(), String> {
        match self.method {
            CredentialsMethod::None => Ok(()),
            CredentialsMethod::ApiToken => {
                if self.config.api_token.as_deref().unwrap_or("").is_empty() {
                    return Err("Credentials.api_token is required for ApiToken method".to_string());
                }
                Ok(())
            }
            CredentialsMethod::ClientCredentials => {
                let required = [
                    (
                        "client_credentials_client_id",
                        &self.config.client_credentials_client_id,
                    ),
                    (
                        "client_credentials_client_secret",
                        &self.config.client_credentials_client_secret,
                    ),
                    (
                        "client_credentials_api_token_issuer",
                        &self.config.client_credentials_api_token_issuer,
                    ),
                    (
                        "client_credentials_api_audience",
                        &self.config.client_credentials_api_audience,
                    ),
                ];
                for (name, val) in required {
                    if val.as_deref().unwrap_or("").is_empty() {
                        return Err(format!(
                            "Credentials.{} is required for ClientCredentials method",
                            name
                        ));
                    }
                }
                Ok(())
            }
        }
    }

    /// Returns static `Authorization` headers (for `ApiToken` only).
    /// For `ClientCredentials`, token injection is done via middleware.
    pub fn static_auth_header(&self) -> Option<String> {
        if self.method == CredentialsMethod::ApiToken {
            self.config
                .api_token
                .as_ref()
                .map(|t| format!("Bearer {}", t))
        } else {
            None
        }
    }

    /// Applies credentials to a `ClientBuilder`.
    ///
    /// - `None`: no-op.
    /// - `ApiToken`: adds a default `Authorization` header.
    /// - `ClientCredentials`: (token refresh middleware added at request time).
    pub(crate) fn apply_to_client_builder(&self, builder: ClientBuilder) -> ClientBuilder {
        if let Some(auth) = self.static_auth_header() {
            let mut headers = reqwest::header::HeaderMap::new();
            if let Ok(val) = reqwest::header::HeaderValue::from_str(&auth) {
                headers.insert(reqwest::header::AUTHORIZATION, val);
            }
            builder.default_headers(headers)
        } else {
            builder
        }
    }
}
