// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use biomeos_test_utils::MockJsonRpcServer;
use serde_json::json;

use super::super::common::create_test_server;

#[tokio::test]
async fn test_semantic_fallback_routes_through_capability_call() {
    let (server, _temp) = create_test_server();
    // "provenance.begin" is not in ROUTE_TABLE but has domain.operation format,
    // so it routes through capability.call via the semantic fallback.
    let req = r#"{"jsonrpc":"2.0","method":"provenance.begin","params":{},"id":1}"#;
    let result = server.handle_request_json(req).await;
    // No provider registered in test server → ApplicationError (-32603), not MethodNotFound
    assert!(result.get("error").is_some());
    assert_ne!(
        result["error"]["code"], -32601,
        "semantic fallback should route through capability.call, not MethodNotFound"
    );
}

#[tokio::test]
async fn test_semantic_fallback_birdsong_decrypt() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"birdsong.decrypt","params":{"data":"test"},"id":2}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("error").is_some());
    assert_ne!(result["error"]["code"], -32601);
}

#[tokio::test]
async fn test_semantic_fallback_dag_dehydrate() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"dag.dehydrate","params":{"session_id":"s1"},"id":3}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("error").is_some());
    assert_ne!(result["error"]["code"], -32601);
}

#[tokio::test]
async fn test_semantic_fallback_content_get_routes_to_nestgate() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"content.get","params":{"key":"test"},"id":106}"#;
    let result = server.handle_request_json(req).await;
    // No NestGate registered → ApplicationError, not MethodNotFound.
    // This confirms content.get routes through semantic fallback → capability.call.
    assert!(result.get("error").is_some());
    assert_ne!(
        result["error"]["code"], -32601,
        "content.get should route through capability.call, not MethodNotFound"
    );
}

#[tokio::test]
async fn test_semantic_fallback_content_resolve_routes_to_nestgate() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"content.resolve","params":{"path":"/test"},"id":107}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("error").is_some());
    assert_ne!(
        result["error"]["code"], -32601,
        "content.resolve should route through capability.call"
    );
}

#[tokio::test]
async fn test_content_routes_to_nestgate_with_registered_provider() {
    let (server, _temp) = create_test_server();
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("nestgate.sock");
    let _mock =
        MockJsonRpcServer::spawn_echo_success(&sock, json!({"found": true, "data": "hello"})).await;

    server
        .capability_handler
        .register(&Some(json!({
            "capability": "content",
            "primal": "nestgate",
            "socket": sock.to_str().unwrap(),
            "source": "content-routing-test"
        })))
        .await
        .expect("register content capability");

    let req = json!({
        "jsonrpc": "2.0",
        "method": "content.get",
        "params": { "key": "test-content" },
        "id": 108
    })
    .to_string();

    let result = server.handle_request_json(&req).await;
    assert!(
        result.get("result").is_some(),
        "content.get should route to registered NestGate: {result}"
    );
    assert_eq!(result["result"]["found"], true);
}
