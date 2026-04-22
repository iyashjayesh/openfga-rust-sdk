# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of the OpenFGA Rust SDK
- `OpenFgaClient` - high-level async client covering all 20 OpenFGA API endpoints
- Three authentication modes: None, API Token, OAuth2 Client Credentials (with thread-safe token caching)
- Automatic retry with exponential backoff and `Retry-After` / `X-RateLimit-Reset` header support
- Non-transaction write chunking (configurable chunk size, parallel execution)
- Client-side parallel `batch_check` (any FGA version)
- Server-side `BatchCheck` support (FGA ≥ 1.8.0)
- Streaming `ListObjects` via NDJSON async iteration
- Full data model coverage: tuples, authorization models, stores, checks, queries
- ULID validation for store IDs and authorization model IDs
- OpenTelemetry metric skeleton (attribute/metric name constants, `NoopTelemetry`)
- TLS support via `rustls` (default) and `native-tls` feature flags
- GitHub Actions CI: fmt, clippy, cross-platform test matrix, MSRV (1.75), cargo audit, cargo deny
