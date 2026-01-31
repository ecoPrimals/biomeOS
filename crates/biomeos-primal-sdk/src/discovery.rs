//! Runtime Primal Discovery Patterns
//!
//! Provides standard patterns for primals to discover each other at runtime
//! following the deep debt principle: "Primal code only has self-knowledge
//! and discovers other primals at runtime."
//!
//! # Design Principles
//!
//! - **Self-Knowledge Only**: Primal knows its own capabilities, not others
//! - **Runtime Discovery**: Find primals by capability, not by name
//! - **Capability-Based**: "I need security" not "I need BearDog"
//! - **Graceful Degradation**: Handle absence of primals elegantly
//! - **Zero Hardcoding**: No paths, no names, no assumptions
//!
//! # Example
//!
//! ```rust,no_run
//! use biomeos_primal_sdk::discovery::{PrimalDiscovery, DiscoveryQuery};
//! use biomeos_primal_sdk::PrimalCapability;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Discover security provider (e.g., BearDog)
//! let security = PrimalDiscovery::find_by_capability(
//!     PrimalCapability::Security
//! ).await?;
//!
//! println!("Found security provider: {}", security.name);
//! println!("Socket: {}", security.socket_path);
//! # Ok(())
//! # }
//! ```

use anyhow::{anyhow, Context, Result};
use biomeos_types::{PrimalCapability, PrimalType};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::timeout;

/// Discovered primal information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPrimal {
    /// Primal name (e.g., "beardog", "songbird")
    pub name: String,
    /// Primal type
    pub primal_type: PrimalType,
    /// Primary capabilities
    pub capabilities: Vec<PrimalCapability>,
    /// Socket path for IPC
    pub socket_path: PathBuf,
    /// Network endpoint (if available)
    pub network_endpoint: Option<String>,
    /// Family ID
    pub family_id: String,
    /// Node ID
    pub node_id: String,
}

/// Discovery query configuration
#[derive(Debug, Clone)]
pub struct DiscoveryQuery {
    /// Required capability
    pub capability: PrimalCapability,
    /// Optional timeout (default: 5 seconds)
    pub timeout: Duration,
    /// Require specific family (default: any)
    pub family_id: Option<String>,
    /// Prefer local over remote
    pub prefer_local: bool,
}

impl Default for DiscoveryQuery {
    fn default() -> Self {
        Self {
            capability: PrimalCapability::new("discovery", "discovery", "1.0"),
            timeout: Duration::from_secs(5),
            family_id: None,
            prefer_local: true,
        }
    }
}

impl DiscoveryQuery {
    /// Create query for specific capability
    pub fn capability(capability: PrimalCapability) -> Self {
        Self {
            capability,
            ..Default::default()
        }
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Require specific family
    pub fn in_family(mut self, family_id: impl Into<String>) -> Self {
        self.family_id = Some(family_id.into());
        self
    }

    /// Allow remote primals
    pub fn allow_remote(mut self) -> Self {
        self.prefer_local = false;
        self
    }
}

/// Primal discovery engine
pub struct PrimalDiscovery;

impl PrimalDiscovery {
    /// Discover primal by capability (simple API)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use biomeos_primal_sdk::discovery::PrimalDiscovery;
    /// # use biomeos_primal_sdk::PrimalCapability;
    /// # async fn example() -> anyhow::Result<()> {
    /// // Find security provider
    /// let security = PrimalDiscovery::find_by_capability(
    ///     PrimalCapability::Security
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_by_capability(
        capability: PrimalCapability,
    ) -> Result<DiscoveredPrimal> {
        Self::discover(DiscoveryQuery::capability(capability)).await
    }

    /// Discover primal with advanced query
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use biomeos_primal_sdk::discovery::{PrimalDiscovery, DiscoveryQuery};
    /// # use biomeos_primal_sdk::PrimalCapability;
    /// # use std::time::Duration;
    /// # async fn example() -> anyhow::Result<()> {
    /// let query = DiscoveryQuery::capability(PrimalCapability::Discovery)
    ///     .with_timeout(Duration::from_secs(10))
    ///     .in_family("prod-cluster-1");
    ///
    /// let discovery = PrimalDiscovery::discover(query).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn discover(query: DiscoveryQuery) -> Result<DiscoveredPrimal> {
        // First, try to find Songbird (discovery service) via environment
        let songbird_result = timeout(
            query.timeout,
            Self::discover_songbird(),
        ).await;

        match songbird_result {
            Ok(Ok(songbird)) => {
                // Use Songbird to find the requested primal
                Self::query_songbird(&songbird, query).await
            }
            _ => {
                // Fallback: Direct environment-based discovery
                Self::discover_via_environment(query).await
            }
        }
    }

    /// Discover Songbird (discovery service) via environment
    async fn discover_songbird() -> Result<DiscoveredPrimal> {
        // Check environment variables for Songbird location
        let socket_path = std::env::var("SONGBIRD_SOCKET")
            .or_else(|_| std::env::var("DISCOVERY_SOCKET"))
            .ok()
            .map(PathBuf::from);

        let family_id = std::env::var("FAMILY_ID")
            .unwrap_or_else(|_| "default".to_string());
        
        let node_id = std::env::var("NODE_ID")
            .unwrap_or_else(|_| "node0".to_string());

        if let Some(socket) = socket_path {
            return Ok(DiscoveredPrimal {
                name: "songbird".to_string(),
                primal_type: PrimalType::new("discovery", "songbird", "1.0.0"),
                capabilities: vec![PrimalCapability::new("discovery", "mdns", "1.0")],
                socket_path: socket,
                network_endpoint: None,
                family_id,
                node_id,
            });
        }

        // Try XDG runtime directory
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            let socket = PathBuf::from(runtime_dir)
                .join("songbird")
                .join("songbird.sock");
            
            if socket.exists() {
                return Ok(DiscoveredPrimal {
                    name: "songbird".to_string(),
                    primal_type: PrimalType::new("discovery", "songbird", "1.0.0"),
                    capabilities: vec![PrimalCapability::new("discovery", "mdns", "1.0")],
                    socket_path: socket,
                    network_endpoint: None,
                    family_id,
                    node_id,
                });
            }
        }

        Err(anyhow!("Songbird not found via environment discovery"))
    }

    /// Query Songbird for a specific capability
    async fn query_songbird(
        songbird: &DiscoveredPrimal,
        query: DiscoveryQuery,
    ) -> Result<DiscoveredPrimal> {
        // TODO: Implement JSON-RPC query to Songbird
        // For now, return error to trigger fallback
        Err(anyhow!("Songbird query not yet implemented - using fallback"))
    }

    /// Discover primal via environment variables (fallback)
    async fn discover_via_environment(query: DiscoveryQuery) -> Result<DiscoveredPrimal> {
        let capability_name = query.capability.category.clone();
        
        // Try environment variable based on category: SECURITY_SOCKET, DISCOVERY_SOCKET, etc.
        let socket_env = format!("{}_SOCKET", capability_name.to_uppercase());
        
        if let Ok(socket_path) = std::env::var(&socket_env) {
            let family_id = std::env::var("FAMILY_ID")
                .unwrap_or_else(|_| "default".to_string());
            
            let node_id = std::env::var("NODE_ID")
                .unwrap_or_else(|_| "node0".to_string());

            return Ok(DiscoveredPrimal {
                name: capability_name.to_lowercase(),
                primal_type: Self::capability_to_type(query.capability.clone()),
                capabilities: vec![query.capability],
                socket_path: PathBuf::from(socket_path),
                network_endpoint: None,
                family_id,
                node_id,
            });
        }

        Err(anyhow!(
            "Could not discover primal with capability {:?} via environment",
            query.capability
        ))
    }

    /// Map capability to primal type (heuristic for common primals)
    /// 
    /// Since PrimalCapability is a struct with category/name fields,
    /// we match on the category string to determine the likely primal type.
    fn capability_to_type(capability: PrimalCapability) -> PrimalType {
        // Match based on capability category
        match capability.category.as_str() {
            "security" | "encryption" | "identity" | "trust" => {
                PrimalType::new("security", "beardog", "1.0.0")
            }
            "discovery" | "p2p" | "federation" => {
                PrimalType::new("discovery", "songbird", "1.0.0")
            }
            "compute" | "workload" | "execution" | "gpu" => {
                PrimalType::new("compute", "toadstool", "1.0.0")
            }
            "storage" | "data" | "persistence" => {
                PrimalType::new("storage", "nestgate", "1.0.0")
            }
            _ => PrimalType::new("generic", "unknown", "1.0.0"),
        }
    }

    /// Discover all primals with a specific capability
    pub async fn find_all_by_capability(
        capability: PrimalCapability,
    ) -> Result<Vec<DiscoveredPrimal>> {
        // Try Songbird first
        if let Ok(songbird) = Self::discover_songbird().await {
            // TODO: Query Songbird for all primals with capability
            // For now, return single result from simple discovery
            let primal = Self::find_by_capability(capability).await?;
            return Ok(vec![primal]);
        }

        // Fallback: Return single result
        let primal = Self::find_by_capability(capability).await?;
        Ok(vec![primal])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_query_builder() {
        let query = DiscoveryQuery::capability(PrimalCapability::new("security", "encryption", "1.0"))
            .with_timeout(Duration::from_secs(10))
            .in_family("prod")
            .allow_remote();

        assert_eq!(query.capability.category, "security");
        assert_eq!(query.timeout, Duration::from_secs(10));
        assert_eq!(query.family_id, Some("prod".to_string()));
        assert!(!query.prefer_local);
    }

    #[test]
    fn test_capability_to_type() {
        let security_cap = PrimalCapability::new("security", "encryption", "1.0");
        let security_type = PrimalDiscovery::capability_to_type(security_cap);
        assert_eq!(security_type.category, "security");
        assert_eq!(security_type.name, "beardog");
        
        let discovery_cap = PrimalCapability::new("discovery", "mdns", "1.0");
        let discovery_type = PrimalDiscovery::capability_to_type(discovery_cap);
        assert_eq!(discovery_type.category, "discovery");
        assert_eq!(discovery_type.name, "songbird");
    }
}
