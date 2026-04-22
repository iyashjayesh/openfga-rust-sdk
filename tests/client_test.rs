//! Unit tests for the OpenFGA Rust SDK.
//!
//! These tests use `wiremock` to stub the HTTP layer so no real server is needed.

use std::collections::HashMap;

use openfga_sdk::{
    client::{ClientConfiguration, OpenFgaClient},
    error::OpenFgaError,
    models::{
        BatchCheckItem, BatchCheckRequest, CheckRequest, CheckRequestTupleKey, CreateStoreRequest,
        ListObjectsRequest, ReadRequest, TupleKey, WriteRequest, WriteRequestWrites,
    },
};
use serde_json::{json, Value};
use wiremock::{
    matchers::{body_json, header, method, path, path_regex},
    Mock, MockServer, ResponseTemplate,
};

// ────────────────────────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────────────────────────

/// Build a client pointing at the wiremock server.
async fn client_for(server: &MockServer) -> OpenFgaClient {
    OpenFgaClient::new(&ClientConfiguration {
        api_url: server.uri(),
        store_id: Some("01GXSA8YR785C4FYS3C0RTG7B1".to_string()),
        authorization_model_id: Some("01GXSA8YR785C4FYS3C0RTG7B2".to_string()),
        ..Default::default()
    })
    .expect("client should build")
}

fn store_response() -> Value {
    json!({
        "id": "01GXSA8YR785C4FYS3C0RTG7B1",
        "name": "test-store",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z"
    })
}

// ────────────────────────────────────────────────────────────────────────────
// Stores API
// ────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_store() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/stores"))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(json!({
                    "id": "01GXSA8YR785C4FYS3C0RTG7B1",
                    "name": "my-store",
                    "created_at": "2024-01-01T00:00:00Z",
                    "updated_at": "2024-01-01T00:00:00Z"
                })),
        )
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let resp = client
        .create_store(CreateStoreRequest { name: "my-store".to_string() })
        .await
        .expect("create_store should succeed");

    assert_eq!(resp.id, "01GXSA8YR785C4FYS3C0RTG7B1");
    assert_eq!(resp.name, "my-store");
}

#[tokio::test]
async fn test_list_stores() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/stores"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "stores": [store_response()],
                "continuation_token": ""
            })),
        )
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let resp = client.list_stores(None, None).await.expect("list_stores should succeed");
    assert_eq!(resp.stores.len(), 1);
    assert_eq!(resp.stores[0].id, "01GXSA8YR785C4FYS3C0RTG7B1");
}

#[tokio::test]
async fn test_get_store() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/stores/01GXSA8YR785C4FYS3C0RTG7B1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(store_response()))
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let resp = client.get_store(None).await.expect("get_store should succeed");
    assert_eq!(resp.name, "test-store");
}

#[tokio::test]
async fn test_delete_store() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/stores/01GXSA8YR785C4FYS3C0RTG7B1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    client.delete_store(None).await.expect("delete_store should succeed");
}

// ────────────────────────────────────────────────────────────────────────────
// Authorization Models API
// ────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_read_authorization_models() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/stores/01GXSA8YR785C4FYS3C0RTG7B1/authorization-models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "authorization_models": [{
                "id": "01GXSA8YR785C4FYS3C0RTG7B2",
                "schema_version": "1.1",
                "type_definitions": []
            }],
            "continuation_token": ""
        })))
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let resp = client
        .read_authorization_models(None, None, None)
        .await
        .expect("read_authorization_models should succeed");
    assert_eq!(resp.authorization_models.len(), 1);
}

#[tokio::test]
async fn test_read_latest_authorization_model() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/stores/01GXSA8YR785C4FYS3C0RTG7B1/authorization-models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "authorization_models": [{
                "id": "01GXSA8YR785C4FYS3C0RTG7B2",
                "schema_version": "1.1",
                "type_definitions": []
            }],
            "continuation_token": ""
        })))
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let model = client
        .read_latest_authorization_model(None)
        .await
        .expect("should succeed");
    assert!(model.is_some());
    assert_eq!(model.unwrap().id, "01GXSA8YR785C4FYS3C0RTG7B2");
}

// ────────────────────────────────────────────────────────────────────────────
// Relationship Tuples
// ────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_write_tuples() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/stores/01GXSA8YR785C4FYS3C0RTG7B1/write"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    client
        .write_tuples(
            vec![TupleKey::new("user:alice", "viewer", "document:roadmap")],
            None,
        )
        .await
        .expect("write_tuples should succeed");
}

#[tokio::test]
async fn test_read_returns_tuples() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/stores/01GXSA8YR785C4FYS3C0RTG7B1/read"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "tuples": [{
                "key": {
                    "user": "user:alice",
                    "relation": "viewer",
                    "object": "document:roadmap"
                },
                "timestamp": "2024-01-01T00:00:00Z"
            }],
            "continuation_token": ""
        })))
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let resp = client
        .read(ReadRequest::default(), None)
        .await
        .expect("read should succeed");
    assert_eq!(resp.tuples.len(), 1);
    assert_eq!(resp.tuples[0].key.user, "user:alice");
}

// ────────────────────────────────────────────────────────────────────────────
// Check API
// ────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_check_allowed() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/stores/01GXSA8YR785C4FYS3C0RTG7B1/check"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "allowed": true, "resolution": "" })),
        )
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let resp = client
        .check(
            CheckRequest::new(CheckRequestTupleKey::new(
                "user:alice",
                "viewer",
                "document:roadmap",
            )),
            None,
        )
        .await
        .expect("check should succeed");

    assert!(resp.is_allowed());
}

#[tokio::test]
async fn test_check_denied() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/stores/01GXSA8YR785C4FYS3C0RTG7B1/check"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "allowed": false, "resolution": "" })),
        )
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let resp = client
        .check(
            CheckRequest::new(CheckRequestTupleKey::new("user:bob", "editor", "document:secret")),
            None,
        )
        .await
        .expect("check should succeed");

    assert!(!resp.is_allowed());
}

#[tokio::test]
async fn test_check_400_returns_validation_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/stores/01GXSA8YR785C4FYS3C0RTG7B1/check"))
        .respond_with(
            ResponseTemplate::new(400).set_body_json(json!({
                "code": "validation_error",
                "message": "tuple_key.user is required"
            })),
        )
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let err = client
        .check(
            CheckRequest::new(CheckRequestTupleKey::new("", "viewer", "document:roadmap")),
            None,
        )
        .await
        .expect_err("should return error on 400");

    assert!(matches!(err, OpenFgaError::Validation(_)));
}

#[tokio::test]
async fn test_check_401_returns_authentication_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/stores/01GXSA8YR785C4FYS3C0RTG7B1/check"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "code": "unauthenticated",
            "message": "Unauthorized"
        })))
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let err = client
        .check(
            CheckRequest::new(CheckRequestTupleKey::new("user:alice", "viewer", "doc:1")),
            None,
        )
        .await
        .expect_err("should return auth error");

    assert!(matches!(err, OpenFgaError::Authentication(_)));
}

#[tokio::test]
async fn test_check_404_returns_not_found_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/stores/01GXSA8YR785C4FYS3C0RTG7B1/check"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "code": "store_id_not_found",
            "message": "Store not found"
        })))
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let err = client
        .check(
            CheckRequest::new(CheckRequestTupleKey::new("user:alice", "viewer", "doc:1")),
            None,
        )
        .await
        .expect_err("should return not found error");

    assert!(matches!(err, OpenFgaError::NotFound(_)));
}

#[tokio::test]
async fn test_check_500_returns_internal_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/stores/01GXSA8YR785C4FYS3C0RTG7B1/check"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "code": "internal_error",
            "message": "Internal server error"
        })))
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let err = client
        .check(
            CheckRequest::new(CheckRequestTupleKey::new("user:alice", "viewer", "doc:1")),
            None,
        )
        .await
        .expect_err("should return internal error");

    assert!(matches!(err, OpenFgaError::Internal(_)));
}

// ────────────────────────────────────────────────────────────────────────────
// ListObjects
// ────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_objects() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/stores/01GXSA8YR785C4FYS3C0RTG7B1/list-objects"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "objects": ["document:roadmap", "document:budget"]
        })))
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let resp = client
        .list_objects(
            ListObjectsRequest {
                user: "user:alice".to_string(),
                relation: "viewer".to_string(),
                object_type: "document".to_string(),
                authorization_model_id: None,
                contextual_tuples: None,
                context: None,
                consistency: None,
            },
            None,
        )
        .await
        .expect("list_objects should succeed");

    assert_eq!(resp.objects, vec!["document:roadmap", "document:budget"]);
}

// ────────────────────────────────────────────────────────────────────────────
// ClientConfiguration validation
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn test_invalid_store_id_rejected() {
    let err = OpenFgaClient::new(&ClientConfiguration {
        api_url: "http://localhost:8080".to_string(),
        store_id: Some("not-a-ulid".to_string()),
        ..Default::default()
    })
    .unwrap_err();

    assert!(matches!(err, OpenFgaError::InvalidParam { param, .. } if param == "store_id"));
}

#[test]
fn test_invalid_model_id_rejected() {
    let err = OpenFgaClient::new(&ClientConfiguration {
        api_url: "http://localhost:8080".to_string(),
        authorization_model_id: Some("bad-model-id".to_string()),
        ..Default::default()
    })
    .unwrap_err();

    assert!(
        matches!(err, OpenFgaError::InvalidParam { param, .. } if param == "authorization_model_id")
    );
}

#[test]
fn test_empty_store_id_passes_validation() {
    // Empty string is allowed (means "no default store_id").
    let client = OpenFgaClient::new(&ClientConfiguration {
        api_url: "http://localhost:8080".to_string(),
        store_id: Some(String::new()),
        ..Default::default()
    });
    assert!(client.is_ok());
}

#[test]
fn test_valid_ulid_passes_validation() {
    let client = OpenFgaClient::new(&ClientConfiguration {
        api_url: "http://localhost:8080".to_string(),
        store_id: Some("01GXSA8YR785C4FYS3C0RTG7B1".to_string()),
        ..Default::default()
    });
    assert!(client.is_ok());
}
