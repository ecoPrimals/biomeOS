//! Graph executor for deterministic deployment orchestration
//!
//! This module executes Neural API graphs with:
//! - Topological sorting for dependency resolution
//! - Parallel execution within phases
//! - Checkpoint/rollback support
//! - Live monitoring and metrics

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::neural_graph::{Graph, GraphNode};

/// Execution status for a node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    Pending,
    Running,
    Completed(serde_json::Value),
    Failed(String),
    Skipped,
}

/// Execution context shared across nodes
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Node outputs (for dependency resolution)
    pub outputs: Arc<Mutex<HashMap<String, serde_json::Value>>>,
    /// Execution status of nodes
    pub status: Arc<Mutex<HashMap<String, NodeStatus>>>,
    /// Checkpoint directory
    pub checkpoint_dir: Option<PathBuf>,
}

impl ExecutionContext {
    /// Create new execution context
    pub fn new(env: HashMap<String, String>) -> Self {
        Self {
            env,
            outputs: Arc::new(Mutex::new(HashMap::new())),
            status: Arc::new(Mutex::new(HashMap::new())),
            checkpoint_dir: None,
        }
    }

    /// Set output for a node
    pub async fn set_output(&self, node_id: &str, value: serde_json::Value) {
        let mut outputs = self.outputs.lock().await;
        outputs.insert(node_id.to_string(), value);
    }

    /// Get output from a node
    pub async fn get_output(&self, node_id: &str) -> Option<serde_json::Value> {
        let outputs = self.outputs.lock().await;
        outputs.get(node_id).cloned()
    }

    /// Set node status
    pub async fn set_status(&self, node_id: &str, status: NodeStatus) {
        let mut statuses = self.status.lock().await;
        statuses.insert(node_id.to_string(), status);
    }

    /// Get node status
    pub async fn get_status(&self, node_id: &str) -> Option<NodeStatus> {
        let statuses = self.status.lock().await;
        statuses.get(node_id).cloned()
    }
}

/// Graph executor
pub struct GraphExecutor {
    graph: Graph,
    context: ExecutionContext,
    max_parallelism: usize,
}

impl GraphExecutor {
    /// Create new graph executor
    pub fn new(graph: Graph, env: HashMap<String, String>) -> Self {
        Self {
            graph,
            context: ExecutionContext::new(env),
            max_parallelism: 3, // Default from graph spec
        }
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
                Ok(phase_results) => {
                    report.phase_results.push(phase_results);
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

        for node_id in nodes {
            let node = self
                .graph
                .nodes
                .iter()
                .find(|n| &n.id == node_id)
                .ok_or_else(|| anyhow::anyhow!("Node not found: {}", node_id))?
                .clone();

            let context = self.context.clone();
            let permit = semaphore.clone().acquire_owned().await?;

            let handle = tokio::spawn(async move {
                let result = Self::execute_node(&node, &context).await;
                drop(permit); // Release semaphore
                (node.id.clone(), result)
            });

            handles.push(handle);
        }

        // Wait for all nodes to complete
        for handle in handles {
            let (node_id, result) = handle.await?;

            match result {
                Ok(output) => {
                    phase_result.completed += 1;
                    self.context
                        .set_status(&node_id, NodeStatus::Completed(output.clone()))
                        .await;
                    self.context.set_output(&node_id, output).await;
                }
                Err(e) => {
                    phase_result.failed += 1;
                    let error_msg = e.to_string();
                    self.context
                        .set_status(&node_id, NodeStatus::Failed(error_msg.clone()))
                        .await;
                    phase_result.errors.push((node_id, error_msg));
                }
            }
        }

        phase_result.duration_ms = phase_start.elapsed().as_millis() as u64;

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
    async fn execute_node(
        node: &GraphNode,
        context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
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

        // Execute based on node type
        let result = match node_type_str {
            "filesystem.check_exists" => Self::node_filesystem_check_exists(node, context).await,
            "crypto.derive_child_seed" => Self::node_crypto_derive_seed(node, context).await,
            "primal.launch" => Self::node_primal_launch(node, context).await,
            "primal_start" => Self::node_primal_start(node, context).await, // NEW: Phase 2
            "verification" => Self::node_verification(node, context).await, // NEW: Phase 2
            "health.check" => Self::node_health_check(node, context).await,
            "health.check_atomic" => Self::node_health_check(node, context).await,
            "health.check_all" => Self::node_health_check_all(node, context).await,
            "lineage.verify_siblings" => Self::node_lineage_verify(node, context).await,
            "report.deployment_success" => Self::node_deployment_report(node, context).await,
            "log.info" => Self::node_log_info(node, context).await,
            "log.warn" => Self::node_log_warn(node, context).await,
            "log.error" => Self::node_log_error(node, context).await,
            _ => {
                warn!("Unknown node type: {}, skipping", node_type_str);
                Ok(serde_json::json!({"skipped": true}))
            }
        };

        result.context(format!("Node execution failed: {}", node.id))
    }

    /// Node executor: filesystem.check_exists
    async fn node_filesystem_check_exists(
        node: &GraphNode,
        context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        let path = node
            .config
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' in config"))?;

        // Substitute environment variables
        let path = Self::substitute_env(path, &context.env);
        let path = PathBuf::from(path);

        if !path.exists() {
            anyhow::bail!("Path does not exist: {}", path.display());
        }

        // Check size if specified
        if let Some(expected_size) = node.config.get("expected_size").and_then(|v| v.as_u64()) {
            let metadata = std::fs::metadata(&path)?;
            if metadata.len() != expected_size {
                anyhow::bail!(
                    "File size mismatch: expected {}, got {}",
                    expected_size,
                    metadata.len()
                );
            }
        }

        Ok(serde_json::json!({
            "exists": true,
            "path": path.to_string_lossy()
        }))
    }

    /// Node executor: crypto.derive_child_seed
    async fn node_crypto_derive_seed(
        node: &GraphNode,
        context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        use biomeos_spore::seed::FamilySeed;

        let parent_seed = node
            .config
            .get("parent_seed")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'parent_seed'"))?;
        let parent_seed = Self::substitute_env(parent_seed, &context.env);

        let node_id = node
            .config
            .get("node_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'node_id'"))?;

        let output_path = node
            .config
            .get("output_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'output_path'"))?;
        let output_path = Self::substitute_env(output_path, &context.env);

        let deployment_batch = node
            .config
            .get("deployment_batch")
            .and_then(|v| v.as_str())
            .map(|s| Self::substitute_env(s, &context.env));

        // Derive child seed
        FamilySeed::derive_sibling(
            PathBuf::from(parent_seed),
            PathBuf::from(&output_path),
            node_id,
            deployment_batch.as_deref(),
        )?;

        Ok(serde_json::json!({
            "derived": true,
            "output_path": output_path
        }))
    }

    /// Node executor: primal.launch
    async fn node_primal_launch(
        node: &GraphNode,
        context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        info!("   🟢 node_primal_launch called for: {}", node.id);
        info!("   🔀 Delegating to node_primal_start...");
        // Delegate to the full primal_start implementation
        let result = Self::node_primal_start(node, context).await;
        info!(
            "   🟢 node_primal_launch result for {}: {:?}",
            node.id,
            result.is_ok()
        );
        result
    }

    /// Node executor: health.check_atomic
    async fn node_health_check(
        node: &GraphNode,
        _context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        // Placeholder for health checking
        Ok(serde_json::json!({
            "healthy": true,
            "atomic": node.config.get("atomic_type")
        }))
    }

    /// Node executor: lineage.verify_siblings
    async fn node_lineage_verify(
        node: &GraphNode,
        _context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        // Placeholder for lineage verification
        Ok(serde_json::json!({
            "verified": true,
            "siblings": true
        }))
    }

    /// Node executor: report.deployment_success
    async fn node_deployment_report(
        node: &GraphNode,
        context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        let atomics = node
            .config
            .get("atomics_deployed")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();

        Ok(serde_json::json!({
            "success": true,
            "atomics_deployed": atomics,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    /// Substitute environment variables in a string
    fn substitute_env(s: &str, env: &HashMap<String, String>) -> String {
        let mut result = s.to_string();

        for (key, value) in env {
            let placeholder = format!("${{{}}}", key);
            result = result.replace(&placeholder, value);
        }

        result
    }

    /// Perform topological sort to determine execution phases
    fn topological_sort(&self) -> Result<Vec<Vec<String>>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut graph_map: HashMap<String, Vec<String>> = HashMap::new();

        // Build adjacency list and in-degree map
        for node in &self.graph.nodes {
            in_degree.entry(node.id.clone()).or_insert(0);

            for dep in &node.dependencies {
                graph_map
                    .entry(dep.clone())
                    .or_insert_with(Vec::new)
                    .push(node.id.clone());
                *in_degree.entry(node.id.clone()).or_insert(0) += 1;
            }
        }

        // Kahn's algorithm for topological sort
        let mut phases = Vec::new();
        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
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
    async fn rollback(&self) -> Result<()> {
        warn!("🔄 Rollback not yet implemented");
        // TODO: Implement rollback strategy
        Ok(())
    }
}

/// Execution report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub graph_id: String,
    pub success: bool,
    pub duration_ms: u64,
    pub phase_results: Vec<PhaseResult>,
    pub error: Option<String>,
}

impl ExecutionReport {
    fn new(graph_id: String) -> Self {
        Self {
            graph_id,
            success: true,
            duration_ms: 0,
            phase_results: Vec::new(),
            error: None,
        }
    }
}

// =============================================================================
// Phase 2 Node Executors: primal_start & verification
// =============================================================================

impl GraphExecutor {
    /// Node executor: primal_start
    /// Spawns a primal binary as a child process with environment configuration
    async fn node_primal_start(
        node: &GraphNode,
        context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        use std::process::Stdio;
        use std::time::Duration;
        use tokio::process::Command;

        info!("   🔵 Starting node_primal_start for: {}", node.id);
        info!("   📋 Node config: {:?}", node.config);

        // Get binary path from config (try both 'binary_path' and 'binary')
        let binary = node
            .config
            .get("binary_path")
            .or_else(|| node.config.get("binary"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                error!(
                    "Config keys available: {:?}",
                    node.config.keys().collect::<Vec<_>>()
                );
                anyhow::anyhow!("Missing 'binary_path' or 'binary' in config")
            })?;
        let binary = Self::substitute_env(binary, &context.env);
        info!("   🔍 Found binary: {}", binary);

        // Get family_id from config
        let family_id = node
            .config
            .get("family_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'family_id' in config"))?;
        let family_id = Self::substitute_env(family_id, &context.env);

        // Get socket path from config (try both 'socket_path' and 'socket')
        let socket = node
            .config
            .get("socket_path")
            .or_else(|| node.config.get("socket"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'socket_path' or 'socket' in config"))?;
        let socket = Self::substitute_env(socket, &context.env);

        info!("   Starting primal: {} (family: {})", node.id, family_id);
        info!("   Binary: {}", binary);
        info!("   Socket: {}", socket);

        // Check if binary exists
        if !tokio::fs::metadata(&binary).await.is_ok() {
            anyhow::bail!("Binary not found: {}", binary);
        }

        // Clean old socket if exists
        let socket_path = std::path::PathBuf::from(&socket);
        if socket_path.exists() {
            tokio::fs::remove_file(&socket_path).await.ok();
        }

        // Create log directory
        std::fs::create_dir_all("/tmp/primals").ok();
        let log_path = format!("/tmp/primals/{}-{}.log", node.id, family_id);

        // Build command
        let mut cmd = Command::new(&binary);

        // Add arguments if specified
        if let Some(args_array) = node.config.get("args").and_then(|v| v.as_array()) {
            for arg in args_array {
                if let Some(arg_str) = arg.as_str() {
                    cmd.arg(arg_str);
                }
            }
        }

        // Set environment variables
        cmd.env("BIOMEOS_FAMILY_ID", &family_id);
        cmd.env("BIOMEOS_SOCKET_PATH", &socket);

        // Add primal-specific variants (backward compat)
        // Use primal_name if available, otherwise fall back to node.id
        let primal_for_env = node
            .config
            .get("primal_name")
            .and_then(|v| v.as_str())
            .unwrap_or(&node.id);
        let primal_upper = primal_for_env.to_uppercase().replace("-", "_");

        // Pass socket path with BOTH primal-specific AND generic names
        // This ensures primals like ToadStool can find their socket path
        cmd.env(format!("{}_SOCKET", primal_upper), &socket);
        cmd.env(format!("{}_SOCKET_PATH", primal_upper), &socket); // Also set with _PATH suffix
        cmd.env(format!("{}_FAMILY", primal_upper), &family_id);
        cmd.env(format!("{}_FAMILY_ID", primal_upper), &family_id);

        info!("   🔧 Environment variables set:");
        info!("      BIOMEOS_FAMILY_ID: {}", family_id);
        info!("      BIOMEOS_SOCKET_PATH: {}", socket);
        info!("      {}_SOCKET: {}", primal_upper, socket);
        info!("      {}_FAMILY: {}", primal_upper, family_id);

        // Add security_provider for primals that need it (e.g., Songbird, NestGate)
        if let Some(security_provider) = node
            .config
            .get("security_provider")
            .and_then(|v| v.as_str())
        {
            let security_provider = Self::substitute_env(security_provider, &context.env);

            // Set generic security endpoint
            cmd.env("SECURITY_ENDPOINT", &security_provider);

            // Add primal-specific variants
            cmd.env("SONGBIRD_SECURITY_PROVIDER", &security_provider);
            cmd.env("NESTGATE_SECURITY_PROVIDER", &security_provider);

            // Request JWT_SECRET from BearDog for primals that need it
            // This is TRUE PRIMAL: runtime capability-based secret management
            if primal_for_env.contains("nestgate") || primal_for_env.contains("NESTGATE") {
                info!("   🔐 Requesting JWT_SECRET from BearDog security provider...");

                // Use new beardog_jwt_client module (cleaner implementation)
                let jwt_purpose = format!("{}_authentication", primal_for_env);
                match crate::beardog_jwt_client::provision_jwt_secret(
                    Some(&security_provider),
                    &jwt_purpose,
                )
                .await
                {
                    Ok(jwt_secret) => {
                        info!(
                            "   ✅ Received JWT_SECRET from BearDog ({} bytes)",
                            jwt_secret.len()
                        );
                        cmd.env("JWT_SECRET", jwt_secret.clone());
                        cmd.env("NESTGATE_JWT_SECRET", jwt_secret);
                    }
                    Err(e) => {
                        warn!("   ⚠️ Failed to provision JWT_SECRET: {}. This will block NestGate startup!", e);
                        return Err(e.context("JWT provisioning failed for NestGate"));
                    }
                }
            }
        }

        // Redirect stdio to log file
        let log_file = std::fs::File::create(&log_path)?;
        cmd.stdout(Stdio::from(log_file.try_clone()?));
        cmd.stderr(Stdio::from(log_file));
        cmd.stdin(Stdio::null());

        // Spawn process
        let mut child = cmd
            .spawn()
            .context(format!("Failed to spawn {}", node.id))?;

        let pid = child.id().unwrap_or(0);
        info!("   ✅ Spawned {} (PID: {})", node.id, pid);

        // Modern async: Check if process crashes immediately using timeout
        let crash_check = async {
            let mut interval = tokio::time::interval(Duration::from_millis(50));
            for _ in 0..6 {
                // Check 6 times over 300ms
                interval.tick().await;
                match child.try_wait()? {
                    Some(status) => {
                        anyhow::bail!(
                            "Process {} exited immediately with status: {}. Check log: {}",
                            node.id,
                            status,
                            log_path
                        );
                    }
                    None => continue,
                }
            }
            Ok::<_, anyhow::Error>(())
        };

        crash_check.await?;
        info!("   ✅ {} running (log: {})", node.id, log_path);
        // Don't wait - let it run in background
        std::mem::forget(child);

        // Modern async: Wait for socket with exponential backoff
        let socket_timeout = Duration::from_secs(10);
        let mut interval = tokio::time::interval(Duration::from_millis(100));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let socket_wait = async {
            loop {
                interval.tick().await;
                if socket_path.exists() {
                    return Ok::<(), anyhow::Error>(());
                }
            }
        };

        match tokio::time::timeout(socket_timeout, socket_wait).await {
            Ok(_) => {
                info!("   ✅ Socket created: {}", socket);
                Ok(serde_json::json!({
                    "primal": node.id,
                    "pid": pid,
                    "socket": socket,
                    "log": log_path,
                    "status": "running"
                }))
            }
            Err(_) => {
                anyhow::bail!(
                    "Socket not created after {}s: {}",
                    socket_timeout.as_secs(),
                    socket
                )
            }
        }
    }

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
            let socket_dir = context
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
                            anyhow::bail!("Socket not found for {}: {}", dep_id, socket);
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

    /// Node executor: log.info
    async fn node_log_info(
        node: &GraphNode,
        _context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        let message = node
            .config
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("(no message)");

        info!("📢 {}", message);

        Ok(serde_json::json!({
            "logged": true,
            "level": "info",
            "message": message
        }))
    }

    /// Node executor: log.warn
    async fn node_log_warn(
        node: &GraphNode,
        _context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        let message = node
            .config
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("(no message)");

        warn!("⚠️  {}", message);

        Ok(serde_json::json!({
            "logged": true,
            "level": "warn",
            "message": message
        }))
    }

    /// Node executor: log.error
    async fn node_log_error(
        node: &GraphNode,
        _context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        let message = node
            .config
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("(no message)");

        error!("❌ {}", message);

        Ok(serde_json::json!({
            "logged": true,
            "level": "error",
            "message": message
        }))
    }

    /// Find BearDog socket from execution context
    /// Used for JWT provisioning and other security capabilities
    async fn find_beardog_socket(context: &ExecutionContext) -> Option<String> {
        // Try to get from outputs first (from launch_beardog node)
        if let Some(beardog_output) = context.get_output("launch_beardog").await {
            if let Some(socket) = beardog_output.get("socket").and_then(|v| v.as_str()) {
                return Some(socket.to_string());
            }
        }

        // Try standard location
        let default_socket = "/tmp/beardog-nat0.sock";
        if tokio::fs::metadata(default_socket).await.is_ok() {
            return Some(default_socket.to_string());
        }

        None
    }

    /// Request JWT_SECRET from BearDog security provider
    /// This is TRUE PRIMAL: runtime capability-based secret management
    ///
    /// NOTE: This is the legacy implementation. New code should use
    /// beardog_jwt_client module for better separation of concerns.
    async fn request_jwt_secret_from_beardog(beardog_socket: &str) -> Result<String> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;
        use tokio::time::{timeout, Duration};

        info!("   🔐 Connecting to BearDog at: {}", beardog_socket);

        // Connect to BearDog with timeout
        let stream = timeout(Duration::from_secs(5), UnixStream::connect(beardog_socket))
            .await
            .context("Timeout connecting to BearDog")?
            .context(format!(
                "Failed to connect to BearDog at {}",
                beardog_socket
            ))?;

        let (read_half, mut write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);

        // JSON-RPC request to generate JWT secret
        // BearDog should have a "generate_secret" or "derive_jwt_secret" method
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "beardog.generate_jwt_secret",
            "params": {
                "purpose": "nestgate_authentication",
                "strength": "high"
            },
            "id": 1
        });

        let request_str = serde_json::to_string(&request)?;
        info!("   📤 Sending request to BearDog: {}", request_str);

        // Send request with newline delimiter
        write_half.write_all(request_str.as_bytes()).await?;
        write_half.write_all(b"\n").await?;
        write_half.flush().await?;

        // Read response with timeout
        let mut response_line = String::new();
        timeout(Duration::from_secs(5), reader.read_line(&mut response_line))
            .await
            .context("Timeout waiting for BearDog response")?
            .context("Failed to read response from BearDog")?;

        info!("   📥 Received from BearDog: {}", response_line.trim());

        // Parse JSON-RPC response
        let response: serde_json::Value = serde_json::from_str(&response_line)
            .context("Failed to parse BearDog response as JSON")?;

        // Extract secret from response
        if let Some(error) = response.get("error") {
            anyhow::bail!("BearDog returned error: {}", error);
        }

        if let Some(result) = response.get("result") {
            if let Some(secret) = result.get("secret").and_then(|s| s.as_str()) {
                return Ok(secret.to_string());
            } else if let Some(secret) = result.as_str() {
                return Ok(secret.to_string());
            }
        }

        anyhow::bail!("BearDog response did not contain a valid secret")
    }

    /// Generate a secure JWT_SECRET as fallback
    /// Used when BearDog is not available
    fn generate_jwt_secret() -> String {
        use rand::Rng;

        info!("   🔐 Generating secure fallback JWT_SECRET...");

        // Generate 64 bytes of cryptographically secure random data
        let mut rng = rand::thread_rng();
        let secret: Vec<u8> = (0..64).map(|_| rng.gen()).collect();

        // Base64 encode for use as JWT secret
        use base64::{engine::general_purpose, Engine as _};
        general_purpose::STANDARD.encode(&secret)
    }
}

/// Phase execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseResult {
    pub total_nodes: usize,
    pub completed: usize,
    pub failed: usize,
    pub duration_ms: u64,
    pub errors: Vec<(String, String)>,
}

impl PhaseResult {
    fn new(total_nodes: usize) -> Self {
        Self {
            total_nodes,
            completed: 0,
            failed: 0,
            duration_ms: 0,
            errors: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_substitution() {
        let mut env = HashMap::new();
        env.insert("FOO".to_string(), "bar".to_string());
        env.insert("BAZ".to_string(), "qux".to_string());

        let result = GraphExecutor::substitute_env("${FOO}/${BAZ}/test", &env);
        assert_eq!(result, "bar/qux/test");
    }
}
