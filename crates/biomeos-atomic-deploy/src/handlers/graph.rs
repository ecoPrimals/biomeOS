// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Graph CRUD and execution handlers.
//!
//! This module handles all graph-related JSON-RPC methods:
//! - `graph.list` - List all available graphs
//! - `graph.get` - Get a specific graph
//! - `graph.save` - Save/update a graph
//! - `graph.execute` - Execute a graph
//! - `graph.status` - Get execution status
//!
//! # Capability-Based Design
//!
//! Graph execution uses capability-based primal discovery instead of hardcoded names.
//! Instead of mapping "security" → "beardog", we discover which primal provides "security".
//!
//! # XDG Compliance (EVOLVED Jan 27, 2026)
//!
//! Socket directory is determined via SystemPaths, not hardcoded.

use crate::capability_translation::CapabilityTranslationRegistry;
use crate::neural_executor::GraphExecutor;
use crate::neural_graph::Graph;
use crate::neural_router::NeuralRouter;
use anyhow::{Context, Result};
use biomeos_graph::continuous::{ContinuousExecutor, SessionCommand, SessionState};
use biomeos_graph::events::GraphEventBroadcaster;
use biomeos_graph::graph::{CoordinationPattern, DeploymentGraph};
use biomeos_graph::pipeline::{PipelineExecutor, StreamItem};
use biomeos_types::{constants::files, defaults::DEFAULT_SOCKET_DIR, SystemPaths};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

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
struct ContinuousSession {
    graph_id: String,
    command_tx: tokio::sync::mpsc::Sender<SessionCommand>,
    state_rx: tokio::sync::watch::Receiver<SessionState>,
    started_at: String,
}

/// Graph handler for CRUD and execution operations.
#[derive(Clone)]
pub struct GraphHandler {
    /// Path to graphs directory
    graphs_dir: PathBuf,

    /// Active executions (transactional)
    executions: Arc<RwLock<HashMap<String, ExecutionStatus>>>,

    /// Active continuous sessions (keyed by session_id)
    continuous_sessions: Arc<RwLock<HashMap<String, ContinuousSession>>>,

    /// Family ID
    family_id: String,

    /// Neural Router for capability discovery
    router: Arc<NeuralRouter>,

    /// Capability Translation Registry
    translation_registry: Arc<RwLock<CapabilityTranslationRegistry>>,
}

impl GraphHandler {
    /// Create a new graph handler.
    pub fn new(
        graphs_dir: impl Into<PathBuf>,
        family_id: impl Into<String>,
        executions: Arc<RwLock<HashMap<String, ExecutionStatus>>>,
        router: Arc<NeuralRouter>,
        translation_registry: Arc<RwLock<CapabilityTranslationRegistry>>,
    ) -> Self {
        Self {
            graphs_dir: graphs_dir.into(),
            family_id: family_id.into(),
            executions,
            continuous_sessions: Arc::new(RwLock::new(HashMap::new())),
            router,
            translation_registry,
        }
    }

    /// List all available graphs.
    ///
    /// JSON-RPC method: `graph.list`
    pub async fn list(&self) -> Result<Value> {
        let mut graphs = Vec::new();

        let entries =
            std::fs::read_dir(&self.graphs_dir).context("Failed to read graphs directory")?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Ok(graph) = Graph::from_toml_file(&path) {
                    graphs.push(json!({
                        "id": graph.id,
                        "version": graph.version,
                        "description": graph.description,
                        "node_count": graph.nodes.len(),
                        "coordination": graph.coordination.as_deref().unwrap_or("sequential"),
                        "continuous": graph.is_continuous(),
                        "estimated_time_ms": null,
                        "tags": []
                    }));
                }
            }
        }

        Ok(json!(graphs))
    }

    /// Get a specific graph.
    ///
    /// JSON-RPC method: `graph.get`
    pub async fn get(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let graph_id = params["graph_id"].as_str().context("Missing graph_id")?;

        let graph_path = self.graphs_dir.join(format!("{graph_id}.toml"));
        let graph = Graph::from_toml_file(&graph_path).context("Failed to load graph")?;

        Ok(serde_json::to_value(graph)?)
    }

    /// Save a graph.
    ///
    /// JSON-RPC method: `graph.save`
    pub async fn save(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let graph: Graph =
            serde_json::from_value(params.clone()).context("Failed to parse graph")?;

        let graph_path = self.graphs_dir.join(format!("{}.toml", graph.id));

        let toml_str =
            toml::to_string_pretty(&graph).context("Failed to serialize graph to TOML")?;

        std::fs::write(&graph_path, toml_str).context("Failed to write graph file")?;

        info!("💾 Saved graph: {} to {}", graph.id, graph_path.display());

        Ok(json!({"graph_id": graph.id}))
    }

    /// Execute a graph.
    ///
    /// JSON-RPC method: `graph.execute`
    ///
    /// # Capability-Based Registration
    ///
    /// After execution, primals are registered by their CAPABILITIES, not hardcoded names.
    /// This enables TRUE PRIMAL discovery - consumers ask for capabilities, not primal names.
    pub async fn execute(&self, raw_params: &Option<Value>) -> Result<Value> {
        let params = raw_params.as_ref().context("Missing parameters")?;
        let graph_id = params["graph_id"].as_str().context("Missing graph_id")?;
        let family_id_param = params["family_id"].as_str().unwrap_or(&self.family_id);

        let graph_path = self.graphs_dir.join(format!("{graph_id}.toml"));

        info!("🔍 Loading graph: {}", graph_id);
        debug!("   Graph path: {}", graph_path.display());

        if !graph_path.exists() {
            error!("❌ Graph file not found: {}", graph_path.display());
            anyhow::bail!("Graph file not found: {}", graph_path.display());
        }

        let graph = Graph::from_toml_file(&graph_path)
            .with_context(|| format!("Failed to load graph from: {}", graph_path.display()))?;

        info!(
            "✅ Graph loaded: {} (version: {}, {} nodes, coordination: {})",
            graph.id,
            graph.version,
            graph.nodes.len(),
            graph.coordination.as_deref().unwrap_or("sequential"),
        );

        // Auto-redirect continuous graphs to start_continuous
        if graph.is_continuous() {
            info!("🔄 Graph is continuous — redirecting to start_continuous");
            return self.start_continuous(raw_params).await;
        }

        // Auto-redirect pipeline graphs to execute_pipeline
        if graph.coordination.as_deref() == Some("pipeline") {
            info!("🔗 Graph is pipeline — redirecting to execute_pipeline");
            return self.execute_pipeline(raw_params).await;
        }

        // Load capability translations from graph
        self.load_translations_from_graph(&graph).await?;

        // Generate execution ID
        let execution_id = format!("{}-{}", graph_id, chrono::Utc::now().timestamp());
        let started_at = chrono::Utc::now().to_rfc3339();

        // Create execution status
        let status = ExecutionStatus {
            execution_id: execution_id.clone(),
            state: "running".to_string(),
            current_phase: Some(0),
            total_phases: graph.nodes.len(),
            completed_nodes: Vec::new(),
            failed_nodes: Vec::new(),
            duration_ms: 0,
            error: None,
        };

        self.executions
            .write()
            .await
            .insert(execution_id.clone(), status);

        // Clone values for async move
        let executions = self.executions.clone();
        let execution_id_clone = execution_id.clone();
        let family_id_owned = family_id_param.to_string();
        let router = self.router.clone();

        // Execute graph in background
        tokio::spawn(async move {
            let mut env = HashMap::new();
            env.insert("FAMILY_ID".to_string(), family_id_owned.clone());
            env.insert(
                "UID".to_string(),
                rustix::process::getuid().as_raw().to_string(),
            );

            // EVOLVED (Jan 27, 2026): XDG-compliant socket directory (no hardcoding!)
            let socket_dir = SystemPaths::new()
                .map(|p| p.runtime_dir().to_string_lossy().to_string())
                .unwrap_or_else(|_| {
                    std::env::var("BIOMEOS_SOCKET_DIR").unwrap_or_else(|_| "/tmp".to_string())
                });
            env.insert("SOCKET_DIR".to_string(), socket_dir);
            env.insert(
                "JWT_SECRET".to_string(),
                std::env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "CHANGE_ME_IN_PRODUCTION".to_string()),
            );

            // Wire PathwayLearner metrics: record per-node and per-graph execution data
            let metrics_db_path = SystemPaths::new()
                .map(|p| p.data_dir().join(files::DEFAULT_NEURAL_METRICS_DB))
                .unwrap_or_else(|_| {
                    PathBuf::from(DEFAULT_SOCKET_DIR).join(files::DEFAULT_NEURAL_METRICS_DB)
                });
            let metrics = biomeos_graph::metrics::MetricsCollector::new(&metrics_db_path).await;

            let mut executor = GraphExecutor::new(graph.clone(), env);
            if let Ok(m) = metrics {
                executor = executor.with_metrics(m);
            }
            let start = std::time::Instant::now();

            match executor.execute().await {
                Ok(report) => {
                    if report.success {
                        // Register capabilities by CAPABILITY, not hardcoded names
                        Self::register_capabilities_from_graph(&router, &graph, &family_id_owned)
                            .await;
                    }

                    let mut status = executions.write().await;
                    if let Some(exec_status) = status.get_mut(&execution_id_clone) {
                        exec_status.state = if report.success {
                            "completed".to_string()
                        } else {
                            "failed".to_string()
                        };
                        exec_status.duration_ms = start.elapsed().as_millis() as u64;
                        exec_status.error = report.error;
                    }
                }
                Err(e) => {
                    let mut status = executions.write().await;
                    if let Some(exec_status) = status.get_mut(&execution_id_clone) {
                        exec_status.state = "failed".to_string();
                        exec_status.duration_ms = start.elapsed().as_millis() as u64;
                        exec_status.error = Some(format!("Execution failed: {e}"));
                    }
                }
            }
        });

        Ok(json!({
            "execution_id": execution_id,
            "graph_id": graph_id,
            "started_at": started_at
        }))
    }

    /// Register capabilities from a deployed graph.
    ///
    /// This is the TRUE PRIMAL way: register by capability, discover the primal name
    /// from the node configuration instead of hardcoding.
    async fn register_capabilities_from_graph(
        router: &NeuralRouter,
        graph: &Graph,
        family_id: &str,
    ) {
        info!("📝 Registering capabilities from deployed graph...");

        for node in &graph.nodes {
            if node.capabilities.is_empty() {
                continue;
            }

            // Get primal name from node config - NO HARDCODING
            let primal_name = Self::resolve_primal_name(node);

            // Determine socket path (XDG-compliant, no hardcoding!)
            let runtime_dir = SystemPaths::new()
                .map(|p| p.runtime_dir().to_string_lossy().to_string())
                .unwrap_or_else(|_| {
                    std::env::var("BIOMEOS_RUNTIME_DIR")
                        .or_else(|_| std::env::var("TMPDIR"))
                        .unwrap_or_else(|_| "/tmp".to_string())
                });

            let socket_path = format!("{runtime_dir}/{primal_name}-{family_id}.sock");

            // Register each capability
            for capability in &node.capabilities {
                if let Err(e) = router
                    .register_capability(
                        capability,
                        &primal_name,
                        PathBuf::from(&socket_path),
                        "graph_deployment",
                    )
                    .await
                {
                    warn!("Failed to register capability {}: {}", capability, e);
                } else {
                    info!("   ✅ {} → {} @ {}", capability, primal_name, socket_path);
                }
            }
        }
    }

    /// Resolve primal name from node configuration.
    ///
    /// Order of precedence:
    /// 1. Explicit `primal.by_name` in node
    /// 2. Node ID (fallback)
    ///
    /// REMOVED: Hardcoded capability → primal mapping
    pub(crate) fn resolve_primal_name(node: &crate::neural_graph::GraphNode) -> String {
        node.primal
            .as_ref()
            .and_then(|p| p.by_name.clone())
            .unwrap_or_else(|| node.id.clone())
    }

    /// Load translations from graph.
    ///
    /// Translations are loaded from nodes that define capability mappings.
    async fn load_translations_from_graph(&self, graph: &Graph) -> Result<()> {
        info!("📝 Loading capability translations from graph...");

        let mut registry = self.translation_registry.write().await;

        // Extract translations from node capabilities
        for node in &graph.nodes {
            for capability in &node.capabilities {
                // Register each capability as a potential translation target
                let primal_name = Self::resolve_primal_name(node);
                let semantic_name = format!("{capability}.default");
                let method = format!("{capability}.invoke");

                // Determine socket path (XDG-compliant, no hardcoding!)
                let runtime_dir = SystemPaths::new()
                    .map(|p| p.runtime_dir().to_string_lossy().to_string())
                    .unwrap_or_else(|_| {
                        std::env::var("BIOMEOS_RUNTIME_DIR")
                            .or_else(|_| std::env::var("TMPDIR"))
                            .unwrap_or_else(|_| "/tmp".to_string())
                    });
                let socket_path =
                    format!("{}/{}-{}.sock", runtime_dir, primal_name, self.family_id);

                debug!(
                    "   Registering: {} → {}:{} @ {}",
                    semantic_name, primal_name, method, socket_path
                );

                registry.register_translation(
                    &semantic_name,
                    &primal_name,
                    &method,
                    &socket_path,
                    None,
                );
            }
        }

        info!("✅ Capability translations loaded");
        Ok(())
    }

    /// Get execution status.
    ///
    /// JSON-RPC method: `graph.status`
    pub async fn get_status(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let execution_id = params["execution_id"]
            .as_str()
            .context("Missing execution_id")?;

        // Check transactional executions
        let executions = self.executions.read().await;
        if let Some(status) = executions.get(execution_id) {
            return Ok(serde_json::to_value(status)?);
        }
        drop(executions);

        // Check continuous sessions
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

    // -------------------------------------------------------------------
    // Continuous session management
    // -------------------------------------------------------------------

    /// Start a continuous graph execution session.
    ///
    /// JSON-RPC method: `graph.start_continuous`
    ///
    /// Loads the graph from disk as a `DeploymentGraph`, creates a
    /// `ContinuousExecutor`, and runs it in a background task. Returns
    /// a `session_id` that can be used for pause/resume/stop.
    pub async fn start_continuous(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let graph_id = params["graph_id"].as_str().context("Missing graph_id")?;

        let graph_path = self.graphs_dir.join(format!("{graph_id}.toml"));
        if !graph_path.exists() {
            anyhow::bail!("Graph file not found: {}", graph_path.display());
        }

        let toml_str = std::fs::read_to_string(&graph_path)
            .with_context(|| format!("Failed to read: {}", graph_path.display()))?;

        let deployment_graph: DeploymentGraph = toml::from_str(&toml_str)
            .with_context(|| format!("Failed to parse DeploymentGraph: {graph_id}"))?;

        let coordination = &deployment_graph.definition.coordination;
        if *coordination != biomeos_graph::graph::CoordinationPattern::Continuous {
            anyhow::bail!("Graph '{graph_id}' has coordination '{coordination:?}', not Continuous");
        }

        let session_id = format!("{graph_id}-{}", chrono::Utc::now().timestamp_millis());
        let broadcaster = GraphEventBroadcaster::new(16);
        let mut executor = ContinuousExecutor::new(deployment_graph, broadcaster);

        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel::<SessionCommand>(16);
        let state_rx = executor.state_receiver();
        let session_id_log = session_id.clone();

        // Spawn the continuous loop in the background
        tokio::spawn(async move {
            info!("🎮 Starting continuous session: {}", session_id_log);
            executor
                .run(cmd_rx, |node_id, _params, _feedback| {
                    let node_id = node_id.to_string();
                    Box::pin(async move {
                        debug!("  tick node: {}", node_id);
                        Ok(serde_json::json!({"node": node_id, "status": "ok"}))
                    })
                })
                .await;
            info!("🛑 Continuous session stopped: {}", session_id_log);
        });

        let started_at = chrono::Utc::now().to_rfc3339();

        self.continuous_sessions.write().await.insert(
            session_id.clone(),
            ContinuousSession {
                graph_id: graph_id.to_string(),
                command_tx: cmd_tx,
                state_rx,
                started_at: started_at.clone(),
            },
        );

        info!(
            "✅ Continuous session started: {} ({})",
            session_id, graph_id
        );

        Ok(json!({
            "session_id": session_id,
            "graph_id": graph_id,
            "started_at": started_at,
        }))
    }

    /// Execute a pipeline graph — streaming coordination.
    ///
    /// JSON-RPC method: `graph.execute_pipeline`
    ///
    /// Loads the graph as a `DeploymentGraph`, validates that it uses
    /// `Pipeline` coordination, then runs it via `PipelineExecutor`.
    /// The source node produces items, each transform node processes them,
    /// and the result collects all outputs.
    ///
    /// Returns the full `PipelineResult` with per-node throughput stats.
    pub async fn execute_pipeline(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let graph_id = params["graph_id"].as_str().context("Missing graph_id")?;

        let graph_path = self.graphs_dir.join(format!("{graph_id}.toml"));
        if !graph_path.exists() {
            anyhow::bail!("Graph file not found: {}", graph_path.display());
        }

        let toml_str = std::fs::read_to_string(&graph_path)
            .with_context(|| format!("Failed to read: {}", graph_path.display()))?;

        let deployment_graph: DeploymentGraph = toml::from_str(&toml_str)
            .with_context(|| format!("Failed to parse DeploymentGraph: {graph_id}"))?;

        if deployment_graph.definition.coordination != CoordinationPattern::Pipeline {
            anyhow::bail!(
                "Graph '{}' has coordination '{:?}', not Pipeline",
                graph_id,
                deployment_graph.definition.coordination
            );
        }

        let channel_capacity = params["channel_capacity"].as_u64().unwrap_or(64) as usize;

        info!(
            "🔗 Executing pipeline graph: {} ({} nodes, capacity {})",
            graph_id,
            deployment_graph.definition.nodes.len(),
            channel_capacity,
        );

        let broadcaster = GraphEventBroadcaster::new(16);
        let executor = PipelineExecutor::new(deployment_graph, broadcaster)
            .with_channel_capacity(channel_capacity);

        let router = self.router.clone();
        let translation_registry = self.translation_registry.clone();

        let result = executor
            .run(move |node_id, node, item| {
                let router = router.clone();
                let translation_registry = translation_registry.clone();
                async move {
                    let capability = match &node.capability {
                        Some(c) => c.clone(),
                        None => {
                            return StreamItem::Error {
                                node_id: node_id.clone(),
                                message: format!("Node '{node_id}' has no capability"),
                            };
                        }
                    };

                    let input = match item {
                        StreamItem::Data(v) => v,
                        other => return other,
                    };

                    let call_params = json!({
                        "capability": capability,
                        "node_id": node_id,
                        "input": input,
                    });

                    // Try translation registry first
                    let registry = translation_registry.read().await;
                    if let Some(translation) = registry.get_translation(&capability) {
                        let socket = PathBuf::from(&translation.socket);
                        let method = translation.actual_method.clone();
                        drop(registry);
                        match router.forward_request(&socket, &method, &call_params).await {
                            Ok(result) => StreamItem::Data(result),
                            Err(e) => {
                                debug!("Pipeline node '{}' capability call failed: {}", node_id, e);
                                StreamItem::Error {
                                    node_id,
                                    message: format!("{e}"),
                                }
                            }
                        }
                    } else {
                        drop(registry);
                        // No translation — discover capability and forward
                        match router.discover_capability(&capability).await {
                            Ok(discovered) => {
                                match router
                                    .forward_request(
                                        &discovered.primary_socket,
                                        &capability,
                                        &call_params,
                                    )
                                    .await
                                {
                                    Ok(result) => StreamItem::Data(result),
                                    Err(e) => StreamItem::Error {
                                        node_id,
                                        message: format!("{e}"),
                                    },
                                }
                            }
                            Err(e) => {
                                debug!("Pipeline node '{}' discovery failed: {}", node_id, e);
                                StreamItem::Error {
                                    node_id,
                                    message: format!("Capability not found: {capability}: {e}"),
                                }
                            }
                        }
                    }
                }
            })
            .await
            .context("Pipeline execution failed")?;

        Ok(serde_json::to_value(result)?)
    }

    /// Pause a running continuous session.
    ///
    /// JSON-RPC method: `graph.pause_continuous`
    pub async fn pause_continuous(&self, params: &Option<Value>) -> Result<Value> {
        let session_id = Self::extract_session_id(params)?;
        let sessions = self.continuous_sessions.read().await;
        let session = sessions
            .get(&session_id)
            .with_context(|| format!("Continuous session not found: {session_id}"))?;

        session
            .command_tx
            .send(SessionCommand::Pause)
            .await
            .context("Session command channel closed")?;

        info!("⏸️  Paused continuous session: {}", session_id);
        Ok(json!({"session_id": session_id, "command": "pause"}))
    }

    /// Resume a paused continuous session.
    ///
    /// JSON-RPC method: `graph.resume_continuous`
    pub async fn resume_continuous(&self, params: &Option<Value>) -> Result<Value> {
        let session_id = Self::extract_session_id(params)?;
        let sessions = self.continuous_sessions.read().await;
        let session = sessions
            .get(&session_id)
            .with_context(|| format!("Continuous session not found: {session_id}"))?;

        session
            .command_tx
            .send(SessionCommand::Resume)
            .await
            .context("Session command channel closed")?;

        info!("▶️  Resumed continuous session: {}", session_id);
        Ok(json!({"session_id": session_id, "command": "resume"}))
    }

    /// Stop a continuous session.
    ///
    /// JSON-RPC method: `graph.stop_continuous`
    pub async fn stop_continuous(&self, params: &Option<Value>) -> Result<Value> {
        let session_id = Self::extract_session_id(params)?;

        let session = self
            .continuous_sessions
            .write()
            .await
            .remove(&session_id)
            .with_context(|| format!("Continuous session not found: {session_id}"))?;

        // Send stop command (best effort — the executor may have already exited)
        let _ = session.command_tx.send(SessionCommand::Stop).await;

        info!("🛑 Stopped continuous session: {}", session_id);
        Ok(json!({"session_id": session_id, "command": "stop"}))
    }

    /// Analyze a graph's execution history and suggest optimizations.
    ///
    /// JSON-RPC method: `graph.suggest_optimizations`
    ///
    /// Loads the graph as a `DeploymentGraph`, connects to the metrics
    /// database, and runs the `PathwayLearner` analysis. Returns
    /// optimization suggestions sorted by estimated impact.
    pub async fn suggest_optimizations(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let graph_id = params["graph_id"].as_str().context("Missing graph_id")?;

        let min_samples = params["min_samples"].as_u64().unwrap_or(10);

        let graph_path = self.graphs_dir.join(format!("{graph_id}.toml"));
        if !graph_path.exists() {
            anyhow::bail!("Graph file not found: {}", graph_path.display());
        }

        let toml_str = std::fs::read_to_string(&graph_path)
            .with_context(|| format!("Failed to read: {}", graph_path.display()))?;

        let deployment_graph: DeploymentGraph = toml::from_str(&toml_str)
            .with_context(|| format!("Failed to parse DeploymentGraph: {graph_id}"))?;

        let metrics_db_path = SystemPaths::new()
            .map(|p| p.data_dir().join(files::DEFAULT_NEURAL_METRICS_DB))
            .unwrap_or_else(|_| {
                PathBuf::from(DEFAULT_SOCKET_DIR).join(files::DEFAULT_NEURAL_METRICS_DB)
            });

        let collector = biomeos_graph::metrics::MetricsCollector::new(&metrics_db_path)
            .await
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

    /// Extract session_id from params (pure logic, testable).
    pub(crate) fn extract_session_id(params: &Option<Value>) -> Result<String> {
        let params = params.as_ref().context("Missing parameters")?;
        Ok(params["session_id"]
            .as_str()
            .context("Missing session_id")?
            .to_string())
    }
}
