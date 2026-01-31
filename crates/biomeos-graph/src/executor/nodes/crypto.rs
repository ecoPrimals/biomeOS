//! Crypto node executors
//!
//! **TRUE ecoBin v2.0:** Delegates ALL crypto to BearDog (no reimplementation).
//!
//! Node types handled:
//! - `crypto.derive_child_seed` - Derives child seed via BearDog
//!
//! ## Deep Debt Principles
//!
//! - **No reimplementation:** BearDog handles ALL cryptographic operations
//! - **Capability-based:** Discovers BearDog by capability, not hardcoded name
//! - **Pure Rust:** JSON-RPC over Unix socket (no HTTP/TLS)
//! - **Platform-agnostic:** Works on any platform with Unix sockets

use crate::executor::context::ExecutionContext;
use crate::executor::helpers::{discover_beardog_socket, parse_config, substitute_env};
use crate::graph::GraphNode;
use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::debug;

/// Execute: crypto.derive_child_seed
///
/// Derives a child seed from a parent seed using BearDog's secure derivation.
///
/// **EVOLVED (Jan 27, 2026):** Now delegates to BearDog primal via JSON-RPC.
///
/// # Config Parameters
///
/// - `parent_seed` (required): Path to parent seed file or environment variable
/// - `node_id` (required): Identifier for this child node
/// - `output_path` (required): Where to write the derived seed
/// - `deployment_batch` (optional): Batch identifier for deployment tracking
///
/// # Returns
///
/// ```json
/// {
///   "seed_path": "/path/to/derived/seed",
///   "node_id": "nat0_child1",
///   "derivation_success": true
/// }
/// ```
pub async fn derive_child_seed(
    node: &GraphNode,
    context: &ExecutionContext,
) -> Result<serde_json::Value> {
    // Extract required parameters
    let parent_seed: String = parse_config(&node.config, "parent_seed")?;
    let parent_seed = substitute_env(&parent_seed, &context.env);

    let node_id: String = parse_config(&node.config, "node_id")?;

    let output_path: String = parse_config(&node.config, "output_path")?;
    let output_path = substitute_env(&output_path, &context.env);

    // Optional deployment batch
    let deployment_batch: Option<String> = node
        .config
        .get("deployment_batch")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Discover BearDog socket (capability-based, NO hardcoding)
    let beardog_socket = discover_beardog_socket(&context.env)?;

    debug!(
        "Calling BearDog for seed derivation: node_id={}, output={}",
        node_id, output_path
    );

    // Connect to BearDog via Unix socket
    let stream = UnixStream::connect(&beardog_socket)
        .await
        .context(format!(
            "Failed to connect to BearDog at {}. Ensure BearDog is running.",
            beardog_socket
        ))?;

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Prepare JSON-RPC 2.0 request to BearDog
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

    // Send request (newline-delimited JSON)
    let request_str = serde_json::to_string(&request)? + "\n";
    writer
        .write_all(request_str.as_bytes())
        .await
        .context("Failed to send request to BearDog")?;
    writer.flush().await?;

    // Read response (newline-delimited)
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .await
        .context("Failed to read response from BearDog")?;

    // Parse JSON-RPC 2.0 response
    let response: serde_json::Value =
        serde_json::from_str(response_line.trim()).context("Invalid JSON from BearDog")?;

    // Check for JSON-RPC error
    if let Some(error) = response.get("error") {
        let message = error
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown BearDog error");
        let code = error.get("code").and_then(|c| c.as_i64()).unwrap_or(-1);

        anyhow::bail!(
            "BearDog seed derivation failed (code {}): {}",
            code,
            message
        );
    }

    // Extract and return the result
    let result = response
        .get("result")
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("BearDog returned empty result"))?;

    debug!("   Seed derived successfully: {}", output_path);

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Note: These tests require a running BearDog instance
    // They should be in integration tests, not unit tests

    #[tokio::test]
    #[ignore] // Requires BearDog running
    async fn test_derive_child_seed() {
        let node = GraphNode {
            id: "test_crypto".to_string(),
            node_type: "crypto.derive_child_seed".to_string(),
            config: serde_json::json!({
                "parent_seed": "/tmp/parent.seed",
                "node_id": "test_node",
                "output_path": "/tmp/child.seed"
            }),
            dependencies: vec![],
        };

        let env = HashMap::from([("XDG_RUNTIME_DIR".to_string(), "/tmp".to_string())]);
        let context = ExecutionContext::new(env);

        // This will fail unless BearDog is actually running
        let result = derive_child_seed(&node, &context).await;

        // In a real integration test, we would:
        // 1. Start BearDog
        // 2. Run this test
        // 3. Stop BearDog
        // For now, we just verify it compiles
        drop(result);
    }
}
