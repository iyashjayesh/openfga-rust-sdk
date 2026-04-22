//! Unit tests for internal retry logic and ULID validation.
//! Uses only publicly exported types so no `pub(crate)` changes are needed.

// ────────────────────────────────────────────────────────────────────────────
// ULID tests
// ────────────────────────────────────────────────────────────────────────────

mod ulid_tests {
    use openfga_sdk::{
        client::{ClientConfiguration, OpenFgaClient},
        error::OpenFgaError,
    };

    #[test]
    fn valid_ulids_accepted() {
        for id in [
            "01GXSA8YR785C4FYS3C0RTG7B1",
            "01FQH7V8BEG3GPQW93KTRFR8JB",
        ] {
            let result = OpenFgaClient::new(&ClientConfiguration {
                api_url: "http://localhost:8080".to_string(),
                store_id: Some(id.to_string()),
                ..Default::default()
            });
            assert!(result.is_ok(), "expected {id} to be accepted as a valid ULID");
        }
    }

    #[test]
    fn invalid_store_id_rejected() {
        let err = OpenFgaClient::new(&ClientConfiguration {
            api_url: "http://localhost:8080".to_string(),
            store_id: Some("not-a-ulid".to_string()),
            ..Default::default()
        })
        .unwrap_err();
        assert!(matches!(err, OpenFgaError::InvalidParam { param, .. } if param == "store_id"));
    }

    #[test]
    fn too_short_ulid_rejected() {
        let err = OpenFgaClient::new(&ClientConfiguration {
            api_url: "http://localhost:8080".to_string(),
            store_id: Some("01FQH7V8BEG3GPQW93KTRFR8J".to_string()), // 25 chars
            ..Default::default()
        })
        .unwrap_err();
        assert!(matches!(err, OpenFgaError::InvalidParam { .. }));
    }

    #[test]
    fn too_long_ulid_rejected() {
        let err = OpenFgaClient::new(&ClientConfiguration {
            api_url: "http://localhost:8080".to_string(),
            store_id: Some("01FQH7V8BEG3GPQW93KTRFR8JB1".to_string()), // 27 chars
            ..Default::default()
        })
        .unwrap_err();
        assert!(matches!(err, OpenFgaError::InvalidParam { .. }));
    }

    #[test]
    fn empty_store_id_is_allowed() {
        // Empty string means "no default store_id" - allowed at construction time.
        let result = OpenFgaClient::new(&ClientConfiguration {
            api_url: "http://localhost:8080".to_string(),
            store_id: Some(String::new()),
            ..Default::default()
        });
        assert!(result.is_ok());
    }

    #[test]
    fn invalid_model_id_rejected() {
        let err = OpenFgaClient::new(&ClientConfiguration {
            api_url: "http://localhost:8080".to_string(),
            authorization_model_id: Some("bad-model".to_string()),
            ..Default::default()
        })
        .unwrap_err();
        assert!(
            matches!(err, OpenFgaError::InvalidParam { param, .. } if param == "authorization_model_id")
        );
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Error classification (should_retry / get_time_to_wait)
// ────────────────────────────────────────────────────────────────────────────

mod error_tests {
    use std::time::Duration;

    use openfga_sdk::error::{
        ApiErrorContext, FgaApiAuthenticationError, FgaApiInternalError,
        FgaApiRateLimitExceededError, FgaApiValidationError, OpenFgaError,
    };

    fn make_ctx(status: u16) -> ApiErrorContext {
        ApiErrorContext {
            response_status_code: status,
            ..Default::default()
        }
    }

    #[test]
    fn rate_limit_429_should_retry() {
        let err = OpenFgaError::RateLimitExceeded(FgaApiRateLimitExceededError::new(make_ctx(429)));
        assert!(err.should_retry());
    }

    #[test]
    fn internal_500_should_retry() {
        let err = OpenFgaError::Internal(FgaApiInternalError::new(make_ctx(500)));
        assert!(err.should_retry());
    }

    #[test]
    fn internal_501_should_not_retry() {
        let err = OpenFgaError::Internal(FgaApiInternalError::new(make_ctx(501)));
        assert!(!err.should_retry());
    }

    #[test]
    fn validation_400_should_not_retry() {
        let err = OpenFgaError::Validation(FgaApiValidationError::new(make_ctx(400)));
        assert!(!err.should_retry());
    }

    #[test]
    fn authentication_401_should_not_retry() {
        let err = OpenFgaError::Authentication(FgaApiAuthenticationError::new(make_ctx(401)));
        assert!(!err.should_retry());
    }

    #[test]
    fn http_error_should_retry() {
        let err = OpenFgaError::Http("connection reset".to_string());
        assert!(err.should_retry());
    }

    #[test]
    fn configuration_error_should_not_retry() {
        let err = OpenFgaError::Configuration("bad config".to_string());
        assert!(!err.should_retry());
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Model serde round-trips
// ────────────────────────────────────────────────────────────────────────────

mod serde_tests {
    use openfga_sdk::models::{
        CheckRequest, CheckRequestTupleKey, ConsistencyPreference, TupleKey,
        TupleKeyWithoutCondition,
    };

    #[test]
    fn tuple_key_round_trip() {
        let key = TupleKey::new("user:alice", "viewer", "document:roadmap");
        let json = serde_json::to_string(&key).unwrap();
        let back: TupleKey = serde_json::from_str(&json).unwrap();
        assert_eq!(back.user, "user:alice");
        assert_eq!(back.relation, "viewer");
        assert_eq!(back.object, "document:roadmap");
    }

    #[test]
    fn tuple_key_without_condition_round_trip() {
        let key = TupleKeyWithoutCondition::new("user:bob", "editor", "doc:1");
        let json = serde_json::to_string(&key).unwrap();
        let back: TupleKeyWithoutCondition = serde_json::from_str(&json).unwrap();
        assert_eq!(back.user, "user:bob");
    }

    #[test]
    fn check_request_tuple_key_new() {
        let tk = CheckRequestTupleKey::new("user:alice", "owner", "document:budget");
        assert_eq!(tk.user, "user:alice");
        assert_eq!(tk.relation, "owner");
        assert_eq!(tk.object, "document:budget");
    }

    #[test]
    fn check_request_omits_none_fields() {
        let req = CheckRequest::new(CheckRequestTupleKey::new("user:bob", "editor", "doc:1"));
        let json: serde_json::Value = serde_json::to_value(&req).unwrap();
        // optional fields should be absent when None
        assert!(json.get("authorization_model_id").and_then(|v| v.as_str()).is_none());
        assert!(json.get("trace").and_then(|v| v.as_bool()).is_none());
    }

    #[test]
    fn consistency_preference_serializes_as_screaming_snake_case() {
        let pref = ConsistencyPreference::HigherConsistency;
        let json = serde_json::to_value(&pref).unwrap();
        assert_eq!(json.as_str().unwrap(), "HIGHER_CONSISTENCY");
    }
}
