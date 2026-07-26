// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::path::{Path, PathBuf};

use biomeos_test_utils::MockJsonRpcServer;
use serde_json::json;

use super::super::*;
use crate::graph::{
    CoordinationPattern, GraphId, Operation, PrimalGraph, PrimalNode, PrimalSelector,
};
use crate::modification::GraphModification;

pub(super) fn create_test_graph() -> PrimalGraph {
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

pub(super) fn sample_suggestion() -> AiSuggestion {
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

pub(super) fn sample_suggestion_json() -> serde_json::Value {
    serde_json::to_value(sample_suggestion()).expect("serialize suggestion")
}

pub(super) fn sample_learning_event(graph: &PrimalGraph) -> LearningEvent {
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

pub(super) fn jsonrpc_success(id: &serde_json::Value, result: &serde_json::Value) -> String {
    format!(
        r#"{{"jsonrpc":"2.0","id":{},"result":{}}}"#,
        id,
        serde_json::to_string(result).expect("serialize result")
    )
}

pub(super) fn jsonrpc_error(id: &serde_json::Value, code: i64, message: &str) -> String {
    format!(
        r#"{{"jsonrpc":"2.0","id":{id},"error":{{"code":{code},"message":{message}}}}}"#,
        message = serde_json::to_string(message).expect("serialize message")
    )
}

pub(super) fn non_utf8_socket_path(root: &Path) -> PathBuf {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    root.join(OsString::from_vec(vec![0x73, 0x6f, 0xFF, 0x63, 0x6b]))
}

pub(super) fn parse_request(req: &str) -> (serde_json::Value, String) {
    let value: serde_json::Value = serde_json::from_str(req).expect("request json");
    let method = value
        .get("method")
        .and_then(|m| m.as_str())
        .unwrap_or("")
        .to_string();
    let id = value.get("id").cloned().unwrap_or_else(|| json!(1));
    (id, method)
}

pub(super) async fn setup_ai_socket(
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
pub(super) async fn run_with_socket_dir<F, Fut>(socket_dir: &Path, test: F)
where
    F: FnOnce() -> Fut + Send,
    Fut: std::future::Future<Output = ()> + Send,
{
    let socket_dir_str = socket_dir.to_str().expect("utf8 socket dir");
    temp_env::async_with_vars([("BIOMEOS_SOCKET_DIR", Some(socket_dir_str))], test()).await;
}

/// Bind a Unix socket that accepts one connection, reads the request, then hangs.
pub(super) async fn bind_hanging_socket() -> (tempfile::TempDir, PathBuf) {
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
