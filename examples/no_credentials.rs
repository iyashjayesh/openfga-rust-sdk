//! Basic usage with no credentials.
//!
//! Run with:
//! ```bash
//! FGA_API_URL=http://localhost:8080 \
//!   cargo run --example no_credentials
//! ```

use openfga_sdk::{
    client::{ClientConfiguration, OpenFgaClient},
    models::{CheckRequest, CheckRequestTupleKey, CreateStoreRequest},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_url =
        std::env::var("FGA_API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    // Step 1: create a bootstrapping client (no store yet) to call CreateStore.
    let bootstrap = OpenFgaClient::new(&ClientConfiguration {
        api_url: api_url.clone(),
        ..Default::default()
    })?;

    let store = bootstrap
        .create_store(CreateStoreRequest {
            name: "my-store".to_string(),
        })
        .await?;
    println!("Created store: {} ({})", store.name, store.id);

    // Step 2: build the real client with the store ID fixed at construction time.
    // store_id and authorization_model_id are immutable for the lifetime of the
    // client — to switch stores, construct a new OpenFgaClient.
    let client = OpenFgaClient::new(&ClientConfiguration {
        api_url: api_url.clone(),
        store_id: Some(store.id.clone()),
        ..Default::default()
    })?;

    // Check a permission (will return false — no tuples written yet).
    let resp = client
        .check(
            CheckRequest::new(CheckRequestTupleKey::new(
                "user:alice",
                "viewer",
                "document:roadmap",
            )),
            None,
        )
        .await?;
    println!("alice viewer document:roadmap => {}", resp.is_allowed());

    // Clean up — use the same client (store_id is already set).
    client.delete_store(None).await?;
    println!("Store deleted.");

    Ok(())
}
