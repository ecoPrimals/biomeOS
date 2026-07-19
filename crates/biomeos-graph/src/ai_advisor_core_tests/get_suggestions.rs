// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::time::Duration;

use serde_json::json;

use super::super::*;
use super::helpers::*;
use crate::graph::{EdgeType, GraphEdge};

#[tokio::test]
async fn ai_advisor_core_get_suggestions_squirrel_partial_valid_suggestions() {
    let valid = sample_suggestion_json();
    let (_temp, socket_dir, _server) = setup_ai_socket(move |req| {
        let (id, method) = parse_request(req);
        match method.as_str() {
            "capabilities.list" => jsonrpc_success(&id, &json!({"capabilities": ["ai"]})),
            "health.check" => jsonrpc_success(&id, &json!({"status": "healthy"})),
            "ai.analyze_graph" => jsonrpc_success(
                &id,
                &json!({"suggestions": [valid.clone(), {"not_a_valid_suggestion": true}]}),
            ),
            other => panic!("unexpected method: {other}"),
        }
    })
    .await;

    run_with_socket_dir(&socket_dir, || async {
        let mut advisor = AiGraphAdvisor::new();
        assert!(advisor.check_squirrel_availability().await.unwrap());

        let graph = create_test_graph();
        let suggestions = advisor.get_suggestions(&graph).await.unwrap();
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].id, "squirrel-1");
    })
    .await;
}

#[tokio::test]
async fn ai_advisor_core_get_suggestions_via_squirrel_with_valid_suggestions() {
    let suggestion = sample_suggestion_json();
    let (_temp, socket_dir, _server) = setup_ai_socket(move |req| {
        let (id, method) = parse_request(req);
        match method.as_str() {
            "capabilities.list" => {
                jsonrpc_success(&id, &json!({"capabilities": ["ai", "ai.analyze_graph"]}))
            }
            "health.check" => jsonrpc_success(&id, &json!({"status": "healthy"})),
            "ai.analyze_graph" => {
                jsonrpc_success(&id, &json!({"suggestions": [suggestion.clone()]}))
            }
            other => panic!("unexpected method: {other}"),
        }
    })
    .await;

    run_with_socket_dir(&socket_dir, || async {
        let mut advisor = AiGraphAdvisor::new();
        assert!(advisor.check_squirrel_availability().await.unwrap());

        let graph = create_test_graph();
        let suggestions = advisor.get_suggestions(&graph).await.unwrap();
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].id, "squirrel-1");
        assert_eq!(suggestions[0].reasoning, "from squirrel");
    })
    .await;
}

#[tokio::test]
async fn ai_advisor_core_get_suggestions_squirrel_empty_suggestions_falls_back_local() {
    let (_temp, socket_dir, _server) = setup_ai_socket(|req| {
        let (id, method) = parse_request(req);
        match method.as_str() {
            "capabilities.list" => jsonrpc_success(&id, &json!({"capabilities": ["ai"]})),
            "health.check" => jsonrpc_success(&id, &json!({"status": "healthy"})),
            "ai.analyze_graph" => jsonrpc_success(&id, &json!({"suggestions": []})),
            other => panic!("unexpected method: {other}"),
        }
    })
    .await;

    run_with_socket_dir(&socket_dir, || async {
        let mut advisor = AiGraphAdvisor::new();
        assert!(advisor.check_squirrel_availability().await.unwrap());

        let graph = create_test_graph();
        let suggestions = advisor.get_suggestions(&graph).await.unwrap();
        assert!(!suggestions.is_empty());
        assert!(
            suggestions
                .iter()
                .any(|s| s.suggestion_type == SuggestionType::PerformanceImprovement)
        );
    })
    .await;
}

#[tokio::test]
async fn ai_advisor_core_get_suggestions_squirrel_invalid_suggestions_falls_back_local() {
    let (_temp, socket_dir, _server) = setup_ai_socket(|req| {
        let (id, method) = parse_request(req);
        match method.as_str() {
            "capabilities.list" => jsonrpc_success(&id, &json!({"capabilities": ["ai"]})),
            "health.check" => jsonrpc_success(&id, &json!({"status": "healthy"})),
            "ai.analyze_graph" => jsonrpc_success(
                &id,
                &json!({"suggestions": [{"not_a_valid_suggestion": true}]}),
            ),
            other => panic!("unexpected method: {other}"),
        }
    })
    .await;

    run_with_socket_dir(&socket_dir, || async {
        let mut advisor = AiGraphAdvisor::new();
        assert!(advisor.check_squirrel_availability().await.unwrap());

        let graph = create_test_graph();
        let suggestions = advisor.get_suggestions(&graph).await.unwrap();
        assert!(!suggestions.is_empty());
    })
    .await;
}

#[tokio::test]
async fn ai_advisor_core_get_suggestions_squirrel_rpc_error_falls_back_local() {
    let (_temp, socket_dir, _server) = setup_ai_socket(|req| {
        let (id, method) = parse_request(req);
        match method.as_str() {
            "capabilities.list" => jsonrpc_success(&id, &json!({"capabilities": ["ai"]})),
            "health.check" => jsonrpc_success(&id, &json!({"status": "healthy"})),
            "ai.analyze_graph" => jsonrpc_error(&id, -32603, "analyze failed"),
            other => panic!("unexpected method: {other}"),
        }
    })
    .await;

    run_with_socket_dir(&socket_dir, || async {
        let mut advisor = AiGraphAdvisor::new();
        assert!(advisor.check_squirrel_availability().await.unwrap());

        let graph = create_test_graph();
        let suggestions = advisor.get_suggestions(&graph).await.unwrap();
        assert!(!suggestions.is_empty());
    })
    .await;
}

#[tokio::test(start_paused = true)]
async fn ai_advisor_core_get_suggestions_squirrel_timeout_falls_back_local() {
    let (_temp, sock) = bind_hanging_socket().await;

    let mut advisor = AiGraphAdvisor::with_timeout(Duration::from_millis(50));
    advisor.test_set_squirrel_state(true, Some(sock));

    let graph = create_test_graph();
    let suggestions_fut = advisor.get_suggestions(&graph);
    tokio::pin!(suggestions_fut);
    tokio::time::advance(Duration::from_millis(100)).await;
    let suggestions = suggestions_fut.await.unwrap();
    assert!(!suggestions.is_empty());
}

#[tokio::test]
async fn ai_advisor_core_get_suggestions_uses_local_when_unavailable() {
    let advisor = AiGraphAdvisor::new();
    let graph = create_test_graph();
    let suggestions = advisor.get_suggestions(&graph).await.unwrap();
    assert!(!suggestions.is_empty());
    assert!(
        suggestions
            .iter()
            .any(|s| s.suggestion_type == SuggestionType::PerformanceImprovement)
    );
}

#[tokio::test]
async fn ai_advisor_core_get_suggestions_squirrel_missing_suggestions_key_falls_back_local() {
    let (_temp, socket_dir, _server) = setup_ai_socket(|req| {
        let (id, method) = parse_request(req);
        match method.as_str() {
            "capabilities.list" => jsonrpc_success(&id, &json!({"capabilities": ["ai"]})),
            "health.check" => jsonrpc_success(&id, &json!({"status": "healthy"})),
            "ai.analyze_graph" => jsonrpc_success(&id, &json!({"analysis": "complete"})),
            other => panic!("unexpected method: {other}"),
        }
    })
    .await;

    run_with_socket_dir(&socket_dir, || async {
        let mut advisor = AiGraphAdvisor::new();
        assert!(advisor.check_squirrel_availability().await.unwrap());

        let graph = create_test_graph();
        let suggestions = advisor.get_suggestions(&graph).await.unwrap();
        assert!(!suggestions.is_empty());
    })
    .await;
}

#[tokio::test]
async fn ai_advisor_core_get_suggestions_non_utf8_socket_errors() {
    let temp = tempfile::tempdir().expect("tempdir");
    let bad_sock = non_utf8_socket_path(temp.path());

    let mut advisor = AiGraphAdvisor::new();
    advisor.test_set_squirrel_state(true, Some(bad_sock));

    let graph = create_test_graph();
    let err = advisor.get_suggestions(&graph).await.unwrap_err();
    assert!(
        err.to_string()
            .contains("AI socket path is not valid UTF-8"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn ai_advisor_core_get_suggestions_errors_when_squirrel_available_without_socket() {
    let mut advisor = AiGraphAdvisor::new();
    advisor.test_set_squirrel_state(true, None);

    let graph = create_test_graph();
    let err = advisor.get_suggestions(&graph).await.unwrap_err();
    assert!(
        err.to_string()
            .contains("AI provider socket not discovered"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn ai_advisor_core_get_suggestions_with_edges_uses_squirrel_path() {
    let (_temp, socket_dir, _server) = setup_ai_socket(|req| {
        let (id, method) = parse_request(req);
        match method.as_str() {
            "capabilities.list" => jsonrpc_success(&id, &json!({"capabilities": ["ai"]})),
            "health.check" => jsonrpc_success(&id, &json!({"status": "healthy"})),
            "ai.analyze_graph" => jsonrpc_success(&id, &json!({"suggestions": []})),
            other => panic!("unexpected method: {other}"),
        }
    })
    .await;

    run_with_socket_dir(&socket_dir, || async {
        let mut advisor = AiGraphAdvisor::new();
        assert!(advisor.check_squirrel_availability().await.unwrap());

        let mut graph = create_test_graph();
        graph.edges.push(GraphEdge {
            from: "node1".to_string(),
            to: "node2".to_string(),
            edge_type: EdgeType::DataFlow,
        });

        let suggestions = advisor.get_suggestions(&graph).await.unwrap();
        assert!(
            !suggestions
                .iter()
                .any(|s| s.suggestion_type == SuggestionType::PerformanceImprovement)
        );
    })
    .await;
}

#[tokio::test]
async fn ai_advisor_core_get_suggestions_squirrel_non_array_suggestions_falls_back_local() {
    let (_temp, socket_dir, _server) = setup_ai_socket(|req| {
        let (id, method) = parse_request(req);
        match method.as_str() {
            "capabilities.list" => jsonrpc_success(&id, &json!({"capabilities": ["ai"]})),
            "health.check" => jsonrpc_success(&id, &json!({"status": "healthy"})),
            "ai.analyze_graph" => jsonrpc_success(&id, &json!({"suggestions": "not-an-array"})),
            other => panic!("unexpected method: {other}"),
        }
    })
    .await;

    run_with_socket_dir(&socket_dir, || async {
        let mut advisor = AiGraphAdvisor::new();
        assert!(advisor.check_squirrel_availability().await.unwrap());

        let graph = create_test_graph();
        let suggestions = advisor.get_suggestions(&graph).await.unwrap();
        assert!(!suggestions.is_empty());
        assert!(
            suggestions
                .iter()
                .any(|s| s.suggestion_type == SuggestionType::PerformanceImprovement)
        );
    })
    .await;
}
