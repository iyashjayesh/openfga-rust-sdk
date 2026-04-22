//! Basic usage with no credentials.
//!
//! Run with:
//! ```bash
//! FGA_API_URL=http://localhost:8080 \
//! FGA_STORE_ID=<your-store-id> \
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

    // Initialize the client (no credentials).
    let client = OpenFgaClient::new(&ClientConfiguration {
        api_url,
        ..Default::default()
    })?;

    // Create a store.
    let store = client
        .create_store(CreateStoreRequest {
            name: "my-store".to_string(),
        })
        .await?;
    println!("Created store: {} ({})", store.name, store.id);

    // Set the new store as active.
    client.set_store_id(&store.id).await?;

    // Check a permission (will return false - no tuples written yet).
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

    // Clean up.
    client.delete_store(None).await?;
    println!("Store deleted.");

    Ok(())
}
