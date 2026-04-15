// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Capability handler tests - extracted to keep capability.rs under 1000 lines

#![expect(clippy::unwrap_used, reason = "test")]
#![expect(clippy::expect_used, reason = "test assertions")]

use biomeos_test_utils::MockJsonRpcServer;
use serde_json::json;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::sync::RwLock;

use crate::capability_translation::CapabilityTranslationRegistry;
use crate::neural_router::NeuralRouter;

use super::capability::CapabilityHandler;

pub(crate) fn make_handler() -> CapabilityHandler {
    let router = Arc::new(NeuralRouter::new("test-family"));
    let registry = Arc::new(RwLock::new(CapabilityTranslationRegistry::new()));
    CapabilityHandler::new(router, registry)
}

pub(crate) async fn handler_with_registration() -> CapabilityHandler {
    let handler = make_handler();
    let params = Some(json!({
        "capability": "crypto",
        "primal": "beardog",
        "socket": "/tmp/beardog-test.sock",
        "source": "test"
    }));
    handler.register(&params).await.unwrap();
    handler
}

#[tokio::test]
async fn test_register_basic() {
    let handler = make_handler();
    let params = Some(json!({
        "capability": "http",
        "primal": "songbird",
        "socket": "/tmp/songbird.sock",
        "source": "unit_test"
    }));
    let result = handler.register(&params).await.unwrap();
    assert_eq!(result["success"], true);
    assert_eq!(result["capability"], "http");
    assert_eq!(result["primal"], "songbird");
}

#[tokio::test]
async fn test_register_missing_params() {
    let handler = make_handler();
    let result = handler.register(&None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_register_missing_capability() {
    let handler = make_handler();
    let params = Some(json!({
        "primal": "beardog",
        "socket": "/tmp/test.sock"
    }));
    let result = handler.register(&params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_register_missing_primal() {
    let handler = make_handler();
    let params = Some(json!({
        "capability": "crypto",
        "socket": "/tmp/test.sock"
    }));
    let result = handler.register(&params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_register_missing_socket() {
    let handler = make_handler();
    let params = Some(json!({
        "capability": "crypto",
        "primal": "beardog"
    }));
    let result = handler.register(&params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_register_with_semantic_mappings() {
    let handler = make_handler();
    let params = Some(json!({
        "capability": "crypto",
        "primal": "beardog",
        "socket": "/tmp/beardog.sock",
        "semantic_mappings": {
            "sha256": "crypto.blake3_hash",
            "sign": "crypto.sign"
        }
    }));
    let result = handler.register(&params).await.unwrap();
    assert_eq!(result["success"], true);

    let translations_result = handler.list_translations().await.unwrap();
    assert!(translations_result["count"].as_u64().unwrap() >= 2);
}

#[tokio::test]
async fn test_register_default_source() {
    let handler = make_handler();
    let params = Some(json!({
        "capability": "crypto",
        "primal": "beardog",
        "socket": "/tmp/beardog.sock"
    }));
    let result = handler.register(&params).await.unwrap();
    assert_eq!(result["success"], true);
}

#[tokio::test]
async fn test_metrics_empty() {
    let handler = make_handler();
    let result = handler.get_metrics().await.unwrap();
    assert_eq!(result["total_requests"], 0);
    assert!(result["metrics"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_discover_registered_capability() {
    let handler = handler_with_registration().await;
    let params = Some(json!({ "capability": "crypto" }));
    let result = handler.discover(&params).await.unwrap();
    assert_eq!(result["capability"], "crypto");
    let primals = result["primals"].as_array().unwrap();
    assert!(!primals.is_empty());
    assert_eq!(primals[0]["name"], "beardog");
}

#[tokio::test]
async fn test_discover_missing_params() {
    let handler = make_handler();
    let result = handler.discover(&None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_route_missing_params() {
    let handler = make_handler();
    let result = handler.route(&None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_route_missing_method() {
    let handler = make_handler();
    let params = Some(json!({ "capability": "crypto" }));
    let result = handler.route(&params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_discover_unregistered_capability() {
    let handler = make_handler();
    let params = Some(json!({ "capability": "nonexistent_capability_xyz" }));
    let result = handler.discover(&params).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("not registered") || err.contains("Capability") || err.contains("not found"),
        "expected capability error, got: {err}"
    );
}

#[tokio::test]
async fn test_mcp_tools_list_empty() {
    let handler = make_handler();
    let result = handler.mcp_tools_list().await.unwrap();
    assert_eq!(result["tool_count"], 0);
    assert!(result["tools"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_mcp_tools_list_after_register() {
    let handler = handler_with_registration().await;
    let params = Some(json!({
        "capability": "crypto",
        "primal": "beardog",
        "socket": "/tmp/beardog.sock",
        "semantic_mappings": {
            "sha256": "crypto.hash",
            "sign": "crypto.sign"
        }
    }));
    handler.register(&params).await.unwrap();

    let result = handler.mcp_tools_list().await.unwrap();
    let tool_count = result["tool_count"].as_u64().unwrap();
    assert!(
        tool_count >= 2,
        "expected at least 2 tools, got {tool_count}"
    );
}

#[tokio::test]
async fn test_route_success_via_mock_socket() {
    let dir = tempdir().expect("tempdir");
    let sock = dir.path().join("route-test.sock");
    let _server =
        MockJsonRpcServer::spawn_echo_success(&sock, json!({ "echo": true, "method": "pong" }))
            .await;

    let handler = make_handler();
    let reg = Some(json!({
        "capability": "mesh",
        "primal": "songbird",
        "socket": sock.to_str().unwrap(),
        "source": "test"
    }));
    handler.register(&reg).await.expect("register");

    let params = Some(json!({
        "capability": "mesh",
        "method": "any.method",
        "params": { "a": 1 }
    }));
    let result = handler.route(&params).await.expect("route");
    assert_eq!(result["echo"], true);

    let metrics = handler.get_metrics().await.expect("metrics");
    assert_eq!(metrics["total_requests"], 1);
    let m0 = &metrics["metrics"].as_array().expect("arr")[0];
    assert_eq!(m0["success"], true);
}

#[tokio::test]
async fn test_discover_missing_capability_field() {
    let handler = make_handler();
    let params = Some(json!({}));
    let err = handler.discover(&params).await.unwrap_err();
    assert!(err.to_string().contains("capability") || err.to_string().contains("Missing"));
}

#[tokio::test]
async fn test_route_missing_capability_field() {
    let handler = make_handler();
    let params = Some(json!({ "method": "x", "params": {} }));
    assert!(handler.route(&params).await.is_err());
}

#[tokio::test]
async fn test_discover_uses_domain_alias_instead_of_capability() {
    let handler = handler_with_registration().await;
    let params = Some(json!({ "domain": "crypto" }));
    let result = handler.discover(&params).await.expect("domain alias");
    assert_eq!(result["capability"], "crypto");
}

#[tokio::test]
async fn test_register_route_tcp_transport() {
    let handler = make_handler();
    let params = Some(json!({
        "primal": "remote",
        "transport": "127.0.0.1:19999",
        "capabilities": ["http.request", "relay"],
        "source": "route-test"
    }));
    let out = handler
        .register_route(&params)
        .await
        .expect("register_route");
    assert_eq!(out["registered"], 2);
    assert_eq!(out["primal"], "remote");
    let caps = out["capabilities"].as_array().expect("caps");
    assert_eq!(caps.len(), 2);
}

#[tokio::test]
async fn test_register_route_with_gate_in_source_tag() {
    let handler = make_handler();
    let params = Some(json!({
        "primal": "p",
        "transport": "/tmp/reg-route.sock",
        "capabilities": ["z"],
        "gate": "gate-a"
    }));
    let out = handler.register_route(&params).await.expect("ok");
    assert_eq!(out["gate"], "gate-a");
}

#[tokio::test]
async fn test_register_route_empty_capabilities_errors() {
    let handler = make_handler();
    let params = Some(json!({
        "primal": "p",
        "transport": "/tmp/x.sock",
        "capabilities": []
    }));
    assert!(handler.register_route(&params).await.is_err());
}

#[tokio::test]
async fn test_register_route_missing_transport() {
    let handler = make_handler();
    let params = Some(json!({
        "primal": "p",
        "capabilities": ["c"]
    }));
    assert!(handler.register_route(&params).await.is_err());
}

#[tokio::test]
async fn test_register_route_non_string_capability_errors() {
    let handler = make_handler();
    let params = Some(json!({
        "primal": "p",
        "transport": "/tmp/y.sock",
        "capabilities": [123]
    }));
    assert!(handler.register_route(&params).await.is_err());
}

#[tokio::test]
async fn test_route_uses_default_empty_params_object() {
    let dir = tempdir().expect("tempdir");
    let sock = dir.path().join("route-default-params.sock");
    let _server = MockJsonRpcServer::spawn_echo_success(&sock, json!({ "ok": true })).await;

    let handler = make_handler();
    handler
        .register(&Some(json!({
            "capability": "mesh",
            "primal": "songbird",
            "socket": sock.to_str().unwrap(),
            "source": "t"
        })))
        .await
        .unwrap();

    let params = Some(json!({
        "capability": "mesh",
        "method": "ping"
    }));
    let result = handler.route(&params).await.expect("route");
    assert_eq!(result["ok"], true);
}

#[tokio::test]
async fn test_register_missing_params_for_route_register_alias() {
    let handler = make_handler();
    assert!(handler.register_route(&None).await.is_err());
}

#[tokio::test]
async fn test_discover_missing_capability_field_errors() {
    let handler = make_handler();
    let err = handler.discover(&Some(json!({}))).await;
    assert!(err.is_err());
}

// --- capability.resolve tests ---

#[tokio::test]
async fn test_resolve_missing_params() {
    let handler = make_handler();
    assert!(handler.resolve(&None).await.is_err());
}

#[tokio::test]
async fn test_resolve_missing_capability_field() {
    let handler = make_handler();
    let params = Some(json!({}));
    let err = handler.resolve(&params).await;
    assert!(err.is_err());
    let msg = err.unwrap_err().to_string();
    assert!(
        msg.contains("capability") || msg.contains("domain") || msg.contains("Missing"),
        "expected missing field error, got: {msg}"
    );
}

#[tokio::test]
async fn test_resolve_unregistered_capability() {
    let handler = make_handler();
    let params = Some(json!({ "capability": "nonexistent_xyz" }));
    assert!(handler.resolve(&params).await.is_err());
}

#[tokio::test]
async fn test_resolve_registered_capability() {
    let handler = handler_with_registration().await;
    let params = Some(json!({ "capability": "crypto" }));
    let result = handler.resolve(&params).await.unwrap();
    assert_eq!(result["resolved"], true);
    assert_eq!(result["capability"], "crypto");
    assert_eq!(result["primal"], "beardog");
    assert!(result["provider_count"].as_u64().unwrap() >= 1);
    assert!(result["endpoint"].is_string());
}

#[tokio::test]
async fn test_resolve_uses_domain_alias() {
    let handler = handler_with_registration().await;
    let params = Some(json!({ "domain": "crypto" }));
    let result = handler.resolve(&params).await.unwrap();
    assert_eq!(result["resolved"], true);
    assert_eq!(result["capability"], "crypto");
}

#[tokio::test]
async fn test_resolve_logs_metrics() {
    let handler = handler_with_registration().await;
    let params = Some(json!({ "capability": "crypto" }));
    handler.resolve(&params).await.unwrap();

    let metrics = handler.get_metrics().await.unwrap();
    assert_eq!(
        metrics["total_requests"].as_u64().unwrap(),
        1,
        "resolve should log a routing metric"
    );
    let m0 = &metrics["metrics"].as_array().unwrap()[0];
    assert_eq!(m0["method"], "capability.resolve");
    assert_eq!(m0["success"], true);
    assert_eq!(m0["capability"], "crypto");
}

#[tokio::test]
async fn test_resolve_logs_failure_metric() {
    let handler = make_handler();
    let params = Some(json!({ "capability": "nonexistent" }));
    let _ = handler.resolve(&params).await;

    let metrics = handler.get_metrics().await.unwrap();
    assert_eq!(metrics["total_requests"].as_u64().unwrap(), 1);
    let m0 = &metrics["metrics"].as_array().unwrap()[0];
    assert_eq!(m0["success"], false);
    assert!(m0["error"].is_string());
}

#[tokio::test]
async fn test_register_route_missing_primal_field_errors() {
    let handler = make_handler();
    let params = Some(json!({
        "transport": "127.0.0.1:19998",
        "capabilities": ["a"]
    }));
    assert!(handler.register_route(&params).await.is_err());
}

#[tokio::test]
async fn test_route_logs_metric_on_success() {
    let dir = tempdir().expect("tempdir");
    let sock = dir.path().join("metric-route.sock");
    let _server = MockJsonRpcServer::spawn_echo_success(&sock, json!({ "metric": true })).await;

    let handler = make_handler();
    handler
        .register(&Some(json!({
            "capability": "metric-cap",
            "primal": "m",
            "socket": sock.to_str().unwrap(),
            "source": "t"
        })))
        .await
        .unwrap();

    let params = Some(json!({
        "capability": "metric-cap",
        "method": "x",
        "params": {}
    }));
    let out = handler.route(&params).await.expect("route");
    assert_eq!(out["metric"], true);
    let m = handler.get_metrics().await.expect("metrics");
    assert_eq!(m["total_requests"], 1);
    assert_eq!(m["metrics"].as_array().unwrap()[0]["success"], true);
}
