// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use serde_json::json;

use super::super::*;
use super::helpers::*;

#[tokio::test]
async fn ai_advisor_core_check_squirrel_availability_healthy_discovery() {
    let (_temp, socket_dir, _server) = setup_ai_socket(|req| {
        let (id, method) = parse_request(req);
        match method.as_str() {
            "capabilities.list" => {
                jsonrpc_success(&id, &json!({"capabilities": ["ai", "ai.analyze_graph"]}))
            }
            "health.check" => jsonrpc_success(&id, &json!({"status": "healthy"})),
            other => panic!("unexpected method: {other}"),
        }
    })
    .await;

    run_with_socket_dir(&socket_dir, || async {
        let mut advisor = AiGraphAdvisor::new();
        let available = advisor.check_squirrel_availability().await.unwrap();
        assert!(available);
        assert!(advisor.squirrel_available);
    })
    .await;
}

#[tokio::test]
async fn ai_advisor_core_check_squirrel_availability_ok_status() {
    let (_temp, socket_dir, _server) = setup_ai_socket(|req| {
        let (id, method) = parse_request(req);
        match method.as_str() {
            "capabilities.list" => jsonrpc_success(&id, &json!({"capabilities": ["ai"]})),
            "health.check" => jsonrpc_success(&id, &json!({"status": "ok"})),
            other => panic!("unexpected method: {other}"),
        }
    })
    .await;

    run_with_socket_dir(&socket_dir, || async {
        let mut advisor = AiGraphAdvisor::new();
        assert!(advisor.check_squirrel_availability().await.unwrap());
    })
    .await;
}

#[tokio::test]
async fn ai_advisor_core_check_squirrel_availability_unhealthy_status() {
    let (_temp, socket_dir, _server) = setup_ai_socket(|req| {
        let (id, method) = parse_request(req);
        match method.as_str() {
            "capabilities.list" => jsonrpc_success(&id, &json!({"capabilities": ["ai"]})),
            "health.check" => jsonrpc_success(&id, &json!({"status": "degraded"})),
            other => panic!("unexpected method: {other}"),
        }
    })
    .await;

    run_with_socket_dir(&socket_dir, || async {
        let mut advisor = AiGraphAdvisor::new();
        let available = advisor.check_squirrel_availability().await.unwrap();
        assert!(!available);
        assert!(!advisor.squirrel_available);
    })
    .await;
}

#[tokio::test]
async fn ai_advisor_core_check_squirrel_availability_health_rpc_failure() {
    let (_temp, socket_dir, _server) = setup_ai_socket(|req| {
        let (id, method) = parse_request(req);
        match method.as_str() {
            "capabilities.list" => jsonrpc_success(&id, &json!({"capabilities": ["ai"]})),
            "health.check" => jsonrpc_error(&id, -32603, "health unavailable"),
            other => panic!("unexpected method: {other}"),
        }
    })
    .await;

    run_with_socket_dir(&socket_dir, || async {
        let mut advisor = AiGraphAdvisor::new();
        let available = advisor.check_squirrel_availability().await.unwrap();
        assert!(!available);
        assert!(!advisor.squirrel_available);
    })
    .await;
}

#[tokio::test]
async fn ai_advisor_core_check_squirrel_availability_missing_status() {
    let (_temp, socket_dir, _server) = setup_ai_socket(|req| {
        let (id, method) = parse_request(req);
        match method.as_str() {
            "capabilities.list" => jsonrpc_success(&id, &json!({"capabilities": ["ai"]})),
            "health.check" => jsonrpc_success(&id, &json!({"uptime_secs": 42})),
            other => panic!("unexpected method: {other}"),
        }
    })
    .await;

    run_with_socket_dir(&socket_dir, || async {
        let mut advisor = AiGraphAdvisor::new();
        let available = advisor.check_squirrel_availability().await.unwrap();
        assert!(!available);
        assert!(!advisor.squirrel_available);
    })
    .await;
}

#[tokio::test]
async fn ai_advisor_core_check_squirrel_availability_discovery_failure() {
    let temp = tempfile::tempdir().expect("tempdir");
    let empty_dir = temp.path().join("empty-runtime");
    std::fs::create_dir_all(&empty_dir).expect("empty runtime");

    run_with_socket_dir(&empty_dir, || async {
        let mut advisor = AiGraphAdvisor::new();
        let available = advisor.check_squirrel_availability().await.unwrap();
        assert!(!available);
        assert!(!advisor.squirrel_available);
    })
    .await;
}

#[tokio::test]
async fn ai_advisor_core_check_squirrel_availability_stores_socket_path() {
    let (_temp, socket_dir, _server) = setup_ai_socket(|req| {
        let (id, method) = parse_request(req);
        match method.as_str() {
            "capabilities.list" => jsonrpc_success(&id, &json!({"capabilities": ["ai"]})),
            "health.check" => jsonrpc_success(&id, &json!({"status": "healthy"})),
            other => panic!("unexpected method: {other}"),
        }
    })
    .await;

    run_with_socket_dir(&socket_dir, || async {
        let mut advisor = AiGraphAdvisor::new();
        assert!(advisor.ai_socket_path.is_none());

        assert!(advisor.check_squirrel_availability().await.unwrap());
        assert!(advisor.ai_socket_path.is_some());
    })
    .await;
}
