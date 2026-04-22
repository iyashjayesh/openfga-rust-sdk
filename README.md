# OpenFGA Rust SDK

[![crates.io](https://img.shields.io/crates/v/openfga-sdk.svg)](https://crates.io/crates/openfga-sdk)
[![docs.rs](https://docs.rs/openfga-sdk/badge.svg)](https://docs.rs/openfga-sdk)
[![CI](https://github.com/openfga/rust-sdk/actions/workflows/ci.yml/badge.svg)](https://github.com/openfga/rust-sdk/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

The official Rust SDK for [OpenFGA](https://openfga.dev) — an open-source Fine-Grained Authorization system inspired by Google Zanzibar.

## Features

- ✅ Full coverage of the [OpenFGA HTTP API](https://openfga.dev/api/service) v1
- ✅ Async/await via [Tokio](https://tokio.rs)
- ✅ Automatic retry with exponential backoff + `Retry-After` header respect
- ✅ Three credential modes: **None**, **API Token**, **OAuth2 Client Credentials**
- ✅ Non-transaction write batching (chunked parallel writes)
- ✅ Client-side parallel batch check
- ✅ Server-side `BatchCheck` (FGA ≥ 1.8.0)
- ✅ Streaming `ListObjects` (NDJSON async iteration)
- ✅ OpenTelemetry metrics (optional `opentelemetry` feature)
- ✅ TLS via `rustls` (default) or `native-tls`

## Installation

```toml
[dependencies]
openfga-sdk = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

### No credentials (local dev)

```rust,no_run
use openfga_sdk::{
    client::{ClientConfiguration, OpenFgaClient},
    models::{CheckRequest, CheckRequestTupleKey, CreateStoreRequest, TupleKey},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenFgaClient::new(&ClientConfiguration {
        api_url: "http://localhost:8080".to_string(),
        ..Default::default()
    })?;

    // 1. Create a store
    let store = client.create_store(CreateStoreRequest {
        name: "my-store".to_string(),
    }).await?;
    client.set_store_id(&store.id).await?;

    // 2. Write a tuple
    client.write_tuples(
        vec![TupleKey::new("user:alice", "viewer", "document:roadmap")],
        None,
    ).await?;

    // 3. Check a permission
    let resp = client.check(
        CheckRequest::new(CheckRequestTupleKey::new(
            "user:alice", "viewer", "document:roadmap",
        )),
        None,
    ).await?;

    println!("alice can view roadmap: {}", resp.is_allowed()); // true
    Ok(())
}
```

### API Token authentication

```rust,no_run
use openfga_sdk::{client::{ClientConfiguration, OpenFgaClient}, credentials::Credentials};

let client = OpenFgaClient::new(&ClientConfiguration {
    api_url: "https://api.fga.example".to_string(),
    store_id: Some(std::env::var("FGA_STORE_ID")?),
    credentials: Some(Credentials::api_token(std::env::var("FGA_API_TOKEN")?)),
    ..Default::default()
})?;
```

### OAuth2 Client Credentials

```rust,no_run
use openfga_sdk::{client::{ClientConfiguration, OpenFgaClient}, credentials::Credentials};

let client = OpenFgaClient::new(&ClientConfiguration {
    api_url: std::env::var("FGA_API_URL")?,
    store_id: Some(std::env::var("FGA_STORE_ID")?),
    credentials: Some(Credentials::client_credentials(
        std::env::var("FGA_CLIENT_ID")?,
        std::env::var("FGA_CLIENT_SECRET")?,
        std::env::var("FGA_TOKEN_ISSUER")?,
        std::env::var("FGA_API_AUDIENCE")?,
    )),
    ..Default::default()
})?;
```

## API Reference

### Stores

```rust,no_run
use openfga_sdk::models::CreateStoreRequest;

let resp = client.list_stores(None, None).await?;
let store = client.create_store(CreateStoreRequest { name: "my-store".to_string() }).await?;
let info = client.get_store(None).await?;
client.delete_store(None).await?;
```

### Authorization Models

```rust,no_run
// Read the latest model
let model = client.read_latest_authorization_model(None).await?;
// List all models (paginated)
let resp = client.read_authorization_models(Some(10), None, None).await?;
```

### Tuples

```rust,no_run
use openfga_sdk::models::{TupleKey, TupleKeyWithoutCondition, ReadRequest};

client.write_tuples(vec![TupleKey::new("user:bob", "editor", "doc:1")], None).await?;
client.delete_tuples(vec![TupleKeyWithoutCondition::new("user:bob", "editor", "doc:1")], None).await?;
let resp = client.read(ReadRequest::default(), None).await?;
let changes = client.read_changes(Some("document".to_string()), None, None, None).await?;
```

### Checks

```rust,no_run
use openfga_sdk::models::{CheckRequest, CheckRequestTupleKey};

// Single check
let resp = client.check(
    CheckRequest::new(CheckRequestTupleKey::new("user:alice", "viewer", "doc:1")),
    None,
).await?;
println!("allowed: {}", resp.is_allowed());

// Client-side parallel batch check (any FGA version)
use openfga_sdk::client::ClientBatchCheckItem;
let resp = client.client_batch_check(vec![
    ClientBatchCheckItem {
        user: "user:alice".to_string(),
        relation: "viewer".to_string(),
        object: "doc:1".to_string(),
        correlation_id: "c1".to_string(),
        contextual_tuples: None,
        context: None,
    },
], None).await?;
```

### Queries

```rust,no_run
use openfga_sdk::models::ListObjectsRequest;

let resp = client.list_objects(ListObjectsRequest {
    user: "user:alice".to_string(),
    relation: "viewer".to_string(),
    object_type: "document".to_string(),
    authorization_model_id: None,
    contextual_tuples: None,
    context: None,
    consistency: None,
}, None).await?;

// Streaming (NDJSON) list objects
let mut stream = client.stream_list_objects(request, None).await?;
while let Some(item) = stream.next().await {
    println!("{}", item?.object);
}
```

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `rustls` | ✅ | Use `rustls` for TLS |
| `native-tls` | ❌ | Use the system TLS library |
| `opentelemetry` | ❌ | Emit OpenTelemetry metrics |

## Environment Variables

The SDK reads no environment variables by default. For convenience, your application can read:

| Variable | Description |
|----------|-------------|
| `FGA_API_URL` | OpenFGA API base URL |
| `FGA_STORE_ID` | Default store ID (ULID) |
| `FGA_MODEL_ID` | Default authorization model ID (ULID) |
| `FGA_API_TOKEN` | Static API token |
| `FGA_CLIENT_ID` | OAuth2 client ID |
| `FGA_CLIENT_SECRET` | OAuth2 client secret |
| `FGA_TOKEN_ISSUER` | OAuth2 token endpoint |
| `FGA_API_AUDIENCE` | OAuth2 audience |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Apache 2.0. See [LICENSE](LICENSE).