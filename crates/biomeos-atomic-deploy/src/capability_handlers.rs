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
        .unwrap_or("nat0");

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
    configure_primal_socket(&mut cmd, primal_name, &socket_path, family_id, context).await;

    cmd.env("FAMILY_ID", family_id);

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
        .unwrap_or("nat0");

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
        // Check all known primals
        vec!["beardog", "songbird", "squirrel", "toadstool", "nestgate"]
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

/// Call primal health endpoint via JSON-RPC
async fn call_primal_health(socket_path: &str) -> Result<bool> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;

    let stream = UnixStream::connect(socket_path).await?;
    let (read_half, mut write_half) = stream.into_split();

    let request = json!({
        "jsonrpc": "2.0",
        "method": "health.check",
        "params": {},
        "id": 1
    });

    write_half.write_all(request.to_string().as_bytes()).await?;
    write_half.write_all(b"\n").await?;
    write_half.flush().await?;

    let mut reader = BufReader::new(read_half);
    let mut response_line = String::new();
    reader.read_line(&mut response_line).await?;

    let response: serde_json::Value = serde_json::from_str(&response_line)?;

    Ok(response
        .get("result")
        .and_then(|r| r.get("healthy"))
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

    #[test]
    fn test_resolve_capability_to_primal() {
        assert_eq!(resolve_capability_to_primal("security"), Some("beardog"));
        assert_eq!(resolve_capability_to_primal("discovery"), Some("songbird"));
        assert_eq!(resolve_capability_to_primal("ai"), Some("squirrel"));
        assert_eq!(resolve_capability_to_primal("unknown"), None);
    }
}
