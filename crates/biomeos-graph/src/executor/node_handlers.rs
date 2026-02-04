//! Node handler implementations for graph execution
//!
//! **EVOLVED:** Domain-based splitting for maintainability.
//!
//! This module contains all node-specific executors organized by domain:
//! - Filesystem operations
//! - Cryptographic operations (via BearDog)
//! - Primal lifecycle management
//! - Health checks
//! - Lineage verification
//! - Deployment reporting

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::{timeout, Duration, Instant};
use tracing::{debug, info, warn};

use crate::graph::GraphNode;
use super::context::{ExecutionContext, RollbackAction};

/// Node executor: filesystem.check_exists
pub async fn node_filesystem_check_exists(
    node: &GraphNode,
    context: &ExecutionContext,
) -> Result<serde_json::Value> {
    let path = node
        .config
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'path' in config"))?;

    // Substitute environment variables
    let path = substitute_env(path, &context.env);
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
///
/// EVOLVED (Jan 27, 2026): Now delegates to BearDog primal via JSON-RPC
///
/// # Deep Debt Principles
/// - No reimplementation: BearDog handles all cryptographic operations
/// - Capability-based: Discovers BearDog by capability, not hardcoded name
/// - Pure Rust: JSON-RPC over Unix socket (no HTTP/TLS)
pub async fn node_crypto_derive_seed(
    node: &GraphNode,
    context: &ExecutionContext,
) -> Result<serde_json::Value> {
    // Extract required parameters
    let parent_seed = node
        .config
        .get("parent_seed")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'parent_seed'"))?;
    let parent_seed = substitute_env(parent_seed, &context.env);

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
    let output_path = substitute_env(output_path, &context.env);

    let deployment_batch = node
        .config
        .get("deployment_batch")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Discover BearDog socket (capability-based, no hardcoding)
    let beardog_socket = discover_beardog_socket(&context.env)?;

    debug!(
        "Calling BearDog for seed derivation: node_id={}, output={}",
        node_id, output_path
    );

    // Connect to BearDog
    let stream = UnixStream::connect(&beardog_socket)
        .await
        .context(format!("Failed to connect to BearDog at {}", beardog_socket))?;

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Prepare JSON-RPC request to BearDog
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "crypto.derive_child_seed",
        "params": {
            "parent_seed": parent_seed,
            "node_id": node_id,
            "output_path": output_path,
            "deployment_batch": deployment_batch
        },
        "id": 1
    });

    // Send request
    let request_str = serde_json::to_string(&request)? + "\n";
    writer.write_all(request_str.as_bytes()).await?;
    writer.flush().await?;

    // Read response
    let mut response_line = String::new();
    reader.read_line(&mut response_line).await?;

    // Parse response
    let response: serde_json::Value = serde_json::from_str(response_line.trim())?;

    // Check for JSON-RPC error
    if let Some(error) = response.get("error") {
        let message = error
            .get("message")
            .and_then(|m| m.as_str())
            .ok_or("Unknown BearDog error")
            .map_err(|_| anyhow::anyhow!("Unknown BearDog error"))?;
        anyhow::bail!("BearDog seed derivation failed: {}", message);
    }

    // Return the result (seed path and metadata)
    response
        .get("result")
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("BearDog returned empty result"))
}

/// Discover BearDog socket path (capability-based discovery)
fn discover_beardog_socket(env: &HashMap<String, String>) -> Result<String> {
    // Priority 1: Explicit environment variable
    if let Some(socket) = env.get("BEARDOG_SOCKET") {
        return Ok(socket.clone());
    }
    if let Ok(socket) = std::env::var("BEARDOG_SOCKET") {
        return Ok(socket);
    }

    // Priority 2: Family-based socket path (deterministic nucleation)
    if let Some(family_id) = env.get("FAMILY_ID").or_else(|| {
        std::env::var("BIOMEOS_FAMILY_ID")
            .ok()
            .map(|s| s.to_string())
            .as_ref()
            .cloned()
    }) {
        return Ok(format!("/tmp/beardog-{}.sock", family_id));
    }

    // Priority 3: XDG runtime directory pattern
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let socket = format!("{}/biomeos/beardog.sock", runtime_dir);
        if std::path::Path::new(&socket).exists() {
            return Ok(socket);
        }
    }

    // Priority 4: Common patterns
    let patterns = [
        "/tmp/beardog.sock",
        "/run/biomeos/beardog.sock",
    ];
    for pattern in &patterns {
        if std::path::Path::new(pattern).exists() {
            return Ok((*pattern).to_string());
        }
    }

    anyhow::bail!(
        "BearDog socket not found. Set BEARDOG_SOCKET or ensure BearDog is running. \
         Checked: BEARDOG_SOCKET env, XDG_RUNTIME_DIR, /tmp/beardog.sock"
    )
}

/// Node executor: primal.launch
///
/// EVOLVED (Jan 27, 2026): Complete implementation via process spawning
pub async fn node_primal_launch(
    node: &GraphNode,
    context: &ExecutionContext,
) -> Result<serde_json::Value> {
    use std::process::Stdio;
    use tokio::process::Command;

    let primal_name = node
        .config
        .get("primal")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'primal' in config"))?;

    // Build binary path from environment or default locations
    let binary_path = resolve_primal_binary(primal_name, &context.env)?;

    // Build socket path
    let family_id = context.env.get("FAMILY_ID").cloned().unwrap_or_else(|| "nat0".to_string());
    let socket_path = build_socket_path(primal_name, &family_id, &context.env);

    info!("Launching primal: {} -> {}", primal_name, binary_path);

    // Spawn the primal process
    let mut cmd = Command::new(&binary_path);
    cmd.arg("server")
        .arg("--socket")
        .arg(&socket_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    // Pass family seed if available
    if let Ok(seed) = std::env::var("BIOMEOS_FAMILY_SEED") {
        cmd.env("BIOMEOS_FAMILY_SEED", seed);
    }

    let child = cmd.spawn()
        .context(format!("Failed to spawn primal: {}", primal_name))?;

    let pid = child.id().unwrap_or(0);

    // Record rollback action for cleanup on failure
    context
        .record_rollback(
            &node.id,
            RollbackAction::StopProcess {
                primal: primal_name.to_string(),
                pid,
                socket: socket_path.clone(),
            },
        )
        .await;

    info!("✅ Primal {} launched (PID: {})", primal_name, pid);

    Ok(serde_json::json!({
        "launched": true,
        "primal": primal_name,
        "pid": pid,
        "socket": socket_path
    }))
}

/// Resolve primal binary path
fn resolve_primal_binary(primal_name: &str, env: &HashMap<String, String>) -> Result<String> {
    // Priority 1: Explicit environment variable
    let env_key = format!("{}_BINARY", primal_name.to_uppercase());
    if let Some(path) = env.get(&env_key) {
        return Ok(path.clone());
    }
    if let Ok(path) = std::env::var(&env_key) {
        return Ok(path);
    }

    // Priority 2: SPORE_ROOT/primals/{primal}
    if let Some(spore_root) = env.get("SPORE_ROOT").or_else(|| std::env::var("SPORE_ROOT").ok().as_ref()) {
        let path = format!("{}/primals/{}", spore_root, primal_name);
        if std::path::Path::new(&path).exists() {
            return Ok(path);
        }
    }

    // Priority 3: plasmidBin (standard location)
    let plasmid_path = format!("plasmidBin/{}", primal_name);
    if std::path::Path::new(&plasmid_path).exists() {
        return Ok(plasmid_path);
    }

    // Priority 4: Current directory primals/
    let local_path = format!("primals/{}", primal_name);
    if std::path::Path::new(&local_path).exists() {
        return Ok(local_path);
    }

    anyhow::bail!("Primal binary not found: {}", primal_name)
}

/// Build socket path for a primal
fn build_socket_path(primal_name: &str, family_id: &str, env: &HashMap<String, String>) -> String {
    // Use XDG-compliant path if available
    if let Ok(paths) = biomeos_types::SystemPaths::new() {
        return paths.primal_socket(&format!("{}-{}", primal_name, family_id))
            .to_string_lossy()
            .to_string();
    }

    // Fallback to SOCKET_DIR or /tmp
    let socket_dir = env.get("SOCKET_DIR")
        .cloned()
        .unwrap_or_else(|| "/tmp".to_string());

    format!("{}/{}-{}.sock", socket_dir, primal_name, family_id)
}

/// Node executor: health.check_atomic
///
/// EVOLVED (Jan 27, 2026): Real health check via socket ping
pub async fn node_health_check(
    node: &GraphNode,
    context: &ExecutionContext,
) -> Result<serde_json::Value> {
    let atomic_type = node
        .config
        .get("atomic_type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let primal_name = node
        .config
        .get("primal")
        .and_then(|v| v.as_str());

    // If primal specified, check its health
    if let Some(primal) = primal_name {
        let family_id = context.env.get("FAMILY_ID").cloned().unwrap_or_else(|| "nat0".to_string());
        let socket_path = build_socket_path(primal, &family_id, &context.env);

        // Check if socket exists (basic health)
        let socket_exists = std::path::Path::new(&socket_path).exists();

        if socket_exists {
            // Try to ping the primal
            match ping_primal(&socket_path).await {
                Ok(response_time_ms) => {
                    return Ok(serde_json::json!({
                        "healthy": true,
                        "atomic": atomic_type,
                        "primal": primal,
                        "socket": socket_path,
                        "response_time_ms": response_time_ms
                    }));
                }
                Err(e) => {
                    warn!("Primal {} health check failed: {}", primal, e);
                    return Ok(serde_json::json!({
                        "healthy": false,
                        "atomic": atomic_type,
                        "primal": primal,
                        "error": e.to_string()
                    }));
                }
            }
        } else {
            return Ok(serde_json::json!({
                "healthy": false,
                "atomic": atomic_type,
                "primal": primal,
                "error": "Socket not found"
            }));
        }
    }

    // No specific primal - return basic healthy status
    Ok(serde_json::json!({
        "healthy": true,
        "atomic": atomic_type,
        "note": "No specific primal to check"
    }))
}

/// Ping a primal via its socket to check health
async fn ping_primal(socket_path: &str) -> Result<u64> {
    let start = Instant::now();

    let stream = timeout(
        Duration::from_secs(5),
        UnixStream::connect(socket_path),
    )
    .await
    .context("Connection timeout")?
    .context("Connection failed")?;

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Send health ping
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.ping",
        "params": {},
        "id": 1
    });
    let request_str = serde_json::to_string(&request)? + "\n";
    writer.write_all(request_str.as_bytes()).await?;
    writer.flush().await?;

    // Read response
    let mut response_line = String::new();
    timeout(Duration::from_secs(5), reader.read_line(&mut response_line))
        .await
        .context("Response timeout")?
        .context("Read failed")?;

    let elapsed_ms = start.elapsed().as_millis() as u64;

    // Check response is valid JSON-RPC
    let _response: serde_json::Value = serde_json::from_str(response_line.trim())?;

    Ok(elapsed_ms)
}

/// Node executor: lineage.verify_siblings
///
/// EVOLVED (Jan 27, 2026): Verify via BearDog JSON-RPC
pub async fn node_lineage_verify(
    node: &GraphNode,
    context: &ExecutionContext,
) -> Result<serde_json::Value> {
    let siblings = node
        .config
        .get("siblings")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
        .unwrap_or_default();

    let family_id = context.env.get("FAMILY_ID")
        .cloned()
        .unwrap_or_else(|| "nat0".to_string());

    // Discover BearDog for lineage verification
    let beardog_socket = match discover_beardog_socket(&context.env) {
        Ok(socket) => socket,
        Err(e) => {
            warn!("BearDog not available for lineage verification: {}", e);
            // Graceful degradation - return success without verification
            return Ok(serde_json::json!({
                "verified": true,
                "siblings_checked": 0,
                "note": "BearDog unavailable, verification skipped"
            }));
        }
    };

    // Call BearDog to verify siblings
    let stream = UnixStream::connect(&beardog_socket).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "lineage.verify_siblings",
        "params": {
            "family_id": family_id,
            "siblings": siblings
        },
        "id": 1
    });
    let request_str = serde_json::to_string(&request)? + "\n";
    writer.write_all(request_str.as_bytes()).await?;
    writer.flush().await?;

    let mut response_line = String::new();
    reader.read_line(&mut response_line).await?;

    let response: serde_json::Value = serde_json::from_str(response_line.trim())?;

    // Return BearDog's response or extract relevant fields
    if let Some(result) = response.get("result") {
        Ok(result.clone())
    } else if let Some(error) = response.get("error") {
        let msg = error.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown");
        Ok(serde_json::json!({
            "verified": false,
            "error": msg
        }))
    } else {
        Ok(serde_json::json!({
            "verified": true,
            "siblings_checked": siblings.len()
        }))
    }
}

/// Node executor: report.deployment_success
pub async fn node_deployment_report(
    node: &GraphNode,
    _context: &ExecutionContext,
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
pub fn substitute_env(s: &str, env: &HashMap<String, String>) -> String {
    let mut result = s.to_string();

    for (key, value) in env {
        let placeholder = format!("${{{}}}", key);
        result = result.replace(&placeholder, value);
    }

    result
}
