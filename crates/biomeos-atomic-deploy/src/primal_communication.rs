//! Primal Communication Module
//!
//! **Universal IPC v3.0**: Uses AtomicClient for multi-transport support.
//!
//! Pure Rust JSON-RPC for primal health verification and secure tunnel
//! establishment. Follows Deep Debt principles:
//! - Self-knowledge only - discover primals at runtime
//! - Capability-based communication
//! - Pure Rust JSON-RPC (no C dependencies)
//! - Multi-transport support (Unix, Abstract, TCP)

use anyhow::{Context, Result};
use biomeos_core::atomic_client::AtomicClient;
use serde_json::{json, Value};
use std::path::Path;
use std::time::Duration;
use tracing::debug;

/// Verify a primal is healthy and query its capabilities (Universal IPC v3.0)
///
/// Uses `AtomicClient` for multi-transport support.
/// Returns the list of capabilities the primal provides.
///
/// # Arguments
/// * `socket_path` - Path to the primal's Unix socket
/// * `primal_name` - Name of the primal (for logging)
///
/// # Returns
/// Vector of capability strings the primal provides
pub async fn verify_primal_health(socket_path: &Path, primal_name: &str) -> Result<Vec<String>> {
    // Create AtomicClient with 5 second timeout
    let client = AtomicClient::unix(socket_path).with_timeout(Duration::from_secs(5));

    // Query capabilities
    let response = client
        .call("primal.capabilities", json!({}))
        .await
        .context(format!("Failed to query capabilities from {}", primal_name))?;

    // Extract capabilities from result
    let capabilities = response
        .get("capabilities")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    debug!(
        "Primal {} provides capabilities: {:?}",
        primal_name, capabilities
    );

    Ok(capabilities)
}

/// Verify a primal is healthy using auto-discovery (Universal IPC v3.0)
///
/// Uses `AtomicClient::discover()` for automatic transport selection.
///
/// # Arguments
/// * `primal_name` - Name of the primal to discover and verify
///
/// # Returns
/// Vector of capability strings the primal provides
pub async fn verify_primal_health_with_discovery(primal_name: &str) -> Result<Vec<String>> {
    // Discover primal with automatic transport fallback
    let client = AtomicClient::discover(primal_name)
        .await
        .context(format!("Failed to discover primal: {}", primal_name))?;

    debug!("Discovered {} via {}", primal_name, client.endpoint());

    // Query capabilities
    let response = client
        .call("primal.capabilities", json!({}))
        .await
        .context(format!("Failed to query capabilities from {}", primal_name))?;

    // Extract capabilities from result
    let capabilities = response
        .get("capabilities")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    debug!(
        "Primal {} provides capabilities: {:?}",
        primal_name, capabilities
    );

    Ok(capabilities)
}

/// Establish a BTSP (BearDog Transport Security Protocol) tunnel (Universal IPC v3.0)
///
/// This creates a cryptographically secured channel for inter-primal
/// communication. The tunnel is authenticated using family lineage.
///
/// # Arguments
/// * `beardog_socket` - Path to BearDog's Unix socket
/// * `family_id` - The family identifier for lineage authentication
///
/// # Returns
/// Session ID for the established tunnel
pub async fn establish_btsp_tunnel(beardog_socket: &Path, family_id: &str) -> Result<String> {
    // Create AtomicClient with 10 second timeout (BTSP can take longer)
    let client = AtomicClient::unix(beardog_socket).with_timeout(Duration::from_secs(10));

    // Request BTSP tunnel establishment
    let response = client
        .call(
            "btsp.establish_tunnel",
            json!({
                "family_id": family_id,
                "tunnel_type": "local",
                "require_lineage_proof": false  // Local deployments don't require proof
            }),
        )
        .await
        .context("BTSP tunnel establishment failed")?;

    // Extract session ID
    let session_id = response
        .get("session_id")
        .and_then(|s| s.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing session_id in BTSP response"))?;

    Ok(session_id.to_string())
}

/// Establish a BTSP tunnel using auto-discovery (Universal IPC v3.0)
///
/// Uses `AtomicClient::discover()` to find BearDog automatically.
pub async fn establish_btsp_tunnel_with_discovery(family_id: &str) -> Result<String> {
    // Discover BearDog with automatic transport fallback
    let client = AtomicClient::discover("beardog")
        .await
        .context("Failed to discover BearDog for BTSP tunnel")?;

    debug!("Discovered BearDog via {} for BTSP", client.endpoint());

    // Request BTSP tunnel establishment
    let response = client
        .call(
            "btsp.establish_tunnel",
            json!({
                "family_id": family_id,
                "tunnel_type": "local",
                "require_lineage_proof": false
            }),
        )
        .await
        .context("BTSP tunnel establishment failed")?;

    // Extract session ID
    let session_id = response
        .get("session_id")
        .and_then(|s| s.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing session_id in BTSP response"))?;

    Ok(session_id.to_string())
}
