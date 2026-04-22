# Contributing to the OpenFGA Rust SDK

Thank you for contributing! This document covers the development setup,
conventions, and workflow to keep the SDK high-quality and consistent with
other OpenFGA SDKs.

## Development Setup

```bash
# Clone
git clone https://github.com/openfga/rust-sdk.git
cd rust-sdk

# Check everything builds
cargo check --all-features

# Run tests
cargo test --all-features

# Format
cargo fmt --all

# Lint
cargo clippy --all-features -- -D warnings

# All checks (mirrors CI)
make check
```

**Minimum Supported Rust Version (MSRV):** 1.75.0

## Project Structure

```
src/
├── lib.rs                  # Crate root, feature flags, module declarations
├── error.rs                # OpenFgaError hierarchy + ApiErrorContext
├── client/
│   └── mod.rs              # OpenFgaClient (high-level, user-facing)
├── api/
│   ├── configuration.rs    # Configuration struct + validation
│   ├── api_client.rs       # Low-level reqwest wrapper
│   └── executor.rs         # ApiExecutorImpl - retry + decode
├── credentials/
│   └── mod.rs              # Credentials: None / ApiToken / ClientCredentials
├── oauth2/
│   └── mod.rs              # ClientCredentialsProvider (token cache)
├── models/
│   ├── mod.rs              # Re-exports all model types
│   ├── tuple.rs            # TupleKey, TupleChange, etc.
│   ├── authorization_model.rs
│   ├── check.rs / batch_check.rs / read.rs / write.rs / expand.rs
│   ├── list_objects.rs / list_users.rs
│   ├── store.rs / consistency.rs / contextual_tuples.rs
│   └── error_codes.rs / misc.rs
├── streaming/
│   └── mod.rs              # StreamedListObjects NDJSON reader
├── telemetry/
│   └── mod.rs              # TelemetryConfiguration + OTel stubs
└── internal/
    ├── constants.rs         # SDK version, user-agent, retry defaults
    ├── retry.rs             # RetryParams, backoff calculation
    └── ulid.rs              # ULID validation
tests/
├── unit/                   # Pure unit tests (no server required)
└── integration/            # Tests that spin up a real FGA server
examples/
├── no_credentials.rs
├── api_token.rs
└── client_credentials.rs
```

## Consistency Rules

This SDK must maintain **behavioural consistency** with the Go SDK:

1. **Retry behaviour** - same defaults: `max_retry = 3`, `min_wait_ms = 100`.
   Respect `Retry-After` and `X-RateLimit-Reset` headers.
2. **Error variants** - every API error carries `store_id`, `endpoint`, HTTP
   status, request-id, and raw body bytes.
3. **ULID validation** - store IDs and model IDs are validated before being
   sent to the API.
4. **Non-transaction writes** - chunked at 100 tuples per chunk by default.
5. **User-Agent** - `openfga-sdk rust/<version>`.

## Adding a New API Endpoint

1. Add request/response types to the appropriate file in `src/models/`.
2. Add a method to `OpenFgaClient` in `src/client/mod.rs` following the existing pattern.
3. Add a unit test (mock HTTP) and update the example if appropriate.
4. Update `CHANGELOG.md`.

## Telemetry

Every API call should eventually emit the following OTel metrics
(currently behind the `opentelemetry` feature flag):

| Metric | Type | Description |
|--------|------|-------------|
| `fga_client.request.duration` | Histogram | Total SDK call duration |
| `fga_client.query.duration` | Histogram | Server-side query duration (from response header) |
| `fga_client.request.count` | Counter | Total requests |
| `http.client.request.duration` | Histogram | HTTP round-trip duration |

Attributes follow the OTel semantic conventions:
`http.request.method`, `http.response.status_code`,
`fga_client.request.store_id`, `fga_client.request.model_id`, etc.

High-cardinality attributes (`url.full`, `fga_client.user`) are **disabled by
default** and must be explicitly enabled in `MetricsConfiguration`.

## Pull Request Checklist

- [ ] `cargo fmt --all` - no formatting changes
- [ ] `cargo clippy --all-features -- -D warnings` - no new warnings
- [ ] `cargo test --all-features` - all tests pass
- [ ] New public types/methods have doc comments
- [ ] `CHANGELOG.md` updated under `[Unreleased]`
- [ ] Consistent with Go SDK behaviour (retry, errors, user-agent)

## Releasing

Releases are managed by the maintainers. Version bumps follow [SemVer](https://semver.org).

```bash
# Bump version in Cargo.toml, then:
git tag -s v0.x.y -m "Release v0.x.y"
git push origin v0.x.y
# CI will publish to crates.io automatically
```
