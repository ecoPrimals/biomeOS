// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

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
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Resolve a primal's Unix socket path.
///
/// Priority (per wateringHole `UNIVERSAL_IPC_STANDARD_V3`):
/// 1. `socket_dir` when provided (e.g. [`run_controlled_with_socket_dir`])
/// 2. `BIOMEOS_SOCKET_DIR/{primal}.sock` (explicit override)
/// 3. XDG-compliant path via SystemPaths (runtime_dir + primal.sock)
fn resolve_primal_socket_with(primal: &str, socket_dir: Option<String>) -> PathBuf {
    if let Some(dir) = socket_dir {
        return PathBuf::from(dir).join(format!("{primal}.sock"));
    }
    SystemPaths::new_lazy().primal_socket(primal)
}

/// Effective socket directory: explicit override wins; otherwise `BIOMEOS_SOCKET_DIR`, then XDG.
fn effective_socket_dir(socket_dir_override: Option<&String>) -> Option<String> {
    socket_dir_override
        .cloned()
        .or_else(|| std::env::var(biomeos_types::env_config::vars::SOCKET_DIR).ok())
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
    .with_context(|| format!("Timeout waiting for {method} response"))?
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
///
/// `socket_dir_override`: if `Some`, resolve sockets under that directory only; if `None`, use
/// `BIOMEOS_SOCKET_DIR` / XDG.
async fn execute_node_with_socket_dir(
    node_id: String,
    node: GraphNode,
    feedback: Option<serde_json::Value>,
    socket_dir_override: Option<String>,
) -> Result<serde_json::Value> {
    let primal = node.config.primal.as_deref().unwrap_or("unknown");
    let capability = node.capability.as_deref().unwrap_or("health.check");

    let socket =
        resolve_primal_socket_with(primal, effective_socket_dir(socket_dir_override.as_ref()));
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

#[cfg(test)]
async fn execute_node(
    node_id: String,
    node: GraphNode,
    feedback: Option<serde_json::Value>,
) -> Result<serde_json::Value> {
    execute_node_with_socket_dir(node_id, node, feedback, None).await
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
    run_controlled_with_socket_dir(graph_path, dry_run, cmd_rx, None).await
}

/// Like [`run_controlled`], but resolve primal sockets under `socket_dir_override` (tests) instead
/// of relying on `BIOMEOS_SOCKET_DIR`.
pub(crate) async fn run_controlled_with_socket_dir(
    graph_path: PathBuf,
    dry_run: bool,
    cmd_rx: mpsc::Receiver<SessionCommand>,
    socket_dir_override: Option<String>,
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
            let socket = resolve_primal_socket_with(
                primal,
                effective_socket_dir(socket_dir_override.as_ref()),
            );
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

    let socket_override = Arc::new(socket_dir_override);
    info!("Starting continuous execution...");
    executor
        .run(cmd_rx, {
            let socket_override = Arc::clone(&socket_override);
            move |node_id, node, feedback| {
                let socket_override = Arc::clone(&socket_override);
                async move {
                    execute_node_with_socket_dir(
                        node_id,
                        node,
                        feedback,
                        (*socket_override).clone(),
                    )
                    .await
                }
            }
        })
        .await;
    info!("Continuous session ended.");

    Ok(())
}

#[cfg(test)]
#[path = "continuous_tests.rs"]
mod tests;
