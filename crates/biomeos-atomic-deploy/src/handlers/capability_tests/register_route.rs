// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use serde_json::json;

use super::make_handler;

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
async fn test_register_route_missing_primal_field_errors() {
    let handler = make_handler();
    let params = Some(json!({
        "transport": "127.0.0.1:19998",
        "capabilities": ["a"]
    }));
    assert!(handler.register_route(&params).await.is_err());
}
