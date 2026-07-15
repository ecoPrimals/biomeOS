// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use biomeos_test_utils::MockJsonRpcServer;
use serde_json::json;

use super::super::common::create_test_server;

#[tokio::test]
async fn test_handle_request_capability_resolve_route() {
    let (server, _temp) = create_test_server();
    server
        .capability_handler
        .register(&Some(serde_json::json!({
            "capability": "crypto",
            "primal": "beardog",
            "socket": "/tmp/beardog.sock",
            "source": "test"
        })))
        .await
        .unwrap();

    let req = r#"{"jsonrpc":"2.0","method":"capability.resolve","params":{"capability":"crypto"},"id":90}"#;
    let result = server.handle_request_json(req).await;
    assert!(
        result.get("result").is_some(),
        "resolve should succeed: {result}"
    );
    assert_eq!(result["result"]["resolved"], true);
    assert_eq!(result["result"]["primal"], "beardog");
    assert_eq!(result["id"], 90);
}

#[tokio::test]
async fn test_handle_request_capability_resolve_missing_capability_errors() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"capability.resolve","params":{},"id":91}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("error").is_some());
}

#[tokio::test]
async fn test_handle_request_inference_register_provider_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"inference.register_provider","params":{"name":"neuralSpring","endpoint":"/tmp/neural.sock"},"id":92}"#;
    let result = server.handle_request_json(req).await;
    assert!(
        result.get("result").is_some(),
        "register_provider should succeed: {result}"
    );
    assert_eq!(result["result"]["registered"], true);
    assert_eq!(result["result"]["name"], "neuralSpring");
}

#[tokio::test]
async fn test_handle_request_inference_providers_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"inference.providers","id":93}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some());
    assert_eq!(result["result"]["count"], 0);
}

#[tokio::test]
async fn test_handle_request_inference_complete_no_provider() {
    let (server, _temp) = create_test_server();
    let req =
        r#"{"jsonrpc":"2.0","method":"inference.complete","params":{"prompt":"hello"},"id":94}"#;
    let result = server.handle_request_json(req).await;
    assert!(
        result.get("error").is_some(),
        "inference.complete with no provider should error"
    );
}

#[tokio::test]
async fn test_handle_request_inference_embed_no_provider() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"inference.embed","params":{"text":"test"},"id":95}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("error").is_some());
}

#[tokio::test]
async fn test_handle_request_inference_models_no_provider() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"inference.models","id":96}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("error").is_some());
}

#[tokio::test]
async fn test_handle_request_inference_register_provider_missing_name_errors() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"inference.register_provider","params":{"endpoint":"/tmp/x.sock"},"id":97}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("error").is_some());
}

#[tokio::test]
async fn test_handle_request_capability_call_includes_routing_trace_when_enabled() {
    let (server, _temp) = create_test_server();
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("routing-trace.sock");
    let _mock = MockJsonRpcServer::spawn_echo_success(&sock, json!({ "hashed": "z" })).await;

    server
        .capability_handler
        .register(&Some(json!({
            "capability": "crypto",
            "primal": "beardog",
            "socket": sock.to_str().unwrap(),
            "source": "routing_test",
            "semantic_mappings": { "sha256": "crypto.blake3_hash" }
        })))
        .await
        .expect("register");

    let req = json!({
        "jsonrpc": "2.0",
        "method": "capability.call",
        "params": {
            "capability": "crypto",
            "operation": "sha256",
            "args": {},
            "_routing_trace": true
        },
        "id": 42
    })
    .to_string();

    let result = server.handle_request_json(&req).await;
    assert!(
        result.get("_routing_trace").is_some(),
        "expected top-level _routing_trace: {result}"
    );
    assert_eq!(result["result"]["hashed"], "z");
    assert_eq!(result["id"], 42);
    let phases = result["_routing_trace"]["phases"]
        .as_array()
        .expect("phases");
    assert_eq!(phases.len(), 3);
}
