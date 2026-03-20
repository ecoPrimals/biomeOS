// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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
use serde_json::json;
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
        .context(format!("Failed to query capabilities from {primal_name}"))?;

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
        .context(format!("Failed to discover primal: {primal_name}"))?;

    debug!("Discovered {} via {}", primal_name, client.endpoint());

    // Query capabilities
    let response = client
        .call("primal.capabilities", json!({}))
        .await
        .context(format!("Failed to query capabilities from {primal_name}"))?;

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
    // Discover security provider with automatic transport fallback
    // DEEP DEBT EVOLUTION: Resolve provider name from env, not hardcoded
    let security_provider =
        std::env::var("BIOMEOS_SECURITY_PROVIDER").unwrap_or_else(|_| "beardog".to_string());
    let client = AtomicClient::discover(&security_provider)
        .await
        .context(format!(
            "Failed to discover {security_provider} for BTSP tunnel"
        ))?;

    debug!(
        "Discovered {} via {} for BTSP",
        security_provider,
        client.endpoint()
    );

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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use biomeos_test_utils::MockJsonRpcServer;
    use serde_json::json;

    #[test]
    fn test_capabilities_response_parsing() {
        let response =
            json!({ "capabilities": ["crypto.encrypt", "crypto.decrypt", "genetic.verify"] });
        let capabilities = response
            .get("capabilities")
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        assert_eq!(capabilities.len(), 3);
        assert!(capabilities.contains(&"crypto.encrypt".to_string()));
    }

    #[test]
    fn test_capabilities_response_empty() {
        let response = json!({ "capabilities": [] });
        let capabilities = response
            .get("capabilities")
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        assert!(capabilities.is_empty());
    }

    #[test]
    fn test_capabilities_response_missing() {
        let response = json!({ "status": "ok" });
        let capabilities = response
            .get("capabilities")
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        assert!(capabilities.is_empty());
    }

    #[test]
    fn test_btsp_request_structure() {
        let request = json!({ "family_id": "test-family", "tunnel_type": "local", "require_lineage_proof": false });
        assert_eq!(request["family_id"], "test-family");
        assert_eq!(request["tunnel_type"], "local");
    }

    #[test]
    fn test_btsp_session_id_parsing() {
        let response = json!({ "session_id": "btsp-session-abc123", "established": true });
        let session_id = response
            .get("session_id")
            .and_then(|s| s.as_str())
            .map(String::from);
        assert_eq!(session_id, Some("btsp-session-abc123".to_string()));
    }

    #[test]
    fn test_btsp_session_id_missing() {
        let response = json!({ "error": "tunnel establishment failed" });
        assert!(
            response
                .get("session_id")
                .and_then(|s| s.as_str())
                .is_none()
        );
    }

    /// verify_primal_health with non-existent socket — connection refused
    #[tokio::test]
    async fn test_verify_primal_health_connection_refused() {
        let path = std::path::Path::new("/nonexistent/socket/primal_12345.sock");
        let result = verify_primal_health(path, "test-primal").await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Failed")
                || err.contains("connect")
                || err.contains("No such file")
                || err.contains("Connection"),
            "Expected connection error, got: {err}"
        );
    }

    /// verify_primal_health_with_discovery with nonexistent primal
    #[tokio::test]
    async fn test_verify_primal_health_with_discovery_failure() {
        let result = verify_primal_health_with_discovery("nonexistent_primal_xyz").await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Failed") || err.contains("discover") || err.contains("not found"));
    }

    /// establish_btsp_tunnel with non-existent beardog socket
    #[tokio::test]
    async fn test_establish_btsp_tunnel_connection_refused() {
        let path = std::path::Path::new("/nonexistent/beardog_12345.sock");
        let result = establish_btsp_tunnel(path, "test-family").await;
        assert!(result.is_err());
    }

    /// establish_btsp_tunnel_with_discovery — fails when beardog not found
    #[tokio::test]
    async fn test_establish_btsp_tunnel_with_discovery_failure() {
        let result = establish_btsp_tunnel_with_discovery("test-family").await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Failed") || err.contains("discover") || err.contains("not found"));
    }

    #[test]
    fn test_capabilities_response_non_string_items_filtered() {
        let response = json!({ "capabilities": ["a", 123, "b", null, "c"] });
        let capabilities = response
            .get("capabilities")
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        assert_eq!(capabilities, vec!["a", "b", "c"]);
    }

    #[tokio::test]
    async fn test_verify_primal_health_success_mock_socket() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("primal-mock.sock");
        let _server = MockJsonRpcServer::spawn_echo_success(
            &sock,
            json!({ "capabilities": ["security", "crypto.lineage"] }),
        )
        .await;

        let caps = verify_primal_health(&sock, "mock-p").await.expect("health");
        assert_eq!(caps.len(), 2);
        assert!(caps.iter().any(|c| c == "security"));
    }

    #[tokio::test]
    async fn test_establish_btsp_tunnel_success_mock_socket() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("beardog-mock.sock");
        let _server =
            MockJsonRpcServer::spawn_echo_success(&sock, json!({ "session_id": "btsp-unit-1" }))
                .await;

        let sid = establish_btsp_tunnel(&sock, "family-x")
            .await
            .expect("btsp");
        assert_eq!(sid, "btsp-unit-1");
    }

    #[tokio::test]
    async fn test_establish_btsp_tunnel_missing_session_id() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("beardog-mock2.sock");
        let _server =
            MockJsonRpcServer::spawn_echo_success(&sock, json!({ "established": true })).await;

        let err = establish_btsp_tunnel(&sock, "f").await.unwrap_err();
        assert!(err.to_string().contains("session_id") || err.to_string().contains("Missing"));
    }
}
