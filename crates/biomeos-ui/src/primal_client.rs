// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Capability-oriented JSON-RPC clients for BiomeOS UI
//!
//! EVOLVED (Jan 27, 2026): Extracted from orchestrator.rs for reuse
//!
//! This module provides thin wrappers around [`AtomicClient`] for talking to
//! ecosystem services over Unix sockets. Callers should think in **capabilities**
//! (discovery, security, storage, …), not fixed primal product names.
//!
//! ## Principles
//!
//! - **No C dependencies**: Uses AtomicClient (pure Rust)
//! - **Capability-based**: Resolves services at runtime via [`biomeos_types::CapabilityTaxonomy`]
//! - **No hardcoded paths**: Socket paths via [`biomeos_types::SystemPaths`]
//! - **Graceful degradation**: Missing providers do not panic the UI

use std::sync::Arc;

use anyhow::Result;
use biomeos_core::atomic_client::AtomicClient;
use serde_json::Value;
use tracing::{debug, info};

/// JSON-RPC client for a single discovered service (Unix socket)
///
/// Wraps [`AtomicClient`] with logging and a stable service label for debugging.
#[derive(Debug, Clone)]
pub struct PrimalClient {
    /// Underlying atomic client
    client: AtomicClient,
    /// Service label (often the socket stem; may match a taxonomy name)
    service_name: String,
}

impl PrimalClient {
    /// Discover a service by taxonomy name using capability-based discovery
    ///
    /// Uses [`biomeos_types::SystemPaths`] for XDG-compliant socket discovery.
    pub async fn discover(service_name: &str) -> Result<Self> {
        debug!("Discovering service: {}", service_name);
        let client = AtomicClient::discover(service_name).await?;
        info!("✅ Discovered service: {}", service_name);
        Ok(Self {
            client,
            service_name: service_name.to_string(),
        })
    }

    /// Create a client with an explicit socket path
    pub fn with_socket(service_name: &str, socket_path: impl AsRef<std::path::Path>) -> Self {
        Self {
            client: AtomicClient::new(socket_path),
            service_name: service_name.to_string(),
        }
    }

    /// Call a JSON-RPC method
    pub async fn call(&self, method: &str, params: Value) -> Result<Value> {
        debug!("{}: calling {}", self.service_name, method);
        self.client.call(method, params).await
    }

    /// Service label used for logging and diagnostics
    #[must_use]
    pub fn name(&self) -> &str {
        &self.service_name
    }

    /// Whether the Unix socket is present and connectable
    #[must_use]
    pub fn is_available(&self) -> bool {
        self.client.is_available()
    }
}

// --- Capability-oriented aliases (preferred) ---

/// UI / visualization client (capabilities: `ui`, `visualization`)
pub type UiClient = PrimalClient;
/// Discovery and registry client (capabilities: `discovery`, `network`)
pub type DiscoveryClient = PrimalClient;
/// Security and encryption client (capabilities: `encryption`, `security`, `crypto`)
pub type SecurityClient = PrimalClient;
/// Storage client (capability: `storage`)
pub type StorageClient = PrimalClient;
/// Compute / GPU client (capability: `compute`)
pub type ComputeClient = PrimalClient;
/// AI / inference client (capability: `ai`)
pub type AiClient = PrimalClient;

/// Dynamic registry of discovered JSON-RPC clients (name → client)
#[derive(Debug, Clone, Default)]
pub struct PrimalConnections {
    /// Discovered services (socket stem or taxonomy name → client)
    clients: std::collections::HashMap<Arc<str>, PrimalClient>,
}

impl PrimalConnections {
    /// Discover all registered services for the given family
    pub async fn discover_all(family_id: &str) -> Self {
        Self::discover_all_with_xdg(family_id, None).await
    }

    /// Like [`Self::discover_all`], with an optional XDG runtime parent (e.g. temp dir in tests).
    pub async fn discover_all_with_xdg(
        family_id: &str,
        xdg_runtime_parent: Option<&std::path::Path>,
    ) -> Self {
        let mut connections = Self::default();

        info!("🔍 Discovering services for family: {}", family_id);

        let paths_result = if let Some(p) = xdg_runtime_parent {
            biomeos_types::SystemPaths::new_with_xdg_overrides(Some(p), None::<&std::path::Path>)
        } else {
            biomeos_types::SystemPaths::new()
        };

        if let Ok(paths) = paths_result {
            let runtime_dir = paths.runtime_dir();
            if let Ok(entries) = std::fs::read_dir(runtime_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "sock")
                        && let Some(name) = path.file_stem().and_then(|s| s.to_str())
                    {
                        // Strip family_id suffix if present (e.g., "crypto-family" → "crypto")
                        let base_name = name.split('-').next().unwrap_or(name);
                        let client = PrimalClient::with_socket(base_name, &path);
                        debug!("   Found socket: {} → {}", base_name, path.display());
                        connections.clients.insert(Arc::from(base_name), client);
                    }
                }
            }
        }

        {
            let bootstrap_names = biomeos_types::CapabilityTaxonomy::known_primals();
            for name in bootstrap_names {
                if !connections.clients.contains_key(*name) {
                    match PrimalClient::discover(name).await {
                        Ok(client) => {
                            connections.clients.insert(Arc::from(*name), client);
                        }
                        Err(_) => {
                            debug!("   {} not available", name);
                        }
                    }
                }
            }
        }

        let count = connections.count_available();
        info!("✅ Discovered {} services", count);

        connections
    }

    /// Look up a client by service name
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&PrimalClient> {
        self.clients.get(name)
    }

    /// Resolve a client by capability string via [`biomeos_types::CapabilityTaxonomy`]
    #[must_use]
    pub fn get_by_capability(&self, capability: &str) -> Option<&PrimalClient> {
        biomeos_types::CapabilityTaxonomy::resolve_to_primal(capability)
            .and_then(|name| self.clients.get(name))
    }

    /// Number of discovered service clients in the registry
    #[must_use]
    pub fn count_available(&self) -> usize {
        self.clients.len()
    }

    /// Names of all services currently in the registry
    pub fn available_primals(&self) -> Vec<&str> {
        self.clients.keys().map(Arc::as_ref).collect()
    }

    // -------------------------------------------------------------------
    // Capability-first accessors
    // -------------------------------------------------------------------

    /// UI / visualization provider
    #[must_use]
    pub fn ui_provider(&self) -> Option<&UiClient> {
        self.get_by_capability("ui")
            .or_else(|| self.get_by_capability("visualization"))
            .or_else(|| self.get(biomeos_types::primal_names::PETALTONGUE))
    }

    /// Discovery / registry provider
    #[must_use]
    pub fn discovery_provider(&self) -> Option<&DiscoveryClient> {
        self.get_by_capability("discovery")
            .or_else(|| self.get_by_capability("network"))
            .or_else(|| self.get(biomeos_types::primal_names::SONGBIRD))
    }

    /// Security / encryption provider
    #[must_use]
    pub fn security_provider(&self) -> Option<&SecurityClient> {
        self.get_by_capability("encryption")
            .or_else(|| self.get_by_capability("security"))
            .or_else(|| self.get(biomeos_types::primal_names::BEARDOG))
    }

    /// Storage provider
    #[must_use]
    pub fn storage_provider(&self) -> Option<&StorageClient> {
        self.get_by_capability("storage")
            .or_else(|| self.get(biomeos_types::primal_names::NESTGATE))
    }

    /// Compute / GPU provider
    #[must_use]
    pub fn compute_provider(&self) -> Option<&ComputeClient> {
        self.get_by_capability("compute")
            .or_else(|| self.get(biomeos_types::primal_names::TOADSTOOL))
    }

    /// AI / inference provider
    #[must_use]
    pub fn ai_provider(&self) -> Option<&AiClient> {
        self.get_by_capability("ai")
            .or_else(|| self.get(biomeos_types::primal_names::SQUIRREL))
    }

    /// Add a client for testing (allows discovery/orchestrator tests to inject mock connections)
    #[cfg(test)]
    pub fn add_client(&mut self, name: impl AsRef<str>, client: PrimalClient) {
        self.clients.insert(Arc::from(name.as_ref()), client);
    }
}

#[cfg(test)]
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
        assert!(connections.security_provider().is_none());
        assert!(connections.discovery_provider().is_none());
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
    fn test_capability_type_aliases() {
        let _ui: UiClient = PrimalClient::with_socket("petaltongue", "/tmp/pt.sock");
        let _discovery: DiscoveryClient = PrimalClient::with_socket("songbird", "/tmp/sb.sock");
        let _security: SecurityClient = PrimalClient::with_socket("beardog", "/tmp/bd.sock");
        let _storage: StorageClient = PrimalClient::with_socket("nestgate", "/tmp/ng.sock");
        let _compute: ComputeClient = PrimalClient::with_socket("toadstool", "/tmp/ts.sock");
        let _ai: AiClient = PrimalClient::with_socket("squirrel", "/tmp/sq.sock");
    }

    #[test]
    fn test_primal_connections_dynamic_get() {
        let mut connections = PrimalConnections::default();
        connections.clients.insert(
            Arc::from("beardog"),
            PrimalClient::with_socket("beardog", "/tmp/bd.sock"),
        );
        connections.clients.insert(
            Arc::from("songbird"),
            PrimalClient::with_socket("songbird", "/tmp/sb.sock"),
        );
        connections.clients.insert(
            Arc::from("nestgate"),
            PrimalClient::with_socket("nestgate", "/tmp/ng.sock"),
        );

        assert_eq!(connections.count_available(), 3);
        assert!(connections.security_provider().is_some());
        assert!(connections.discovery_provider().is_some());
        assert!(connections.storage_provider().is_some());
        assert!(connections.compute_provider().is_none());
        assert!(connections.ai_provider().is_none());
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
                Arc::from(*name),
                PrimalClient::with_socket(name, format!("/tmp/{name}.sock")),
            );
        }
        assert_eq!(connections.count_available(), 6);
        assert_eq!(connections.available_primals().len(), 6);
    }

    #[test]
    fn test_primal_connections_custom_primal() {
        let mut connections = PrimalConnections::default();
        connections.clients.insert(
            Arc::from("my-custom-primal"),
            PrimalClient::with_socket("my-custom-primal", "/tmp/custom.sock"),
        );
        assert_eq!(connections.count_available(), 1);
        assert!(connections.get("my-custom-primal").is_some());
    }

    #[tokio::test]
    async fn test_primal_client_discover_nonexistent() {
        let result = PrimalClient::discover("nonexistent-primal-xyz").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_primal_client_call_nonexistent_socket() {
        let client = PrimalClient::with_socket("test", "/nonexistent/socket.sock");
        let result = client.call("ping", serde_json::json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_primal_connections_discover_all() {
        let connections = PrimalConnections::discover_all("test-family").await;
        let _count = connections.count_available();
    }
}
