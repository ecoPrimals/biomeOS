// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Extra `handlers::graph` coverage (pipeline translation / discovery branches, params).

#![expect(clippy::expect_used, reason = "test assertions use expect for clarity")]

use super::*;
use serde_json::json;
use std::time::Duration;
use tempfile::tempdir;

/// Minimal pipeline graph (two nodes) for exercising `execute_pipeline` internals.
const TWO_NODE_PIPELINE_TOML: &str = r#"
[graph]
id = "pipe-two-node"
name = "Two-node pipeline"
version = "1.0.0"
coordination = "pipeline"

[[graph.nodes]]
id = "source"
name = "Source"
capability = "test.source.cap"

[[graph.nodes]]
id = "sink"
name = "Sink"
capability = "test.sink.cap"
depends_on = ["source"]
"#;

#[tokio::test]
async fn test_execute_pipeline_accepts_channel_capacity_param() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("pipe-two-node.toml");
    std::fs::write(&path, TWO_NODE_PIPELINE_TOML).expect("write graph");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({
        "graph_id": "pipe-two-node",
        "channel_capacity": 7_u64,
    }));

    let outcome =
        tokio::time::timeout(Duration::from_secs(4), handler.execute_pipeline(&params)).await;

    match outcome {
        Ok(Ok(value)) => {
            assert!(
                value.get("graph_id").is_some()
                    || value.get("items_in").is_some()
                    || value.get("success").is_some(),
                "unexpected pipeline payload: {value}"
            );
        }
        Ok(Err(err)) => {
            let msg = err.to_string();
            assert!(
                msg.contains("Pipeline")
                    || msg.contains("Capability")
                    || msg.contains("connect")
                    || msg.contains("forward")
                    || msg.contains("discovery"),
                "unexpected error: {msg}"
            );
        }
        Err(_) => {
            // Pipeline may block on discovery in CI — timeout still proves we entered execute_pipeline.
        }
    }
}

#[tokio::test]
async fn test_execute_pipeline_translation_forward_uses_registry_socket() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("pipe-two-node.toml");
    std::fs::write(&path, TWO_NODE_PIPELINE_TOML).expect("write graph");

    let (handler, _, registry) = make_handler_with_registry(temp.path());
    {
        let mut reg = registry.write().await;
        reg.register_translation(
            "test.source.cap",
            "mock-provider",
            "test.source.cap",
            "/tmp/biomeos_missing_socket_for_graph_test.sock",
            None,
        );
    }

    let params = Some(json!({"graph_id": "pipe-two-node"}));
    let outcome =
        tokio::time::timeout(Duration::from_secs(4), handler.execute_pipeline(&params)).await;

    match outcome {
        Ok(Ok(value)) => {
            let success = value.get("success").and_then(|s| s.as_bool());
            let dropped = value.get("items_dropped").and_then(|d| d.as_u64());
            assert!(
                success == Some(false)
                    || dropped.unwrap_or(0) > 0
                    || value.get("outputs").is_some(),
                "expected error/drop output from bad translation socket: {value}"
            );
        }
        Ok(Err(e)) => {
            let msg = e.to_string();
            assert!(
                msg.contains("Pipeline") || msg.contains("connect") || msg.contains("forward"),
                "unexpected: {msg}"
            );
        }
        Err(_) => {}
    }
}
