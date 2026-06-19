// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Request parsing, dispatch outcomes, health checks, and dispatch error mapping.

#![expect(clippy::unwrap_used, reason = "test assertions")]

use crate::neural_api_server::rpc::DispatchOutcome;

use super::common::create_test_server;

#[tokio::test]
async fn test_handle_request_unknown_single_word_method() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"nonexistent","id":1}"#;
    let result = server.handle_request_json(req).await;
    assert_eq!(result["jsonrpc"], "2.0");
    assert_eq!(result["error"]["code"], -32601);
    assert!(
        result["error"]["message"]
            .as_str()
            .unwrap()
            .contains("nonexistent")
    );
    assert_eq!(result["id"], 1);
}
#[tokio::test]
async fn test_handle_request_invalid_json() {
    let (server, _temp) = create_test_server();
    let result = server.handle_request_json("{broken").await;
    assert_eq!(result["error"]["code"], -32700);
    // serde_json error message varies (e.g. "expected value", "EOF while parsing")
    assert!(!result["error"]["message"].as_str().unwrap_or("").is_empty());
}

#[tokio::test]
async fn test_handle_request_mesh_method_invalid_format_single_part() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"mesh","id":2}"#;
    let result = server.handle_request_json(req).await;
    assert_eq!(result["error"]["code"], -32601);
    assert!(
        result["error"]["message"]
            .as_str()
            .unwrap()
            .contains("mesh")
    );
}

#[tokio::test]
async fn test_semantic_fallback_multipart_method() {
    let (server, _temp) = create_test_server();
    // "a.b.c" has a dot so semantic fallback splits on first dot: domain="a", operation="b.c"
    let req = r#"{"jsonrpc":"2.0","method":"a.b.c","id":3}"#;
    let result = server.handle_request_json(req).await;
    // Routes through capability.call (no provider), not MethodNotFound
    assert!(result.get("error").is_some());
    assert_ne!(result["error"]["code"], -32601);
}

#[tokio::test]
async fn test_handle_request_empty_method() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"","id":4}"#;
    let result = server.handle_request_json(req).await;
    assert_eq!(result["error"]["code"], -32601);
}

#[tokio::test]
async fn test_handle_request_method_not_found_response_structure() {
    let (server, _temp) = create_test_server();
    // Single-word methods with no dot hit MethodNotFound (no semantic fallback)
    let req = r#"{"jsonrpc":"2.0","method":"nonexistent_verb","id":99}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_none());
    assert!(result.get("error").is_some());
    assert_eq!(result["error"]["code"], -32601);
    assert_eq!(result["id"], 99);
}
#[tokio::test]
async fn test_handle_request_missing_id() {
    // JSON-RPC 2.0 allows omitting id (notification); we accept and echo null
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"graph.list"}"#;
    let result = server.handle_request_json(req).await;
    assert_eq!(result["id"], serde_json::Value::Null);
}
#[tokio::test]
async fn test_handle_request_dispatch_outcome_success_structure() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"graph.list","id":76}"#;
    let outcome = server.handle_request(req).await;
    let response = outcome.into_response();
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response.get("result").is_some());
    assert!(response.get("error").is_none());
}

#[tokio::test]
async fn test_handle_request_dispatch_outcome_parse_error() {
    let (server, _temp) = create_test_server();
    let outcome = server.handle_request("not json").await;
    let response = outcome.into_response();
    assert_eq!(response["error"]["code"], -32700);
    assert!(response["id"].is_null());
}

#[tokio::test]
async fn test_handle_request_dispatch_outcome_method_not_found() {
    let (server, _temp) = create_test_server();
    // No-dot method → pure MethodNotFound (no semantic fallback)
    let req = r#"{"jsonrpc":"2.0","method":"nonexistent","id":77}"#;
    let outcome = server.handle_request(req).await;
    let response = outcome.into_response();
    assert_eq!(response["error"]["code"], -32601);
    assert!(
        response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("nonexistent")
    );
}

#[tokio::test]
async fn test_handle_request_health_check() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"health.check","id":80}"#;
    let result = server.handle_request_json(req).await;
    assert_eq!(result["result"]["status"], "alive");
    assert_eq!(result["result"]["family_id"], "test_family");
}

#[tokio::test]
async fn test_handle_request_health_liveness() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"health.liveness","id":81}"#;
    let result = server.handle_request_json(req).await;
    assert_eq!(result["result"]["status"], "alive");
    assert!(result["result"]["version"].is_string());
}

#[tokio::test]
async fn test_handle_request_health_readiness() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"health.readiness","id":82}"#;
    let result = server.handle_request_json(req).await;
    assert!(result["result"]["ready"].is_boolean());
    assert!(result["result"]["mode"].is_string());
}
#[test]
fn dispatch_preserves_primal_json_rpc_error_code() {
    let err = biomeos_types::IpcError::JsonRpcError {
        primal: "beardog".to_string(),
        code: -32601,
        message: "Method not found".to_string(),
    };
    let id = serde_json::json!(42);
    let outcome = super::super::dispatch(Err(err.into()), id);
    match outcome {
        DispatchOutcome::ApplicationError { code, message, .. } => {
            assert_eq!(code, -32601, "primal error code must be preserved");
            assert_eq!(message, "Method not found");
        }
        other => panic!("expected ApplicationError, got: {other:?}"),
    }
}

#[test]
fn dispatch_uses_generic_code_for_non_ipc_errors() {
    let err = anyhow::anyhow!("connection refused");
    let id = serde_json::json!(1);
    let outcome = super::super::dispatch(Err(err), id);
    match outcome {
        DispatchOutcome::ApplicationError { code, .. } => {
            assert_eq!(code, -32603, "non-IPC errors use generic code");
        }
        other => panic!("expected ApplicationError, got: {other:?}"),
    }
}
