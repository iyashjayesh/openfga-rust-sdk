//! ULID validation — mirrors `internal/utils/ulid.go`.

use ulid::Ulid;

/// Returns `true` if `s` is a well-formed [ULID](https://github.com/ulid/spec) string.
///
/// OpenFGA uses ULIDs for store IDs and authorization model IDs.
///
/// # Examples
///
/// ```rust,ignore
/// // internal module — see unit tests below for validated examples.
/// assert!(is_well_formed_ulid("01FQH7V8BEG3GPQW93KTRFR8JB"));
/// assert!(!is_well_formed_ulid("not-a-ulid"));
/// assert!(!is_well_formed_ulid(""));
/// ```
pub fn is_well_formed_ulid(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    s.parse::<Ulid>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_ulid() {
        assert!(is_well_formed_ulid("01FQH7V8BEG3GPQW93KTRFR8JB"));
        assert!(is_well_formed_ulid("01GXSA8YR785C4FYS3C0RTG7B1"));
    }

    #[test]
    fn invalid_ulid() {
        assert!(!is_well_formed_ulid(""));
        assert!(!is_well_formed_ulid("not-a-ulid"));
        assert!(!is_well_formed_ulid("01FQH7V8BEG3GPQW93KTRFR8J")); // too short
        assert!(!is_well_formed_ulid("01FQH7V8BEG3GPQW93KTRFR8JB1")); // too long
    }
}
