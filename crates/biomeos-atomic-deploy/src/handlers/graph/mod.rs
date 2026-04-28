// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Graph CRUD, execution, and session management.
//!
//! This module handles all graph-related JSON-RPC methods:
//! - `graph.list` / `graph.get` / `graph.save` — CRUD (this file)
//! - `graph.execute` — Sequential execution (`execute` submodule)
//! - `graph.execute_pipeline` — Pipeline/streaming execution (`pipeline` submodule)
//! - `graph.start_continuous` / pause / resume / stop — Continuous sessions (`continuous` submodule)
//! - `graph.status` — Execution status (this file)
//! - `graph.suggest_optimizations` — PathwayLearner analysis (this file)
//!
//! # Capability-Based Design
//!
//! Graph execution uses capability-based primal discovery instead of hardcoded names.

mod continuous;
mod execute;
mod pipeline;

use crate::capability_translation::CapabilityTranslationRegistry;
use crate::neural_graph::Graph;
use crate::neural_router::NeuralRouter;
use anyhow::{Context, Result};
use biomeos_graph::continuous::SessionState;
use biomeos_graph::events::GraphEventBroadcaster;
use biomeos_graph::graph::DeploymentGraph;
use biomeos_types::{SystemPaths, constants::files, defaults::DEFAULT_SOCKET_DIR};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Execution status tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStatus {
    /// Unique execution ID
    pub execution_id: String,
    /// Current state (running, completed, failed)
    pub state: String,
    /// Current phase index (if phased execution)
    pub current_phase: Option<usize>,
    /// Total number of phases
    pub total_phases: usize,
    /// Node IDs that completed successfully
    pub completed_nodes: Vec<String>,
    /// Node IDs that failed
    pub failed_nodes: Vec<String>,
    /// Elapsed time in milliseconds
    pub duration_ms: u64,
    /// Error message if failed
    pub error: Option<String>,
}

/// Tracks an active continuous execution session.
pub(super) struct ContinuousSession {
    pub(super) graph_id: String,
    pub(super) command_tx: tokio::sync::mpsc::Sender<biomeos_graph::continuous::SessionCommand>,
    pub(super) state_rx: tokio::sync::watch::Receiver<SessionState>,
    pub(super) started_at: String,
}

/// Graph handler for CRUD and execution operations.
///
/// Graphs are split into two tiers:
/// - **Nucleus graphs** (`graphs_dir`): biomeOS's own bootstrap/health/routing
///   graphs. Bundled with the binary and loaded at build time. Not writable via API.
/// - **Runtime graphs** (`runtime_graphs_dir`): Consumer compositions deployed via
///   `graph.save`. Stored under `$XDG_DATA_HOME/biomeos/graphs/` or a sibling
///   `runtime_graphs/` directory.
#[derive(Clone)]
pub struct GraphHandler {
    /// Nucleus graphs directory (read-only, bundled with binary)
    pub(super) graphs_dir: PathBuf,
    /// Runtime graphs directory (writable via `graph.save`)
    pub(super) runtime_graphs_dir: PathBuf,
    /// Active executions (transactional)
    pub(super) executions: Arc<RwLock<HashMap<String, ExecutionStatus>>>,
    /// Active continuous sessions (keyed by `session_id`)
    pub(super) continuous_sessions: Arc<RwLock<HashMap<String, ContinuousSession>>>,
    /// Family ID
    pub(super) family_id: String,
    /// Neural Router for capability discovery
    pub(super) router: Arc<NeuralRouter>,
    /// Capability Translation Registry
    pub(super) translation_registry: Arc<RwLock<CapabilityTranslationRegistry>>,
    /// Override neural metrics DB path (e.g. tests); when `None`, use XDG data dir.
    pub(super) metrics_db_path: Option<PathBuf>,
    /// Shared event broadcaster for continuous tick events.
    /// When set, continuous sessions relay `TickCompleted` / session lifecycle events
    /// to this broadcaster so WebSocket/SSE subscribers can observe the tick loop.
    pub(super) event_broadcaster: Option<Arc<GraphEventBroadcaster>>,
}

/// Recursively collect all `.toml` files under `dir`, including subdirectories.
fn collect_toml_files_recursive(dir: &std::path::Path, files: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_toml_files_recursive(&path, files);
        } else if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            files.push(path);
        }
    }
}

impl GraphHandler {
    /// Create a new graph handler.
    ///
    /// `graphs_dir` is the nucleus graphs directory (bundled / read-only).
    /// Runtime graphs are stored in a `runtime_graphs` sibling directory,
    /// created on first `graph.save` if it does not exist.
    pub fn new(
        graphs_dir: impl Into<PathBuf>,
        family_id: impl Into<String>,
        executions: Arc<RwLock<HashMap<String, ExecutionStatus>>>,
        router: Arc<NeuralRouter>,
        translation_registry: Arc<RwLock<CapabilityTranslationRegistry>>,
    ) -> Self {
        Self::new_with_metrics_db(
            graphs_dir,
            family_id,
            executions,
            router,
            translation_registry,
            None,
        )
    }

    /// Same as [`Self::new`], with an explicit metrics database path for isolated analysis/tests.
    pub fn new_with_metrics_db(
        graphs_dir: impl Into<PathBuf>,
        family_id: impl Into<String>,
        executions: Arc<RwLock<HashMap<String, ExecutionStatus>>>,
        router: Arc<NeuralRouter>,
        translation_registry: Arc<RwLock<CapabilityTranslationRegistry>>,
        metrics_db_path: Option<PathBuf>,
    ) -> Self {
        let graphs_dir: PathBuf = graphs_dir.into();
        let graphs_dir = if graphs_dir.is_relative() {
            std::env::current_dir()
                .map(|cwd| cwd.join(&graphs_dir))
                .unwrap_or(graphs_dir)
        } else {
            graphs_dir
        };
        let runtime_graphs_dir = graphs_dir.parent().map_or_else(
            || graphs_dir.join("runtime_graphs"),
            |parent| parent.join("runtime_graphs"),
        );
        Self {
            graphs_dir,
            runtime_graphs_dir,
            family_id: family_id.into(),
            executions,
            continuous_sessions: Arc::new(RwLock::new(HashMap::new())),
            router,
            translation_registry,
            metrics_db_path,
            event_broadcaster: None,
        }
    }

    /// Set a shared event broadcaster for continuous session tick events.
    pub fn with_event_broadcaster(mut self, broadcaster: Arc<GraphEventBroadcaster>) -> Self {
        self.event_broadcaster = Some(broadcaster);
        self
    }

    /// Resolve a graph ID to a file path, searching runtime graphs first,
    /// then nucleus graphs. Runtime graphs take precedence so consumers can
    /// override built-in compositions.
    pub(super) fn resolve_graph_path(&self, graph_id: &str) -> Option<PathBuf> {
        let runtime_path = self.runtime_graphs_dir.join(format!("{graph_id}.toml"));
        if runtime_path.exists() {
            return Some(runtime_path);
        }
        let nucleus_path = self.graphs_dir.join(format!("{graph_id}.toml"));
        if nucleus_path.exists() {
            return Some(nucleus_path);
        }
        None
    }

    // -----------------------------------------------------------------
    // CRUD operations
    // -----------------------------------------------------------------

    /// List all available graphs from both nucleus and runtime directories.
    ///
    /// JSON-RPC method: `graph.list`
    pub async fn list(&self) -> Result<Value> {
        let mut graphs = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();
        let mut any_dir_readable = false;

        for (dir, tier) in [
            (&self.runtime_graphs_dir, "runtime"),
            (&self.graphs_dir, "nucleus"),
        ] {
            if !dir.is_dir() {
                continue;
            }
            any_dir_readable = true;
            let mut toml_files = Vec::new();
            collect_toml_files_recursive(dir, &mut toml_files);

            info!(
                "graph.list: scanning {} ({} .toml files found)",
                dir.display(),
                toml_files.len()
            );

            for path in toml_files {
                match Graph::from_toml_file(&path) {
                    Ok(graph) => {
                        if seen_ids.insert(graph.id.clone()) {
                            graphs.push(json!({
                                "id": graph.id,
                                "version": graph.version,
                                "description": graph.description,
                                "node_count": graph.nodes.len(),
                                "coordination": graph.coordination.as_deref().unwrap_or("sequential"),
                                "continuous": graph.is_continuous(),
                                "tier": tier,
                                "estimated_time_ms": null,
                                "tags": []
                            }));
                        }
                    }
                    Err(neural_err) => match biomeos_graph::GraphLoader::from_file(&path) {
                        Ok(dg) => {
                            let def = &dg.definition;
                            let id_str = def.id.as_str().to_string();
                            if seen_ids.insert(id_str.clone()) {
                                graphs.push(json!({
                                    "id": id_str,
                                    "version": if def.version.is_empty() { "0.0.0" } else { &def.version },
                                    "description": def.description,
                                    "node_count": def.nodes.len(),
                                    "coordination": format!("{:?}", def.coordination).to_lowercase(),
                                    "continuous": matches!(def.coordination, biomeos_graph::CoordinationPattern::Continuous),
                                    "tier": tier,
                                    "estimated_time_ms": null,
                                    "tags": []
                                }));
                            }
                        }
                        Err(deploy_err) => {
                            tracing::warn!(
                                path = %path.display(),
                                neural_err = %neural_err,
                                deploy_err = %deploy_err,
                                "graph.list: TOML file failed both parsers — skipping"
                            );
                        }
                    },
                }
            }
        }

        if !any_dir_readable {
            tracing::debug!(
                graphs_dir = %self.graphs_dir.display(),
                runtime_dir = %self.runtime_graphs_dir.display(),
                "graph.list: no readable graphs directory found"
            );
        }

        Ok(json!(graphs))
    }

    /// Get a specific graph.
    ///
    /// JSON-RPC method: `graph.get`
    pub async fn get(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let graph_id = params["graph_id"].as_str().context("Missing graph_id")?;

        let graph_path = self.resolve_graph_path(graph_id).with_context(|| {
            format!("Graph '{graph_id}' not found in nucleus or runtime directories")
        })?;
        let graph = Graph::from_toml_file(&graph_path).context("Failed to load graph")?;

        Ok(serde_json::to_value(graph)?)
    }

    /// Save a runtime graph (consumer-deployed composition).
    ///
    /// JSON-RPC method: `graph.save`
    pub async fn save(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;

        let graph: Graph = if let Ok(g) = serde_json::from_value::<Graph>(params.clone()) {
            g
        } else if let Some(toml_str) = params.get("toml").and_then(|v| v.as_str()) {
            Graph::from_toml_str(toml_str)
                .context("Failed to parse TOML graph from 'toml' field")?
        } else if let Some(graph_obj) = params.get("graph") {
            let mut flat = graph_obj.clone();
            if let (Some(obj), Some(nodes)) = (flat.as_object_mut(), params.get("nodes")) {
                obj.insert("nodes".to_string(), nodes.clone());
            }
            serde_json::from_value::<Graph>(flat)
                .context("Failed to parse graph from {graph: {...}, nodes: [...]}")?
        } else {
            anyhow::bail!(
                "Failed to parse graph. Accepted formats: flat JSON, \
                 {{\"toml\": \"...\"}}, or {{\"graph\": {{...}}, \"nodes\": [...]}}"
            );
        };

        std::fs::create_dir_all(&self.runtime_graphs_dir).with_context(|| {
            format!(
                "Failed to create runtime graphs directory: {}",
                self.runtime_graphs_dir.display()
            )
        })?;

        let graph_path = self.runtime_graphs_dir.join(format!("{}.toml", graph.id));

        let toml_str =
            toml::to_string_pretty(&graph).context("Failed to serialize graph to TOML")?;

        std::fs::write(&graph_path, toml_str).context("Failed to write graph file")?;

        info!(
            "💾 Saved runtime graph: {} to {}",
            graph.id,
            graph_path.display()
        );

        Ok(json!({"graph_id": graph.id, "location": "runtime"}))
    }

    // -----------------------------------------------------------------
    // Status & utilities
    // -----------------------------------------------------------------

    /// Get execution status.
    ///
    /// JSON-RPC method: `graph.status`
    pub async fn get_status(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let execution_id = params["execution_id"]
            .as_str()
            .context("Missing execution_id")?;

        let executions = self.executions.read().await;
        if let Some(status) = executions.get(execution_id) {
            return Ok(serde_json::to_value(status)?);
        }
        drop(executions);

        let sessions = self.continuous_sessions.read().await;
        if let Some(session) = sessions.get(execution_id) {
            let state = *session.state_rx.borrow();
            return Ok(json!({
                "execution_id": execution_id,
                "graph_id": session.graph_id,
                "state": state.to_string(),
                "continuous": true,
                "started_at": session.started_at,
            }));
        }

        anyhow::bail!("Execution not found: {execution_id}")
    }

    /// Analyze a graph's execution history and suggest optimizations.
    ///
    /// JSON-RPC method: `graph.suggest_optimizations`
    pub async fn suggest_optimizations(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let graph_id = params["graph_id"].as_str().context("Missing graph_id")?;

        let min_samples = params["min_samples"].as_u64().unwrap_or(10);

        let graph_path = self.resolve_graph_path(graph_id).with_context(|| {
            format!("Graph '{graph_id}' not found in nucleus or runtime directories")
        })?;

        let toml_str = std::fs::read_to_string(&graph_path)
            .with_context(|| format!("Failed to read: {}", graph_path.display()))?;

        let deployment_graph: DeploymentGraph = toml::from_str(&toml_str)
            .with_context(|| format!("Failed to parse DeploymentGraph: {graph_id}"))?;

        let metrics_db_path = self.metrics_db_path.clone().unwrap_or_else(|| {
            SystemPaths::new()
                .map(|p| p.data_dir().join(files::DEFAULT_NEURAL_METRICS_DB))
                .unwrap_or_else(|_| {
                    PathBuf::from(DEFAULT_SOCKET_DIR).join(files::DEFAULT_NEURAL_METRICS_DB)
                })
        });

        let collector = biomeos_graph::metrics::MetricsCollector::new(&metrics_db_path)
            .context("Failed to open metrics database")?;

        let learner = biomeos_graph::pathway_learner::PathwayLearner::new(collector, min_samples);
        let analysis = learner.analyze(&deployment_graph).await;

        info!(
            "🧠 PathwayLearner analysis for '{}': {} suggestions from {} samples",
            graph_id,
            analysis.suggestions.len(),
            analysis.sample_size
        );

        Ok(serde_json::to_value(analysis)?)
    }

    /// Resolve primal name from node configuration.
    ///
    /// Order of precedence:
    /// 1. Explicit `primal.by_name` in node
    /// 2. Node ID (fallback)
    pub(crate) fn resolve_primal_name(node: &crate::neural_graph::GraphNode) -> String {
        node.primal
            .as_ref()
            .and_then(|p| p.by_name.clone())
            .unwrap_or_else(|| node.id.clone())
    }

    /// Extract `session_id` from params (pure logic, testable).
    pub(crate) fn extract_session_id(params: &Option<Value>) -> Result<String> {
        let params = params.as_ref().context("Missing parameters")?;
        Ok(params["session_id"]
            .as_str()
            .context("Missing session_id")?
            .to_string())
    }

    /// Report status of all active continuous sessions.
    ///
    /// JSON-RPC method: `graph.tick_status`
    pub async fn tick_status(&self) -> Result<Value> {
        let sessions = self.continuous_sessions.read().await;
        let mut session_list = Vec::new();
        for (sid, session) in sessions.iter() {
            let state = *session.state_rx.borrow();
            session_list.push(json!({
                "session_id": sid,
                "graph_id": session.graph_id,
                "state": format!("{state:?}"),
                "started_at": session.started_at,
            }));
        }
        Ok(json!({
            "sessions": session_list,
            "count": session_list.len(),
            "has_shared_broadcaster": self.event_broadcaster.is_some(),
        }))
    }

    /// Verify graph integrity (content hash + signature check).
    ///
    /// Params: `{ "path": "/path/to/graph.toml" }` or `{ "id": "graph-id" }`
    pub async fn verify_graph(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;

        let path = if let Some(p) = params.get("path").and_then(|v| v.as_str()) {
            PathBuf::from(p)
        } else if let Some(graph_id) = params.get("id").and_then(|v| v.as_str()) {
            self.resolve_graph_path(graph_id)
                .ok_or_else(|| anyhow::anyhow!("Graph not found: {graph_id}"))?
        } else {
            anyhow::bail!("graph.verify requires 'path' or 'id' parameter");
        };

        let content = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Failed to read graph: {e}"))?;

        // Try typed parse for metadata
        let (embedded_hash, embedded_sig, embedded_signer, genetics_tier) =
            if let Ok(graph) = biomeos_graph::GraphLoader::from_str(&content, Some(&path)) {
                (
                    graph.definition.metadata.content_hash.clone(),
                    graph.definition.metadata.signature.clone(),
                    graph.definition.metadata.signed_by.clone(),
                    graph.definition.metadata.genetics_tier,
                )
            } else {
                (None, None, None, None)
            };

        let report = biomeos_graph::verify_integrity(
            &content,
            embedded_hash.as_deref(),
            embedded_sig.as_deref(),
            embedded_signer.as_deref(),
        );

        Ok(json!({
            "path": path.display().to_string(),
            "content_hash": report.computed_hash,
            "hash_match": report.hash_match,
            "signature_valid": report.signature_valid,
            "signer": report.signer,
            "genetics_tier": genetics_tier.map(|t| format!("{t:?}")),
            "acceptable": report.acceptable_for_tier(genetics_tier),
        }))
    }
}
