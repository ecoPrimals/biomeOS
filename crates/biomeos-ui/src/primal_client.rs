// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Primal Client - Pure Rust JSON-RPC Communication
//!
//! EVOLVED (Jan 27, 2026): Extracted from orchestrator.rs for reuse
//!
//! This module provides type-safe wrappers around AtomicClient for
//! communicating with different primals via JSON-RPC over Unix sockets.
//!
//! ## Deep Debt Principles
//!
//! - **No C dependencies**: Uses AtomicClient (pure Rust)
//! - **Capability-based**: Discovers primals at runtime
//! - **No hardcoding**: Socket paths via SystemPaths
//! - **Graceful degradation**: Missing primals don't break the system

use anyhow::Result;
use biomeos_core::atomic_client::AtomicClient;
use serde_json::Value;
use tracing::{debug, info};

/// Primal client wrapper for type-safe JSON-RPC communication
///
/// This provides a thin wrapper around AtomicClient that adds:
/// - Primal-specific method naming
/// - Logging and debugging
/// - Error context
#[derive(Debug, Clone)]
pub struct PrimalClient {
    /// The underlying atomic client
    client: AtomicClient,
    /// Primal name for debugging
    primal_name: String,
}

impl PrimalClient {
    /// Discover a primal by name using capability-based discovery
    ///
    /// This uses SystemPaths for XDG-compliant socket discovery.
    /// No hardcoded paths!
    pub async fn discover(primal_name: &str) -> Result<Self> {
        debug!("Discovering primal: {}", primal_name);
        let client = AtomicClient::discover(primal_name).await?;
        info!("✅ Discovered primal: {}", primal_name);
        Ok(Self {
            client,
            primal_name: primal_name.to_string(),
        })
    }

    /// Create a client with explicit socket path
    ///
    /// Use this when you already know the socket location.
    pub fn with_socket(primal_name: &str, socket_path: impl AsRef<std::path::Path>) -> Self {
        Self {
            client: AtomicClient::new(socket_path),
            primal_name: primal_name.to_string(),
        }
    }

    /// Call a JSON-RPC method on this primal
    pub async fn call(&self, method: &str, params: Value) -> Result<Value> {
        debug!("{}: calling {}", self.primal_name, method);
        self.client.call(method, params).await
    }

    /// Get the primal name
    pub fn name(&self) -> &str {
        &self.primal_name
    }

    /// Check if the primal socket is available
    pub fn is_available(&self) -> bool {
        self.client.is_available()
    }
}

/// Type alias for PetalTongue UI framework client
pub type PetalTongueClient = PrimalClient;
/// Type alias for Songbird discovery/networking client
pub type SongbirdClient = PrimalClient;
/// Type alias for BearDog security/crypto client
pub type BearDogClient = PrimalClient;
/// Type alias for NestGate storage client
pub type NestGateClient = PrimalClient;
/// Type alias for ToadStool compute/GPU client
pub type ToadStoolClient = PrimalClient;
/// Type alias for Squirrel lightweight storage client
pub type SquirrelClient = PrimalClient;

/// Dynamic primal connection registry
///
/// DEEP DEBT EVOLUTION (Feb 7, 2026): Replaced fixed-field struct with
/// `HashMap<String, PrimalClient>`. This allows ANY primal to be discovered
/// at runtime without code changes. Typed accessors provide backward compatibility.
#[derive(Debug, Clone, Default)]
pub struct PrimalConnections {
    /// Dynamic registry of discovered primals (name → client)
    clients: std::collections::HashMap<String, PrimalClient>,
}

impl PrimalConnections {
    /// Discover all primals for the given family
    ///
    /// DEEP DEBT EVOLUTION: Scans the runtime socket directory for ANY primal,
    /// rather than hardcoding a list of 6 names. Unknown primals are discovered
    /// and accessible via `get()`.
    pub async fn discover_all(family_id: &str) -> Self {
        let mut connections = Self::default();

        info!("🔍 Discovering primals for family: {}", family_id);

        // Phase 1: Scan runtime directory for all .sock files (dynamic discovery)
        if let Ok(paths) = biomeos_types::SystemPaths::new() {
            let runtime_dir = paths.runtime_dir();
            if let Ok(entries) = std::fs::read_dir(runtime_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "sock")
                        && let Some(name) = path.file_stem().and_then(|s| s.to_str())
                    {
                        // Strip family_id suffix if present (e.g., "beardog-family" → "beardog")
                        let base_name = name.split('-').next().unwrap_or(name);
                        let client = PrimalClient::with_socket(base_name, &path);
                        debug!("   Found socket: {} → {}", base_name, path.display());
                        connections.clients.insert(base_name.to_string(), client);
                    }
                }
            }
        }

        // Phase 2: Try bootstrap names for any not found via directory scan
        // Uses CapabilityTaxonomy::known_primals() for the bootstrap hint list
        // In strict discovery mode, this returns empty (no hardcoded names)
        {
            let bootstrap_names = biomeos_types::CapabilityTaxonomy::known_primals();
            for name in bootstrap_names {
                if !connections.clients.contains_key(*name) {
                    match PrimalClient::discover(name).await {
                        Ok(client) => {
                            connections.clients.insert(name.to_string(), client);
                        }
                        Err(_) => {
                            debug!("   {} not available", name);
                        }
                    }
                }
            }
        }

        let count = connections.count_available();
        info!("✅ Discovered {} primals", count);

        connections
    }

    /// Get a primal client by name
    pub fn get(&self, name: &str) -> Option<&PrimalClient> {
        self.clients.get(name)
    }

    /// Get a primal client by capability (capability-based discovery)
    ///
    /// Uses CapabilityTaxonomy to resolve capability → primal name, then looks up
    /// in the discovered registry. No hardcoded primal names.
    pub fn get_by_capability(&self, capability: &str) -> Option<&PrimalClient> {
        biomeos_types::CapabilityTaxonomy::resolve_to_primal(capability)
            .and_then(|name| self.clients.get(name))
    }

    /// Count available primals
    pub fn count_available(&self) -> usize {
        self.clients.len()
    }

    /// List all discovered primal names
    pub fn available_primals(&self) -> Vec<&str> {
        self.clients
            .keys()
            .map(std::string::String::as_str)
            .collect()
    }

    // ===================================================================
    // Typed accessors — capability-based with name fallback
    // Primary: CapabilityTaxonomy lookup. Fallback: direct name (runtime-discovered primals).
    // ===================================================================

    /// PetalTongue UI framework connection (capability: ui, visualization)
    #[deprecated(
        note = "use get_by_capability(\"ui\") or get_by_capability(\"visualization\") instead"
    )]
    pub fn petaltongue(&self) -> Option<&PetalTongueClient> {
        self.get_by_capability("ui")
            .or_else(|| self.get_by_capability("visualization"))
            .or_else(|| self.get("petaltongue"))
    }
    /// Songbird discovery/networking connection (capability: discovery, network)
    #[deprecated(
        note = "use get_by_capability(\"discovery\") or get_by_capability(\"network\") instead"
    )]
    pub fn songbird(&self) -> Option<&SongbirdClient> {
        self.get_by_capability("discovery")
            .or_else(|| self.get_by_capability("network"))
            .or_else(|| self.get(biomeos_types::primal_names::SONGBIRD))
    }
    /// BearDog security/crypto connection (capability: encryption, security)
    #[deprecated(
        note = "use get_by_capability(\"crypto\") or get_by_capability(\"encryption\") instead"
    )]
    pub fn beardog(&self) -> Option<&BearDogClient> {
        self.get_by_capability("encryption")
            .or_else(|| self.get_by_capability("security"))
            .or_else(|| self.get(biomeos_types::primal_names::BEARDOG))
    }
    /// NestGate storage connection (capability: storage)
    #[deprecated(note = "use get_by_capability(\"storage\") instead")]
    pub fn nestgate(&self) -> Option<&NestGateClient> {
        self.get_by_capability("storage")
            .or_else(|| self.get(biomeos_types::primal_names::NESTGATE))
    }
    /// ToadStool compute/GPU connection (capability: compute)
    #[deprecated(note = "use get_by_capability(\"compute\") instead")]
    pub fn toadstool(&self) -> Option<&ToadStoolClient> {
        self.get_by_capability("compute")
            .or_else(|| self.get(biomeos_types::primal_names::TOADSTOOL))
    }
    /// Squirrel AI connection (capability: ai)
    #[deprecated(note = "use get_by_capability(\"ai\") instead")]
    pub fn squirrel(&self) -> Option<&SquirrelClient> {
        self.get_by_capability("ai")
            .or_else(|| self.get(biomeos_types::primal_names::SQUIRREL))
    }

    /// Add a client for testing (allows discovery/orchestrator tests to inject mock connections)
    #[cfg(test)]
    pub fn add_client(&mut self, name: impl Into<String>, client: PrimalClient) {
        self.clients.insert(name.into(), client);
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_primal_connections_default() {
        let connections = PrimalConnections::default();
        assert_eq!(connections.count_available(), 0);
    }

    #[test]
    fn test_primal_connections_empty() {
        let connections = PrimalConnections::default();
        assert_eq!(connections.count_available(), 0);
        assert!(connections.beardog().is_none());
        assert!(connections.songbird().is_none());
        assert!(connections.available_primals().is_empty());
    }

    #[test]
    fn test_primal_client_with_socket() {
        let client = PrimalClient::with_socket("test-primal", "/tmp/test.sock");
        assert_eq!(client.name(), "test-primal");
    }

    #[test]
    fn test_primal_client_name() {
        let client = PrimalClient::with_socket("beardog", "/tmp/beardog.sock");
        assert_eq!(client.name(), "beardog");
    }

    #[test]
    fn test_primal_client_is_available_nonexistent() {
        let client = PrimalClient::with_socket("test", "/nonexistent/path/test.sock");
        // Socket doesn't exist, so not available
        assert!(!client.is_available());
    }

    #[test]
    fn test_primal_client_with_socket_pathbuf() {
        let path = PathBuf::from("/tmp/songbird.sock");
        let client = PrimalClient::with_socket("songbird", &path);
        assert_eq!(client.name(), "songbird");
    }

    #[test]
    fn test_primal_client_debug() {
        let client = PrimalClient::with_socket("nestgate", "/tmp/nestgate.sock");
        let debug_str = format!("{client:?}");
        assert!(debug_str.contains("nestgate"));
    }

    #[test]
    fn test_primal_client_clone() {
        let client = PrimalClient::with_socket("toadstool", "/tmp/toadstool.sock");
        let cloned = client.clone();
        assert_eq!(client.name(), cloned.name());
    }

    #[test]
    fn test_type_aliases() {
        // Test that type aliases work correctly
        let _petaltongue: PetalTongueClient =
            PrimalClient::with_socket("petaltongue", "/tmp/pt.sock");
        let _songbird: SongbirdClient = PrimalClient::with_socket("songbird", "/tmp/sb.sock");
        let _beardog: BearDogClient = PrimalClient::with_socket("beardog", "/tmp/bd.sock");
        let _nestgate: NestGateClient = PrimalClient::with_socket("nestgate", "/tmp/ng.sock");
        let _toadstool: ToadStoolClient = PrimalClient::with_socket("toadstool", "/tmp/ts.sock");
        let _squirrel: SquirrelClient = PrimalClient::with_socket("squirrel", "/tmp/sq.sock");
    }

    #[test]
    fn test_primal_connections_dynamic_get() {
        let mut connections = PrimalConnections::default();
        connections.clients.insert(
            "beardog".to_string(),
            PrimalClient::with_socket("beardog", "/tmp/bd.sock"),
        );
        connections.clients.insert(
            "songbird".to_string(),
            PrimalClient::with_socket("songbird", "/tmp/sb.sock"),
        );
        connections.clients.insert(
            "nestgate".to_string(),
            PrimalClient::with_socket("nestgate", "/tmp/ng.sock"),
        );

        assert_eq!(connections.count_available(), 3);
        assert!(connections.beardog().is_some());
        assert!(connections.songbird().is_some());
        assert!(connections.nestgate().is_some());
        assert!(connections.toadstool().is_none());
        assert!(connections.squirrel().is_none());
        // Dynamic access
        assert!(connections.get("beardog").is_some());
        assert!(connections.get("unknown").is_none());
    }

    #[test]
    fn test_primal_connections_all_available() {
        let mut connections = PrimalConnections::default();
        for name in &[
            "petaltongue",
            "songbird",
            "beardog",
            "nestgate",
            "toadstool",
            "squirrel",
        ] {
            connections.clients.insert(
                name.to_string(),
                PrimalClient::with_socket(name, format!("/tmp/{name}.sock")),
            );
        }
        assert_eq!(connections.count_available(), 6);
        assert_eq!(connections.available_primals().len(), 6);
    }

    #[test]
    fn test_primal_connections_custom_primal() {
        let mut connections = PrimalConnections::default();
        // Any primal can be added dynamically — not limited to hardcoded 6
        connections.clients.insert(
            "my-custom-primal".to_string(),
            PrimalClient::with_socket("my-custom-primal", "/tmp/custom.sock"),
        );
        assert_eq!(connections.count_available(), 1);
        assert!(connections.get("my-custom-primal").is_some());
    }

    #[tokio::test]
    async fn test_primal_client_discover_nonexistent() {
        // Trying to discover a primal that doesn't exist should fail
        let result = PrimalClient::discover("nonexistent-primal-xyz").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_primal_client_call_nonexistent_socket() {
        let client = PrimalClient::with_socket("test", "/nonexistent/socket.sock");
        let result = client.call("ping", serde_json::json!({})).await;
        // Should fail because socket doesn't exist
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_primal_connections_discover_all() {
        // In test environment, no primals are running, so all should fail
        // but discover_all should complete gracefully
        let connections = PrimalConnections::discover_all("test-family").await;

        // The function should not panic even with no primals available
        let _count = connections.count_available();
    }
}
