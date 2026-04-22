//! Internal constants — mirrors `internal/constants/constants.go`.
//! These values must stay consistent across all OpenFGA SDKs.

/// Current SDK version.
pub const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default User-Agent header sent with every request.
pub const USER_AGENT: &str = concat!("openfga-sdk rust/", env!("CARGO_PKG_VERSION"));

/// Alias for `USER_AGENT` — used by modules that import `DEFAULT_USER_AGENT`.
pub const DEFAULT_USER_AGENT: &str = USER_AGENT;

/// Default maximum number of retries on 429 / 5xx responses.
pub const DEFAULT_MAX_RETRY: u32 = 3;

/// Default minimum wait between retries (milliseconds).
pub const DEFAULT_MIN_WAIT_MS: u64 = 100;

/// Maximum number of parallel requests issued by client-level batch methods.
pub const CLIENT_MAX_METHOD_PARALLEL_REQUESTS: usize = 10;

/// Default number of items per chunk in non-transaction write mode.
pub const CLIENT_BATCH_CHECK_DEFAULT_SIZE: usize = 50;

/// Default batch size for server-side BatchCheck splits.
pub const CLIENT_BATCH_CHECK_MAX_SIZE: usize = 50;

/// Token expiry jitter (seconds subtracted from `expires_in`).
/// Prevents thundering-herd refresh against token issuers.
pub const TOKEN_EXPIRY_JITTER_SECS: u64 = 300;
