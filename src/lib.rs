//! # OpenFGA Rust SDK
//!
//! The official Rust SDK for [OpenFGA](https://openfga.dev) — an open-source
//! Fine-Grained Authorization solution inspired by Google's Zanzibar paper.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use openfga_sdk::client::{ClientConfiguration, OpenFgaClient};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = OpenFgaClient::new(&ClientConfiguration {
//!         api_url: "https://api.fga.example".to_string(),
//!         store_id: Some(std::env::var("FGA_STORE_ID").unwrap_or_default()),
//!         authorization_model_id: Some(std::env::var("FGA_MODEL_ID").unwrap_or_default()),
//!         ..Default::default()
//!     })?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Feature Flags
//!
//! - `rustls` *(default)*: Use `rustls` for TLS.
//! - `native-tls`: Use the system TLS library.
//! - `opentelemetry`: Enable OpenTelemetry metric emission.

#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod client;
pub mod credentials;
pub mod error;
pub mod models;
pub mod oauth2;
pub mod streaming;
pub mod telemetry;

pub(crate) mod api;
pub(crate) mod internal;

// Re-export commonly used types at the crate root.
pub use client::{ClientConfiguration, OpenFgaClient};
pub use error::OpenFgaError;
pub use models::*;
