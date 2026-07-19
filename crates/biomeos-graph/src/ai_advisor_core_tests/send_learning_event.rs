// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::json;

use super::super::*;
use super::helpers::*;

#[tokio::test]
async fn ai_advisor_core_send_learning_event_skipped_when_unavailable() {
    let advisor = AiGraphAdvisor::new();
    let graph = create_test_graph();
    let event = sample_learning_event(&graph);
    assert!(advisor.send_learning_event(event).await.is_ok());
}

#[tokio::test]
async fn ai_advisor_core_send_learning_event_success_when_available() {
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_clone = Arc::clone(&calls);
    let (_temp, socket_dir, _server) = setup_ai_socket(move |req| {
        let (id, method) = parse_request(req);
        match method.as_str() {
            "capabilities.list" => jsonrpc_success(&id, &json!({"capabilities": ["ai"]})),
            "health.check" => jsonrpc_success(&id, &json!({"status": "healthy"})),
            "ai.learn_from_event" => {
                calls_clone.fetch_add(1, Ordering::SeqCst);
                jsonrpc_success(&id, &json!({"recorded": true}))
            }
            other => panic!("unexpected method: {other}"),
        }
    })
    .await;

    run_with_socket_dir(&socket_dir, || async {
        let mut advisor = AiGraphAdvisor::new();
        assert!(advisor.check_squirrel_availability().await.unwrap());

        let graph = create_test_graph();
        let event = sample_learning_event(&graph);
        assert!(advisor.send_learning_event(event).await.is_ok());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    })
    .await;
}

#[tokio::test]
async fn ai_advisor_core_send_learning_event_rpc_error_still_ok() {
    let (_temp, socket_dir, _server) = setup_ai_socket(|req| {
        let (id, method) = parse_request(req);
        match method.as_str() {
            "capabilities.list" => jsonrpc_success(&id, &json!({"capabilities": ["ai"]})),
            "health.check" => jsonrpc_success(&id, &json!({"status": "healthy"})),
            "ai.learn_from_event" => jsonrpc_error(&id, -32603, "learn failed"),
            other => panic!("unexpected method: {other}"),
        }
    })
    .await;

    run_with_socket_dir(&socket_dir, || async {
        let mut advisor = AiGraphAdvisor::new();
        assert!(advisor.check_squirrel_availability().await.unwrap());

        let graph = create_test_graph();
        let event = sample_learning_event(&graph);
        assert!(advisor.send_learning_event(event).await.is_ok());
    })
    .await;
}

#[tokio::test]
async fn ai_advisor_core_send_learning_event_non_utf8_socket_errors() {
    let temp = tempfile::tempdir().expect("tempdir");
    let bad_sock = non_utf8_socket_path(temp.path());

    let mut advisor = AiGraphAdvisor::new();
    advisor.test_set_squirrel_state(true, Some(bad_sock));

    let graph = create_test_graph();
    let event = sample_learning_event(&graph);
    let err = advisor.send_learning_event(event).await.unwrap_err();
    assert!(
        err.to_string()
            .contains("AI socket path is not valid UTF-8"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn ai_advisor_core_send_learning_event_errors_without_socket_when_marked_available() {
    let mut advisor = AiGraphAdvisor::new();
    advisor.test_set_squirrel_state(true, None);

    let graph = create_test_graph();
    let event = sample_learning_event(&graph);
    let err = advisor.send_learning_event(event).await.unwrap_err();
    assert!(
        err.to_string()
            .contains("AI provider socket not discovered"),
        "unexpected error: {err}"
    );
}
