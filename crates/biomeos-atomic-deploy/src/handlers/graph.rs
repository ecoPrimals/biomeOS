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
use biomeos_types::SystemPaths;
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

/// Graph handler for CRUD and execution operations.
#[derive(Clone)]
pub struct GraphHandler {
    /// Path to graphs directory
    graphs_dir: PathBuf,

    /// Active executions
    executions: Arc<RwLock<HashMap<String, ExecutionStatus>>>,

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
    pub async fn execute(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
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
            "✅ Graph loaded: {} (version: {}, {} nodes)",
            graph.id,
            graph.version,
            graph.nodes.len()
        );

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

            let mut executor = GraphExecutor::new(graph.clone(), env);
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
    fn resolve_primal_name(node: &crate::neural_graph::GraphNode) -> String {
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

        let executions = self.executions.read().await;
        let status = executions
            .get(execution_id)
            .context("Execution not found")?;

        Ok(serde_json::to_value(status)?)
    }
}
