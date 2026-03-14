// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Continuous mode — fixed-timestep graph execution via JSON-RPC IPC
//!
//! Loads a continuous coordination graph (e.g. `game_engine_tick.toml`) and runs
//! it in a real-time tick loop. Each node's capability is dispatched to the
//! appropriate primal via JSON-RPC over Unix domain sockets.
//!
//! Usage:
//!   biomeos continuous graphs/game_engine_tick.toml

use anyhow::{Context, Result};
use biomeos_graph::continuous::SessionCommand;
use biomeos_graph::{
    ContinuousExecutor, CoordinationPattern, GraphEventBroadcaster, GraphLoader, GraphNode,
};
use biomeos_types::paths::SystemPaths;
use biomeos_types::JsonRpcRequest;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, info, warn};

/// Resolve a primal's Unix socket path.
///
/// Priority (per wateringHole `UNIVERSAL_IPC_STANDARD_V3`):
/// 1. `BIOMEOS_SOCKET_DIR/{primal}.sock` (explicit override)
/// 2. XDG-compliant path via SystemPaths (runtime_dir + primal.sock)
fn resolve_primal_socket(primal: &str) -> PathBuf {
    if let Ok(dir) = std::env::var("BIOMEOS_SOCKET_DIR") {
        return PathBuf::from(dir).join(format!("{primal}.sock"));
    }

    SystemPaths::new_lazy().primal_socket(primal)
}

/// Send a JSON-RPC capability call to a primal over Unix socket.
async fn call_primal(
    socket_path: &PathBuf,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value> {
    let stream = UnixStream::connect(socket_path)
        .await
        .with_context(|| format!("Connect to {}", socket_path.display()))?;

    let (reader, mut writer) = stream.into_split();

    let request = JsonRpcRequest::new(method, params);

    let payload = format!("{}\n", serde_json::to_string(&request)?);
    writer.write_all(payload.as_bytes()).await?;
    writer.shutdown().await?;

    let mut buf_reader = BufReader::new(reader);
    let mut response_line = String::new();

    tokio::time::timeout(
        std::time::Duration::from_secs(5),
        buf_reader.read_line(&mut response_line),
    )
    .await
    .map_err(|_| anyhow::anyhow!("Timeout waiting for {method} response"))?
    .context("Read primal response")?;

    let response: serde_json::Value =
        serde_json::from_str(&response_line).context("Parse JSON-RPC response")?;

    if let Some(result) = response.get("result") {
        Ok(result.clone())
    } else if let Some(err) = response.get("error") {
        anyhow::bail!("RPC error from {method}: {err}")
    } else {
        anyhow::bail!("Malformed response from {method}")
    }
}

/// Build JSON-RPC params from a node's configuration and optional feedback.
fn build_call_params(node: &GraphNode, feedback: Option<serde_json::Value>) -> serde_json::Value {
    let mut params = node.params.to_json();
    if let Some(fb) = feedback {
        if let Some(obj) = params.as_object_mut() {
            obj.insert("_feedback".to_string(), fb);
        }
    }
    params
}

/// Execute a single node by routing its capability to the target primal.
async fn execute_node(
    node_id: String,
    node: GraphNode,
    feedback: Option<serde_json::Value>,
) -> Result<serde_json::Value> {
    let primal = node.config.primal.as_deref().unwrap_or("unknown");
    let capability = node.capability.as_deref().unwrap_or("health.check");

    let socket = resolve_primal_socket(primal);
    let params = build_call_params(&node, feedback);

    match call_primal(&socket, capability, params).await {
        Ok(result) => Ok(result),
        Err(e) => {
            debug!(
                "Node '{}' capability '{}' @ '{}' error: {}",
                node_id,
                capability,
                socket.display(),
                e
            );
            Err(e)
        }
    }
}

/// Run a continuous coordination graph.
///
/// Loads the graph, validates it uses `Continuous` coordination, and starts
/// the fixed-timestep tick loop. Each tick dispatches node capabilities to
/// primals over JSON-RPC IPC.
pub async fn run(graph_path: PathBuf, dry_run: bool) -> Result<()> {
    info!("biomeOS Continuous Mode");
    info!("  graph: {}", graph_path.display());

    if !graph_path.exists() {
        anyhow::bail!("Graph file not found: {}", graph_path.display());
    }

    let graph = GraphLoader::from_file(&graph_path)
        .with_context(|| format!("Failed to load graph: {}", graph_path.display()))?;

    if graph.definition.coordination != CoordinationPattern::Continuous {
        anyhow::bail!(
            "Graph '{}' uses {:?} coordination, expected Continuous",
            graph.id(),
            graph.definition.coordination
        );
    }

    let tick_hz = graph
        .definition
        .tick
        .as_ref()
        .map(|t| t.target_hz)
        .unwrap_or(60.0);

    info!(
        "  id:    {} ({} nodes @ {tick_hz} Hz)",
        graph.id(),
        graph.nodes().len(),
    );

    if dry_run {
        info!("  [dry run] Would execute continuous loop:");
        for node in graph.nodes_in_order() {
            let cap = node.capability.as_deref().unwrap_or("(none)");
            let primal = node.config.primal.as_deref().unwrap_or("(auto)");
            let budget = node
                .budget_ms
                .map(|b| format!("{b}ms"))
                .unwrap_or_else(|| "-".to_string());
            let fb = node
                .feedback_to
                .as_deref()
                .map(|t| format!(" -> feedback to {t}"))
                .unwrap_or_default();
            info!("    {} : {cap} @ {primal} [budget {budget}]{fb}", node.id);
        }

        let feedback_count = graph
            .nodes()
            .iter()
            .filter(|n| n.feedback_to.is_some())
            .count();
        info!("  feedback edges: {feedback_count}");
        return Ok(());
    }

    // Verify primal sockets exist before starting the hot loop
    let mut missing = Vec::new();
    for node in graph.nodes() {
        if let Some(primal) = node.config.primal.as_deref() {
            let socket = resolve_primal_socket(primal);
            if !socket.exists() {
                missing.push((primal.to_string(), socket));
            }
        }
    }
    if !missing.is_empty() {
        for (primal, path) in &missing {
            warn!("Primal '{}' socket not found: {}", primal, path.display());
        }
        warn!(
            "Missing {} primal socket(s) — nodes will fail until primals start",
            missing.len()
        );
    }

    let broadcaster = GraphEventBroadcaster::new(1024);
    let mut executor = ContinuousExecutor::new(graph, broadcaster);

    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel::<SessionCommand>(16);

    let cmd_tx_signal = cmd_tx.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        info!("Shutdown signal received");
        let _ = cmd_tx_signal.send(SessionCommand::Stop).await;
    });

    info!("Starting continuous execution...");
    executor.run(cmd_rx, execute_node).await;
    info!("Continuous session ended.");

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_primal_socket_default() {
        // Clear env vars that would override
        std::env::remove_var("BIOMEOS_SOCKET_DIR");

        let path = resolve_primal_socket("ludospring");
        assert!(path.is_absolute());
        assert_eq!(path.file_name().unwrap(), "ludospring.sock");
    }

    #[test]
    fn test_resolve_primal_socket_biomeos_dir() {
        std::env::set_var("BIOMEOS_SOCKET_DIR", "/run/biomeos");
        let path = resolve_primal_socket("petaltongue");
        std::env::remove_var("BIOMEOS_SOCKET_DIR");
        assert_eq!(path, PathBuf::from("/run/biomeos/petaltongue.sock"));
    }

    #[test]
    fn test_build_call_params_no_feedback() {
        let node: GraphNode = toml::from_str(
            r#"
            id = "test"
            name = "Test"
            [params]
            dt = "fixed"
        "#,
        )
        .unwrap();

        let params = build_call_params(&node, None);
        assert_eq!(params.get("dt").and_then(|v| v.as_str()), Some("fixed"));
        assert!(params.get("_feedback").is_none());
    }

    #[test]
    fn test_build_call_params_with_feedback() {
        let node: GraphNode = toml::from_str(
            r#"
            id = "test"
            name = "Test"
            [params]
            dt = "fixed"
        "#,
        )
        .unwrap();

        let feedback = serde_json::json!({"collision": true});
        let params = build_call_params(&node, Some(feedback));
        assert_eq!(params.get("dt").and_then(|v| v.as_str()), Some("fixed"));
        assert_eq!(
            params
                .get("_feedback")
                .and_then(|v| v.get("collision"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[tokio::test]
    async fn test_run_missing_graph() {
        let result = run(PathBuf::from("/nonexistent/graph.toml"), false).await;
        let err = result.expect_err("missing graph should fail");
        assert!(err.to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_run_non_continuous_graph() {
        let dir = tempfile::tempdir().expect("temp dir");
        let graph_path = dir.path().join("seq.toml");
        std::fs::write(
            &graph_path,
            r#"
            [graph]
            id = "seq-graph"
            name = "Sequential"
            version = "1.0.0"
            coordination = "sequential"
        "#,
        )
        .expect("write graph");

        let result = run(graph_path, false).await;
        let err = result.expect_err("sequential graph should fail");
        assert!(err.to_string().contains("Continuous"), "error: {}", err);
    }

    #[tokio::test]
    async fn test_run_dry_run() {
        let dir = tempfile::tempdir().expect("temp dir");
        let graph_path = dir.path().join("game.toml");
        std::fs::write(
            &graph_path,
            r#"
            [graph]
            id = "dry-run-test"
            name = "Dry Run"
            version = "1.0.0"
            coordination = "continuous"

            [graph.tick]
            target_hz = 60.0

            [[graph.nodes]]
            id = "input"
            name = "Input"
            capability = "interaction.poll_sensors"
            budget_ms = 1.0

            [graph.nodes.config]
            primal = "petaltongue"

            [[graph.nodes]]
            id = "logic"
            name = "Logic"
            capability = "game.tick_logic"
            depends_on = ["input"]
            budget_ms = 4.0

            [graph.nodes.config]
            primal = "ludospring"

            [[graph.nodes]]
            id = "physics"
            name = "Physics"
            capability = "game.tick_physics"
            depends_on = ["logic"]
            feedback_to = "logic"
            budget_ms = 4.0

            [graph.nodes.config]
            primal = "ludospring"
        "#,
        )
        .expect("write graph");

        let result = run(graph_path, true).await;
        result.expect("dry run should succeed");
    }

    #[tokio::test]
    async fn test_run_invalid_toml() {
        let dir = tempfile::tempdir().expect("temp dir");
        let graph_path = dir.path().join("bad.toml");
        std::fs::write(&graph_path, "not valid toml {{{").expect("write");

        let result = run(graph_path, false).await;
        let err = result.expect_err("invalid toml should fail");
        assert!(
            err.to_string().contains("parse") || err.to_string().contains("Failed"),
            "error: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_call_primal_nonexistent_socket() {
        let path = PathBuf::from("/tmp/nonexistent-primal-test-xyz.sock");
        let result = call_primal(&path, "health.check", serde_json::json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_call_primal_roundtrip() {
        let dir = tempfile::tempdir().expect("temp dir");
        let sock = dir.path().join("test-primal.sock");
        let sock_clone = sock.clone();

        let server = tokio::spawn(async move {
            let listener = tokio::net::UnixListener::bind(&sock_clone).expect("bind");
            if let Ok((stream, _)) = listener.accept().await {
                let (reader, mut writer) = stream.into_split();
                let mut br = BufReader::new(reader);
                let mut line = String::new();
                br.read_line(&mut line).await.ok();
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "result": {"status": "healthy"},
                    "id": 1
                });
                writer
                    .write_all(format!("{response}\n").as_bytes())
                    .await
                    .ok();
            }
        });

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let result = call_primal(&sock, "health.check", serde_json::json!({})).await;
        assert!(result.is_ok(), "call_primal failed: {:?}", result.err());
        let val = result.unwrap();
        assert_eq!(val.get("status").and_then(|v| v.as_str()), Some("healthy"));

        server.abort();
    }

    #[tokio::test]
    async fn test_execute_node_no_socket() {
        let node: GraphNode = toml::from_str(
            r#"
            id = "test"
            name = "Test"
            capability = "game.tick_logic"
            [config]
            primal = "nonexistent-primal-xyz"
        "#,
        )
        .unwrap();

        let result = execute_node("test".to_string(), node, None).await;
        assert!(result.is_err());
    }
}
