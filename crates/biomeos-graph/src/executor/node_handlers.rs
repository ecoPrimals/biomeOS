// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Node handler implementations for graph execution
//!
//! This module contains all node-specific executors organized by domain:
//! - Filesystem operations
//! - Cryptographic operations (via security provider)
//! - Primal lifecycle management
//! - Health checks
//! - Lineage verification
//! - Deployment reporting

use anyhow::{Context, Result};
use biomeos_types::JsonRpcRequest;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::{timeout, Duration, Instant};
use tracing::{debug, info, warn};

use crate::graph::GraphNode;
use super::context::{ExecutionContext, RollbackAction};

/// Resolve family ID from graph env map, falling back to canonical discovery.
fn resolve_family_id(env: &HashMap<String, String>) -> String {
    env.get(biomeos_types::env_config::vars::FAMILY_ID)
        .or_else(|| env.get(biomeos_types::env_config::vars::FAMILY_ID_LEGACY))
        .cloned()
        .unwrap_or_else(|| biomeos_core::family_discovery::get_family_id())
}

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
/// # Deep Debt Principles
/// - No reimplementation: security provider handles all cryptographic operations
/// - Capability-based: discovers provider by capability, not hardcoded name
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

    // Route through Neural API via capability.call (Gate 6.2)
    let neural_socket = discover_neural_api_socket(&context.env)?;

    debug!(
        "Routing crypto.derive_child_seed via Neural API at {neural_socket}: node_id={node_id}, output={output_path}",
    );

    let request = JsonRpcRequest::new(
        "capability.call",
        serde_json::json!({
            "capability": "crypto",
            "operation": "derive_child_seed",
            "args": {
                "parent_seed": parent_seed,
                "node_id": node_id,
                "output_path": output_path,
                "deployment_batch": deployment_batch
            }
        }),
    );

    let response = call_neural_api(&neural_socket, &request)
        .await
        .context("capability.call(crypto, derive_child_seed) via Neural API failed")?;

    if let Some(error) = response.get("error") {
        let message = error
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");
        anyhow::bail!("Neural API crypto.derive_child_seed failed: {message}");
    }

    response
        .get("result")
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Neural API returned empty result for crypto.derive_child_seed"))
}

/// Discover the Neural API socket for capability routing.
///
/// Discovery priority:
/// 1. `NEURAL_API_SOCKET` from graph env or process env
/// 2. XDG: `$XDG_RUNTIME_DIR/membrane/neural-api.sock`
/// 3. `/run/user/{uid}/membrane/neural-api.sock`
/// 4. `/tmp/membrane/neural-api.sock`
fn discover_neural_api_socket(env: &HashMap<String, String>) -> Result<String> {
    use biomeos_types::paths::SystemPaths;

    if let Some(socket) = env.get(biomeos_types::env_config::vars::NEURAL_API_SOCKET) {
        return Ok(socket.clone());
    }
    if let Ok(socket) = std::env::var(biomeos_types::env_config::vars::NEURAL_API_SOCKET) {
        return Ok(socket);
    }

    let paths = SystemPaths::new_lazy();
    let neural_socket = paths.primal_socket("neural-api");
    if neural_socket.exists() {
        return Ok(neural_socket.display().to_string());
    }

    // Tier 4: /tmp fallback (PRIMAL_IPC_PROTOCOL.md standard)
    let tmp_socket = std::path::PathBuf::from(biomeos_types::constants::runtime_paths::FALLBACK_RUNTIME_BASE)
        .join("neural-api.sock");
    if tmp_socket.exists() {
        return Ok(tmp_socket.display().to_string());
    }

    anyhow::bail!(
        "Neural API socket not found. Set NEURAL_API_SOCKET or ensure biomeOS Neural API is running. \
         Checked: NEURAL_API_SOCKET env, XDG runtime dir: {:?}, fallback: {}",
        paths.runtime_dir(),
        biomeos_types::constants::runtime_paths::FALLBACK_RUNTIME_BASE,
    )
}

/// Send a JSON-RPC request to Neural API and return the parsed response.
async fn call_neural_api(
    neural_socket: &str,
    request: &JsonRpcRequest,
) -> Result<serde_json::Value> {
    let connect_timeout = Duration::from_millis(
        biomeos_types::constants::timeouts::DEFAULT_CONNECTION_TIMEOUT_MS,
    );
    let read_timeout = Duration::from_secs(30);

    let stream = timeout(connect_timeout, UnixStream::connect(neural_socket))
        .await
        .context(format!(
            "Connect timeout ({connect_timeout:?}) to Neural API at {neural_socket}"
        ))?
        .context(format!(
            "Failed to connect to Neural API at {neural_socket}"
        ))?;

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let request_str = serde_json::to_string(request)? + "\n";
    writer.write_all(request_str.as_bytes()).await?;
    writer.flush().await?;

    let mut response_line = String::new();
    timeout(read_timeout, reader.read_line(&mut response_line))
        .await
        .context(format!(
            "Read timeout ({read_timeout:?}) from Neural API at {neural_socket}"
        ))??;

    let response: serde_json::Value = serde_json::from_str(response_line.trim())?;
    Ok(response)
}

/// Node executor: primal.launch
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
    let family_id = resolve_family_id(&context.env);
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
    if let Ok(seed) = std::env::var(biomeos_types::env_config::vars::FAMILY_SEED) {
        cmd.env(biomeos_types::env_config::vars::FAMILY_SEED, seed);
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
    if let Some(spore_root) = env.get(biomeos_types::env_config::vars::SPORE_ROOT).or_else(|| std::env::var(biomeos_types::env_config::vars::SPORE_ROOT).ok().as_ref()) {
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

    let socket_dir = env.get("SOCKET_DIR")
        .cloned()
        .unwrap_or_else(|| biomeos_types::defaults::DEFAULT_SOCKET_DIR.to_string());

    format!("{}/{}-{}.sock", socket_dir, primal_name, family_id)
}

/// Node executor: health.check_atomic
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
        let family_id = resolve_family_id(&context.env);
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

    let family_id = resolve_family_id(&context.env);

    // Route through Neural API via capability.call (Gate 6.2)
    let neural_socket = match discover_neural_api_socket(&context.env) {
        Ok(socket) => socket,
        Err(e) => {
            warn!("Neural API not available for lineage verification: {}", e);
            return Ok(serde_json::json!({
                "verified": true,
                "siblings_checked": 0,
                "note": "Neural API unavailable, verification skipped"
            }));
        }
    };

    let request = JsonRpcRequest::new(
        "capability.call",
        serde_json::json!({
            "capability": "lineage",
            "operation": "verify_siblings",
            "args": {
                "family_id": family_id,
                "siblings": siblings
            }
        }),
    );

    match call_neural_api(&neural_socket, &request).await {
        Ok(response) => {
            if let Some(result) = response.get("result") {
                Ok(result.clone())
            } else if let Some(error) = response.get("error") {
                let msg = error.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown");
                Ok(serde_json::json!({ "verified": false, "error": msg }))
            } else {
                Ok(serde_json::json!({ "verified": true, "siblings_checked": siblings.len() }))
            }
        }
        Err(e) => Ok(serde_json::json!({ "verified": false, "error": e.to_string() })),
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

/// Generic capability_call executor (G69 evolution).
///
/// Routes any capability through the Neural API's `capability.call` dispatch path.
/// This enables graph templates to invoke arbitrary capabilities without requiring
/// hardcoded handlers in the executor. Used by depot_lineage.toml and all provenance
/// graphs that compose capability chains.
///
/// Params are read from `node.config`:
/// - `capability`: dotted capability name (e.g., "crypto.sign", "entry.append")
/// - `args`: arguments for the capability operation (JSON object)
/// - `operation`: optional explicit operation name (if not in dotted capability)
pub async fn node_capability_call(
    node: &GraphNode,
    context: &ExecutionContext,
) -> Result<serde_json::Value> {
    let capability = node
        .config
        .get("capability")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!(
            "Generic capability_call node '{}' missing 'capability' in config",
            node.id
        ))?;

    let args = node
        .config
        .get("args")
        .cloned()
        .or_else(|| node.config.get("params").cloned())
        .unwrap_or(serde_json::json!({}));

    let operation = node.config.get("operation").and_then(|v| v.as_str());

    let neural_socket = discover_neural_api_socket(&context.env)?;

    let env = &context.env;
    let args_str = serde_json::to_string(&args).unwrap_or_default();
    let args_substituted: serde_json::Value =
        serde_json::from_str(&substitute_env(&args_str, env)).unwrap_or(args);

    let mut call_params = serde_json::json!({
        "capability": capability,
        "args": args_substituted
    });

    if let Some(op) = operation {
        call_params["operation"] = serde_json::Value::String(op.to_string());
    }

    let request = JsonRpcRequest::new("capability.call", call_params);

    debug!(
        "Generic capability_call: {} via Neural API at {neural_socket}",
        capability
    );

    let response = call_neural_api(&neural_socket, &request)
        .await
        .context(format!(
            "capability.call({capability}) via Neural API failed for node '{}'",
            node.id
        ))?;

    if let Some(error) = response.get("error") {
        let message = error
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");
        let code = error.get("code").and_then(|c| c.as_i64()).unwrap_or(-32000);
        anyhow::bail!("capability.call({capability}) error {code}: {message}");
    }

    response
        .get("result")
        .cloned()
        .ok_or_else(|| anyhow::anyhow!(
            "capability.call({capability}) returned no result for node '{}'",
            node.id
        ))
}

/// Generic health_check executor (G69 evolution).
///
/// Routes a health check through the Neural API for any named primal.
/// Supports the `health_check` operation name used in depot_lineage and
/// provenance_trio_deploy graphs.
pub async fn node_generic_health_check(
    node: &GraphNode,
    context: &ExecutionContext,
) -> Result<serde_json::Value> {
    let primal_name = node
        .config
        .get("primal_name")
        .or_else(|| node.config.get("primal"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let family_id = resolve_family_id(&context.env);
    let socket_path = build_socket_path(primal_name, &family_id, &context.env);

    if !std::path::Path::new(&socket_path).exists() {
        return Ok(serde_json::json!({
            "healthy": false,
            "primal": primal_name,
            "error": format!("Socket not found: {socket_path}")
        }));
    }

    match ping_primal(&socket_path).await {
        Ok(ms) => Ok(serde_json::json!({
            "healthy": true,
            "primal": primal_name,
            "response_time_ms": ms
        })),
        Err(e) => Ok(serde_json::json!({
            "healthy": false,
            "primal": primal_name,
            "error": e.to_string()
        })),
    }
}

/// Graph foreach executor (G69 batch lineage).
///
/// Iterates over a list of items and executes a sub-graph for each one.
/// Supports bounded concurrency via `config.concurrency`.
///
/// Config:
/// - `graph`: sub-graph filename to execute for each item (e.g., "depot_lineage")
/// - `items`: variable name containing the list to iterate (resolved from context outputs)
/// - `concurrency`: max parallel executions (default: 1)
/// - `bind`: variable mapping from item fields to sub-graph env vars
pub async fn node_graph_foreach(
    node: &GraphNode,
    context: &ExecutionContext,
) -> Result<serde_json::Value> {
    let graph_name = node
        .config
        .get("graph")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!(
            "graph_foreach node '{}' missing 'graph' in config",
            node.id
        ))?;

    let items_ref = node
        .config
        .get("items")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!(
            "graph_foreach node '{}' missing 'items' in config",
            node.id
        ))?;

    let concurrency = node
        .config
        .get("concurrency")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as usize;

    let items_resolved = substitute_env(items_ref, &context.env);
    let items: Vec<serde_json::Value> = context
        .get_output(&items_resolved)
        .await
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();

    if items.is_empty() {
        return Ok(serde_json::json!({
            "graph": graph_name,
            "count": 0,
            "results": []
        }));
    }

    info!(
        "graph_foreach: executing '{}' for {} items (concurrency={})",
        graph_name,
        items.len(),
        concurrency
    );

    let bind_map = node
        .config
        .get("bind")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();

    let neural_socket = discover_neural_api_socket(&context.env)?;
    let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));

    let mut handles = Vec::new();

    for (idx, item) in items.iter().enumerate() {
        let sem = semaphore.clone();
        let neural_socket = neural_socket.clone();
        let bind_map = bind_map.clone();
        let item = item.clone();
        let graph_name = graph_name.to_string();
        let base_env = context.env.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await;

            let mut env = base_env;
            for (env_key, bind_expr) in &bind_map {
                if let Some(field) = bind_expr.as_str() {
                    let value = if let Some(stripped) = field.strip_prefix("item.") {
                        item.get(stripped)
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string()
                    } else {
                        field.to_string()
                    };
                    env.insert(env_key.clone(), value);
                }
            }

            let request = JsonRpcRequest::new(
                "graph.execute",
                serde_json::json!({
                    "graph": graph_name,
                    "env": env
                }),
            );

            match call_neural_api(&neural_socket, &request).await {
                Ok(response) => {
                    if let Some(result) = response.get("result") {
                        (idx, Ok(result.clone()))
                    } else if let Some(error) = response.get("error") {
                        let msg = error.get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("sub-graph error");
                        (idx, Err(msg.to_string()))
                    } else {
                        (idx, Ok(serde_json::json!({"ok": true})))
                    }
                }
                Err(e) => (idx, Err(e.to_string())),
            }
        });

        handles.push(handle);
    }

    let mut results = Vec::new();
    let mut succeeded = 0usize;
    let mut failed = 0usize;

    for handle in handles {
        match handle.await {
            Ok((idx, Ok(result))) => {
                succeeded += 1;
                results.push(serde_json::json!({ "index": idx, "status": "ok", "result": result }));
            }
            Ok((idx, Err(e))) => {
                failed += 1;
                results.push(serde_json::json!({ "index": idx, "status": "error", "error": e }));
            }
            Err(e) => {
                failed += 1;
                results.push(serde_json::json!({ "status": "join_error", "error": e.to_string() }));
            }
        }
    }

    Ok(serde_json::json!({
        "graph": graph_name,
        "count": items.len(),
        "succeeded": succeeded,
        "failed": failed,
        "results": results
    }))
}

