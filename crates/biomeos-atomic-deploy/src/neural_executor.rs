// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Graph executor for deterministic deployment orchestration
//!
//! This module executes Neural API graphs with:
//! - Topological sorting for dependency resolution
//! - Parallel execution within phases
//! - Checkpoint/rollback support
//! - Live monitoring and metrics
//!
//! ## Architecture
//!
//! Uses shared types from `crate::executor`:
//! - `ExecutionContext`: Shared state across nodes
//! - `NodeStatus`: Node execution status
//! - `ExecutionReport`: Final execution report
//! - `PhaseResult`: Result from a single phase
//!
//! ## Deep Debt Principles
//!
//! - Capability-based discovery (no hardcoded primal names)
//! - Pure JSON-RPC communication (no HTTP in IPC)
//! - Runtime primal discovery (self-knowledge only)

use anyhow::{Context, Result};
use biomeos_graph::metrics::{GraphResult, MetricsCollector, NodeExecutionParams};
use biomeos_types::JsonRpcRequest;
use serde::Serialize;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::neural_graph::{Graph, GraphNode};

// Re-export from executor module (single source of truth)
// This eliminates duplicate type definitions and ensures consistency
pub use crate::executor::context::{ExecutionContext, NodeStatus};
pub use crate::executor::types::{ExecutionReport, PhaseResult};

/// Graph executor
pub struct GraphExecutor {
    graph: Graph,
    context: ExecutionContext,
    pub(crate) max_parallelism: usize,
    metrics: Option<MetricsCollector>,
}

impl GraphExecutor {
    /// Create new graph executor
    pub fn new(graph: Graph, env: HashMap<String, String>) -> Self {
        Self {
            graph,
            context: ExecutionContext::new(env),
            max_parallelism: 3,
            metrics: None,
        }
    }

    /// Create graph executor with socket nucleation
    pub fn with_nucleation(
        graph: Graph,
        env: HashMap<String, String>,
        nucleation: Arc<tokio::sync::RwLock<crate::nucleation::SocketNucleation>>,
    ) -> Self {
        Self {
            graph,
            context: ExecutionContext::new(env).with_nucleation(nucleation),
            max_parallelism: 3,
            metrics: None,
        }
    }

    /// Attach a `MetricsCollector` for PathwayLearner integration.
    ///
    /// When set, the executor records per-node and per-graph execution metrics
    /// so the PathwayLearner can analyze and suggest optimizations.
    #[must_use]
    pub fn with_metrics(mut self, metrics: MetricsCollector) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Execute the entire graph
    pub async fn execute(&mut self) -> Result<ExecutionReport> {
        info!("🚀 Starting graph execution: {}", self.graph.id);

        let start_time = std::time::Instant::now();
        let mut report = ExecutionReport::new(self.graph.id.clone());

        // Topological sort to get execution phases
        let phases = self.topological_sort()?;

        info!("   Execution plan: {} phases", phases.len());

        // Execute each phase
        for (phase_num, phase_nodes) in phases.iter().enumerate() {
            info!(
                "📍 Phase {}/{}: {} nodes",
                phase_num + 1,
                phases.len(),
                phase_nodes.len()
            );

            match self.execute_phase(phase_nodes).await {
                Ok(phase_result) => {
                    report.add_phase_result(&phase_result);
                }
                Err(e) => {
                    error!("❌ Phase {} failed: {}", phase_num + 1, e);
                    report.success = false;
                    report.error = Some(e.to_string());

                    // Rollback if enabled
                    if self.graph.config.rollback_on_failure {
                        warn!("🔄 Rolling back deployment...");
                        self.rollback().await?;
                    }

                    break;
                }
            }
        }

        report.duration_ms = start_time.elapsed().as_millis() as u64;

        if report.success {
            info!("✅ Graph execution complete: {} ms", report.duration_ms);
        } else {
            error!("❌ Graph execution failed: {} ms", report.duration_ms);
        }

        // Record graph-level metrics for PathwayLearner
        if let Some(ref collector) = self.metrics {
            let graph_result = GraphResult {
                success: report.success,
                node_results: HashMap::new(),
                errors: vec![],
                duration_ms: report.duration_ms,
            };
            if let Err(e) = collector
                .record_execution(&self.graph.id, &graph_result, report.duration_ms)
                .await
            {
                warn!("Failed to record graph metrics: {e}");
            }
        }

        Ok(report)
    }

    /// Execute a single phase (parallel execution of independent nodes)
    async fn execute_phase(&mut self, nodes: &[String]) -> Result<PhaseResult> {
        let phase_start = std::time::Instant::now();
        let mut phase_result = PhaseResult::new(nodes.len());

        // Semaphore for max parallelism
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_parallelism));

        // Execute nodes in parallel
        let mut handles = Vec::new();

        let execution_id = chrono::Utc::now().timestamp_millis();

        for node_id in nodes {
            let node = self
                .graph
                .nodes
                .iter()
                .find(|n| &n.id == node_id)
                .ok_or_else(|| anyhow::anyhow!("Node not found: {node_id}"))?
                .clone();

            let context = self.context.clone();
            let permit = semaphore.clone().acquire_owned().await?;

            let handle = tokio::spawn(async move {
                let node_start = std::time::Instant::now();
                let result = Self::execute_node(&node, &context).await;
                let duration_ms = node_start.elapsed().as_millis() as u64;
                drop(permit);
                (node.id.clone(), result, duration_ms)
            });

            handles.push(handle);
        }

        let graph_id = self.graph.id.clone();

        // Wait for all nodes to complete
        for handle in handles {
            let (node_id, result, duration_ms) = handle.await?;

            let success = result.is_ok();

            match result {
                Ok(output) => {
                    phase_result.completed += 1;
                    self.context
                        .set_status(&node_id, NodeStatus::Completed(output.clone()))
                        .await;
                    self.context.set_output(&node_id, output).await;
                }
                Err(e) => {
                    let node = self
                        .graph
                        .nodes
                        .iter()
                        .find(|n| n.id == node_id);
                    if node.is_some_and(|n| n.is_optional()) {
                        debug!("Optional node {} failed, skipping: {}", node_id, e);
                        phase_result.completed += 1;
                        let skip_value = serde_json::json!({"skipped": true});
                        self.context
                            .set_status(&node_id, NodeStatus::Completed(skip_value.clone()))
                            .await;
                        self.context.set_output(&node_id, skip_value).await;
                    } else {
                        phase_result.failed += 1;
                        let error_msg = e.to_string();
                        self.context
                            .set_status(&node_id, NodeStatus::Failed(error_msg.clone()))
                            .await;
                        phase_result.errors.push((node_id.clone(), error_msg));
                    }
                }
            }

            // Record per-node metrics for PathwayLearner
            if let Some(ref collector) = self.metrics {
                if let Err(e) = collector
                    .record_node_execution(NodeExecutionParams {
                        execution_id,
                        graph_name: &graph_id,
                        node_id: &node_id,
                        primal_id: "",
                        operation: "",
                        success,
                        duration_ms,
                        error: None,
                    })
                    .await
                {
                    warn!("Failed to record node metrics for {node_id}: {e}");
                }
            }
        }

        phase_result.duration_ms = {
            let e = phase_start.elapsed();
            e.as_secs() * 1000 + u64::from(e.subsec_millis())
        };

        if phase_result.failed > 0 {
            error!("❌ {} nodes failed in this phase:", phase_result.failed);
            for (node_id, error_msg) in &phase_result.errors {
                error!("   • {}: {}", node_id, error_msg);
            }
            anyhow::bail!("Phase failed: {} nodes failed", phase_result.failed);
        }

        Ok(phase_result)
    }

    /// Execute a single node
    ///
    /// Delegates to shared handlers in `executor::node_handlers` for consistency
    /// and to avoid code duplication.
    async fn execute_node(
        node: &GraphNode,
        context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        use crate::executor::node_handlers;

        // Determine node type (new format or legacy)
        let node_type_str = if let Some(ref operation) = node.operation {
            operation.name.as_str()
        } else if let Some(ref node_type) = node.node_type {
            node_type.as_str()
        } else {
            "unknown"
        };

        info!(
            "   ⚡ Executing node: {} (type: {})",
            node.id, node_type_str
        );

        // Mark as running
        context.set_status(&node.id, NodeStatus::Running).await;

        // Execute based on node type - delegate to shared handlers
        let result = match node_type_str {
            // Filesystem operations
            "filesystem.check_exists" => {
                node_handlers::filesystem_check_exists(node, context).await
            }

            // Crypto operations
            "crypto.derive_child_seed" => node_handlers::crypto_derive_seed(node, context).await,

            // Primal lifecycle
            "primal.launch" => node_handlers::primal_launch(node, context).await,
            "primal_start" | "start" => {
                crate::capability_handlers::primal_start_capability(node, context).await
            }

            // Health checks
            "health_check" | "health.check" | "health.check_atomic" => {
                node_handlers::health_check(node, context).await
            }
            "health.check_all" => Self::node_health_check_all(node, context).await,

            // Verification
            "verification" => Self::node_verification(node, context).await,
            "lineage.verify_siblings" => node_handlers::lineage_verify(node, context).await,

            // Reporting
            "report.deployment_success" => node_handlers::deployment_report(node, context).await,

            // Logging
            "log.info" => node_handlers::log_info(node, context).await,
            "log.warn" => node_handlers::log_warn(node, context).await,
            "log.error" => node_handlers::log_error(node, context).await,

            // RPC call (NEW - Feb 6, 2026)
            // Allows graph nodes to call arbitrary methods on primals
            "rpc_call" => Self::node_rpc_call(node, context).await,

            // Capability call (NEW - Mar 1, 2026)
            // Routes through neural-api capability.call for semantic resolution.
            // Falls back to direct primal RPC if neural-api is unavailable.
            "capability_call" => Self::node_capability_call(node, context).await,

            // Capability registration for deployment graphs
            "register_capabilities" => node_handlers::register_capabilities(node, context).await,

            // Unknown
            _ => {
                warn!("Unknown node type: {}, skipping", node_type_str);
                Ok(serde_json::json!({"skipped": true}))
            }
        };

        result.context(format!("Node execution failed: {}", node.id))
    }

    /// Split capability string into (domain, operation) for capability.call semantics.
    /// e.g. "ecology.et0_fao56" -> ("ecology", "et0_fao56"), "single" -> ("single", "execute")
    pub(crate) fn split_capability(capability: &str) -> (String, String) {
        if let Some(dot_pos) = capability.find('.') {
            (
                capability[..dot_pos].to_string(),
                capability[dot_pos + 1..].to_string(),
            )
        } else {
            (capability.to_string(), "execute".to_string())
        }
    }

    /// Substitute environment variables in a string (used by tests; production uses executor::substitute_env)
    #[cfg(test)]
    pub(crate) fn substitute_env(s: &str, env: &HashMap<String, String>) -> String {
        let mut result = s.to_string();

        for (key, value) in env {
            let placeholder = format!("${{{key}}}");
            result = result.replace(&placeholder, value);
        }

        result
    }

    /// Perform topological sort to determine execution phases
    pub(crate) fn topological_sort(&self) -> Result<Vec<Vec<String>>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut graph_map: HashMap<String, Vec<String>> = HashMap::new();

        // Build adjacency list and in-degree map
        tracing::info!(
            "🔍 Building dependency graph for {} nodes...",
            self.graph.nodes.len()
        );
        for node in &self.graph.nodes {
            tracing::info!("   Node '{}' depends_on: {:?}", node.id, node.depends_on);
            in_degree.entry(node.id.clone()).or_insert(0);

            for dep in &node.depends_on {
                // FIXED: was node.dependencies, now node.depends_on
                graph_map
                    .entry(dep.clone())
                    .or_default()
                    .push(node.id.clone());
                *in_degree.entry(node.id.clone()).or_insert(0) += 1;
            }
        }

        tracing::info!("🔍 In-degree calculation:");
        for (id, degree) in &in_degree {
            tracing::info!("   {} → in_degree={}", id, degree);
        }

        // Kahn's algorithm for topological sort
        let mut phases = Vec::new();
        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, degree)| **degree == 0)
            .map(|(id, _)| id.clone())
            .collect();

        while !queue.is_empty() {
            let mut current_phase = Vec::new();
            let phase_size = queue.len();

            for _ in 0..phase_size {
                if let Some(node_id) = queue.pop_front() {
                    current_phase.push(node_id.clone());

                    if let Some(dependents) = graph_map.get(&node_id) {
                        for dependent in dependents {
                            if let Some(degree) = in_degree.get_mut(dependent) {
                                *degree -= 1;
                                if *degree == 0 {
                                    queue.push_back(dependent.clone());
                                }
                            }
                        }
                    }
                }
            }

            if !current_phase.is_empty() {
                phases.push(current_phase);
            }
        }

        // Check for cycles
        if phases.iter().map(|p| p.len()).sum::<usize>() != self.graph.nodes.len() {
            anyhow::bail!("Graph contains cycles or unreachable nodes");
        }

        Ok(phases)
    }

    /// Rollback deployment
    ///
    /// Future Enhancement: Implement rollback strategy
    /// - Store checkpoints during execution
    /// - Reverse operations on failure
    /// - Restore previous state
    async fn rollback(&self) -> Result<()> {
        warn!("🔄 Rollback not yet implemented - graph execution is forward-only");
        Ok(())
    }
}

// =============================================================================
// Phase 2 Node Executors: verification
// =============================================================================

impl GraphExecutor {
    /// Node executor: verification
    /// Verifies primal health by checking sockets and optionally querying via JSON-RPC
    async fn node_verification(
        node: &GraphNode,
        context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        let check_sockets = node
            .config
            .get("check_sockets")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let check_health = node
            .config
            .get("check_health")
            .and_then(|v| v.as_bool())
            .unwrap_or(false); // Default false for Phase 2 (JSON-RPC query is expensive)

        info!("   Verifying ecosystem...");

        if check_sockets {
            // Get socket directory
            let _socket_dir = context
                .env
                .get("SOCKET_DIR")
                .ok_or_else(|| anyhow::anyhow!("SOCKET_DIR not set"))?;

            // Check that sockets exist for all dependencies
            let mut verified = Vec::new();
            for dep_id in &node.dependencies {
                // Get socket path from previous node output
                if let Some(dep_output) = context.get_output(dep_id).await {
                    if let Some(socket) = dep_output.get("socket").and_then(|v| v.as_str()) {
                        let socket_path = std::path::PathBuf::from(socket);
                        if socket_path.exists() {
                            info!("      ✅ {} socket exists", dep_id);
                            verified.push(dep_id.clone());
                        } else {
                            anyhow::bail!("Socket not found for {dep_id}: {socket}");
                        }
                    }
                }
            }

            info!("   ✅ Verified {} primals", verified.len());

            Ok(serde_json::json!({
                "verified_count": verified.len(),
                "verified_primals": verified,
                "check_sockets": true,
                "check_health": check_health
            }))
        } else {
            Ok(serde_json::json!({
                "verified_count": 0,
                "check_sockets": false
            }))
        }
    }

    /// Node executor: health.check_all
    /// Checks health of all primals by scanning socket directory
    async fn node_health_check_all(
        _node: &GraphNode,
        context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        let socket_dir = context
            .env
            .get("SOCKET_DIR")
            .ok_or_else(|| anyhow::anyhow!("SOCKET_DIR not set"))?;

        info!("   Checking health of all primals in {}", socket_dir);

        let socket_dir = PathBuf::from(socket_dir);
        let mut healthy_primals = Vec::new();

        if !socket_dir.exists() {
            warn!(
                "   Socket directory does not exist: {}",
                socket_dir.display()
            );
            return Ok(serde_json::json!({
                "healthy_count": 0,
                "primals": []
            }));
        }

        // Scan for .sock files
        let entries = std::fs::read_dir(&socket_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("sock") {
                if let Some(primal_name) = path.file_stem().and_then(|s| s.to_str()) {
                    healthy_primals.push(primal_name.to_string());
                }
            }
        }

        info!("   ✅ Found {} healthy primals", healthy_primals.len());

        Ok(serde_json::json!({
            "healthy_count": healthy_primals.len(),
            "primals": healthy_primals
        }))
    }

    // DEEP DEBT EVOLUTION (Feb 7, 2026): Removed legacy `request_jwt_secret_from_beardog`
    // and `generate_jwt_secret` functions. These are now properly implemented in the
    // `beardog_jwt_client` module with better separation of concerns:
    //   - `crate::beardog_jwt_client::provision_jwt_secret()` for JWT provisioning
    //   - `crate::beardog_jwt_client::generate_secure_random_jwt()` for fallback

    /// Node executor: rpc_call
    /// Makes a JSON-RPC call to a target primal
    ///
    /// NEW (Feb 6, 2026) - Allows graph nodes to orchestrate primal behavior
    /// Used for: onion.start, mesh.init, birdsong.advertise, etc.
    async fn node_rpc_call(
        node: &GraphNode,
        context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        use std::time::Duration;
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;

        // Get target primal from config
        let target = node
            .config
            .get("target")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("rpc_call requires 'target' config (primal name)"))?;

        // Get method name from config
        let method = node
            .config
            .get("method")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("rpc_call requires 'method' config"))?;

        // Get params from config (optional, default to empty object)
        let params = node
            .config
            .get("params")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));

        // Substitute environment variables in params
        let params_str = serde_json::to_string(&params)?;
        let params_expanded = crate::executor::substitute_env(&params_str, context.env());
        let params: serde_json::Value = serde_json::from_str(&params_expanded)?;

        info!("   📞 RPC call to {}: {}({:?})", target, method, params);

        // Get socket path for target primal
        let socket_path = context.get_socket_path(target).await;

        // Build JSON-RPC request
        let request = JsonRpcRequest::new(method, params);

        // Connect to primal
        let stream =
            tokio::time::timeout(Duration::from_secs(10), UnixStream::connect(&socket_path))
                .await
                .context(format!("Timeout connecting to {target} at {socket_path}"))?
                .context(format!("Failed to connect to {target} at {socket_path}"))?;

        let (read_half, mut write_half) = stream.into_split();

        // Send request
        let request_json = serde_json::to_string(&request)?;
        write_half.write_all(request_json.as_bytes()).await?;
        write_half.write_all(b"\n").await?;
        write_half.flush().await?;

        // Read response with timeout
        let mut reader = BufReader::new(read_half);
        let mut response_line = String::new();
        tokio::time::timeout(
            Duration::from_secs(30),
            reader.read_line(&mut response_line),
        )
        .await
        .context(format!("Timeout waiting for {target} response"))?
        .context(format!("Failed to read response from {target}"))?;

        let response: serde_json::Value = serde_json::from_str(&response_line)
            .context(format!("Invalid JSON response from {target}"))?;

        // Check for error
        if let Some(error) = response.get("error") {
            let error_msg = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            anyhow::bail!("RPC error from {target}: {error_msg}");
        }

        // Extract result
        let result = response
            .get("result")
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        info!("   ✅ RPC call successful: {} → {:?}", method, result);

        Ok(serde_json::json!({
            "target": target,
            "method": method,
            "result": result,
            "success": true
        }))
    }

    /// Node executor: capability_call
    /// Routes semantic capability calls through the neural-api or directly to primals.
    ///
    /// Graph nodes specify a `capability` (e.g. "ecology.et0_fao56") and `params`.
    /// This handler:
    /// 1. Tries routing via the neural-api `capability.call` JSON-RPC method
    /// 2. Falls back to direct primal socket resolution via `capability_domains`
    ///
    /// NEW (Mar 1, 2026) — Enables science pipeline graphs (science_pipeline.toml,
    /// neuralspring_spectral_pipeline.toml, airspring_ecology_pipeline.toml)
    async fn node_capability_call(
        node: &GraphNode,
        context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        use std::time::Duration;

        let capability = node
            .config
            .get("capability")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("capability_call requires 'capability' config"))?;

        let params = node
            .config
            .get("params")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));

        let params_str = serde_json::to_string(&params)?;
        let params_expanded = crate::executor::substitute_env(&params_str, context.env());
        let params: serde_json::Value = serde_json::from_str(&params_expanded)?;

        info!("   🔬 Capability call: {}({:?})", capability, params);

        let timeout_ms = node
            .config
            .get("timeout_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(30_000);

        // Split capability into (domain, operation) for capability.call semantics
        let (cap_domain, cap_operation) = Self::split_capability(capability);

        // Strategy 1: Route via neural-api capability.call
        let neural_api_socket = context.get_socket_path("neural-api").await;
        let neural_api_path = std::path::PathBuf::from(&neural_api_socket);

        if neural_api_path.exists() {
            let request = JsonRpcRequest::new(
                "capability.call",
                serde_json::json!({
                    "capability": &cap_domain,
                    "operation": &cap_operation,
                    "args": params,
                }),
            );

            match tokio::time::timeout(
                Duration::from_millis(timeout_ms),
                Self::send_jsonrpc_async(&neural_api_socket, &request),
            )
            .await
            {
                Ok(Ok(response)) => {
                    if let Some(error) = response.get("error") {
                        let msg = error
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("unknown");
                        warn!(
                            "   ⚠️ capability.call({}) via neural-api failed: {}, trying direct",
                            capability, msg
                        );
                    } else {
                        let result = response
                            .get("result")
                            .cloned()
                            .unwrap_or(serde_json::Value::Null);
                        info!(
                            "   ✅ Capability call via neural-api: {} → success",
                            capability
                        );
                        return Ok(serde_json::json!({
                            "capability": capability,
                            "routed_via": "neural-api",
                            "result": result,
                            "success": true,
                        }));
                    }
                }
                Ok(Err(e)) => {
                    warn!(
                        "   ⚠️ neural-api unreachable for {}: {}, trying direct",
                        capability, e
                    );
                }
                Err(_) => {
                    warn!(
                        "   ⚠️ neural-api timeout for {} ({}ms), trying direct",
                        capability, timeout_ms
                    );
                }
            }
        }

        // Strategy 2: Direct primal resolution via capability domains
        let provider = crate::capability_domains::capability_to_provider_fallback(capability)
            .or_else(|| crate::capability_domains::capability_to_provider_fallback(&cap_domain));

        let provider = provider.ok_or_else(|| {
            anyhow::anyhow!(
                "No provider found for capability '{capability}' (neither neural-api nor fallback)"
            )
        })?;

        info!(
            "   📞 Direct capability call: {} → {} ({})",
            capability, provider, cap_operation
        );

        let socket_path = context.get_socket_path(provider).await;

        let request = JsonRpcRequest::new(capability, params);

        let response = tokio::time::timeout(
            Duration::from_millis(timeout_ms),
            Self::send_jsonrpc_async(&socket_path, &request),
        )
        .await
        .context(format!("Timeout on capability call: {capability}"))?
        .context(format!(
            "Failed capability call {capability} → {provider} at {socket_path}"
        ))?;

        if let Some(error) = response.get("error") {
            let msg = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown");
            anyhow::bail!("Capability call {capability} failed: {msg}");
        }

        let result = response
            .get("result")
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        info!("   ✅ Direct capability call: {} → success", capability);

        Ok(serde_json::json!({
            "capability": capability,
            "routed_via": provider,
            "result": result,
            "success": true,
        }))
    }

    /// Helper: send a JSON-RPC request over a Unix socket and return the response.
    async fn send_jsonrpc_async(
        socket_path: &str,
        request: &impl Serialize,
    ) -> Result<serde_json::Value> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;

        let stream = UnixStream::connect(socket_path)
            .await
            .context(format!("Connecting to {socket_path}"))?;

        let (read_half, mut write_half) = stream.into_split();

        let payload = serde_json::to_string(request)?;
        write_half.write_all(payload.as_bytes()).await?;
        write_half.write_all(b"\n").await?;
        write_half.flush().await?;

        let mut reader = BufReader::new(read_half);
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        serde_json::from_str(line.trim()).context("Invalid JSON response")
    }
}
// Tests are in neural_executor_tests.rs to keep this file under 1000 lines
