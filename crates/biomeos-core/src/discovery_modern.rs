//! Modern trait-based primal discovery system
//!
//! This module provides a composable, trait-based architecture for discovering
//! primals in the ecosystem. It replaces hardcoded discovery with pluggable sources.
//!
//! # Architecture
//!
//! The discovery system is built around the [`PrimalDiscovery`] trait, which
//! defines a common interface for discovering primals. Multiple discovery
//! sources can be composed using [`CompositeDiscovery`].
//!
//! # Examples
//!
//! ```ignore
//! use biomeos_core::{CompositeDiscovery, HttpDiscovery, PrimalDiscovery};
//! use biomeos_types::{Endpoint, PrimalId};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create discovery sources
//! let beardog = HttpDiscovery::new(
//!     Endpoint::new("http://localhost:9000")?,
//!     PrimalId::new("beardog-local")?,
//!     "BearDog".to_string(),
//!     biomeos_core::PrimalType::Security,
//! );
//!
//! let songbird = HttpDiscovery::new(
//!     Endpoint::new("http://localhost:8080")?,
//!     PrimalId::new("songbird-local")?,
//!     "Songbird".to_string(),
//!     biomeos_core::PrimalType::Orchestration,
//! );
//!
//! // Compose into unified discovery
//! let discovery = CompositeDiscovery::new()
//!     .add_source(beardog)
//!     .add_source(songbird);
//!
//! // Discover all primals
//! let primals = discovery.discover_all().await?;
//! println!("Found {} primals", primals.len());
//! # Ok(())
//! # }
//! ```
//!
//! # Design Philosophy
//!
//! - **Composability**: Multiple discovery sources can be combined
//! - **Type Safety**: Strong typing prevents configuration errors
//! - **Testability**: Easy to mock for unit tests
//! - **Extensibility**: New discovery methods via trait implementation

use async_trait::async_trait;
use biomeos_types::{Endpoint, FamilyId, PrimalId};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

/// Result type for discovery operations
pub type DiscoveryResult<T> = Result<T, DiscoveryError>;

/// Discovery errors
#[derive(Debug, Error)]
pub enum DiscoveryError {
    /// Primal was not found at the specified endpoint
    #[error("Primal not found at {endpoint}")]
    NotFound {
        /// Endpoint that was probed
        endpoint: String,
    },

    /// Connection timed out
    #[error("Connection timeout after {timeout:?}")]
    Timeout {
        /// Duration before timeout occurred
        timeout: Duration,
    },

    /// Response was malformed or unexpected
    #[error("Invalid response from primal: {message}")]
    InvalidResponse {
        /// Descriptive message about the invalid response
        message: String,
    },

    /// Authentication / identity verification failed
    #[error("Authentication failed for primal {id}")]
    AuthFailed {
        /// Primal identifier that failed auth
        id: String,
    },

    /// Network-level error during discovery
    #[error("Network error: {0}")]
    Network(String),

    /// URL parsing failed
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    /// Catch-all for other errors
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Health status of a primal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    /// Primal is fully operational
    Healthy,

    /// Primal is operational but degraded
    Degraded,

    /// Primal is not operational
    Unhealthy,

    /// Health status unknown
    Unknown,
}

impl HealthStatus {
    /// Check if the primal is operational (healthy or degraded)
    pub fn is_operational(self) -> bool {
        matches!(self, Self::Healthy | Self::Degraded)
    }

    /// Check if the primal is fully healthy
    pub fn is_healthy(self) -> bool {
        matches!(self, Self::Healthy)
    }
}

/// Type of primal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimalType {
    /// Security and trust management (e.g., BearDog)
    Security,

    /// Orchestration and federation (e.g., Songbird)
    Orchestration,

    /// Storage and persistence (e.g., NestGate)
    Storage,

    /// Compute and processing (e.g., Squirrel)
    Compute,

    /// AI and machine learning (e.g., WolfPack)
    Ai,

    /// Tower coordinator
    Tower,

    /// Visualization and UI (e.g., PetalTongue)
    Visualization,

    /// Custom primal type
    Custom,
}

/// Capability that a primal provides
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Capability(String);

impl Capability {
    /// Create a new capability
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Get the capability name
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<S: Into<String>> From<S> for Capability {
    fn from(s: S) -> Self {
        Self(s.into())
    }
}

/// Discovered primal information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPrimal {
    /// Unique identifier
    pub id: PrimalId,

    /// Human-readable name
    pub name: String,

    /// Type of primal
    pub primal_type: PrimalType,

    /// Semantic version
    pub version: semver::Version,

    /// Current health status
    pub health: HealthStatus,

    /// Capabilities provided
    pub capabilities: Vec<Capability>,

    /// API endpoint
    pub endpoint: Endpoint,

    /// Family ID (if part of genetic lineage)
    pub family_id: Option<FamilyId>,

    /// Optional metadata
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Trait for discovering primals in the ecosystem
#[async_trait]
pub trait PrimalDiscovery: Send + Sync {
    /// Discover a specific primal by endpoint
    async fn discover(&self, endpoint: &Endpoint) -> DiscoveryResult<DiscoveredPrimal>;

    /// Discover all available primals
    async fn discover_all(&self) -> DiscoveryResult<Vec<DiscoveredPrimal>>;

    /// Check if a primal is healthy
    async fn check_health(&self, id: &PrimalId) -> DiscoveryResult<HealthStatus>;

    /// Get capabilities of a primal
    async fn get_capabilities(&self, id: &PrimalId) -> DiscoveryResult<Vec<Capability>> {
        // Default implementation: discover and extract capabilities
        let primals = self.discover_all().await?;
        primals
            .into_iter()
            .find(|p| &p.id == id)
            .map(|p| p.capabilities)
            .ok_or_else(|| DiscoveryError::NotFound {
                endpoint: id.as_str().to_string(),
            })
    }
}

/// Composite discovery that aggregates multiple discovery sources
pub struct CompositeDiscovery {
    sources: Vec<Box<dyn PrimalDiscovery>>,
}

impl CompositeDiscovery {
    /// Create a new composite discovery
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    /// Add a discovery source
    pub fn add_source(mut self, source: impl PrimalDiscovery + 'static) -> Self {
        self.sources.push(Box::new(source));
        self
    }

    /// Add a boxed discovery source
    pub fn add_boxed_source(mut self, source: Box<dyn PrimalDiscovery>) -> Self {
        self.sources.push(source);
        self
    }

    /// Add multiple discovery sources
    pub fn add_sources(
        mut self,
        sources: impl IntoIterator<Item = Box<dyn PrimalDiscovery>>,
    ) -> Self {
        self.sources.extend(sources);
        self
    }
}

impl Default for CompositeDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PrimalDiscovery for CompositeDiscovery {
    async fn discover_all(&self) -> DiscoveryResult<Vec<DiscoveredPrimal>> {
        use std::collections::HashMap;

        let mut all_primals = Vec::new();

        // Collect from all sources
        for source in &self.sources {
            match source.discover_all().await {
                Ok(primals) => all_primals.extend(primals),
                Err(e) => {
                    tracing::warn!("Discovery source failed: {}", e);
                }
            }
        }

        // Deduplicate by ID (last wins)
        let mut unique: HashMap<PrimalId, DiscoveredPrimal> = HashMap::new();
        for primal in all_primals {
            unique.insert(primal.id.clone(), primal);
        }

        Ok(unique.into_values().collect())
    }

    async fn discover(&self, endpoint: &Endpoint) -> DiscoveryResult<DiscoveredPrimal> {
        // Try each source until one succeeds
        for source in &self.sources {
            if let Ok(primal) = source.discover(endpoint).await {
                return Ok(primal);
            }
        }

        Err(DiscoveryError::NotFound {
            endpoint: endpoint.as_str().to_string(),
        })
    }

    async fn check_health(&self, id: &PrimalId) -> DiscoveryResult<HealthStatus> {
        // Try each source until one succeeds
        for source in &self.sources {
            if let Ok(health) = source.check_health(id).await {
                return Ok(health);
            }
        }

        Ok(HealthStatus::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockDiscovery {
        primals: Vec<DiscoveredPrimal>,
    }

    #[async_trait]
    impl PrimalDiscovery for MockDiscovery {
        async fn discover(&self, _endpoint: &Endpoint) -> DiscoveryResult<DiscoveredPrimal> {
            self.primals
                .first()
                .cloned()
                .ok_or_else(|| DiscoveryError::NotFound {
                    endpoint: "mock".to_string(),
                })
        }

        async fn discover_all(&self) -> DiscoveryResult<Vec<DiscoveredPrimal>> {
            Ok(self.primals.clone())
        }

        async fn check_health(&self, _id: &PrimalId) -> DiscoveryResult<HealthStatus> {
            Ok(HealthStatus::Healthy)
        }
    }

    #[tokio::test]
    async fn composite_discovery_aggregates_sources() {
        let mock1 = MockDiscovery {
            primals: vec![DiscoveredPrimal {
                id: PrimalId::new("primal1").unwrap(),
                name: "Primal 1".to_string(),
                primal_type: PrimalType::Security,
                version: semver::Version::new(1, 0, 0),
                health: HealthStatus::Healthy,
                capabilities: vec![],
                endpoint: Endpoint::new("http://localhost:9000").unwrap(),
                family_id: None,
                metadata: serde_json::Value::Null,
            }],
        };

        let mock2 = MockDiscovery {
            primals: vec![DiscoveredPrimal {
                id: PrimalId::new("primal2").unwrap(),
                name: "Primal 2".to_string(),
                primal_type: PrimalType::Orchestration,
                version: semver::Version::new(1, 0, 0),
                health: HealthStatus::Healthy,
                capabilities: vec![],
                endpoint: Endpoint::new("http://localhost:8080").unwrap(),
                family_id: None,
                metadata: serde_json::Value::Null,
            }],
        };

        let composite = CompositeDiscovery::new()
            .add_source(mock1)
            .add_source(mock2);

        let primals = composite.discover_all().await.unwrap();
        assert_eq!(primals.len(), 2);
    }

    #[test]
    fn health_status_checks() {
        assert!(HealthStatus::Healthy.is_operational());
        assert!(HealthStatus::Degraded.is_operational());
        assert!(!HealthStatus::Unhealthy.is_operational());
        assert!(!HealthStatus::Unknown.is_operational());

        assert!(HealthStatus::Healthy.is_healthy());
        assert!(!HealthStatus::Degraded.is_healthy());
    }
}
