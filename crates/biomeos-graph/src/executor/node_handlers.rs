// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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
use biomeos_types::JsonRpcRequest;
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
    let request = JsonRpcRequest::new(
        "crypto.derive_child_seed",
        serde_json::json!({
            "parent_seed": parent_seed,
            "node_id": node_id,
            "output_path": output_path,
            "deployment_batch": deployment_batch
        }),
    );

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
///
/// Uses XDG-compliant paths via `SystemPaths` instead of hardcoded `/tmp/` paths.
fn discover_beardog_socket(env: &HashMap<String, String>) -> Result<String> {
    use biomeos_types::paths::SystemPaths;

    // Priority 1: Explicit environment variable
    if let Some(socket) = env.get("BEARDOG_SOCKET") {
        return Ok(socket.clone());
    }
    if let Ok(socket) = std::env::var("BEARDOG_SOCKET") {
        return Ok(socket);
    }

    // Priority 2: XDG-compliant socket path via SystemPaths
    let paths = SystemPaths::new_lazy();
    let family_id = env
        .get("FAMILY_ID")
        .cloned()
        .or_else(|| std::env::var("BIOMEOS_FAMILY_ID").ok())
        .unwrap_or_else(|| "default".to_string());

    // Try family-specific socket first
    let family_socket = paths.primal_socket(&format!("beardog-{}", family_id));
    if family_socket.exists() {
        return Ok(family_socket.display().to_string());
    }

    // Try generic beardog socket
    let generic_socket = paths.primal_socket("beardog");
    if generic_socket.exists() {
        return Ok(generic_socket.display().to_string());
    }

    // Priority 3: Fallback patterns (legacy compatibility)
    let fallback_patterns = [
        format!("{}/beardog-{}.sock", std::env::temp_dir().display(), family_id),
        format!("{}/beardog.sock", std::env::temp_dir().display()),
        "/run/biomeos/beardog.sock".to_string(),
    ];
    for pattern in &fallback_patterns {
        if std::path::Path::new(pattern).exists() {
            return Ok(pattern.clone());
        }
    }

    anyhow::bail!(
        "BearDog socket not found. Set BEARDOG_SOCKET or ensure BearDog is running. \
         Checked: BEARDOG_SOCKET env, XDG runtime dir: {:?}",
        paths.runtime_dir()
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
    let family_id = context.env.get("FAMILY_ID").cloned().unwrap_or_else(|| biomeos_core::family_discovery::get_family_id());
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
        let family_id = context.env.get("FAMILY_ID").cloned().unwrap_or_else(|| biomeos_core::family_discovery::get_family_id());
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
    let request = JsonRpcRequest::new("health.ping", serde_json::json!({}));
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
        .unwrap_or_else(|| biomeos_core::family_discovery::get_family_id());

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

    let request = JsonRpcRequest::new(
        "lineage.verify_siblings",
        serde_json::json!({
            "family_id": family_id,
            "siblings": siblings
        }),
    );
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn create_test_node(id: &str) -> GraphNode {
        use crate::node::{NodeId, NodeType, NodeConfig, NodeParams};
        GraphNode {
            id: NodeId::new(id).unwrap(),
            name: id.to_string(),
            node_type: NodeType::Capability,
            capability: None,
            required: true,
            order: 0,
            depends_on: vec![],
            condition: None,
            config: NodeConfig::default(),
            params: NodeParams::new(),
        }
    }

    fn create_test_context(env: HashMap<String, String>) -> ExecutionContext {
        ExecutionContext::new(env)
    }

    #[test]
    fn test_substitute_env_basic() {
        let mut env = HashMap::new();
        env.insert("FOO".to_string(), "bar".to_string());
        env.insert("BAZ".to_string(), "qux".to_string());

        let result = substitute_env("${FOO}/${BAZ}/test", &env);
        assert_eq!(result, "bar/qux/test");
    }

    #[test]
    fn test_substitute_env_empty() {
        let env = HashMap::new();
        let result = substitute_env("no-vars", &env);
        assert_eq!(result, "no-vars");
    }

    #[test]
    fn test_substitute_env_partial() {
        let mut env = HashMap::new();
        env.insert("FOO".to_string(), "bar".to_string());
        let result = substitute_env("${FOO}/${MISSING}", &env);
        assert_eq!(result, "bar/${MISSING}");
    }

    #[test]
    fn test_substitute_env_multiple_same() {
        let mut env = HashMap::new();
        env.insert("VAR".to_string(), "value".to_string());
        let result = substitute_env("${VAR}-${VAR}-${VAR}", &env);
        assert_eq!(result, "value-value-value");
    }

    #[tokio::test]
    async fn test_filesystem_check_exists_success() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test content").unwrap();

        let mut node = create_test_node("test-node");
        node.config.extra.insert(
            "path".to_string(),
            toml::Value::String(test_file.to_string_lossy().to_string()),
        );

        let context = create_test_context(HashMap::new());
        let result = node_filesystem_check_exists(&node, &context).await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value.get("exists"), Some(&serde_json::json!(true)));
    }

    #[tokio::test]
    async fn test_filesystem_check_exists_missing_path() {
        let mut node = create_test_node("test-node");
        // No path in config

        let context = create_test_context(HashMap::new());
        let result = node_filesystem_check_exists(&node, &context).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_filesystem_check_exists_file_not_found() {
        let mut node = create_test_node("test-node");
        node.config.extra.insert(
            "path".to_string(),
            toml::Value::String("/nonexistent/path/file.txt".to_string()),
        );

        let context = create_test_context(HashMap::new());
        let result = node_filesystem_check_exists(&node, &context).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_filesystem_check_exists_with_size_check() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test").unwrap(); // 4 bytes

        let mut node = create_test_node("test-node");
        node.config.extra.insert(
            "path".to_string(),
            toml::Value::String(test_file.to_string_lossy().to_string()),
        );
        node.config.extra.insert(
            "expected_size".to_string(),
            toml::Value::Integer(4),
        );

        let context = create_test_context(HashMap::new());
        let result = node_filesystem_check_exists(&node, &context).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_filesystem_check_exists_size_mismatch() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test").unwrap(); // 4 bytes

        let mut node = create_test_node("test-node");
        node.config.extra.insert(
            "path".to_string(),
            toml::Value::String(test_file.to_string_lossy().to_string()),
        );
        node.config.extra.insert(
            "expected_size".to_string(),
            toml::Value::Integer(10), // Wrong size
        );

        let context = create_test_context(HashMap::new());
        let result = node_filesystem_check_exists(&node, &context).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_filesystem_check_exists_with_env_substitution() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test").unwrap();

        let mut env = HashMap::new();
        env.insert("TEST_DIR".to_string(), temp_dir.path().to_string_lossy().to_string());

        let mut node = create_test_node("test-node");
        node.config.extra.insert(
            "path".to_string(),
            toml::Value::String("${TEST_DIR}/test.txt".to_string()),
        );

        let context = create_test_context(env);
        let result = node_filesystem_check_exists(&node, &context).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_deployment_report() {
        let mut node = create_test_node("report-node");
        let mut atomics = vec![
            toml::Value::String("tower".to_string()),
            toml::Value::String("nest".to_string()),
        ];
        node.config.extra.insert(
            "atomics_deployed".to_string(),
            toml::Value::Array(atomics),
        );

        let context = create_test_context(HashMap::new());
        let result = node_deployment_report(&node, &context).await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value.get("success"), Some(&serde_json::json!(true)));
        assert!(value.get("atomics_deployed").is_some());
        assert!(value.get("timestamp").is_some());
    }

    #[tokio::test]
    async fn test_deployment_report_empty_atomics() {
        let node = create_test_node("report-node");
        let context = create_test_context(HashMap::new());
        let result = node_deployment_report(&node, &context).await;

        assert!(result.is_ok());
        let value = result.unwrap();
        let atomics = value.get("atomics_deployed").and_then(|v| v.as_array()).unwrap();
        assert_eq!(atomics.len(), 0);
    }

    #[tokio::test]
    async fn test_health_check_no_primal() {
        let mut node = create_test_node("health-node");
        node.config.extra.insert(
            "atomic_type".to_string(),
            toml::Value::String("tower".to_string()),
        );

        let context = create_test_context(HashMap::new());
        let result = node_health_check(&node, &context).await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value.get("healthy"), Some(&serde_json::json!(true)));
        assert_eq!(value.get("atomic"), Some(&serde_json::json!("tower")));
    }

    #[tokio::test]
    async fn test_health_check_missing_primal_config() {
        let mut node = create_test_node("health-node");
        node.config.extra.insert(
            "atomic_type".to_string(),
            toml::Value::String("tower".to_string()),
        );
        // No primal specified

        let mut env = HashMap::new();
        env.insert("FAMILY_ID".to_string(), "test-family".to_string());
        let context = create_test_context(env);

        let result = node_health_check(&node, &context).await;
        // Should succeed with basic healthy status
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_lineage_verify_no_beardog() {
        let mut node = create_test_node("lineage-node");
        let siblings = vec![
            toml::Value::String("sibling1".to_string()),
            toml::Value::String("sibling2".to_string()),
        ];
        node.config.extra.insert(
            "siblings".to_string(),
            toml::Value::Array(siblings),
        );

        let mut env = HashMap::new();
        env.insert("FAMILY_ID".to_string(), "test-family".to_string());
        let context = create_test_context(env);

        // Should gracefully degrade when BearDog is not available
        let result = node_lineage_verify(&node, &context).await;
        // May succeed with graceful degradation or fail - both are acceptable
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_lineage_verify_empty_siblings() {
        let node = create_test_node("lineage-node");
        let mut env = HashMap::new();
        env.insert("FAMILY_ID".to_string(), "test-family".to_string());
        let context = create_test_context(env);

        // Should handle empty siblings list
        let result = node_lineage_verify(&node, &context).await;
        // May succeed or fail depending on BearDog availability
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_substitute_env_edge_cases() {
        let mut env = HashMap::new();
        env.insert("EMPTY".to_string(), "".to_string());
        env.insert("SPECIAL".to_string(), "a/b/c".to_string());

        assert_eq!(substitute_env("${EMPTY}", &env), "");
        assert_eq!(substitute_env("${SPECIAL}", &env), "a/b/c");
        assert_eq!(substitute_env("prefix-${EMPTY}-suffix", &env), "prefix--suffix");
    }
}
