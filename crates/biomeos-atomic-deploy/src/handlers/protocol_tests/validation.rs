// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::{create_test_handler, make_handler};
use serde_json::json;

#[tokio::test]
async fn register_primal_missing_primal_id_is_error() {
    let h = make_handler();
    let params = Some(json!({
        "json_rpc_socket": "/tmp/a.sock"
    }));
    let err = h.register_primal(&params).await.unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("primal_id"),
        "{err}"
    );
}

#[tokio::test]
async fn register_primal_missing_json_rpc_socket_is_error() {
    let h = make_handler();
    let params = Some(json!({
        "primal_id": "only-id"
    }));
    let err = h.register_primal(&params).await.unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("json_rpc_socket"),
        "{err}"
    );
}

#[tokio::test]
async fn record_request_missing_latency_us_is_error() {
    let h = make_handler();
    h.register_connection(&Some(json!({ "from": "a", "to": "b" })))
        .await
        .unwrap();
    let params = Some(json!({
        "from": "a",
        "to": "b"
    }));
    let err = h.record_request(&params).await.unwrap_err();
    assert!(err.to_string().to_lowercase().contains("latency"), "{err}");
}

#[tokio::test]
async fn record_request_missing_from_is_error() {
    let h = make_handler();
    let params = Some(json!({
        "to": "b",
        "latency_us": 1
    }));
    assert!(h.record_request(&params).await.is_err());
}

#[tokio::test]
async fn escalate_missing_to_parameter() {
    let h = make_handler();
    let err = h
        .escalate(&Some(json!({ "from": "only-from" })))
        .await
        .unwrap_err();
    assert!(err.to_string().to_lowercase().contains("to"));
}

#[tokio::test]
async fn record_request_missing_to_is_error() {
    let h = make_handler();
    let err = h
        .record_request(&Some(json!({
            "from": "a",
            "latency_us": 1
        })))
        .await
        .unwrap_err();
    assert!(err.to_string().to_lowercase().contains("to"));
}

#[tokio::test]
async fn test_missing_params() {
    let handler = create_test_handler();

    let params = Some(json!({ "to": "b" }));
    let result = handler.escalate(&params).await;
    assert!(result.is_err());

    let params = Some(json!({ "from": "a" }));
    let result = handler.escalate(&params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_escalate_none_params() {
    let handler = create_test_handler();
    let result = handler.escalate(&None).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .to_lowercase()
            .contains("missing"),
        "Error should mention missing params"
    );
}

#[tokio::test]
async fn test_fallback_missing_params() {
    let handler = create_test_handler();

    let params = Some(json!({ "to": "b" }));
    let result = handler.fallback(&params).await;
    assert!(result.is_err());

    let params = Some(json!({ "from": "a" }));
    let result = handler.fallback(&params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_fallback_none_params() {
    let handler = create_test_handler();
    let result = handler.fallback(&None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_metrics_missing_params() {
    let handler = create_test_handler();

    let params = Some(json!({ "to": "b" }));
    let result = handler.metrics(&params).await;
    assert!(result.is_err());

    let params = Some(json!({ "from": "a" }));
    let result = handler.metrics(&params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_metrics_none_params() {
    let handler = create_test_handler();
    let result = handler.metrics(&None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_metrics_connection_not_found() {
    let handler = create_test_handler();

    let params = Some(json!({
        "from": "nonexistent",
        "to": "also-nonexistent"
    }));
    let result = handler.metrics(&params).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .to_lowercase()
            .contains("not found"),
        "Error should mention connection not found"
    );
}

#[tokio::test]
async fn test_register_primal_missing_params() {
    let handler = create_test_handler();

    let result = handler.register_primal(&None).await;
    assert!(result.is_err());

    let params = Some(json!({}));
    let result = handler.register_primal(&params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_register_connection_missing_params() {
    let handler = create_test_handler();

    let result = handler.register_connection(&None).await;
    assert!(result.is_err());

    let params = Some(json!({ "from": "a" }));
    let result = handler.register_connection(&params).await;
    assert!(result.is_err());

    let params = Some(json!({ "to": "b" }));
    let result = handler.register_connection(&params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_record_request_missing_params() {
    let handler = create_test_handler();

    let result = handler.record_request(&None).await;
    assert!(result.is_err());

    let params = Some(json!({ "from": "a", "to": "b" }));
    let result = handler.record_request(&params).await;
    assert!(result.is_err());
}
