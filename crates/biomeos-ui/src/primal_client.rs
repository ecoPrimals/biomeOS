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
use tracing::{debug, info, warn};

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

/// Type aliases for semantic clarity
///
/// All primals use the same PrimalClient internally, but these aliases
/// make code more readable and self-documenting.
pub type PetalTongueClient = PrimalClient;
pub type SongbirdClient = PrimalClient;
pub type BearDogClient = PrimalClient;
pub type NestGateClient = PrimalClient;
pub type ToadStoolClient = PrimalClient;
pub type SquirrelClient = PrimalClient;

/// Primal connection status
#[derive(Debug, Clone, Default)]
pub struct PrimalConnections {
    pub petaltongue: Option<PetalTongueClient>,
    pub songbird: Option<SongbirdClient>,
    pub beardog: Option<BearDogClient>,
    pub nestgate: Option<NestGateClient>,
    pub toadstool: Option<ToadStoolClient>,
    pub squirrel: Option<SquirrelClient>,
}

impl PrimalConnections {
    /// Discover all primals for the given family
    ///
    /// This attempts to connect to all known primals. Missing primals
    /// are logged but don't cause failure (graceful degradation).
    pub async fn discover_all(family_id: &str) -> Self {
        let mut connections = Self::default();

        // Discover each primal (graceful degradation - missing is OK)
        info!("🔍 Discovering primals for family: {}", family_id);

        // petalTongue - UI framework
        match PrimalClient::discover("petaltongue").await {
            Ok(client) => connections.petaltongue = Some(client),
            Err(e) => warn!("⚠️ petalTongue not available: {}", e),
        }

        // Songbird - Discovery and mesh
        match PrimalClient::discover("songbird").await {
            Ok(client) => connections.songbird = Some(client),
            Err(e) => warn!("⚠️ Songbird not available: {}", e),
        }

        // BearDog - Security and authorization
        match PrimalClient::discover("beardog").await {
            Ok(client) => connections.beardog = Some(client),
            Err(e) => warn!("⚠️ BearDog not available: {}", e),
        }

        // NestGate - Configuration persistence
        match PrimalClient::discover("nestgate").await {
            Ok(client) => connections.nestgate = Some(client),
            Err(e) => warn!("⚠️ NestGate not available: {}", e),
        }

        // ToadStool - Compute metrics
        match PrimalClient::discover("toadstool").await {
            Ok(client) => connections.toadstool = Some(client),
            Err(e) => warn!("⚠️ ToadStool not available: {}", e),
        }

        // Squirrel - AI suggestions
        match PrimalClient::discover("squirrel").await {
            Ok(client) => connections.squirrel = Some(client),
            Err(e) => warn!("⚠️ Squirrel not available: {}", e),
        }

        let count = connections.count_available();
        info!("✅ Discovered {}/6 primals", count);

        connections
    }

    /// Count available primals
    pub fn count_available(&self) -> usize {
        [
            self.petaltongue.is_some(),
            self.songbird.is_some(),
            self.beardog.is_some(),
            self.nestgate.is_some(),
            self.toadstool.is_some(),
            self.squirrel.is_some(),
        ]
        .iter()
        .filter(|&&x| x)
        .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primal_connections_default() {
        let connections = PrimalConnections::default();
        assert_eq!(connections.count_available(), 0);
    }
}
