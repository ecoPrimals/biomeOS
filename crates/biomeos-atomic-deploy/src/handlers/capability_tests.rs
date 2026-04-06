// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Capability handler tests - extracted to keep capability.rs under 1000 lines

#![expect(clippy::unwrap_used, reason = "test")]
#![allow(clippy::expect_used)]

use biomeos_test_utils::MockJsonRpcServer;
use serde_json::json;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::sync::RwLock;

use crate::capability_translation::CapabilityTranslationRegistry;
use crate::neural_router::NeuralRouter;

use super::capability::CapabilityHandler;

fn make_handler() -> CapabilityHandler {
    let router = Arc::new(NeuralRouter::new("test-family"));
    let registry = Arc::new(RwLock::new(CapabilityTranslationRegistry::new()));
    CapabilityHandler::new(router, registry)
}

async fn handler_with_registration() -> CapabilityHandler {
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
async fn test_capability_handler_creation() {
    let handler = make_handler();
    let result = handler.list().await.unwrap();
    assert!(result["capabilities"].as_array().unwrap().is_empty());
    assert_eq!(result["count"], 0);
}

#[tokio::test]
async fn test_list_empty() {
    let handler = make_handler();
    let result = handler.list().await.unwrap();
    assert_eq!(result["count"], 0);
    assert!(result["capabilities"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_list_after_register() {
    let handler = handler_with_registration().await;
    let result = handler.list().await.unwrap();
    assert_eq!(result["count"], 1);
    let caps = result["capabilities"].as_array().unwrap();
    assert!(caps.iter().any(|c| c.as_str() == Some("crypto")));
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
async fn test_providers_empty() {
    let handler = make_handler();
    let params = Some(json!({ "capability": "nonexistent" }));
    let result = handler.providers(&params).await.unwrap();
    assert_eq!(result["count"], 0);
}

#[tokio::test]
async fn test_providers_after_register() {
    let handler = handler_with_registration().await;
    let params = Some(json!({ "capability": "crypto" }));
    let result = handler.providers(&params).await.unwrap();
    assert_eq!(result["count"], 1);
    let providers = result["providers"].as_array().unwrap();
    assert_eq!(providers[0]["primal"], "beardog");
}

#[tokio::test]
async fn test_providers_missing_params() {
    let handler = make_handler();
    let result = handler.providers(&None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_metrics_empty() {
    let handler = make_handler();
    let result = handler.get_metrics().await.unwrap();
    assert_eq!(result["total_requests"], 0);
    assert!(result["metrics"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_discover_translations_empty() {
    let handler = make_handler();
    let params = Some(json!({ "capability": "crypto" }));
    let result = handler.discover_translations(&params).await.unwrap();
    assert_eq!(result["count"], 0);
    assert_eq!(result["capability"], "crypto");
}

#[tokio::test]
async fn test_discover_translations_after_register() {
    let handler = make_handler();
    let params = Some(json!({
        "capability": "crypto",
        "primal": "beardog",
        "socket": "/tmp/beardog.sock",
        "semantic_mappings": {
            "sha256": "crypto.blake3_hash"
        }
    }));
    handler.register(&params).await.unwrap();

    let params = Some(json!({ "capability": "crypto" }));
    let result = handler.discover_translations(&params).await.unwrap();
    assert_eq!(result["capability"], "crypto");
}

#[tokio::test]
async fn test_discover_translations_missing_params() {
    let handler = make_handler();
    let result = handler.discover_translations(&None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_translations_empty() {
    let handler = make_handler();
    let result = handler.list_translations().await.unwrap();
    assert_eq!(result["count"], 0);
}

#[tokio::test]
async fn test_list_translations_after_register() {
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
    handler.register(&params).await.unwrap();

    let result = handler.list_translations().await.unwrap();
    let count = result["count"].as_u64().unwrap();
    assert!(count >= 2, "Expected at least 2 translations, got {count}");

    let translations = result["translations"].as_array().unwrap();
    let semantics: Vec<&str> = translations
        .iter()
        .filter_map(|t| t["semantic"].as_str())
        .collect();
    assert!(semantics.contains(&"crypto.sha256"));
    assert!(semantics.contains(&"crypto.sign"));
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
async fn test_call_missing_params() {
    let handler = make_handler();
    let result = handler.call(&None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_call_missing_capability() {
    let handler = make_handler();
    let params = Some(json!({ "operation": "sha256" }));
    let result = handler.call(&params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_call_missing_operation() {
    let handler = make_handler();
    let params = Some(json!({ "capability": "crypto" }));
    let result = handler.call(&params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_call_dotted_capability() {
    let handler = handler_with_registration().await;
    let params = Some(json!({
        "capability": "crypto.sha256",
        "args": { "data": "test" }
    }));
    let result = handler.call(&params).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(!err.contains("Missing 'operation'"));
}

#[tokio::test]
async fn test_call_params_alias_for_args() {
    let handler = handler_with_registration().await;
    let params = Some(json!({
        "capability": "crypto",
        "operation": "sha256",
        "params": { "data": "test" }
    }));
    let result = handler.call(&params).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(!err.contains("Missing"));
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
async fn test_multiple_capabilities() {
    let handler = make_handler();

    handler
        .register(&Some(json!({
            "capability": "crypto",
            "primal": "beardog",
            "socket": "/tmp/beardog.sock"
        })))
        .await
        .unwrap();

    handler
        .register(&Some(json!({
            "capability": "http",
            "primal": "songbird",
            "socket": "/tmp/songbird.sock"
        })))
        .await
        .unwrap();

    let result = handler.list().await.unwrap();
    assert_eq!(result["count"], 2);
}

#[tokio::test]
async fn test_multiple_providers_same_capability() {
    let handler = make_handler();

    handler
        .register(&Some(json!({
            "capability": "compute",
            "primal": "toadstool-tower",
            "socket": "/tmp/toadstool-tower.sock"
        })))
        .await
        .unwrap();

    handler
        .register(&Some(json!({
            "capability": "compute",
            "primal": "toadstool-gate2",
            "socket": "/tmp/toadstool-gate2.sock"
        })))
        .await
        .unwrap();

    let params = Some(json!({ "capability": "compute" }));
    let result = handler.providers(&params).await.unwrap();
    assert_eq!(result["count"], 2);
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
async fn test_list_includes_cost_estimates() {
    let handler = handler_with_registration().await;
    let result = handler.list().await.unwrap();
    let details = result["details"].as_array().unwrap();
    if !details.is_empty() {
        let first = &details[0];
        assert!(first.get("cost_estimates").is_some());
        assert!(first.get("operation_dependencies").is_some());
        assert!(first.get("locality").is_some());
    }
}

#[tokio::test]
async fn test_register_semantic_mappings_non_object_skipped() {
    let handler = make_handler();
    let params = Some(json!({
        "capability": "crypto",
        "primal": "beardog",
        "socket": "/tmp/x.sock",
        "semantic_mappings": []
    }));
    handler.register(&params).await.unwrap();
    let tr = handler.list_translations().await.unwrap();
    assert_eq!(tr["count"], 0);
}

#[tokio::test]
async fn test_register_semantic_mappings_ignores_non_string_values() {
    let handler = make_handler();
    let params = Some(json!({
        "capability": "crypto",
        "primal": "beardog",
        "socket": "/tmp/x.sock",
        "semantic_mappings": { "op1": 123, "op2": "real.method" }
    }));
    handler.register(&params).await.unwrap();
    let tr = handler.list_translations().await.unwrap();
    assert_eq!(tr["count"], 1);
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
async fn test_call_uses_translation_when_present() {
    let dir = tempdir().expect("tempdir");
    let sock = dir.path().join("call-tr.sock");
    let _server = MockJsonRpcServer::spawn_echo_success(&sock, json!({ "hashed": "abc" })).await;

    let handler = make_handler();
    let reg = Some(json!({
        "capability": "crypto",
        "primal": "beardog",
        "socket": sock.to_str().unwrap(),
        "source": "test",
        "semantic_mappings": { "sha256": "crypto.blake3_hash" }
    }));
    handler.register(&reg).await.expect("reg");

    let params = Some(json!({
        "capability": "crypto",
        "operation": "sha256",
        "args": { "data": "x" }
    }));
    let out = handler.call(&params).await.expect("call");
    assert_eq!(out["hashed"], "abc");
}

#[tokio::test]
async fn test_call_direct_without_translation_warn_path() {
    let dir = tempdir().expect("tempdir");
    let sock = dir.path().join("call-direct.sock");
    let _server = MockJsonRpcServer::spawn_echo_success(&sock, json!({ "direct": true })).await;

    let handler = make_handler();
    let reg = Some(json!({
        "capability": "custom",
        "primal": "p",
        "socket": sock.to_str().unwrap(),
        "source": "test"
    }));
    handler.register(&reg).await.expect("register");

    let params = Some(json!({
        "capability": "custom",
        "operation": "custom.op",
        "args": {}
    }));
    let out = handler.call(&params).await.expect("call");
    assert_eq!(out["direct"], true);
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
async fn test_list_details_compute_locality_and_costs() {
    let handler = make_handler();
    handler
        .register(&Some(json!({
            "capability": "compute",
            "primal": "t",
            "socket": "/tmp/c.sock"
        })))
        .await
        .unwrap();
    handler
        .register(&Some(json!({
            "capability": "relay",
            "primal": "r",
            "socket": "/tmp/r.sock"
        })))
        .await
        .unwrap();
    let list = handler.list().await.unwrap();
    let details = list["details"].as_array().unwrap();
    let compute = details
        .iter()
        .find(|d| d["capability"] == "compute")
        .expect("compute entry");
    assert_eq!(compute["locality"], "local");
    let relay = details
        .iter()
        .find(|d| d["capability"] == "relay")
        .expect("relay");
    assert_eq!(relay["locality"], "mesh");
}

#[tokio::test]
async fn test_discover_translations_missing_capability_field() {
    let handler = make_handler();
    let err = handler.discover_translations(&Some(json!({}))).await;
    assert!(err.is_err());
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
async fn test_call_with_unknown_gate_falls_through() {
    let dir = tempdir().expect("tempdir");
    let sock = dir.path().join("call-gate.sock");
    let _server = MockJsonRpcServer::spawn_echo_success(&sock, json!({ "via": "local" })).await;

    let handler = make_handler();
    handler
        .register(&Some(json!({
            "capability": "zcap",
            "primal": "z",
            "socket": sock.to_str().unwrap(),
            "source": "t"
        })))
        .await
        .unwrap();

    let params = Some(json!({
        "capability": "zcap",
        "operation": "zcap.op",
        "args": {},
        "gate": "nonexistent_gate_label"
    }));
    let out = handler
        .call(&params)
        .await
        .expect("call without remote gate");
    assert_eq!(out["via"], "local");
}

#[tokio::test]
async fn test_list_includes_shader_domain_metadata() {
    let handler = make_handler();
    handler
        .register(&Some(json!({
            "capability": "shader",
            "primal": "gpu",
            "socket": "/tmp/shader.sock"
        })))
        .await
        .unwrap();
    let list = handler.list().await.unwrap();
    let details = list["details"].as_array().unwrap();
    let shader = details
        .iter()
        .find(|d| d["capability"] == "shader")
        .expect("shader entry");
    assert_eq!(shader["locality"], "local");
    assert_eq!(shader["provider_count"], 1);
    assert!(shader["cost_estimates"].is_array());
}

#[tokio::test]
async fn test_list_includes_stun_locality_mesh() {
    let handler = make_handler();
    handler
        .register(&Some(json!({
            "capability": "stun",
            "primal": "s",
            "socket": "/tmp/stun.sock"
        })))
        .await
        .unwrap();
    let list = handler.list().await.unwrap();
    let details = list["details"].as_array().unwrap();
    let stun = details
        .iter()
        .find(|d| d["capability"] == "stun")
        .expect("stun");
    assert_eq!(stun["locality"], "mesh");
}

#[tokio::test]
async fn test_register_missing_params_for_route_register_alias() {
    let handler = make_handler();
    assert!(handler.register_route(&None).await.is_err());
}

#[tokio::test]
async fn test_providers_missing_capability_field_errors() {
    let handler = make_handler();
    let err = handler.providers(&Some(json!({}))).await;
    assert!(err.is_err());
}

#[tokio::test]
async fn test_discover_missing_capability_field_errors() {
    let handler = make_handler();
    let err = handler.discover(&Some(json!({}))).await;
    assert!(err.is_err());
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
