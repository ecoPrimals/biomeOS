// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Node execution handlers for Neural API graphs
//!
//! This module contains handlers for each node type in the graph execution system.
//! Each handler is a pure function that executes a specific operation.
//!
//! **Deep Debt Principle**: Node handlers are isolated, testable, and follow
//! capability-based discovery patterns - no hardcoded primal knowledge.

use anyhow::{Context as AnyhowContext, Result};
use biomeos_types::JsonRpcRequest;
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use super::context::ExecutionContext;
use crate::neural_graph::GraphNode;

/// Execute a filesystem existence check
pub async fn filesystem_check_exists(
    node: &GraphNode,
    context: &ExecutionContext,
) -> Result<Value> {
    let path = node
        .config
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("filesystem.check_exists requires 'path' config"))?;

    // Substitute environment variables
    let expanded_path = substitute_env(path, context.env());

    let exists = tokio::fs::metadata(&expanded_path).await.is_ok();

    info!("📁 Filesystem check: {} exists={}", expanded_path, exists);

    Ok(json!({
        "path": expanded_path,
        "exists": exists
    }))
}

/// Execute seed derivation via capability-based discovery
pub async fn crypto_derive_seed(node: &GraphNode, context: &ExecutionContext) -> Result<Value> {
    let source = node
        .config
        .get("source")
        .and_then(|v| v.as_str())
        .unwrap_or("family");

    let family_id = &context.family_id;

    // Capability-based: Find security provider via environment or discovery
    let security_socket = discover_capability_provider(context, "security").await;

    if let Some(socket_path) = security_socket {
        // Use security provider for real crypto derivation
        info!(
            "🔐 Using security provider at {} for seed derivation",
            socket_path
        );

        let request = JsonRpcRequest::new(
            "crypto.derive_seed",
            json!({
                "family_id": family_id,
                "source": source
            }),
        );

        let response = call_primal_rpc(&socket_path, &request).await?;

        if let Some(result) = response.get("result") {
            Ok(result.clone())
        } else if let Some(err) = response.get("error") {
            anyhow::bail!("Crypto derive failed: {err}");
        } else {
            anyhow::bail!("Invalid response from security provider");
        }
    } else {
        // Fallback: Generate deterministic seed from family_id
        warn!("⚠️  No security provider found, using deterministic fallback");
        let seed = format!("seed-{family_id}-{source}");
        Ok(json!({
            "seed": seed,
            "derived_from": source,
            "method": "deterministic_fallback"
        }))
    }
}

/// Execute primal launch via capability-based discovery
pub async fn primal_launch(node: &GraphNode, context: &ExecutionContext) -> Result<Value> {
    let primal_name = node
        .config
        .get("primal_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("primal.launch requires 'primal_name' config"))?;

    let mode = node
        .config
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("serve");

    let socket_path = context.get_socket_path(primal_name).await;

    info!("🚀 Launching primal: {} at {}", primal_name, socket_path);

    // Spawn the process using the spawner (handles discovery internally)
    let (child, tcp_port) =
        super::primal_spawner::spawn_primal_process(primal_name, mode, context, node).await?;

    if let Some(port) = tcp_port {
        super::primal_spawner::wait_for_tcp_port(port, 300).await?;
    } else {
        super::primal_spawner::wait_for_socket(&socket_path, 300).await?;
    }

    // Post-spawn: register capabilities with the Neural API router so the
    // primal is immediately routable via capability.call.
    if let Some(ref router) = context.neural_router {
        let socket = std::path::PathBuf::from(&socket_path);
        router
            .register_spawned_primal(primal_name, Some(&socket), tcp_port)
            .await;
    }

    let binary_path = super::primal_spawner::discover_primal_binary(primal_name, context).await?;

    info!(
        "✅ Primal {} launched successfully (pid: {})",
        primal_name,
        child.id().unwrap_or(0)
    );

    Ok(json!({
        "primal": primal_name,
        "socket": socket_path,
        "tcp_port": tcp_port,
        "binary": binary_path.display().to_string(),
        "status": "running"
    }))
}

/// Execute health check on a primal
pub async fn health_check(node: &GraphNode, context: &ExecutionContext) -> Result<Value> {
    let primal_name = node
        .config
        .get("primal_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("health_check requires 'primal_name' config"))?;

    let socket_path = context.get_socket_path(primal_name).await;
    let timeout_secs = node
        .config
        .get("timeout_secs")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(10);

    info!("🏥 Health check: {} at {}", primal_name, socket_path);

    let request = JsonRpcRequest::new("health.check", json!({}));

    // Call with timeout
    let response = tokio::time::timeout(
        Duration::from_secs(timeout_secs),
        call_primal_rpc(&socket_path, &request),
    )
    .await
    .map_err(|_| anyhow::anyhow!("Health check timeout after {timeout_secs}s"))??;

    let healthy = response
        .get("result")
        .and_then(|r| r.get("healthy"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    if healthy {
        info!("✅ {} is healthy", primal_name);
    } else {
        warn!("⚠️  {} health check failed", primal_name);
    }

    Ok(json!({
        "primal": primal_name,
        "healthy": healthy,
        "response": response
    }))
}

/// Register capabilities for a primal with the execution context.
///
/// Reads `config.primal_name` and `node.capabilities` to record which capabilities
/// a primal provides. Used by deployment graphs after a primal starts and passes
/// health checks.
pub async fn register_capabilities(node: &GraphNode, _context: &ExecutionContext) -> Result<Value> {
    let primal_name = node
        .config
        .get("primal_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let capabilities = &node.capabilities;

    if capabilities.is_empty() {
        warn!(
            "register_capabilities for '{}': no capabilities listed",
            primal_name
        );
    } else {
        info!(
            "Registering {} capabilities for '{}'",
            capabilities.len(),
            primal_name
        );
        for cap in capabilities {
            info!("   {} -> {}", cap, primal_name);
        }
    }

    Ok(json!({
        "primal": primal_name,
        "registered": capabilities,
        "count": capabilities.len()
    }))
}

/// Execute lineage verification via capability-based discovery
pub async fn lineage_verify(node: &GraphNode, context: &ExecutionContext) -> Result<Value> {
    let primal_name = node
        .config
        .get("primal_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("lineage.verify requires 'primal_name' config"))?;

    info!("🧬 Verifying lineage for: {}", primal_name);

    // Find security provider for verification
    let security_socket = discover_capability_provider(context, "security").await;

    if let Some(socket_path) = security_socket {
        let request = JsonRpcRequest::new(
            "lineage.verify",
            json!({
                "primal_name": primal_name,
                "family_id": context.family_id
            }),
        );

        let response = call_primal_rpc(&socket_path, &request).await?;

        if let Some(result) = response.get("result") {
            info!("✅ Lineage verified for {}", primal_name);
            Ok(result.clone())
        } else {
            warn!("⚠️  Lineage verification response missing result");
            Ok(json!({
                "verified": false,
                "error": "No result in response"
            }))
        }
    } else {
        warn!("⚠️  No security provider for lineage verification, assuming valid");
        Ok(json!({
            "verified": true,
            "method": "assumed_valid_no_provider"
        }))
    }
}

/// Log info message
pub async fn log_info(node: &GraphNode, context: &ExecutionContext) -> Result<Value> {
    let message = node
        .config
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("(no message)");

    let expanded = substitute_env(message, context.env());
    info!("📝 {}", expanded);

    Ok(json!({
        "level": "info",
        "message": expanded
    }))
}

/// Log warning message
pub async fn log_warn(node: &GraphNode, context: &ExecutionContext) -> Result<Value> {
    let message = node
        .config
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("(no message)");

    let expanded = substitute_env(message, context.env());
    warn!("⚠️  {}", expanded);

    Ok(json!({
        "level": "warn",
        "message": expanded
    }))
}

/// Log error message
pub async fn log_error(node: &GraphNode, context: &ExecutionContext) -> Result<Value> {
    let message = node
        .config
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("(no message)");

    let expanded = substitute_env(message, context.env());
    error!("❌ {}", expanded);

    Ok(json!({
        "level": "error",
        "message": expanded
    }))
}

/// Generate deployment report
pub async fn deployment_report(node: &GraphNode, context: &ExecutionContext) -> Result<Value> {
    let title = node
        .config
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Deployment Report");

    let statuses = context.all_statuses().await;

    let completed = statuses
        .values()
        .filter(|s| matches!(s, super::context::NodeStatus::Completed(_)))
        .count();

    let failed = statuses
        .values()
        .filter(|s| matches!(s, super::context::NodeStatus::Failed(_)))
        .count();

    info!("📊 {}: {} completed, {} failed", title, completed, failed);

    Ok(json!({
        "title": title,
        "completed": completed,
        "failed": failed,
        "total": statuses.len(),
        "success": failed == 0
    }))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Substitute environment variables in a string
///
/// Supports both ${VAR} and $VAR syntax.
#[expect(
    clippy::implicit_hasher,
    reason = "default hasher sufficient for env substitution"
)]
#[must_use]
pub fn substitute_env(s: &str, env: &HashMap<String, String>) -> String {
    let mut result = s.to_string();

    for (key, value) in env {
        result = result.replace(&format!("${{{key}}}"), value);
        result = result.replace(&format!("${key}"), value);
    }

    result
}

/// Discover a capability provider via environment-based discovery
///
/// **Deep Debt Principle**: No hardcoded primal names - only capabilities.
/// Discovery order:
/// 1. Environment variable ({CAPABILITY}_SOCKET or {CAPABILITY}_ENDPOINT)
/// 2. (REMOVED) Well-known primal names - evolved to runtime-only discovery
///
/// Note: This function no longer falls back to hardcoded primal names.
/// Users must explicitly configure capability providers via environment variables
/// or ensure the Neural API capability registry is accessible for runtime discovery.
async fn discover_capability_provider(
    context: &ExecutionContext,
    capability: &str,
) -> Option<String> {
    let cap_upper = capability.to_uppercase();

    // 1. Check environment for explicit socket
    if let Some(socket) = context.env().get(&format!("{cap_upper}_SOCKET")) {
        if tokio::fs::metadata(socket).await.is_ok() {
            debug!(
                "Found {} provider via {}_SOCKET: {}",
                capability, cap_upper, socket
            );
            return Some(socket.clone());
        }
    }

    // 2. Check for endpoint environment variable
    if let Some(endpoint) = context.env().get(&format!("{cap_upper}_ENDPOINT")) {
        debug!(
            "Found {} provider via {}_ENDPOINT: {}",
            capability, cap_upper, endpoint
        );
        return Some(endpoint.clone());
    }

    // 3. No hardcoded fallback — require explicit configuration.
    // TRUE PRIMAL principle: no compile-time knowledge of specific primal names.
    debug!(
        "No provider found for capability '{}'. \
         Resolution options: \
         (1) Start a primal providing '{}' capability, \
         (2) Set {}_ENDPOINT environment variable, or \
         (3) Ensure Neural API capability registry is accessible.",
        capability,
        capability,
        capability.to_uppercase()
    );

    None
}

/// Call a primal via JSON-RPC over Unix socket with BTSP awareness.
///
/// Uses BTSP handshake for family-scoped sockets in production mode,
/// falls back to cleartext JSON-RPC for development or non-family sockets.
///
/// Returns the full JSON-RPC response envelope (preserving `result`/`error`
/// fields for callers that inspect the envelope shape).
async fn call_primal_rpc(socket_path: &str, request: &impl Serialize) -> Result<Value> {
    use biomeos_core::btsp_client;
    use std::path::Path;
    use tokio::time::timeout;

    let connect_timeout =
        Duration::from_millis(biomeos_types::constants::timeouts::DEFAULT_CONNECTION_TIMEOUT_MS);
    let read_timeout = biomeos_types::constants::DEFAULT_REQUEST_TIMEOUT;
    let request_json = serde_json::to_string(request)?;

    let path = Path::new(socket_path);
    let use_btsp = btsp_client::is_family_scoped_socket(path)
        && matches!(
            btsp_client::security_mode(),
            btsp_client::SecurityMode::Production {
                btsp_available: true
            }
        );

    let stream = timeout(
        connect_timeout,
        tokio::net::UnixStream::connect(socket_path),
    )
    .await
    .with_context(|| format!("Connect timeout ({connect_timeout:?}) to {socket_path}"))?
    .with_context(|| format!("Failed to connect to {socket_path}"))?;

    if use_btsp {
        debug!("Executor RPC: BTSP handshake + Phase 3 for {socket_path}");
        match biomeos_core::btsp_client_phase3::perform_client_handshake_phase3(stream).await {
            Ok(biomeos_core::btsp_client_phase3::ClientPhase3Outcome::Encrypted {
                keys,
                stream,
            }) => {
                return call_primal_rpc_encrypted(
                    stream,
                    &request_json,
                    &keys,
                    read_timeout,
                    socket_path,
                )
                .await;
            }
            Ok(biomeos_core::btsp_client_phase3::ClientPhase3Outcome::Plaintext { stream }) => {
                return call_primal_rpc_plaintext(stream, &request_json, read_timeout, socket_path)
                    .await;
            }
            Err(e) => {
                warn!(
                    "Executor RPC: BTSP handshake failed for {socket_path}, falling back to cleartext: {e}"
                );
                let stream = timeout(
                    connect_timeout,
                    tokio::net::UnixStream::connect(socket_path),
                )
                .await
                .with_context(|| {
                    format!("Reconnect timeout ({connect_timeout:?}) to {socket_path}")
                })?
                .with_context(|| format!("Failed to reconnect to {socket_path}"))?;
                return call_primal_rpc_plaintext(stream, &request_json, read_timeout, socket_path)
                    .await;
            }
        }
    }

    call_primal_rpc_plaintext(stream, &request_json, read_timeout, socket_path).await
}

async fn call_primal_rpc_plaintext(
    stream: tokio::net::UnixStream,
    request_json: &str,
    read_timeout: Duration,
    socket_path: &str,
) -> Result<Value> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
    use tokio::time::timeout;

    let (read_half, mut write_half) = stream.into_split();

    write_half.write_all(request_json.as_bytes()).await?;
    write_half.write_all(b"\n").await?;
    write_half.flush().await?;

    let mut reader = tokio::io::BufReader::new(read_half);
    let mut response_line = String::new();
    timeout(read_timeout, reader.read_line(&mut response_line))
        .await
        .with_context(|| format!("Read timeout ({read_timeout:?}) from {socket_path}"))??;

    let response: Value = serde_json::from_str(&response_line)
        .with_context(|| format!("Invalid JSON response from {socket_path}"))?;

    Ok(response)
}

async fn call_primal_rpc_encrypted(
    stream: tokio::net::UnixStream,
    request_json: &str,
    keys: &biomeos_core::btsp_crypto::SessionKeys,
    read_timeout: Duration,
    socket_path: &str,
) -> Result<Value> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::time::timeout;

    let frame =
        biomeos_core::btsp_crypto::encrypt_frame(&keys.client_to_server, request_json.as_bytes())
            .map_err(|e| anyhow::anyhow!("encrypt_frame failed for {socket_path}: {e}"))?;

    let (mut read_half, mut write_half) = stream.into_split();
    write_half.write_all(&frame).await?;
    write_half.flush().await?;

    let mut len_buf = [0u8; 4];
    timeout(read_timeout, read_half.read_exact(&mut len_buf))
        .await
        .with_context(|| format!("Read timeout ({read_timeout:?}) from {socket_path}"))??;

    let payload_len = u32::from_be_bytes(len_buf) as usize;
    if payload_len > 16 * 1024 * 1024 {
        anyhow::bail!("Response frame too large from {socket_path}: {payload_len} bytes");
    }

    let mut payload = vec![0u8; payload_len];
    read_half
        .read_exact(&mut payload)
        .await
        .with_context(|| format!("Failed to read response frame from {socket_path}"))?;

    let plaintext = biomeos_core::btsp_crypto::decrypt_frame(&keys.server_to_client, &payload)
        .map_err(|e| anyhow::anyhow!("decrypt_frame failed for {socket_path}: {e}"))?;

    let response: Value = serde_json::from_slice(&plaintext)
        .with_context(|| format!("Invalid decrypted JSON response from {socket_path}"))?;

    Ok(response)
}

#[cfg(test)]
#[path = "node_handlers_tests.rs"]
mod tests;
