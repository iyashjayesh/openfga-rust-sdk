//! Retry utilities - mirrors `internal/utils/retryutils/`.

use std::time::Duration;

use reqwest::header::HeaderMap;

use crate::internal::constants::{DEFAULT_MAX_RETRY, DEFAULT_MIN_WAIT_MS};

/// Name of the `Retry-After` HTTP header.
pub const RETRY_AFTER_HEADER: &str = "Retry-After";

/// Name of the `X-RateLimit-Reset` HTTP header.
pub const RATE_LIMIT_RESET_HEADER: &str = "X-RateLimit-Reset";

/// Configuration for retry behaviour.
#[derive(Debug, Clone)]
pub struct RetryParams {
    /// Maximum number of retries (not counting the initial attempt).
    pub max_retry: u32,
    /// Minimum wait between retries in milliseconds.
    pub min_wait_ms: u64,
}

impl Default for RetryParams {
    fn default() -> Self {
        Self {
            max_retry: DEFAULT_MAX_RETRY,
            min_wait_ms: DEFAULT_MIN_WAIT_MS,
        }
    }
}

impl RetryParams {
    /// Creates a new `RetryParams` with custom values.
    pub fn new(max_retry: u32, min_wait_ms: u64) -> Self {
        Self {
            max_retry,
            min_wait_ms,
        }
    }

    /// Validates retry parameters.
    ///
    /// # Errors
    ///
    /// Returns an error string if `max_retry` exceeds 15.
    pub fn validate(&self) -> Result<(), String> {
        if self.max_retry > 15 {
            return Err(format!(
                "RetryParams.max_retry ({}) must not exceed 15",
                self.max_retry
            ));
        }
        Ok(())
    }
}

/// Calculates how long to wait before the next retry attempt.
///
/// Implements exponential backoff: `min_wait * 2^attempt`, capped so it
/// doesn't exceed a reasonable maximum. Respects `Retry-After` headers.
///
/// Returns `Duration::ZERO` when no more retries should be attempted.
pub fn get_time_to_wait(
    attempt: u32,
    max_retry: u32,
    min_wait_ms: u64,
    headers: &HeaderMap,
    _operation: &str,
) -> Duration {
    if attempt >= max_retry {
        return Duration::ZERO;
    }

    // Check Retry-After header first.
    if let Some(retry_after) = parse_retry_after_header(headers) {
        return retry_after;
    }

    // Exponential backoff: min_wait_ms * 2^attempt
    let wait_ms = min_wait_ms.saturating_mul(1u64 << attempt.min(10));
    Duration::from_millis(wait_ms)
}

/// Parses the `Retry-After` header value.
///
/// Supports both the delay-seconds form (`Retry-After: 30`) and the
/// HTTP-date form (`Retry-After: Wed, 21 Oct 2015 07:28:00 GMT`).
///
/// Returns `None` if the header is absent or unparseable.
pub fn parse_retry_after_header(headers: &HeaderMap) -> Option<Duration> {
    let value = headers.get(RETRY_AFTER_HEADER)?.to_str().ok()?;

    // Try as plain seconds first.
    if let Ok(secs) = value.trim().parse::<u64>() {
        return Some(Duration::from_secs(secs));
    }

    // Try as HTTP-date (RFC 2822 / RFC 7231).
    // chrono parses RFC 2822 dates via `DateTime::parse_from_rfc2822`.
    if let Ok(dt) = chrono::DateTime::parse_from_rfc2822(value.trim()) {
        let retry_at = dt.with_timezone(&chrono::Utc);
        let now = chrono::Utc::now();
        if retry_at > now {
            let diff = (retry_at - now).num_milliseconds().max(0) as u64;
            return Some(Duration::from_millis(diff));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_retry_params() {
        let p = RetryParams::default();
        assert_eq!(p.max_retry, DEFAULT_MAX_RETRY);
        assert_eq!(p.min_wait_ms, DEFAULT_MIN_WAIT_MS);
    }

    #[test]
    fn validate_exceeds_max() {
        let p = RetryParams::new(16, 100);
        assert!(p.validate().is_err());
    }

    #[test]
    fn validate_ok() {
        let p = RetryParams::new(3, 100);
        assert!(p.validate().is_ok());
    }

    #[test]
    fn backoff_grows_exponentially() {
        let headers = HeaderMap::new();
        let d0 = get_time_to_wait(0, 3, 100, &headers, "op");
        let d1 = get_time_to_wait(1, 3, 100, &headers, "op");
        let d2 = get_time_to_wait(2, 3, 100, &headers, "op");
        assert_eq!(d0, Duration::from_millis(100));
        assert_eq!(d1, Duration::from_millis(200));
        assert_eq!(d2, Duration::from_millis(400));
    }

    #[test]
    fn no_retry_when_exhausted() {
        let headers = HeaderMap::new();
        let d = get_time_to_wait(3, 3, 100, &headers, "op");
        assert_eq!(d, Duration::ZERO);
    }

    #[test]
    fn parse_retry_after_seconds() {
        let mut headers = HeaderMap::new();
        headers.insert(RETRY_AFTER_HEADER, "30".parse().unwrap());
        let d = parse_retry_after_header(&headers);
        assert_eq!(d, Some(Duration::from_secs(30)));
    }
}
