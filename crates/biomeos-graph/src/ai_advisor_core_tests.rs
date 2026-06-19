// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use biomeos_test_utils::MockJsonRpcServer;
use chrono::Utc;
use serde_json::json;

use super::*;
use crate::events::GraphEvent;
use crate::graph::{
    CoordinationPattern, EdgeType, GraphEdge, GraphId, Operation, PrimalGraph, PrimalNode,
    PrimalSelector,
};
use crate::modification::GraphModification;

fn create_test_graph() -> PrimalGraph {
    PrimalGraph {
        id: GraphId::new("test").unwrap(),
        name: "test".to_string(),
        description: "Test graph".to_string(),
        version: "1.0.0".to_string(),
        coordination: CoordinationPattern::Sequential,
        nodes: vec![
            PrimalNode {
                id: "node1".to_string(),
                primal: PrimalSelector::ByCapability {
                    by_capability: "compute".to_string(),
                },
                operation: Operation {
                    name: "process".to_string(),
                    params: serde_json::json!({}),
                    environment: None,
                },
                input: None,
                outputs: vec![],
                constraints: None,
            },
            PrimalNode {
                id: "node2".to_string(),
                primal: PrimalSelector::ByCapability {
                    by_capability: "storage".to_string(),
                },
                operation: Operation {
                    name: "store".to_string(),
                    params: serde_json::json!({}),
                    environment: None,
                },
                input: None,
                outputs: vec![],
                constraints: None,
            },
            PrimalNode {
                id: "node3".to_string(),
                primal: PrimalSelector::ByCapability {
                    by_capability: "network".to_string(),
                },
                operation: Operation {
                    name: "send".to_string(),
                    params: serde_json::json!({}),
                    environment: None,
                },
                input: None,
                outputs: vec![],
                constraints: None,
            },
        ],
        edges: vec![],
    }
}

fn sample_suggestion() -> AiSuggestion {
    AiSuggestion {
        id: "squirrel-1".to_string(),
        suggestion_type: SuggestionType::Optimization,
        modification: GraphModification::ChangeCoordination {
            pattern: CoordinationPattern::Parallel,
        },
        reasoning: "from squirrel".to_string(),
        confidence: 0.95,
        evidence: vec!["test evidence".to_string()],
        impact: ImpactEstimate {
            performance: 0.5,
            reliability: 0.0,
            complexity: 0.0,
            summary: "test impact".to_string(),
        },
    }
}

fn sample_suggestion_json() -> serde_json::Value {
    serde_json::to_value(sample_suggestion()).expect("serialize suggestion")
}

fn sample_learning_event(graph: &PrimalGraph) -> LearningEvent {
    let before = GraphSnapshot::from_graph(graph);
    let mut after_graph = graph.clone();
    after_graph.coordination = CoordinationPattern::Parallel;
    LearningEvent {
        event_type: "user_edit".to_string(),
        before,
        after: GraphSnapshot::from_graph(&after_graph),
        action: GraphModification::ChangeCoordination {
            pattern: CoordinationPattern::Parallel,
        },
        context: std::collections::HashMap::from([("reason".to_string(), "test".to_string())]),
    }
}

fn jsonrpc_success(id: &serde_json::Value, result: &serde_json::Value) -> String {
    format!(
        r#"{{"jsonrpc":"2.0","id":{},"result":{}}}"#,
        id,
        serde_json::to_string(result).expect("serialize result")
    )
}

fn jsonrpc_error(id: &serde_json::Value, code: i64, message: &str) -> String {
    format!(
        r#"{{"jsonrpc":"2.0","id":{id},"error":{{"code":{code},"message":{message}}}}}"#,
        message = serde_json::to_string(message).expect("serialize message")
    )
}

fn non_utf8_socket_path(root: &Path) -> PathBuf {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    root.join(OsString::from_vec(vec![0x73, 0x6f, 0xFF, 0x63, 0x6b]))
}

fn parse_request(req: &str) -> (serde_json::Value, String) {
    let value: serde_json::Value = serde_json::from_str(req).expect("request json");
    let method = value
        .get("method")
        .and_then(|m| m.as_str())
        .unwrap_or("")
        .to_string();
    let id = value.get("id").cloned().unwrap_or_else(|| json!(1));
    (id, method)
}

async fn setup_ai_socket(
    handler: impl Fn(&str) -> String + Send + Sync + 'static,
) -> (tempfile::TempDir, PathBuf, MockJsonRpcServer) {
    let temp = tempfile::tempdir().expect("tempdir");
    let socket_dir = temp.path().to_path_buf();
    std::fs::create_dir_all(&socket_dir).expect("socket dir");
    let sock = socket_dir.join("mockai-default.sock");
    let server = MockJsonRpcServer::spawn(&sock, handler).await;
    (temp, socket_dir, server)
}

#[expect(
    clippy::future_not_send,
    reason = "temp_env::async_with_vars uses thread-local state"
)]
async fn run_with_socket_dir<F, Fut>(socket_dir: &Path, test: F)
where
    F: FnOnce() -> Fut + Send,
    Fut: std::future::Future<Output = ()> + Send,
{
    let socket_dir_str = socket_dir.to_str().expect("utf8 socket dir");
    temp_env::async_with_vars([("BIOMEOS_SOCKET_DIR", Some(socket_dir_str))], test()).await;
}

/// Bind a Unix socket that accepts one connection, reads the request, then hangs.
async fn bind_hanging_socket() -> (tempfile::TempDir, PathBuf) {
    use tokio::io::AsyncReadExt;

    let temp = tempfile::tempdir().expect("tempdir");
    let sock = temp.path().join("hanging.sock");
    let listener = tokio::net::UnixListener::bind(&sock).expect("bind hanging socket");
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 4096];
            let _ = stream.read(&mut buf).await;
            std::future::pending::<()>().await;
        }
    });
    (temp, sock)
}

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

#[tokio::test]
async fn ai_advisor_core_send_feedback_skipped_when_unavailable() {
    let advisor = AiGraphAdvisor::new();
    let feedback = SuggestionFeedback {
        suggestion_id: "s1".to_string(),
        accepted: false,
        comments: None,
        outcome: None,
    };
    assert!(advisor.send_feedback(feedback).await.is_ok());
}

#[tokio::test]
async fn ai_advisor_core_send_feedback_success_when_available() {
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_clone = Arc::clone(&calls);
    let (_temp, socket_dir, _server) = setup_ai_socket(move |req| {
        let (id, method) = parse_request(req);
        match method.as_str() {
            "capabilities.list" => jsonrpc_success(&id, &json!({"capabilities": ["ai"]})),
            "health.check" => jsonrpc_success(&id, &json!({"status": "healthy"})),
            "ai.record_feedback" => {
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

        let feedback = SuggestionFeedback {
            suggestion_id: "s1".to_string(),
            accepted: true,
            comments: Some("helpful".to_string()),
            outcome: Some(FeedbackOutcome {
                success: true,
                performance_delta: Some(0.2),
                satisfaction: Some(5),
            }),
        };
        assert!(advisor.send_feedback(feedback).await.is_ok());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    })
    .await;
}

#[tokio::test]
async fn ai_advisor_core_send_feedback_rpc_error_still_ok() {
    let (_temp, socket_dir, _server) = setup_ai_socket(|req| {
        let (id, method) = parse_request(req);
        match method.as_str() {
            "capabilities.list" => jsonrpc_success(&id, &json!({"capabilities": ["ai"]})),
            "health.check" => jsonrpc_success(&id, &json!({"status": "healthy"})),
            "ai.record_feedback" => jsonrpc_error(&id, -32603, "feedback failed"),
            other => panic!("unexpected method: {other}"),
        }
    })
    .await;

    run_with_socket_dir(&socket_dir, || async {
        let mut advisor = AiGraphAdvisor::new();
        assert!(advisor.check_squirrel_availability().await.unwrap());

        let feedback = SuggestionFeedback {
            suggestion_id: "s1".to_string(),
            accepted: false,
            comments: None,
            outcome: None,
        };
        assert!(advisor.send_feedback(feedback).await.is_ok());
    })
    .await;
}

#[tokio::test]
async fn ai_advisor_core_send_feedback_non_utf8_socket_errors() {
    let temp = tempfile::tempdir().expect("tempdir");
    let bad_sock = non_utf8_socket_path(temp.path());

    let mut advisor = AiGraphAdvisor::new();
    advisor.test_set_squirrel_state(true, Some(bad_sock));

    let feedback = SuggestionFeedback {
        suggestion_id: "s1".to_string(),
        accepted: true,
        comments: None,
        outcome: None,
    };
    let err = advisor.send_feedback(feedback).await.unwrap_err();
    assert!(
        err.to_string()
            .contains("AI socket path is not valid UTF-8"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn ai_advisor_core_send_feedback_errors_without_socket_when_marked_available() {
    let mut advisor = AiGraphAdvisor::new();
    advisor.test_set_squirrel_state(true, None);

    let feedback = SuggestionFeedback {
        suggestion_id: "s1".to_string(),
        accepted: true,
        comments: None,
        outcome: None,
    };
    let err = advisor.send_feedback(feedback).await.unwrap_err();
    assert!(
        err.to_string()
            .contains("AI provider socket not discovered"),
        "unexpected error: {err}"
    );
}

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

#[test]
fn ai_advisor_core_new_constructor_initial_state() {
    let advisor = AiGraphAdvisor::new();
    assert!(!advisor.squirrel_available);
    assert!(advisor.ai_socket_path.is_none());
    assert_eq!(advisor.squirrel_timeout, Duration::from_secs(5));
    assert_eq!(advisor.local_patterns.len(), 3);
}

#[test]
fn ai_advisor_core_advisor_with_timeout_constructor_stores_duration() {
    let timeout_secs = 42;
    let advisor = AiGraphAdvisor::with_timeout(Duration::from_secs(timeout_secs));
    assert!(!advisor.squirrel_available);
    assert_eq!(advisor.local_patterns.len(), 3);
    assert_eq!(advisor.squirrel_timeout, Duration::from_secs(timeout_secs));
}

#[test]
fn ai_advisor_core_advisor_default_matches_new() {
    let from_default = AiGraphAdvisor::default();
    let from_new = AiGraphAdvisor::new();
    assert_eq!(from_default.squirrel_available, from_new.squirrel_available);
    assert_eq!(
        from_default.local_patterns.len(),
        from_new.local_patterns.len()
    );
}
