// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use biomeos_test_utils::MockJsonRpcServer;
use serde_json::json;

use super::super::common::create_test_server;

#[tokio::test]
async fn test_spring_status_returns_expected_shape() {
    let (server, _temp) = create_test_server();
    let req = json!({
        "jsonrpc": "2.0",
        "method": "biomeos.spring_status",
        "params": {},
        "id": 120
    })
    .to_string();

    let result = server.handle_request_json(&req).await;
    let inner = &result["result"];

    assert!(
        inner["primals"].is_array(),
        "expected primals array: {result}"
    );
    let primals = inner["primals"].as_array().unwrap();
    assert!(
        !primals.is_empty(),
        "primals array should list known primals"
    );

    // Every entry should have required fields
    for p in primals {
        assert!(p["name"].is_string(), "missing name: {p}");
        assert!(p["display_name"].is_string(), "missing display_name: {p}");
        assert!(
            p["binary_available"].is_boolean(),
            "missing binary_available: {p}"
        );
        assert!(p["capabilities"].is_array(), "missing capabilities: {p}");
    }

    assert!(
        inner["workload_count"].is_number(),
        "expected workload_count"
    );
    assert!(
        inner["workloads_running"].is_number(),
        "expected workloads_running"
    );
    assert!(
        inner["topology_version"].is_number(),
        "expected topology_version"
    );
}

#[tokio::test]
async fn test_spring_status_includes_core_primals() {
    let (server, _temp) = create_test_server();
    let req = json!({
        "jsonrpc": "2.0",
        "method": "biomeos.spring_status",
        "params": {},
        "id": 121
    })
    .to_string();

    let result = server.handle_request_json(&req).await;
    let primals = result["result"]["primals"].as_array().unwrap();
    let names: Vec<&str> = primals.iter().filter_map(|p| p["name"].as_str()).collect();

    // Core primals must appear
    assert!(names.contains(&"beardog"), "missing beardog");
    assert!(names.contains(&"songbird"), "missing songbird");
    assert!(names.contains(&"nestgate"), "missing nestgate");
    assert!(names.contains(&"toadstool"), "missing toadstool");
    // Provenance trio
    assert!(names.contains(&"rhizocrypt"), "missing rhizocrypt");
    assert!(names.contains(&"loamspine"), "missing loamspine");
    assert!(names.contains(&"sweetgrass"), "missing sweetgrass");
}

#[tokio::test]
async fn test_spring_status_has_display_names() {
    let (server, _temp) = create_test_server();
    let req = json!({
        "jsonrpc": "2.0",
        "method": "biomeos.spring_status",
        "params": {},
        "id": 122
    })
    .to_string();

    let result = server.handle_request_json(&req).await;
    let primals = result["result"]["primals"].as_array().unwrap();

    let beardog = primals.iter().find(|p| p["name"] == "beardog").unwrap();
    assert_eq!(beardog["display_name"], "BearDog");

    let nestgate = primals.iter().find(|p| p["name"] == "nestgate").unwrap();
    assert_eq!(nestgate["display_name"], "NestGate");
}

#[tokio::test]
async fn test_handle_request_method_register_registers_domains() {
    let (server, _temp) = create_test_server();
    let req = json!({
        "jsonrpc": "2.0",
        "method": "method.register",
        "params": {
            "primal": "ludoSpring",
            "transport": "/tmp/ludo.sock",
            "methods": ["game.start", "game.join", "game.end", "score.get", "score.leaderboard"]
        },
        "id": 102
    })
    .to_string();

    let result = server.handle_request_json(&req).await;
    let inner = &result["result"];
    assert_eq!(inner["registered"], 5, "should register 5 methods");
    assert_eq!(inner["primal"], "ludoSpring");
    let domains = inner["domains"].as_array().expect("domains array");
    assert!(domains.len() == 2, "should have 2 domains: game + score");
}

#[tokio::test]
async fn test_handle_request_method_register_empty_methods_errors() {
    let (server, _temp) = create_test_server();
    let req = json!({
        "jsonrpc": "2.0",
        "method": "method.register",
        "params": {
            "primal": "test",
            "transport": "/tmp/test.sock",
            "methods": []
        },
        "id": 103
    })
    .to_string();

    let result = server.handle_request_json(&req).await;
    assert!(result.get("error").is_some());
}

#[tokio::test]
async fn test_handle_request_method_register_makes_methods_semantically_routable() {
    let (server, _temp) = create_test_server();
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("ludo.sock");
    let _mock = MockJsonRpcServer::spawn_echo_success(&sock, json!({"started": true})).await;

    let reg_req = json!({
        "jsonrpc": "2.0",
        "method": "method.register",
        "params": {
            "primal": "ludoSpring",
            "transport": sock.to_str().unwrap(),
            "methods": ["game.start"]
        },
        "id": 104
    })
    .to_string();

    let reg_result = server.handle_request_json(&reg_req).await;
    assert!(
        reg_result.get("result").is_some(),
        "registration should succeed"
    );

    let call_req = json!({
        "jsonrpc": "2.0",
        "method": "game.start",
        "params": {},
        "id": 105
    })
    .to_string();

    let call_result = server.handle_request_json(&call_req).await;
    assert!(
        call_result.get("result").is_some(),
        "game.start should route via semantic fallback after method.register: {call_result}"
    );
}
