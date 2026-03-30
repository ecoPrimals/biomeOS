// SPDX-License-Identifier: AGPL-3.0-only
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
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
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
        // Use BearDog for real crypto derivation
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
    let child =
        super::primal_spawner::spawn_primal_process(primal_name, mode, context, node).await?;

    // Wait for socket to be available (300 attempts = 30 seconds at 100ms each)
    super::primal_spawner::wait_for_socket(&socket_path, 300).await?;

    // Get the binary path for the response
    let binary_path = super::primal_spawner::discover_primal_binary(primal_name, context).await?;

    info!(
        "✅ Primal {} launched successfully (pid: {})",
        primal_name,
        child.id().unwrap_or(0)
    );

    Ok(json!({
        "primal": primal_name,
        "socket": socket_path,
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

    // 3. No hardcoded fallback - require explicit configuration
    // EVOLVED: Removed hardcoded capability→primal mappings
    // TRUE PRIMAL principle: No compile-time knowledge of specific primal names
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

/// Call a primal via JSON-RPC over Unix socket
///
/// **Deep Debt Principle**: Pure JSON-RPC, no HTTP dependencies.
async fn call_primal_rpc(socket_path: &str, request: &impl Serialize) -> Result<Value> {
    let request_json = serde_json::to_string(request)?;
    let stream = UnixStream::connect(socket_path)
        .await
        .with_context(|| format!("Failed to connect to {socket_path}"))?;

    let (read_half, mut write_half) = stream.into_split();

    // Send request
    write_half.write_all(request_json.as_bytes()).await?;
    write_half.write_all(b"\n").await?;
    write_half.flush().await?;

    // Read response
    let mut reader = BufReader::new(read_half);
    let mut response_line = String::new();
    reader.read_line(&mut response_line).await?;

    let response: Value = serde_json::from_str(&response_line)
        .with_context(|| format!("Invalid JSON response from {socket_path}"))?;

    Ok(response)
}

#[cfg(test)]
#[path = "node_handlers_tests.rs"]
mod tests;
