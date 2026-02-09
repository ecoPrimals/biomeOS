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
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info, warn};

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

            // Unknown
            _ => {
                warn!("Unknown node type: {}, skipping", node_type_str);
                Ok(serde_json::json!({"skipped": true}))
            }
        };

        result.context(format!("Node execution failed: {}", node.id))
    }

    /// Substitute environment variables in a string
    #[allow(dead_code)] // Reserved for graph environment variable expansion
    pub(crate) fn substitute_env(s: &str, env: &HashMap<String, String>) -> String {
        let mut result = s.to_string();

        for (key, value) in env {
            let placeholder = format!("${{{}}}", key);
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
// Phase 2 Node Executors: primal_start & verification
// =============================================================================

impl GraphExecutor {
    /// Node executor: primal_start
    /// Spawns a primal binary as a child process with environment configuration
    #[allow(dead_code)] // Reserved for graph-based primal spawning
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
        if tokio::fs::metadata(&binary).await.is_err() {
            anyhow::bail!("Binary not found: {}", binary);
        }

        // Clean old socket if exists
        let socket_path = std::path::PathBuf::from(&socket);
        if socket_path.exists() {
            tokio::fs::remove_file(&socket_path).await.ok();
        }

        // Create log directory (use runtime dir, not hardcoded /tmp/)
        let runtime_dir = std::env::var("BIOMEOS_RUNTIME_DIR")
            .or_else(|_| std::env::var("TMPDIR"))
            .unwrap_or_else(|_| "/tmp".to_string());
        let log_dir = format!("{}/primals", runtime_dir);
        std::fs::create_dir_all(&log_dir).ok();
        let log_path = format!("{}/{}-{}.log", log_dir, node.id, family_id);

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

    /// Find security provider socket from execution context
    ///
    /// DEEP DEBT EVOLUTION: Resolves security provider by capability, not name.
    /// Uses graph output first, then env override, then nucleation fallback.
    #[allow(dead_code)] // Used conditionally based on security features
    async fn find_security_socket(context: &ExecutionContext) -> Option<String> {
        use crate::nucleation::SocketNucleation;

        // 1. Try to get from graph execution outputs (capability-based)
        for node_name in &["launch_security", "launch_beardog"] {
            if let Some(output) = context.get_output(node_name).await {
                if let Some(socket) = output.get("socket").and_then(|v| v.as_str()) {
                    return Some(socket.to_string());
                }
            }
        }

        // 2. Try env override (DEEP DEBT: configurable, not hardcoded)
        if let Ok(socket) = std::env::var("BIOMEOS_SECURITY_SOCKET") {
            if tokio::fs::metadata(&socket).await.is_ok() {
                return Some(socket);
            }
        }

        // 3. Fall back to nucleation with resolved provider name
        let provider = std::env::var("BIOMEOS_SECURITY_PROVIDER")
            .unwrap_or_else(|_| "beardog".to_string());
        let mut nucleation = SocketNucleation::default();
        let default_socket = nucleation.assign_socket(&provider, &context.family_id);
        if tokio::fs::metadata(&default_socket).await.is_ok() {
            return Some(default_socket.to_string_lossy().into_owned());
        }

        None
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
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;
        use std::time::Duration;

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
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        // Connect to primal
        let stream = tokio::time::timeout(
            Duration::from_secs(10),
            UnixStream::connect(&socket_path),
        )
        .await
        .context(format!("Timeout connecting to {} at {}", target, socket_path))?
        .context(format!("Failed to connect to {} at {}", target, socket_path))?;

        let (read_half, mut write_half) = stream.into_split();

        // Send request
        let request_json = serde_json::to_string(&request)?;
        write_half.write_all(request_json.as_bytes()).await?;
        write_half.write_all(b"\n").await?;
        write_half.flush().await?;

        // Read response with timeout
        let mut reader = BufReader::new(read_half);
        let mut response_line = String::new();
        tokio::time::timeout(Duration::from_secs(30), reader.read_line(&mut response_line))
            .await
            .context(format!("Timeout waiting for {} response", target))?
            .context(format!("Failed to read response from {}", target))?;

        let response: serde_json::Value = serde_json::from_str(&response_line)
            .context(format!("Invalid JSON response from {}", target))?;

        // Check for error
        if let Some(error) = response.get("error") {
            let error_msg = error.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error");
            anyhow::bail!("RPC error from {}: {}", target, error_msg);
        }

        // Extract result
        let result = response.get("result").cloned().unwrap_or(serde_json::Value::Null);

        info!("   ✅ RPC call successful: {} → {:?}", method, result);

        Ok(serde_json::json!({
            "target": target,
            "method": method,
            "result": result,
            "success": true
        }))
    }
}
// Tests are in neural_executor_tests.rs to keep this file under 1000 lines
