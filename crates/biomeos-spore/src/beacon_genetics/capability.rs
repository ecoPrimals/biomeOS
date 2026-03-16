// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Capability Caller - NeuralAPI and Direct Integration
//!
//! Abstracts capability.call mechanism for beacon genetics operations.
//!
//! ## Architecture
//!
//! ```text
//! BeaconGeneticsManager (ecosystem knowledge)
//!     │
//!     │ capability.call("beacon.encrypt", params)
//!     ▼
//! CapabilityCaller trait
//!     │
//!     ├── NeuralApiCapabilityCaller (production - uses AtomicClient via NeuralAPI)
//!     │   └── Connects to NeuralAPI socket, routes to primals
//!     │
//!     ├── DirectBeardogCaller (enrollment - calls BearDog directly)
//!     │   └── Connects directly to BearDog socket
//!     │
//!     └── MockCapabilityCaller (testing - returns preset responses)
//! ```
//!
//! Uses Universal IPC v3.0 AtomicClient for multi-transport support.

use biomeos_core::atomic_client::AtomicClient;
use biomeos_types::constants::ports;
use tracing::debug;

/// Trait for calling capabilities via neuralAPI.
///
/// This abstracts the actual RPC mechanism, allowing:
/// - Real capability.call via CapabilityTranslationRegistry
/// - Mock implementations for testing
/// - Different transport mechanisms
///
/// Callers pass dotted semantic names (e.g. `"beacon.encrypt"`); the
/// implementation splits them into the canonical `{ capability, operation, args }`
/// format before forwarding to the Neural API.
#[async_trait::async_trait]
pub trait CapabilityCaller: Send + Sync {
    /// Call a semantic capability.
    ///
    /// # Arguments
    /// * `capability` - Dotted semantic name (e.g., `"beacon.encrypt"`)
    /// * `params` - Operation arguments as JSON value
    ///
    /// # Returns
    /// Result from the provider primal
    async fn call(
        &self,
        capability: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String>;
}

/// Default capability caller using AtomicClient to neuralAPI
///
/// Uses Universal IPC v3.0 for multi-transport support.
pub struct NeuralApiCapabilityCaller {
    /// Path to neuralAPI socket
    neural_api_socket: String,
}

impl NeuralApiCapabilityCaller {
    /// Create new caller
    pub fn new(neural_api_socket: &str) -> Self {
        Self {
            neural_api_socket: neural_api_socket.to_string(),
        }
    }

    /// Get default neuralAPI socket path
    pub fn default_socket() -> String {
        // XDG-compliant path
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            format!("{runtime_dir}/biomeos/neural-api.sock")
        } else if let Ok(uid) = std::env::var("UID") {
            format!("/run/user/{uid}/biomeos/neural-api.sock")
        } else {
            std::env::temp_dir()
                .join("biomeos/neural-api.sock")
                .to_string_lossy()
                .to_string()
        }
    }
}

#[async_trait::async_trait]
impl CapabilityCaller for NeuralApiCapabilityCaller {
    async fn call(
        &self,
        capability: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        // Create AtomicClient for neuralAPI socket (Universal IPC v3.0)
        let client = AtomicClient::unix(&self.neural_api_socket);

        // Build capability.call params in canonical format:
        // { capability: "domain", operation: "method", args: {...} }
        // Split dotted capability names (e.g. "beacon.encrypt" → "beacon" + "encrypt")
        let (domain, operation) = if let Some(dot_pos) = capability.find('.') {
            (&capability[..dot_pos], &capability[dot_pos + 1..])
        } else {
            (capability, "call")
        };

        let call_params = serde_json::json!({
            "capability": domain,
            "operation": operation,
            "args": params
        });

        // Call neuralAPI's capability.call method using AtomicClient
        client
            .call("capability.call", call_params)
            .await
            .map_err(|e| format!("Capability call failed: {e}"))
    }
}

/// Direct BearDog caller - calls BearDog methods directly
///
/// Unlike NeuralApiCapabilityCaller which goes through Neural API's semantic
/// routing, this caller connects directly to a BearDog socket and translates
/// capability names to BearDog's actual method names.
///
/// ## Usage
///
/// This is useful for enrollment scenarios where:
/// 1. Neural API may not be running
/// 2. Direct BearDog access is needed for bootstrapping
/// 3. Simpler deployment without full Neural API stack
///
/// ## Endpoint Formats
///
/// - **Unix socket**: `/path/to/socket.sock`
/// - **TCP**: `tcp:host:port` (e.g., `tcp:127.0.0.1:9900`)
pub struct DirectBeardogCaller {
    /// Endpoint to BearDog (socket path or tcp:host:port)
    beardog_endpoint: String,
}

impl DirectBeardogCaller {
    /// Create new direct BearDog caller
    ///
    /// # Arguments
    /// * `endpoint` - Either a Unix socket path or "tcp:host:port"
    pub fn new(endpoint: &str) -> Self {
        Self {
            beardog_endpoint: endpoint.to_string(),
        }
    }

    /// Get default BearDog socket path
    pub fn default_socket() -> String {
        // XDG-compliant path
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            format!("{runtime_dir}/biomeos/beardog.sock")
        } else if let Ok(uid) = std::env::var("UID") {
            format!("/run/user/{uid}/biomeos/beardog.sock")
        } else {
            std::env::temp_dir()
                .join("biomeos/beardog.sock")
                .to_string_lossy()
                .to_string()
        }
    }

    /// Translate semantic capability name to BearDog method
    ///
    /// BearDog uses flat method names like "genetic.derive_lineage_key"
    /// while semantic capabilities might be expressed differently.
    fn translate_capability<'a>(&self, capability: &'a str) -> &'a str {
        // Most capabilities map directly to BearDog methods
        // This provides a hook for future translation if needed
        capability
    }

    /// Create AtomicClient based on endpoint format
    fn create_client(&self) -> AtomicClient {
        if self.beardog_endpoint.starts_with("tcp:") {
            // Parse tcp:host:port format
            let addr = &self.beardog_endpoint[4..]; // Skip "tcp:"
            let parts: Vec<&str> = addr.rsplitn(2, ':').collect();
            if parts.len() == 2 {
                let port: u16 = parts[0].parse().unwrap_or(ports::NEURAL_API);
                let host = parts[1];
                debug!("Creating TCP client to {}:{}", host, port);
                AtomicClient::tcp(host, port)
            } else {
                // Fallback to default if parse fails
                debug!(
                    "Invalid TCP endpoint format, falling back to Unix: {}",
                    self.beardog_endpoint
                );
                AtomicClient::unix(&self.beardog_endpoint)
            }
        } else {
            // Unix socket path
            debug!("Creating Unix socket client: {}", self.beardog_endpoint);
            AtomicClient::unix(&self.beardog_endpoint)
        }
    }
}

#[async_trait::async_trait]
impl CapabilityCaller for DirectBeardogCaller {
    async fn call(
        &self,
        capability: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let method = self.translate_capability(capability);
        debug!(
            "DirectBeardogCaller: calling {} (mapped from {})",
            method, capability
        );

        // Create AtomicClient based on endpoint type (Universal IPC v3.0)
        let client = self.create_client();

        // Call BearDog directly with the translated method name
        client
            .call(method, params)
            .await
            .map_err(|e| format!("Direct BearDog call failed: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════════
    // NeuralApiCapabilityCaller tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_neural_api_default_socket() {
        let socket = NeuralApiCapabilityCaller::default_socket();
        assert!(socket.contains("neural-api.sock"));
    }

    #[test]
    fn test_neural_api_caller_new() {
        let caller = NeuralApiCapabilityCaller::new("/tmp/test.sock");
        assert_eq!(caller.neural_api_socket, "/tmp/test.sock");
    }

    #[test]
    fn test_neural_api_caller_new_custom_path() {
        let caller = NeuralApiCapabilityCaller::new("/run/user/1000/biomeos/custom.sock");
        assert_eq!(
            caller.neural_api_socket,
            "/run/user/1000/biomeos/custom.sock"
        );
    }

    // ═══════════════════════════════════════════════════════════════
    // DirectBeardogCaller tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_direct_beardog_new_unix() {
        let caller = DirectBeardogCaller::new("/tmp/beardog.sock");
        assert_eq!(caller.beardog_endpoint, "/tmp/beardog.sock");
    }

    #[test]
    fn test_direct_beardog_new_tcp() {
        let caller = DirectBeardogCaller::new("tcp:127.0.0.1:9000");
        assert_eq!(caller.beardog_endpoint, "tcp:127.0.0.1:9000");
    }

    #[test]
    fn test_direct_beardog_default_socket() {
        let socket = DirectBeardogCaller::default_socket();
        assert!(socket.contains("beardog.sock"));
    }

    #[test]
    fn test_translate_capability_passthrough() {
        let caller = DirectBeardogCaller::new("/tmp/test.sock");
        assert_eq!(
            caller.translate_capability("beacon.encrypt"),
            "beacon.encrypt"
        );
        assert_eq!(
            caller.translate_capability("genetic.derive_lineage_key"),
            "genetic.derive_lineage_key"
        );
        assert_eq!(caller.translate_capability("crypto.sign"), "crypto.sign");
    }

    #[test]
    fn test_create_client_unix() {
        let caller = DirectBeardogCaller::new("/tmp/beardog.sock");
        let _client = caller.create_client();
        // Just verify no panic on construction
    }

    #[test]
    fn test_create_client_tcp() {
        let caller = DirectBeardogCaller::new("tcp:127.0.0.1:9000");
        let _client = caller.create_client();
        // Just verify no panic on construction
    }

    #[test]
    fn test_create_client_tcp_default_port() {
        // Invalid port should default to 9000
        let caller = DirectBeardogCaller::new("tcp:host:badport");
        let _client = caller.create_client();
    }

    #[test]
    fn test_create_client_tcp_no_port() {
        // Single part after "tcp:" → falls back to unix
        let caller = DirectBeardogCaller::new("tcp:justhost");
        let _client = caller.create_client();
    }

    // ═══════════════════════════════════════════════════════════════
    // CapabilityCaller trait object tests
    // ═══════════════════════════════════════════════════════════════

    /// Verify that callers can be used as trait objects
    #[test]
    fn test_caller_as_boxed_trait() {
        let _caller: Box<dyn CapabilityCaller> =
            Box::new(NeuralApiCapabilityCaller::new("/tmp/test.sock"));
        // Just ensure the boxing works — the trait is object-safe
    }

    #[test]
    fn test_direct_caller_as_boxed_trait() {
        let _caller: Box<dyn CapabilityCaller> =
            Box::new(DirectBeardogCaller::new("/tmp/beardog.sock"));
    }
}
