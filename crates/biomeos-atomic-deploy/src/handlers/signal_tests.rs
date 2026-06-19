// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Unit tests for `handlers/signal.rs` (atomic signal dispatch layer).

#![expect(clippy::expect_used, reason = "test assertions")]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::json;
use tempfile::TempDir;
use tokio::sync::RwLock;

use crate::capability_translation::CapabilityTranslationRegistry;
use crate::handlers::graph::GraphHandler;
use crate::neural_router::NeuralRouter;

use super::signal::{
    SignalInfo, dispatch, is_signal_tier, list, list_signal_graphs, load_signal_schema, schema,
    signal_graph_path,
};

const EMPTY_ENV: [(&str, Option<&str>); 0] = [];

/// Minimal valid graph TOML for signal dispatch execute tests (log.info completes quickly).
const MINIMAL_SIGNAL_GRAPH_TOML: &str = r#"
[graph]
id = "test_minimal"
version = "1.0.0"
description = "Minimal graph for signal tests"

[[nodes]]
id = "log1"
[nodes.operation]
name = "log.info"
[nodes.config]
message = "signal test execution"
"#;

fn repo_graphs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../graphs")
}

fn make_graph_handler(graphs_dir: &Path) -> GraphHandler {
    GraphHandler::new(
        graphs_dir,
        "signal-test-family",
        Arc::new(RwLock::new(HashMap::new())),
        Arc::new(NeuralRouter::new("signal-test-family")),
        Arc::new(RwLock::new(CapabilityTranslationRegistry::new())),
    )
}

fn write_signal_graph(graphs_dir: &Path, tier: &str, signal: &str, toml: &str) {
    let signals_dir = graphs_dir.join("signals");
    std::fs::create_dir_all(&signals_dir).expect("signals dir");
    std::fs::write(signals_dir.join(format!("{tier}_{signal}.toml")), toml).expect("write graph");
}

struct SignalFixture {
    _temp: TempDir,
    graphs_dir: PathBuf,
    handler: GraphHandler,
}

impl SignalFixture {
    fn with_minimal_graph(tier: &str, signal: &str) -> Self {
        let temp = TempDir::new().expect("tempdir");
        let graphs_dir = temp.path().to_path_buf();
        write_signal_graph(&graphs_dir, tier, signal, MINIMAL_SIGNAL_GRAPH_TOML);
        let handler = make_graph_handler(&graphs_dir);
        Self {
            _temp: temp,
            graphs_dir,
            handler,
        }
    }
}

#[test]
fn is_signal_tier_matches_dispatch_table() {
    temp_env::with_vars(EMPTY_ENV, || {
        for tier in ["tower", "node", "nest", "meta", "braid"] {
            assert!(
                is_signal_tier(tier),
                "expected tier {tier} in dispatch table"
            );
        }
        assert!(!is_signal_tier("crypto"));
        assert!(!is_signal_tier("unknown"));
        assert!(!is_signal_tier(""));
    });
}

#[test]
fn signal_graph_path_maps_tier_and_signal_to_toml_file() {
    temp_env::with_vars(EMPTY_ENV, || {
        let graphs_dir = PathBuf::from("/tmp/graphs");
        assert_eq!(
            signal_graph_path(&graphs_dir, "tower", "publish"),
            graphs_dir.join("signals/tower_publish.toml")
        );
        assert_eq!(
            signal_graph_path(&graphs_dir, "nest", "store"),
            graphs_dir.join("signals/nest_store.toml")
        );
        assert_eq!(
            signal_graph_path(&graphs_dir, "braid", "partial_update"),
            graphs_dir.join("signals/braid_partial_update.toml")
        );
    });
}

#[test]
fn list_signal_graphs_builds_dispatch_table_from_directory() {
    temp_env::with_vars(EMPTY_ENV, || {
        let temp = TempDir::new().expect("tempdir");
        let graphs_dir = temp.path();
        write_signal_graph(graphs_dir, "tower", "publish", MINIMAL_SIGNAL_GRAPH_TOML);
        write_signal_graph(graphs_dir, "nest", "store", MINIMAL_SIGNAL_GRAPH_TOML);
        std::fs::write(
            graphs_dir.join("signals/not_a_tier_foo.toml"),
            MINIMAL_SIGNAL_GRAPH_TOML,
        )
        .expect("write ignored tier");
        std::fs::write(graphs_dir.join("signals/tower.txt"), "not toml").expect("write non-toml");

        let signals = list_signal_graphs(graphs_dir);
        assert_eq!(signals.len(), 2);
        assert_eq!(signals[0].name, "nest.store");
        assert_eq!(signals[1].name, "tower.publish");
        assert_eq!(signals[0].tier, "nest");
        assert_eq!(signals[0].signal, "store");
        assert_eq!(
            signals[1].graph_path,
            graphs_dir
                .join("signals/tower_publish.toml")
                .display()
                .to_string()
        );
    });
}

#[test]
fn list_signal_graphs_returns_empty_when_signals_dir_missing() {
    temp_env::with_vars(EMPTY_ENV, || {
        let temp = TempDir::new().expect("tempdir");
        let signals = list_signal_graphs(temp.path());
        assert!(signals.is_empty());
    });
}

#[tokio::test]
async fn signal_list_returns_tiers_and_sorted_signals() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let temp = TempDir::new().expect("tempdir");
        write_signal_graph(temp.path(), "meta", "observe", MINIMAL_SIGNAL_GRAPH_TOML);
        write_signal_graph(temp.path(), "tower", "health", MINIMAL_SIGNAL_GRAPH_TOML);

        let result = list(temp.path()).await.expect("signal.list");
        assert_eq!(result["count"], 2);
        let tiers = result["tiers"].as_array().expect("tiers");
        assert_eq!(tiers.len(), 5);
        let names: Vec<&str> = result["signals"]
            .as_array()
            .expect("signals")
            .iter()
            .map(|s| s["name"].as_str().expect("name"))
            .collect();
        assert_eq!(names, ["meta.observe", "tower.health"]);
    })
    .await;
}

#[test]
fn load_signal_schema_errors_when_config_missing() {
    temp_env::with_vars(EMPTY_ENV, || {
        let temp = TempDir::new().expect("tempdir");
        let err = load_signal_schema(temp.path()).expect_err("missing schema");
        assert!(
            err.to_string().contains("signal_tools.toml not found"),
            "unexpected error: {err}"
        );
    });
}

#[tokio::test]
async fn signal_schema_loads_from_repo_config() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let schema = schema(&repo_graphs_dir())
            .await
            .expect("signal.schema from repo graphs");
        assert!(schema.get("tools").is_some());
    })
    .await;
}

#[tokio::test]
async fn dispatch_rejects_missing_params() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let fixture = SignalFixture::with_minimal_graph("meta", "observe");
        let err = dispatch(
            &fixture.graphs_dir,
            "signal-test-family",
            &fixture.handler,
            &None,
        )
        .await
        .expect_err("missing params");
        assert!(err.to_string().contains("Missing parameters"));
    })
    .await;
}

#[tokio::test]
async fn dispatch_rejects_malformed_dotted_signal_name() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let fixture = SignalFixture::with_minimal_graph("meta", "observe");
        let params = Some(json!({ "signal": "not-a-tier-signal" }));
        let err = dispatch(
            &fixture.graphs_dir,
            "signal-test-family",
            &fixture.handler,
            &params,
        )
        .await
        .expect_err("malformed signal");
        assert!(
            err.to_string().contains("tier.name"),
            "unexpected error: {err}"
        );
    })
    .await;
}

#[tokio::test]
async fn dispatch_rejects_unknown_signal_tier() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let fixture = SignalFixture::with_minimal_graph("meta", "observe");
        let params = Some(json!({ "signal": "crypto.encrypt" }));
        let err = dispatch(
            &fixture.graphs_dir,
            "signal-test-family",
            &fixture.handler,
            &params,
        )
        .await
        .expect_err("unknown tier");
        assert!(
            err.to_string().contains("Unknown signal tier"),
            "unexpected error: {err}"
        );
    })
    .await;
}

#[tokio::test]
async fn dispatch_rejects_missing_signal_graph_file() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let temp = TempDir::new().expect("tempdir");
        let handler = make_graph_handler(temp.path());
        let params = Some(json!({ "signal": "tower.publish" }));
        let err = dispatch(temp.path(), "signal-test-family", &handler, &params)
            .await
            .expect_err("missing graph");
        assert!(
            err.to_string().contains("Signal graph not found"),
            "unexpected error: {err}"
        );
        assert!(
            err.to_string().contains("tower.publish"),
            "unexpected error: {err}"
        );
    })
    .await;
}

#[tokio::test]
async fn dispatch_accepts_tier_and_operation_fields() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let fixture = SignalFixture::with_minimal_graph("nest", "store");
        let params = Some(json!({
            "tier": "nest",
            "operation": "store",
            "params": { "key": "value" }
        }));
        let result = dispatch(
            &fixture.graphs_dir,
            "signal-test-family",
            &fixture.handler,
            &params,
        )
        .await
        .expect("dispatch tier+operation");

        assert_eq!(result["signal"], "nest.store");
        assert_eq!(result["graph_id"], "signals/nest_store");
        assert!(result["execution"].is_object());
    })
    .await;
}

#[tokio::test]
async fn dispatch_maps_dotted_signal_to_graph_execution() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let fixture = SignalFixture::with_minimal_graph("meta", "observe");
        let params = Some(json!({
            "signal": "meta.observe",
            "params": { "remote_gate": "tcp://westgate:9001" }
        }));
        let result = dispatch(
            &fixture.graphs_dir,
            "signal-test-family",
            &fixture.handler,
            &params,
        )
        .await
        .expect("dispatch dotted signal");

        assert_eq!(result["signal"], "meta.observe");
        assert_eq!(result["graph_id"], "signals/meta_observe");
        let execution = result["execution"].as_object().expect("execution object");
        assert!(execution.contains_key("execution_id"));
        assert_eq!(execution["graph_id"], "signals/meta_observe");
    })
    .await;
}

#[test]
fn signal_info_struct_holds_mapping_metadata() {
    temp_env::with_vars(EMPTY_ENV, || {
        let info = SignalInfo {
            name: "tower.publish".to_string(),
            tier: "tower".to_string(),
            signal: "publish".to_string(),
            graph_path: "/graphs/signals/tower_publish.toml".to_string(),
        };
        assert_eq!(info.name, "tower.publish");
        assert_eq!(info.tier, "tower");
        assert_eq!(info.signal, "publish");
    });
}
