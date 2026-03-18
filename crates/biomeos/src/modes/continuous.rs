// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

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
use biomeos_types::JsonRpcRequest;
use biomeos_types::paths::SystemPaths;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Resolve a primal's Unix socket path.
///
/// Priority (per wateringHole `UNIVERSAL_IPC_STANDARD_V3`):
/// 1. `BIOMEOS_SOCKET_DIR/{primal}.sock` (explicit override)
/// 2. XDG-compliant path via SystemPaths (runtime_dir + primal.sock)
fn resolve_primal_socket(primal: &str) -> PathBuf {
    resolve_primal_socket_with(primal, std::env::var("BIOMEOS_SOCKET_DIR").ok())
}

fn resolve_primal_socket_with(primal: &str, socket_dir: Option<String>) -> PathBuf {
    if let Some(dir) = socket_dir {
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

    let (cmd_tx, cmd_rx) = mpsc::channel::<SessionCommand>(16);
    let cmd_tx_signal = cmd_tx.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        info!("Shutdown signal received");
        let _ = cmd_tx_signal.send(SessionCommand::Stop).await;
    });

    run_controlled(graph_path, dry_run, cmd_rx).await
}

/// Run a continuous graph with caller-controlled command channel.
///
/// Used by tests to control when the session stops (e.g. after N ticks).
/// Production code uses `run()` which wires ctrl_c to this.
pub(crate) async fn run_controlled(
    graph_path: PathBuf,
    dry_run: bool,
    cmd_rx: mpsc::Receiver<SessionCommand>,
) -> Result<()> {
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

    let tick_hz = graph.definition.tick.as_ref().map_or(60.0, |t| t.target_hz);

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
                .map_or_else(|| "-".to_string(), |b| format!("{b}ms"));
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

    info!("Starting continuous execution...");
    executor.run(cmd_rx, execute_node).await;
    info!("Continuous session ended.");

    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use biomeos_test_utils::TestEnvGuard;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[test]
    fn test_resolve_primal_socket_default() {
        let path = resolve_primal_socket_with("ludospring", None);
        assert!(path.is_absolute());
        assert_eq!(path.file_name().unwrap(), "ludospring.sock");
    }

    #[test]
    fn test_resolve_primal_socket_biomeos_dir() {
        let path = resolve_primal_socket_with("petaltongue", Some("/run/biomeos".to_string()));
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
        assert!(err.to_string().contains("Continuous"), "error: {err}");
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
            capability = "interaction.poll"
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
            "error: {err}"
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

        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();

        let server = tokio::spawn(async move {
            let listener = tokio::net::UnixListener::bind(&sock_clone).expect("bind");
            let _ = ready_tx.send(());
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

        ready_rx.await.expect("server ready");

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

    /// Live socket validation: Create a temp Unix socket, spawn a mock JSON-RPC server,
    /// run the ContinuousExecutor for a few ticks, verify it sends requests to the socket.
    #[tokio::test]
    async fn test_continuous_executor_with_mock_socket() {
        let dir = tempfile::tempdir().expect("temp dir");
        let sock_path = dir.path().join("mockprimal.sock");
        let received: Arc<Mutex<Vec<serde_json::Value>>> = Arc::new(Mutex::new(Vec::new()));
        let received_clone = Arc::clone(&received);
        let first_request = Arc::new(tokio::sync::Notify::new());
        let first_request_clone = Arc::clone(&first_request);

        let server = tokio::spawn(async move {
            let listener = tokio::net::UnixListener::bind(&sock_path).expect("bind");
            for _ in 0..20 {
                if let Ok(Ok((stream, _))) =
                    tokio::time::timeout(std::time::Duration::from_secs(2), listener.accept()).await
                {
                    let recv = Arc::clone(&received_clone);
                    let first_req = Arc::clone(&first_request_clone);
                    tokio::spawn(async move {
                        let (reader, mut writer) = stream.into_split();
                        let mut br = BufReader::new(reader);
                        let mut line = String::new();
                        let _ = br.read_line(&mut line).await;
                        if let Ok(req) = serde_json::from_str::<serde_json::Value>(&line) {
                            recv.lock().await.push(req);
                            first_req.notify_waiters();
                        }
                        let response = serde_json::json!({
                            "jsonrpc": "2.0",
                            "result": {"ok": true},
                            "id": 1
                        });
                        let _ = writer.write_all(format!("{response}\n").as_bytes()).await;
                    });
                }
            }
        });

        let graph_path = dir.path().join("graph.toml");
        std::fs::write(
            &graph_path,
            r#"
            [graph]
            id = "mock-socket-test"
            name = "Mock Socket Test"
            version = "1.0.0"
            coordination = "continuous"

            [graph.tick]
            target_hz = 20.0

            [[graph.nodes]]
            id = "node1"
            name = "Node 1"
            capability = "game.tick_logic"
            budget_ms = 1.0

            [graph.nodes.config]
            primal = "mockprimal"
        "#,
        )
        .expect("write graph");

        let _env = TestEnvGuard::new("BIOMEOS_SOCKET_DIR", dir.path().to_str());

        let (cmd_tx, cmd_rx) = mpsc::channel::<SessionCommand>(16);
        let cmd_tx_stop = cmd_tx.clone();
        let first_request_waiter = Arc::clone(&first_request);
        tokio::spawn(async move {
            first_request_waiter.notified().await;
            let _ = cmd_tx_stop.send(SessionCommand::Stop).await;
        });

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            run_controlled(graph_path, false, cmd_rx),
        )
        .await;

        server.abort();

        let res = result.expect("run_controlled should complete within timeout");
        res.expect("run_controlled should succeed");

        let reqs = received.lock().await;
        assert!(
            !reqs.is_empty(),
            "Executor should have sent at least one JSON-RPC request to the mock socket"
        );
        assert!(
            reqs.iter()
                .any(|r| r.get("method").and_then(|m| m.as_str()) == Some("game.tick_logic")),
            "Expected game.tick_logic method in received requests"
        );
    }

    /// Run the executor with a non-existent socket path; verify it doesn't panic, just logs warnings.
    #[tokio::test]
    async fn test_continuous_executor_graceful_degradation() {
        let dir = tempfile::tempdir().expect("temp dir");
        let graph_path = dir.path().join("graph.toml");
        std::fs::write(
            &graph_path,
            r#"
            [graph]
            id = "graceful-test"
            name = "Graceful Degradation"
            version = "1.0.0"
            coordination = "continuous"

            [graph.tick]
            target_hz = 10.0

            [[graph.nodes]]
            id = "node1"
            name = "Node 1"
            capability = "health.check"
            budget_ms = 1.0

            [graph.nodes.config]
            primal = "nonexistent-primal-xyz-12345"
        "#,
        )
        .expect("write graph");

        let _env = TestEnvGuard::new("BIOMEOS_SOCKET_DIR", dir.path().to_str());

        let (cmd_tx, cmd_rx) = mpsc::channel::<SessionCommand>(16);
        let cmd_tx_stop = cmd_tx.clone();
        tokio::spawn(async move {
            // Intentional: allow executor to run a few ticks before stop (no event to wait for)
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            let _ = cmd_tx_stop.send(SessionCommand::Stop).await;
        });

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            run_controlled(graph_path, false, cmd_rx),
        )
        .await;

        let res = result.expect("run_controlled should complete within timeout");
        res.expect("run_controlled should succeed (graceful degradation)");
    }

    /// Run at 10 Hz for ~500ms; verify we get approximately 4–6 ticks.
    /// This test lives in biomeos-graph (see continuous.rs there).
    #[tokio::test]
    async fn test_continuous_executor_tick_timing() {
        let toml_str = r#"
            [graph]
            id = "tick-timing-test"
            name = "Tick Timing"
            version = "1.0.0"
            coordination = "continuous"

            [graph.tick]
            target_hz = 10.0

            [[graph.nodes]]
            id = "node1"
            name = "Node 1"
        "#;
        let graph: biomeos_graph::DeploymentGraph = toml::from_str(toml_str).unwrap();
        let broadcaster = GraphEventBroadcaster::new(1024);
        let mut receiver = broadcaster.subscribe();
        let mut executor = biomeos_graph::ContinuousExecutor::new(graph, broadcaster);

        let (cmd_tx, cmd_rx) = mpsc::channel::<SessionCommand>(16);
        let cmd_tx_stop = cmd_tx.clone();
        tokio::spawn(async move {
            // Intentional: testing tick timing over 500ms at 10 Hz (~5 ticks)
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let _ = cmd_tx_stop.send(SessionCommand::Stop).await;
        });

        let handle = tokio::spawn(async move {
            executor
                .run(cmd_rx, |_node_id, _node, _feedback| async {
                    Ok(serde_json::json!({"ok": true}))
                })
                .await;
        });

        let mut tick_count = 0u64;
        loop {
            let event =
                tokio::time::timeout(std::time::Duration::from_secs(2), receiver.recv()).await;
            match event {
                Ok(Ok(biomeos_graph::GraphEvent::TickCompleted { tick, .. })) => {
                    tick_count = tick;
                }
                Ok(Ok(biomeos_graph::GraphEvent::SessionStateChanged { new_state, .. }))
                    if new_state == "stopped" =>
                {
                    break;
                }
                Ok(Err(_)) | Err(_) => break,
                _ => {}
            }
        }

        handle.await.expect("executor task");

        assert!(
            (4..=8).contains(&tick_count),
            "Expected ~5 ticks at 10 Hz over 500ms, got {}",
            tick_count
        );
    }
}
