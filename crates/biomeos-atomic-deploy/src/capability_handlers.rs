//! Capability-based primal handlers
//!
//! This module contains the handlers for capability-based primal operations,
//! extracted from neural_executor for better organization and maintainability.
//!
//! ## Deep Debt Principles
//!
//! - Capability-based discovery (runtime resolution)
//! - No hardcoded primal names where possible
//! - Socket nucleation for deterministic paths
//! - Environment-driven configuration

use anyhow::Result;
use serde_json::json;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn};

use crate::executor::context::ExecutionContext;
use crate::neural_graph::GraphNode;

/// Resolve capability to primal name using the capability taxonomy
///
/// Uses `biomeos_types::CapabilityTaxonomy` for consistent mapping across the codebase.
/// Set `BIOMEOS_STRICT_DISCOVERY=1` to disable fallback and require Songbird discovery.
fn resolve_capability_to_primal(capability: &str) -> Option<&'static str> {
    biomeos_types::CapabilityTaxonomy::resolve_to_primal(capability)
}

/// Get all known primal names from the capability taxonomy
///
/// This provides a canonical list of primals for health checks and deployment.
/// Uses `biomeos_types::CapabilityTaxonomy::known_primals()` for consistency.
fn known_primal_names() -> Vec<&'static str> {
    biomeos_types::CapabilityTaxonomy::known_primals().to_vec()
}

/// Start a primal via capability-based discovery
///
/// This is the capability-based alternative to direct primal name starting.
/// It resolves capabilities to primals at runtime.
pub async fn primal_start_capability(
    node: &GraphNode,
    context: &ExecutionContext,
) -> Result<serde_json::Value> {
    info!("🚀 Starting primal via capability-based discovery");

    // Extract capability and operation parameters
    let capability = node
        .primal
        .as_ref()
        .and_then(|p| p.by_capability.as_ref())
        .ok_or_else(|| anyhow::anyhow!("Missing primal.by_capability"))?;

    let operation = node
        .operation
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Missing operation"))?;

    let params = &operation.params;
    let mode = params
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("server");

    let family_id = params
        .get("family_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(biomeos_core::family_discovery::get_family_id);

    debug!("   Capability: {}", capability);
    debug!("   Mode: {}", mode);
    debug!("   Family ID: {}", family_id);

    // 1. Capability → Primal Name Discovery
    let primal_name = match resolve_capability_to_primal(capability) {
        Some(name) => name,
        None => {
            warn!("Unknown capability '{}', skipping", capability);
            return Ok(json!({
                "started": false,
                "capability": capability,
                "error": format!("Unknown capability: {}", capability)
            }));
        }
    };

    // 2. Discover binary path
    let binary_full_path = match discover_primal_binary(primal_name, context).await {
        Ok(path) => path,
        Err(e) => {
            warn!("   Binary discovery failed for {}: {}", primal_name, e);
            return Ok(json!({
                "started": false,
                "capability": capability,
                "primal": primal_name,
                "error": format!("Binary not found: {}", e)
            }));
        }
    };

    info!(
        "   Discovered: {} → {}",
        primal_name,
        binary_full_path.display()
    );

    // 3. Build socket path using nucleation
    let socket_path = context.get_socket_path(primal_name).await;

    // 4. Build command with primal-specific arguments
    let mut cmd = tokio::process::Command::new(&binary_full_path);
    cmd.arg(mode);

    // Configure primal-specific socket handling
    configure_primal_socket(&mut cmd, primal_name, &socket_path, &family_id, context).await;

    cmd.env("FAMILY_ID", &family_id);

    // Pass SSLKEYLOGFILE if set (for TLS debugging)
    if let Ok(sslkeylogfile) = std::env::var("SSLKEYLOGFILE") {
        if !sslkeylogfile.is_empty() {
            cmd.env("SSLKEYLOGFILE", &sslkeylogfile);
            info!("   🔐 Passing SSLKEYLOGFILE to primal: {}", sslkeylogfile);
        }
    }

    // Capture stdout/stderr for visibility
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    info!(
        "   Starting: {} {} (socket: {})",
        primal_name, mode, socket_path
    );

    // Pass environment variables from graph TOML
    if let Some(ref operation) = node.operation {
        if let Some(ref env_map) = operation.environment {
            info!(
                "   🔧 Passing {} environment variables to primal",
                env_map.len()
            );
            for (key, value) in env_map {
                info!(
                    "   Setting env: {}={}",
                    key,
                    if key.contains("KEY") { "***" } else { value }
                );
                cmd.env(key, value);
            }
        }
    }

    // 5. Start process
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            error!("   Failed to spawn process: {}", e);
            return Ok(json!({
                "started": false,
                "capability": capability,
                "primal": primal_name,
                "error": format!("Failed to spawn: {}", e)
            }));
        }
    };

    let pid = child.id().unwrap_or(0);
    info!("   Process started: PID {}", pid);

    // 6. Relay stdout/stderr to logs
    spawn_output_relays(&mut child, primal_name);

    // 7. Wait for socket
    let socket_confirmed = wait_for_socket_with_timeout(&socket_path, 30).await;

    if socket_confirmed {
        info!("   ✅ Socket available: {}", socket_path);

        // Log capability registration (actual registration via RPC)
        if !node.capabilities.is_empty() {
            info!(
                "   📝 Registering {} capabilities...",
                node.capabilities.len()
            );
            for cap in &node.capabilities {
                info!("      - {} → {} @ {}", cap, primal_name, socket_path);
            }
        }

        Ok(json!({
            "started": true,
            "capability": capability,
            "primal": primal_name,
            "mode": mode,
            "family_id": family_id,
            "pid": pid,
            "socket": socket_path,
            "socket_confirmed": true
        }))
    } else {
        warn!("   ⚠️  Socket not found after 3s: {}", socket_path);

        Ok(json!({
            "started": true,
            "capability": capability,
            "primal": primal_name,
            "mode": mode,
            "family_id": family_id,
            "pid": pid,
            "socket": socket_path,
            "socket_confirmed": false,
            "warning": "Socket not detected within 3 seconds"
        }))
    }
}

/// Configure primal-specific socket handling
async fn configure_primal_socket(
    cmd: &mut tokio::process::Command,
    primal_name: &str,
    socket_path: &str,
    family_id: &str,
    context: &ExecutionContext,
) {
    match primal_name {
        "beardog" => {
            // BearDog: GOLD STANDARD - uses CLI flags
            cmd.arg("--socket").arg(socket_path);
            cmd.arg("--family-id").arg(family_id);
        }
        "squirrel" => {
            // Squirrel: Uses --socket CLI flag
            cmd.arg("--socket").arg(socket_path);
            let neural_api_socket = context.get_socket_path("neural-api").await;
            cmd.env("SERVICE_MESH_ENDPOINT", neural_api_socket);
        }
        "songbird" => {
            // Songbird v3.33.0: CLI flags + environment variables
            // Validated Jan 28, 2026 - matches successful manual startup
            cmd.arg("--socket").arg(socket_path);

            // Bond to BearDog for security (TLS crypto delegation)
            let beardog_socket = context.get_socket_path("beardog").await;
            cmd.arg("--beardog-socket").arg(&beardog_socket);

            // Environment variables for Songbird configuration
            cmd.env("BEARDOG_MODE", "direct"); // Direct RPC to BearDog (Neural API adds routing later)
            cmd.env("BEARDOG_SOCKET", &beardog_socket);
            cmd.env("SONGBIRD_SECURITY_PROVIDER", "beardog"); // Provider name, not socket path!
            cmd.env("FAMILY_ID", family_id);

            // Neural API socket if available (enables capability.call routing)
            let neural_api_socket = context.get_socket_path("neural-api").await;
            cmd.env("NEURAL_API_SOCKET", &neural_api_socket);

            info!("   🧬 Bonding Songbird → BearDog: {}", beardog_socket);
            info!("   🧠 Neural API: {}", neural_api_socket);
        }
        "nestgate" | "toadstool" => {
            // Generic: use CLI flags (follow BearDog pattern)
            cmd.arg("--socket").arg(socket_path);
            cmd.arg("--family-id").arg(family_id);
        }
        _ => {
            // Unknown: try both methods
            cmd.arg("--socket").arg(socket_path);
            cmd.env("PRIMAL_SOCKET", socket_path);
        }
    }
}

/// Spawn tasks to relay child stdout/stderr to tracing
fn spawn_output_relays(child: &mut tokio::process::Child, primal_name: &str) {
    let stdout_name = primal_name.to_string();
    let stderr_name = primal_name.to_string();

    if let Some(stdout) = child.stdout.take() {
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                info!("[{}] {}", stdout_name, line);
            }
        });
    }

    if let Some(stderr) = child.stderr.take() {
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                warn!("[{}] {}", stderr_name, line);
            }
        });
    }
}

/// Wait for socket with timeout
async fn wait_for_socket_with_timeout(socket_path: &str, attempts: u32) -> bool {
    for attempt in 1..=attempts {
        if PathBuf::from(socket_path).exists() {
            debug!("   Socket available after {}00ms", attempt);
            return true;
        }
        sleep(Duration::from_millis(100)).await;
    }
    false
}

/// Health check for capability-based deployment
pub async fn health_check_capability(
    node: &GraphNode,
    context: &ExecutionContext,
) -> Result<serde_json::Value> {
    info!("🏥 Health check for capability-based deployment");

    let operation = node
        .operation
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Missing operation"))?;

    let params = &operation.params;

    debug!("   Health check params: {:?}", params);

    let family_id = params
        .get("family_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(biomeos_core::family_discovery::get_family_id);

    let mut checks_passed = Vec::new();
    let mut checks_failed = Vec::new();

    // Determine which primals to check
    let primals_to_check = if let Some(ref primal) = node.primal {
        if let Some(ref capability) = primal.by_capability {
            // Check specific capability
            resolve_capability_to_primal(capability)
                .map(|p| vec![p])
                .unwrap_or_default()
        } else {
            vec![]
        }
    } else {
        // Check all known primals (from capability domains)
        // See: config/capability_registry.toml for domain definitions
        known_primal_names()
    };

    // Check each primal
    for primal_name in primals_to_check {
        let socket_path = context.get_socket_path(primal_name).await;

        if PathBuf::from(&socket_path).exists() {
            // Try JSON-RPC health check
            match call_primal_health(&socket_path).await {
                Ok(healthy) if healthy => {
                    info!("   ✅ {} @ {} - healthy", primal_name, socket_path);
                    checks_passed.push(primal_name.to_string());
                }
                Ok(_) => {
                    warn!("   ⚠️  {} @ {} - unhealthy", primal_name, socket_path);
                    checks_failed.push(primal_name.to_string());
                }
                Err(e) => {
                    warn!(
                        "   ⚠️  {} @ {} - RPC failed: {}",
                        primal_name, socket_path, e
                    );
                    // Socket exists but RPC failed - could be starting up
                    checks_failed.push(primal_name.to_string());
                }
            }
        } else {
            debug!("   ⏸️  {} - socket not found", primal_name);
            // Not running - not necessarily a failure
        }
    }

    let all_healthy = checks_failed.is_empty() && !checks_passed.is_empty();

    if all_healthy {
        info!(
            "   ✅ All health checks passed ({} checks)",
            checks_passed.len()
        );
    } else if checks_failed.is_empty() {
        info!("   ⏸️  No primals running to check");
    } else {
        warn!("   ⚠️  Some health checks failed: {:?}", checks_failed);
    }

    Ok(json!({
        "healthy": all_healthy,
        "family_id": family_id,
        "checks_passed": checks_passed,
        "checks_failed": checks_failed,
        "total_checks": checks_passed.len() + checks_failed.len()
    }))
}

/// Call primal health endpoint via JSON-RPC using AtomicClient
///
/// Uses Universal IPC v3.0 AtomicClient for multi-transport support.
/// This enables Unix sockets, abstract sockets (Android), and TCP fallback.
async fn call_primal_health(socket_path: &str) -> Result<bool> {
    use biomeos_core::atomic_client::AtomicClient;

    // Create AtomicClient from socket path (supports Unix sockets)
    let client = AtomicClient::unix(socket_path);

    // Use health.check method (semantic naming standard)
    let response = client.call("health.check", json!({})).await?;

    Ok(response
        .get("healthy")
        .and_then(|h| h.as_bool())
        .unwrap_or(false))
}

/// Discover binary path for a primal
///
/// Search order:
/// 1. BIOMEOS_PLASMID_BIN_DIR environment variable
/// 2. ./plasmidBin directory
/// 3. ../plasmidBin directory
/// 4. ../../plasmidBin directory
pub async fn discover_primal_binary(
    primal_name: &str,
    context: &ExecutionContext,
) -> Result<PathBuf> {
    let base_dirs = vec![
        context
            .env()
            .get("BIOMEOS_PLASMID_BIN_DIR")
            .cloned()
            .map(PathBuf::from),
        Some(PathBuf::from("./plasmidBin")),
        Some(PathBuf::from("../plasmidBin")),
        Some(PathBuf::from("../../plasmidBin")),
    ];

    // Auto-detect architecture
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;
    let target = format!("{}-{}", arch, os);

    debug!("   Discovering {} binary for {}", primal_name, target);

    for base_dir_opt in base_dirs {
        let Some(base_dir) = base_dir_opt else {
            continue;
        };

        // Try architecture-specific path first
        let arch_path = base_dir.join(&target).join(primal_name);
        if arch_path.exists() {
            info!("   Found: {}", arch_path.display());
            return Ok(arch_path);
        }

        // Try generic path
        let generic_path = base_dir.join(primal_name);
        if generic_path.exists() {
            info!("   Found: {}", generic_path.display());
            return Ok(generic_path);
        }
    }

    anyhow::bail!("Binary not found for: {}", primal_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::context::ExecutionContext;
    use crate::neural_graph::{GraphNode, Operation, PrimalSelector};
    use std::collections::HashMap;
    use std::sync::OnceLock;

    /// Serializes tests that change cwd to avoid races (async-aware to hold across await)
    static CWD_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    fn cwd_lock() -> &'static tokio::sync::Mutex<()> {
        CWD_LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
    }

    fn create_beardog_stub(temp: &tempfile::TempDir) {
        let bin_path = temp.path().join("beardog");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::copy("/bin/true", &bin_path).expect("copy true");
            std::fs::set_permissions(&bin_path, std::fs::Permissions::from_mode(0o755))
                .expect("chmod");
        }
        #[cfg(not(unix))]
        {
            std::fs::write(&bin_path, "").expect("write stub");
        }
    }

    // -------------------------------------------------------------------------
    // resolve_capability_to_primal - Domain mapping logic
    // -------------------------------------------------------------------------

    #[test]
    fn test_resolve_capability_to_primal_encryption() {
        assert_eq!(
            resolve_capability_to_primal("encryption"),
            Some("beardog"),
            "Encryption capability maps to beardog"
        );
    }

    #[test]
    fn test_resolve_capability_to_primal_discovery() {
        assert_eq!(
            resolve_capability_to_primal("discovery"),
            Some("songbird"),
            "Discovery capability maps to songbird"
        );
    }

    #[test]
    fn test_resolve_capability_to_primal_compute() {
        assert_eq!(
            resolve_capability_to_primal("compute"),
            Some("toadstool"),
            "Compute capability maps to toadstool"
        );
    }

    #[test]
    fn test_resolve_capability_to_primal_storage() {
        assert_eq!(
            resolve_capability_to_primal("storage"),
            Some("nestgate"),
            "Storage capability maps to nestgate"
        );
    }

    #[test]
    fn test_resolve_capability_to_primal_ai() {
        assert_eq!(
            resolve_capability_to_primal("ai"),
            Some("squirrel"),
            "AI capability maps to squirrel"
        );
    }

    #[test]
    fn test_resolve_capability_to_primal_case_insensitive() {
        assert_eq!(
            resolve_capability_to_primal("ENCRYPTION"),
            Some("beardog"),
            "Capability resolution should be case-insensitive"
        );
        assert_eq!(
            resolve_capability_to_primal("Discovery"),
            Some("songbird"),
            "Mixed case should resolve"
        );
    }

    #[test]
    fn test_resolve_capability_to_primal_nat_traversal_aliases() {
        assert_eq!(
            resolve_capability_to_primal("mesh"),
            Some("songbird"),
            "mesh maps to songbird"
        );
        assert_eq!(
            resolve_capability_to_primal("punch"),
            Some("songbird"),
            "punch maps to songbird"
        );
        assert_eq!(
            resolve_capability_to_primal("stun"),
            Some("songbird"),
            "stun maps to songbird"
        );
        assert_eq!(
            resolve_capability_to_primal("federation"),
            Some("songbird"),
            "federation maps to songbird"
        );
    }

    #[test]
    fn test_resolve_capability_to_primal_unknown() {
        assert_eq!(
            resolve_capability_to_primal("unknown"),
            None,
            "Unknown capabilities return None"
        );
        assert_eq!(
            resolve_capability_to_primal("nonexistent"),
            None,
            "Nonexistent capability returns None"
        );
    }

    #[test]
    fn test_resolve_capability_to_primal_empty_input() {
        assert_eq!(
            resolve_capability_to_primal(""),
            None,
            "Empty string should return None"
        );
    }

    #[test]
    fn test_resolve_capability_to_primal_whitespace() {
        assert_eq!(
            resolve_capability_to_primal("  encryption  "),
            None,
            "Whitespace-padded should not match (no trim)"
        );
    }

    // -------------------------------------------------------------------------
    // known_primal_names - Capability taxonomy bootstrap
    // -------------------------------------------------------------------------

    #[test]
    fn test_known_primal_names_contains_core_primals() {
        let primals = known_primal_names();

        assert!(primals.contains(&"beardog"), "Should contain beardog");
        assert!(primals.contains(&"songbird"), "Should contain songbird");
        assert!(primals.contains(&"toadstool"), "Should contain toadstool");
        assert!(primals.contains(&"nestgate"), "Should contain nestgate");
        assert!(primals.contains(&"squirrel"), "Should contain squirrel");
    }

    #[test]
    fn test_known_primal_names_returns_vec() {
        let primals = known_primal_names();
        assert_eq!(
            primals.len(),
            5,
            "Should have exactly 5 core primals when not in strict mode"
        );
    }

    #[test]
    #[ignore = "env var BIOMEOS_STRICT_DISCOVERY races with parallel tests — run with --test-threads=1"]
    fn test_known_primal_names_strict_discovery() {
        std::env::set_var("BIOMEOS_STRICT_DISCOVERY", "1");
        let primals = known_primal_names();
        std::env::remove_var("BIOMEOS_STRICT_DISCOVERY");

        assert!(
            primals.is_empty(),
            "Strict discovery mode should return empty list"
        );
    }

    #[test]
    fn test_known_primal_names_no_duplicates() {
        let primals = known_primal_names();
        let unique: std::collections::HashSet<_> = primals.iter().collect();
        assert_eq!(
            unique.len(),
            primals.len(),
            "Known primals should have no duplicates"
        );
    }

    // -------------------------------------------------------------------------
    // primal_start_capability - Error handling and routing
    // -------------------------------------------------------------------------

    fn make_node(
        capability: Option<&str>,
        mode: &str,
        family_id: Option<&str>,
        env_vars: Option<HashMap<String, String>>,
    ) -> GraphNode {
        let mut params = HashMap::new();
        params.insert("mode".to_string(), serde_json::json!(mode));
        if let Some(fid) = family_id {
            params.insert("family_id".to_string(), serde_json::json!(fid));
        }

        let operation = Some(Operation {
            name: "start".to_string(),
            params,
            environment: env_vars,
        });

        let primal = capability.map(|c| PrimalSelector {
            by_capability: Some(c.to_string()),
            by_name: None,
        });

        GraphNode {
            id: "test_node".to_string(),
            primal,
            output: None,
            operation,
            constraints: None,
            depends_on: vec![],
            capabilities: vec![],
            capabilities_provided: None,
            parameter_mappings: None,
            node_type: None,
            dependencies: vec![],
            config: HashMap::new(),
            outputs: vec![],
        }
    }

    #[tokio::test]
    async fn test_primal_start_capability_missing_by_capability() {
        let node = GraphNode {
            id: "test".to_string(),
            primal: None,
            output: None,
            operation: Some(Operation {
                name: "start".to_string(),
                params: HashMap::new(),
                environment: None,
            }),
            constraints: None,
            depends_on: vec![],
            capabilities: vec![],
            capabilities_provided: None,
            parameter_mappings: None,
            node_type: None,
            dependencies: vec![],
            config: HashMap::new(),
            outputs: vec![],
        };

        let ctx = ExecutionContext::new(HashMap::new());
        let result = primal_start_capability(&node, &ctx).await;

        let err = result.expect_err("Should fail when primal.by_capability is missing");
        assert!(
            err.to_string().contains("by_capability"),
            "Error should mention by_capability: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_primal_start_capability_primal_without_by_capability() {
        let node = GraphNode {
            id: "test".to_string(),
            primal: Some(PrimalSelector {
                by_capability: None,
                by_name: Some("beardog".to_string()),
            }),
            output: None,
            operation: Some(Operation {
                name: "start".to_string(),
                params: HashMap::new(),
                environment: None,
            }),
            constraints: None,
            depends_on: vec![],
            capabilities: vec![],
            capabilities_provided: None,
            parameter_mappings: None,
            node_type: None,
            dependencies: vec![],
            config: HashMap::new(),
            outputs: vec![],
        };

        let ctx = ExecutionContext::new(HashMap::new());
        let result = primal_start_capability(&node, &ctx).await;

        let err = result.expect_err("Should fail when by_capability is None");
        assert!(
            err.to_string().contains("by_capability"),
            "Error should mention by_capability: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_primal_start_capability_missing_operation() {
        let node = GraphNode {
            id: "test".to_string(),
            primal: Some(PrimalSelector {
                by_capability: Some("encryption".to_string()),
                by_name: None,
            }),
            output: None,
            operation: None,
            constraints: None,
            depends_on: vec![],
            capabilities: vec![],
            capabilities_provided: None,
            parameter_mappings: None,
            node_type: None,
            dependencies: vec![],
            config: HashMap::new(),
            outputs: vec![],
        };

        let ctx = ExecutionContext::new(HashMap::new());
        let result = primal_start_capability(&node, &ctx).await;

        let err = result.expect_err("Should fail when operation is missing");
        assert!(
            err.to_string().contains("operation"),
            "Error should mention operation: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_primal_start_capability_unknown_capability() {
        let node = make_node(Some("nonexistent_capability"), "server", None, None);
        let ctx = ExecutionContext::new(HashMap::new());

        let result = primal_start_capability(&node, &ctx)
            .await
            .expect("Unknown capability returns Ok with started: false");

        assert_eq!(
            result["started"], false,
            "Unknown capability should not start"
        );
        assert_eq!(result["capability"], "nonexistent_capability");
        assert!(
            result["error"]
                .as_str()
                .unwrap()
                .contains("Unknown capability"),
            "Error field should describe unknown capability"
        );
    }

    #[tokio::test]
    async fn test_primal_start_capability_binary_not_found() {
        let _guard = cwd_lock().lock().await;
        let temp = tempfile::tempdir().expect("temp dir");
        let orig = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(temp.path()).expect("chdir");

        let node = make_node(Some("encryption"), "server", None, None);
        let mut env = HashMap::new();
        env.insert(
            "BIOMEOS_PLASMID_BIN_DIR".to_string(),
            temp.path().to_string_lossy().to_string(),
        );
        let ctx = ExecutionContext::new(env);

        let result = primal_start_capability(&node, &ctx)
            .await
            .expect("Binary not found returns Ok with started: false");

        std::env::set_current_dir(&orig).expect("restore cwd");

        assert_eq!(result["started"], false);
        assert_eq!(result["capability"], "encryption");
        assert_eq!(result["primal"], "beardog");
        assert!(
            result["error"]
                .as_str()
                .unwrap()
                .contains("Binary not found"),
            "Error should indicate binary discovery failure"
        );
    }

    #[tokio::test]
    async fn test_primal_start_capability_mode_default() {
        let _guard = cwd_lock().lock().await;
        let temp = tempfile::tempdir().expect("temp dir");
        let orig = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(temp.path()).expect("chdir");

        create_beardog_stub(&temp);

        let node = make_node(Some("encryption"), "server", None, None);
        let mut env = HashMap::new();
        env.insert(
            "BIOMEOS_PLASMID_BIN_DIR".to_string(),
            temp.path().to_string_lossy().to_string(),
        );
        let ctx = ExecutionContext::new(env);

        let result = primal_start_capability(&node, &ctx).await.unwrap();

        std::env::set_current_dir(&orig).expect("restore cwd");

        assert_eq!(result["mode"], "server", "Default mode should be server");
    }

    #[tokio::test]
    async fn test_primal_start_capability_mode_from_params() {
        let _guard = cwd_lock().lock().await;
        let temp = tempfile::tempdir().expect("temp dir");
        let orig = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(temp.path()).expect("chdir");

        create_beardog_stub(&temp);

        let node = make_node(Some("encryption"), "client", None, None);
        let mut env = HashMap::new();
        env.insert(
            "BIOMEOS_PLASMID_BIN_DIR".to_string(),
            temp.path().to_string_lossy().to_string(),
        );
        let ctx = ExecutionContext::new(env);

        let result = primal_start_capability(&node, &ctx).await.unwrap();

        std::env::set_current_dir(&orig).expect("restore cwd");

        assert_eq!(result["mode"], "client");
    }

    #[tokio::test]
    async fn test_primal_start_capability_family_id_from_params() {
        let _guard = cwd_lock().lock().await;
        let temp = tempfile::tempdir().expect("temp dir");
        let orig = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(temp.path()).expect("chdir");

        create_beardog_stub(&temp);

        let node = make_node(
            Some("encryption"),
            "server",
            Some("custom_family_123"),
            None,
        );
        let mut env = HashMap::new();
        env.insert(
            "BIOMEOS_PLASMID_BIN_DIR".to_string(),
            temp.path().to_string_lossy().to_string(),
        );
        let ctx = ExecutionContext::new(env);

        let result = primal_start_capability(&node, &ctx).await.unwrap();

        std::env::set_current_dir(&orig).expect("restore cwd");

        assert_eq!(result["family_id"], "custom_family_123");
    }

    #[tokio::test]
    async fn test_primal_start_capability_output_json_structure() {
        let node = make_node(Some("unknown_cap"), "server", None, None);
        let ctx = ExecutionContext::new(HashMap::new());

        let result = primal_start_capability(&node, &ctx).await.unwrap();

        assert!(result.get("started").is_some());
        assert!(result.get("capability").is_some());
        assert!(result.get("error").is_some());
        let serialized = serde_json::to_string(&result).expect("Output should serialize to JSON");
        assert!(!serialized.is_empty());
    }

    // -------------------------------------------------------------------------
    // health_check_capability - Capability routing and domain mapping
    // -------------------------------------------------------------------------

    fn make_health_node(capability: Option<&str>, family_id: Option<&str>) -> GraphNode {
        let mut params = HashMap::new();
        if let Some(fid) = family_id {
            params.insert("family_id".to_string(), serde_json::json!(fid));
        }

        let operation = Some(Operation {
            name: "health_check".to_string(),
            params,
            environment: None,
        });

        let primal = capability.map(|c| PrimalSelector {
            by_capability: Some(c.to_string()),
            by_name: None,
        });

        GraphNode {
            id: "health_node".to_string(),
            primal,
            output: None,
            operation,
            constraints: None,
            depends_on: vec![],
            capabilities: vec![],
            capabilities_provided: None,
            parameter_mappings: None,
            node_type: None,
            dependencies: vec![],
            config: HashMap::new(),
            outputs: vec![],
        }
    }

    #[tokio::test]
    async fn test_health_check_capability_missing_operation() {
        let node = GraphNode {
            id: "test".to_string(),
            primal: Some(PrimalSelector {
                by_capability: Some("encryption".to_string()),
                by_name: None,
            }),
            output: None,
            operation: None,
            constraints: None,
            depends_on: vec![],
            capabilities: vec![],
            capabilities_provided: None,
            parameter_mappings: None,
            node_type: None,
            dependencies: vec![],
            config: HashMap::new(),
            outputs: vec![],
        };

        let ctx = ExecutionContext::new(HashMap::new());
        let result = health_check_capability(&node, &ctx).await;

        let err = result.expect_err("Should fail when operation is missing");
        assert!(
            err.to_string().contains("operation"),
            "Error should mention operation: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_health_check_capability_specific_capability() {
        let node = make_health_node(Some("encryption"), Some("test_family"));
        let ctx = ExecutionContext::new(HashMap::new());

        let result = health_check_capability(&node, &ctx)
            .await
            .expect("Health check should succeed");

        assert_eq!(result["family_id"], "test_family");
        assert!(
            result.get("healthy").is_some(),
            "Result should have healthy field"
        );
        assert!(result.get("checks_passed").is_some());
        assert!(result.get("checks_failed").is_some());
        assert!(result.get("total_checks").is_some());
        let total = result["total_checks"].as_u64().unwrap_or(0);
        assert!(
            total <= 1,
            "Single capability should check at most 1 primal"
        );
    }

    #[tokio::test]
    async fn test_health_check_capability_unknown_capability() {
        let node = make_health_node(Some("unknown_cap"), None);
        let ctx = ExecutionContext::new(HashMap::new());

        let result = health_check_capability(&node, &ctx)
            .await
            .expect("Health check should succeed with empty primals");

        assert_eq!(result["total_checks"], 0);
        assert!(!result["healthy"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_health_check_capability_all_primals() {
        let node = make_health_node(None, None);
        let ctx = ExecutionContext::new(HashMap::new());

        let result = health_check_capability(&node, &ctx)
            .await
            .expect("Health check should succeed");

        let total = result["total_checks"].as_u64().unwrap_or(0);
        assert!(
            total <= 5,
            "Should check at most 5 known primals, got {}",
            total
        );
        assert!(result.get("healthy").is_some());
        assert!(result.get("checks_passed").is_some());
        assert!(result.get("checks_failed").is_some());
    }

    #[tokio::test]
    async fn test_health_check_capability_output_json_structure() {
        let node = make_health_node(Some("encryption"), None);
        let ctx = ExecutionContext::new(HashMap::new());

        let result = health_check_capability(&node, &ctx).await.unwrap();

        assert!(result.get("healthy").is_some());
        assert!(result.get("family_id").is_some());
        assert!(result.get("checks_passed").is_some());
        assert!(result.get("checks_failed").is_some());
        assert!(result.get("total_checks").is_some());
        let serialized = serde_json::to_string(&result).expect("Output should serialize");
        assert!(!serialized.is_empty());
    }

    // -------------------------------------------------------------------------
    // discover_primal_binary - Binary discovery logic
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn test_discover_primal_binary_success_via_env() {
        let temp = tempfile::tempdir().expect("temp dir");
        let bin_path = temp.path().join("beardog");
        std::fs::write(&bin_path, "#!/bin/sh\nexit 0").expect("write stub");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bin_path, std::fs::Permissions::from_mode(0o755))
                .expect("chmod");
        }

        let mut env = HashMap::new();
        env.insert(
            "BIOMEOS_PLASMID_BIN_DIR".to_string(),
            temp.path().to_string_lossy().to_string(),
        );
        let ctx = ExecutionContext::new(env);

        let result = discover_primal_binary("beardog", &ctx)
            .await
            .expect("Should find beardog in BIOMEOS_PLASMID_BIN_DIR");

        assert!(result.exists(), "Resolved path should exist");
        assert_eq!(result.file_name().unwrap(), "beardog");
    }

    #[tokio::test]
    async fn test_discover_primal_binary_success_arch_specific() {
        let temp = tempfile::tempdir().expect("temp dir");
        let arch = std::env::consts::ARCH;
        let os = std::env::consts::OS;
        let target_dir = temp.path().join(format!("{}-{}", arch, os));
        std::fs::create_dir_all(&target_dir).expect("create dir");
        let bin_path = target_dir.join("squirrel");
        std::fs::write(&bin_path, "#!/bin/sh\nexit 0").expect("write stub");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bin_path, std::fs::Permissions::from_mode(0o755))
                .expect("chmod");
        }

        let mut env = HashMap::new();
        env.insert(
            "BIOMEOS_PLASMID_BIN_DIR".to_string(),
            temp.path().to_string_lossy().to_string(),
        );
        let ctx = ExecutionContext::new(env);

        let result = discover_primal_binary("squirrel", &ctx)
            .await
            .expect("Should find arch-specific squirrel");

        assert!(result.exists());
        assert_eq!(result.file_name().unwrap(), "squirrel");
    }

    #[tokio::test]
    async fn test_discover_primal_binary_not_found() {
        let temp = tempfile::tempdir().expect("temp dir");
        let mut env = HashMap::new();
        env.insert(
            "BIOMEOS_PLASMID_BIN_DIR".to_string(),
            temp.path().to_string_lossy().to_string(),
        );
        let ctx = ExecutionContext::new(env);

        let result = discover_primal_binary("nonexistent_primal", &ctx).await;

        let err = result.expect_err("Should fail when binary not found");
        assert!(
            err.to_string().contains("Binary not found"),
            "Error should mention binary: {}",
            err
        );
        assert!(
            err.to_string().contains("nonexistent_primal"),
            "Error should mention primal name"
        );
    }

    #[tokio::test]
    async fn test_discover_primal_binary_empty_dir() {
        let _guard = cwd_lock().lock().await;
        let temp = tempfile::tempdir().expect("temp dir");
        let orig = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(temp.path()).expect("chdir");

        let mut env = HashMap::new();
        env.insert(
            "BIOMEOS_PLASMID_BIN_DIR".to_string(),
            temp.path().to_string_lossy().to_string(),
        );
        let ctx = ExecutionContext::new(env);

        let result = discover_primal_binary("beardog", &ctx).await;

        std::env::set_current_dir(&orig).expect("restore cwd");

        let err = result.expect_err("Empty dir should not find beardog");
        assert!(err.to_string().contains("Binary not found"));
    }

    #[tokio::test]
    async fn test_discover_primal_binary_prefers_env_over_default_paths() {
        let temp = tempfile::tempdir().expect("temp dir");
        let bin_path = temp.path().join("nestgate");
        std::fs::write(&bin_path, "x").expect("write");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bin_path, std::fs::Permissions::from_mode(0o755))
                .expect("chmod");
        }

        let mut env = HashMap::new();
        env.insert(
            "BIOMEOS_PLASMID_BIN_DIR".to_string(),
            temp.path().to_string_lossy().to_string(),
        );
        let ctx = ExecutionContext::new(env);

        let result = discover_primal_binary("nestgate", &ctx)
            .await
            .expect("Should find in env dir");

        assert!(result.starts_with(temp.path()));
    }

    // -------------------------------------------------------------------------
    // wait_for_socket_with_timeout - Edge cases
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn test_wait_for_socket_exists_immediately() {
        let temp = tempfile::NamedTempFile::new().expect("temp file");
        let path = temp.path().to_string_lossy().to_string();

        let found = wait_for_socket_with_timeout(&path, 5).await;

        assert!(found, "Should find socket that exists immediately");
    }

    #[tokio::test]
    async fn test_wait_for_socket_not_found() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp
            .path()
            .join("nonexistent.sock")
            .to_string_lossy()
            .to_string();

        let found = wait_for_socket_with_timeout(&path, 2).await;

        assert!(
            !found,
            "Should not find nonexistent socket within 2 attempts"
        );
    }

    #[tokio::test]
    async fn test_wait_for_socket_zero_attempts() {
        let temp = tempfile::NamedTempFile::new().expect("temp file");
        let path = temp.path().to_string_lossy().to_string();

        let found = wait_for_socket_with_timeout(&path, 0).await;

        assert!(
            !found,
            "Zero attempts should not check (1..=0 is empty range)"
        );
    }

    // -------------------------------------------------------------------------
    // Path construction and type serialization
    // -------------------------------------------------------------------------

    #[test]
    fn test_binary_discovery_path_construction() {
        let arch = std::env::consts::ARCH;
        let os = std::env::consts::OS;
        let target = format!("{}-{}", arch, os);

        assert!(target.contains(arch));
        assert!(target.contains(os));
    }

    #[test]
    fn test_plasmid_bin_paths() {
        let paths = [
            PathBuf::from("./plasmidBin"),
            PathBuf::from("../plasmidBin"),
            PathBuf::from("../../plasmidBin"),
        ];

        for path in paths {
            assert!(path.to_string_lossy().contains("plasmidBin"));
        }
    }

    #[test]
    fn test_json_output_serialization_roundtrip() {
        let output = json!({
            "started": false,
            "capability": "encryption",
            "error": "Binary not found: test"
        });
        let s = serde_json::to_string(&output).expect("serialize");
        let parsed: serde_json::Value = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(parsed["started"], false);
        assert_eq!(parsed["capability"], "encryption");
    }

    #[test]
    fn test_health_check_json_serialization() {
        let output = json!({
            "healthy": false,
            "family_id": "test",
            "checks_passed": [],
            "checks_failed": ["beardog"],
            "total_checks": 1
        });
        let s = serde_json::to_string(&output).expect("serialize");
        let parsed: serde_json::Value = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(parsed["healthy"], false);
        assert_eq!(parsed["total_checks"], 1);
    }
}
