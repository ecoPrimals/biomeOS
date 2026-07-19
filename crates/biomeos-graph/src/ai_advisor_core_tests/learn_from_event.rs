// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use chrono::Utc;
use serde_json::json;

use super::super::*;
use super::helpers::*;
use crate::events::GraphEvent;

#[tokio::test]
async fn ai_advisor_core_learn_from_event_ignores_unhandled_events() {
    let advisor = AiGraphAdvisor::new();
    let started = GraphEvent::GraphStarted {
        graph_id: "g".to_string(),
        graph_name: "g".to_string(),
        total_nodes: 1,
        coordination: "sequential".to_string(),
        timestamp: Utc::now(),
    };
    assert!(advisor.learn_from_event(&started).await.is_ok());
}

#[tokio::test]
async fn ai_advisor_core_learn_from_event_forwards_to_squirrel_when_available() {
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_clone = Arc::clone(&calls);
    let (_temp, socket_dir, _server) = setup_ai_socket(move |req| {
        let (id, method) = parse_request(req);
        match method.as_str() {
            "capabilities.list" => jsonrpc_success(&id, &json!({"capabilities": ["ai"]})),
            "health.check" => jsonrpc_success(&id, &json!({"status": "healthy"})),
            "ai.learn_event" => {
                calls_clone.fetch_add(1, Ordering::SeqCst);
                jsonrpc_success(&id, &json!({"learned": true}))
            }
            other => panic!("unexpected method: {other}"),
        }
    })
    .await;

    run_with_socket_dir(&socket_dir, || async {
        let mut advisor = AiGraphAdvisor::new();
        assert!(advisor.check_squirrel_availability().await.unwrap());

        let failed = GraphEvent::NodeFailed {
            graph_id: "g".to_string(),
            node_id: "n".to_string(),
            error: "boom".to_string(),
            retry_attempt: 0,
            will_retry: false,
            timestamp: Utc::now(),
        };
        assert!(advisor.learn_from_event(&failed).await.is_ok());
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        let decision = GraphEvent::DecisionMade {
            graph_id: "g".to_string(),
            decision_type: "retry".to_string(),
            reasoning: vec!["backoff".to_string()],
            confidence: 0.5,
            timestamp: Utc::now(),
        };
        assert!(advisor.learn_from_event(&decision).await.is_ok());
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    })
    .await;
}

#[tokio::test]
async fn ai_advisor_core_learn_from_event_without_squirrel_does_not_call_rpc() {
    let advisor = AiGraphAdvisor::new();
    let failed = GraphEvent::NodeFailed {
        graph_id: "g".to_string(),
        node_id: "n".to_string(),
        error: "e".to_string(),
        retry_attempt: 0,
        will_retry: false,
        timestamp: Utc::now(),
    };
    assert!(advisor.learn_from_event(&failed).await.is_ok());
}

#[tokio::test]
async fn ai_advisor_core_learn_from_event_rpc_error_still_ok() {
    let (_temp, socket_dir, _server) = setup_ai_socket(|req| {
        let (id, method) = parse_request(req);
        match method.as_str() {
            "capabilities.list" => jsonrpc_success(&id, &json!({"capabilities": ["ai"]})),
            "health.check" => jsonrpc_success(&id, &json!({"status": "healthy"})),
            "ai.learn_event" => jsonrpc_error(&id, -32603, "learn failed"),
            other => panic!("unexpected method: {other}"),
        }
    })
    .await;

    run_with_socket_dir(&socket_dir, || async {
        let mut advisor = AiGraphAdvisor::new();
        assert!(advisor.check_squirrel_availability().await.unwrap());

        let failed = GraphEvent::NodeFailed {
            graph_id: "g".to_string(),
            node_id: "n".to_string(),
            error: "rpc error".to_string(),
            retry_attempt: 0,
            will_retry: false,
            timestamp: Utc::now(),
        };
        assert!(advisor.learn_from_event(&failed).await.is_ok());
    })
    .await;
}

#[tokio::test(start_paused = true)]
async fn ai_advisor_core_learn_from_event_rpc_timeout_still_ok() {
    let (_temp, sock) = bind_hanging_socket().await;

    let mut advisor = AiGraphAdvisor::with_timeout(Duration::from_millis(50));
    advisor.test_set_squirrel_state(true, Some(sock));

    let failed = GraphEvent::NodeFailed {
        graph_id: "g".to_string(),
        node_id: "n".to_string(),
        error: "timeout".to_string(),
        retry_attempt: 0,
        will_retry: false,
        timestamp: Utc::now(),
    };
    let learn_fut = advisor.learn_from_event(&failed);
    tokio::pin!(learn_fut);
    tokio::time::advance(Duration::from_millis(100)).await;
    assert!(learn_fut.await.is_ok());
}

#[tokio::test]
async fn ai_advisor_core_learn_from_event_skips_rpc_when_socket_present_but_unavailable() {
    let (_temp, sock) = bind_hanging_socket().await;
    let mut advisor = AiGraphAdvisor::new();
    advisor.test_set_squirrel_state(false, Some(sock));

    let failed = GraphEvent::NodeFailed {
        graph_id: "g".to_string(),
        node_id: "n".to_string(),
        error: "offline".to_string(),
        retry_attempt: 0,
        will_retry: false,
        timestamp: Utc::now(),
    };
    assert!(advisor.learn_from_event(&failed).await.is_ok());
}

#[tokio::test]
async fn ai_advisor_core_learn_from_event_rpc_success_still_ok() {
    let (_temp, socket_dir, _server) = setup_ai_socket(|req| {
        let (id, method) = parse_request(req);
        match method.as_str() {
            "capabilities.list" => jsonrpc_success(&id, &json!({"capabilities": ["ai"]})),
            "health.check" => jsonrpc_success(&id, &json!({"status": "healthy"})),
            "ai.learn_event" => jsonrpc_success(&id, &json!({"learned": true})),
            other => panic!("unexpected method: {other}"),
        }
    })
    .await;

    run_with_socket_dir(&socket_dir, || async {
        let mut advisor = AiGraphAdvisor::new();
        assert!(advisor.check_squirrel_availability().await.unwrap());

        let decision = GraphEvent::DecisionMade {
            graph_id: "g".to_string(),
            decision_type: "route".to_string(),
            reasoning: vec!["path-a".to_string(), "path-b".to_string()],
            confidence: 0.9,
            timestamp: Utc::now(),
        };
        assert!(advisor.learn_from_event(&decision).await.is_ok());
    })
    .await;
}
