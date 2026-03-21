// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Layer 5: Registry and Tracking
//!
//! Local registry of discovered and verified primals.
//! Tracks primal state, health, and availability.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::{
    TrustLevel, VerifiedPrimal, capability::CapabilityVerification, discovery::DiscoveredPrimal,
    identity::IdentityVerification, trust::TrustEvaluation,
};

/// Primal information (full context)
#[derive(Debug, Clone)]
pub struct PrimalInfo {
    /// Discovered primal
    pub discovered: DiscoveredPrimal,
    /// Identity verification
    pub identity: Option<IdentityVerification>,
    /// Capability verification
    pub capability: Option<CapabilityVerification>,
    /// Trust evaluation
    pub trust: Option<TrustEvaluation>,
    /// Registration timestamp
    pub registered_at: chrono::DateTime<chrono::Utc>,
    /// Last seen timestamp
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

/// Registered primal (verified and tracked)
#[derive(Debug, Clone)]
pub struct RegisteredPrimal {
    /// Verified primal
    pub primal: VerifiedPrimal,
    /// Registration timestamp
    pub registered_at: chrono::DateTime<chrono::Utc>,
    /// Last health check
    pub last_health_check: Option<chrono::DateTime<chrono::Utc>>,
    /// Health status
    pub healthy: bool,
}

/// Primal registry
pub struct Registry {
    /// Registered primals (by primal name + node ID)
    primals: Arc<RwLock<HashMap<String, RegisteredPrimal>>>,
}

impl Registry {
    /// Create a new registry
    pub fn new() -> Self {
        info!("Initializing NUCLEUS Registry");
        Self {
            primals: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a verified primal
    pub async fn register(&self, primal: VerifiedPrimal) {
        let key = format!("{}:{}", primal.name, primal.node_id);

        info!(
            primal = %primal.name,
            node = %primal.node_id,
            capabilities = ?primal.capabilities,
            "Registering primal in NUCLEUS registry"
        );

        let registered = RegisteredPrimal {
            primal,
            registered_at: chrono::Utc::now(),
            last_health_check: None,
            healthy: true,
        };

        let mut primals = self.primals.write().await;
        primals.insert(key, registered);
    }

    /// Get a registered primal
    pub async fn get(&self, name: &str, node_id: &str) -> Option<RegisteredPrimal> {
        let key = format!("{name}:{node_id}");
        let primals = self.primals.read().await;
        primals.get(&key).cloned()
    }

    /// Find primals by capability
    pub async fn find_by_capability(&self, capability: &str) -> Vec<RegisteredPrimal> {
        debug!(capability = %capability, "Finding primals by capability in registry");

        let primals = self.primals.read().await;
        primals
            .values()
            .filter(|p| p.primal.capabilities.contains(&capability.to_string()))
            .cloned()
            .collect()
    }

    /// Find primals by family
    pub async fn find_by_family(&self, family_id: &str) -> Vec<RegisteredPrimal> {
        debug!(family = %family_id, "Finding primals by family in registry");

        let primals = self.primals.read().await;
        primals
            .values()
            .filter(|p| p.primal.family_id == family_id)
            .cloned()
            .collect()
    }

    /// Find primals by trust level
    pub async fn find_by_trust_level(&self, min_trust: TrustLevel) -> Vec<RegisteredPrimal> {
        debug!(min_trust = ?min_trust, "Finding primals by trust level");

        let primals = self.primals.read().await;
        primals
            .values()
            .filter(|p| p.primal.trust_level.is_sufficient(&min_trust))
            .cloned()
            .collect()
    }

    /// Update primal health status
    pub async fn update_health(&self, name: &str, node_id: &str, healthy: bool) {
        let key = format!("{name}:{node_id}");
        let mut primals = self.primals.write().await;

        if let Some(registered) = primals.get_mut(&key) {
            registered.healthy = healthy;
            registered.last_health_check = Some(chrono::Utc::now());

            debug!(
                primal = %name,
                node = %node_id,
                healthy = healthy,
                "Updated primal health status"
            );
        }
    }

    /// Remove a primal from the registry
    pub async fn unregister(&self, name: &str, node_id: &str) {
        let key = format!("{name}:{node_id}");
        let mut primals = self.primals.write().await;

        if primals.remove(&key).is_some() {
            info!(
                primal = %name,
                node = %node_id,
                "Unregistered primal from NUCLEUS registry"
            );
        }
    }

    /// Get all registered primals
    pub async fn list_all(&self) -> Vec<RegisteredPrimal> {
        let primals = self.primals.read().await;
        primals.values().cloned().collect()
    }

    /// Get registry statistics
    pub async fn stats(&self) -> RegistryStats {
        let primals = self.primals.read().await;

        let total = primals.len();
        let healthy = primals.values().filter(|p| p.healthy).count();
        let by_trust: HashMap<String, usize> =
            primals.values().fold(HashMap::new(), |mut acc, p| {
                let level = format!("{:?}", p.primal.trust_level);
                *acc.entry(level).or_insert(0) += 1;
                acc
            });

        RegistryStats {
            total_primals: total,
            healthy_primals: healthy,
            unhealthy_primals: total - healthy,
            by_trust_level: by_trust,
        }
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry statistics
#[derive(Debug, Clone)]
pub struct RegistryStats {
    /// Total registered primals
    pub total_primals: usize,
    /// Healthy primals
    pub healthy_primals: usize,
    /// Unhealthy primals
    pub unhealthy_primals: usize,
    /// Primals by trust level
    pub by_trust_level: HashMap<String, usize>,
}

#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Endpoint, EndpointType};

    fn create_test_primal(name: &str, node: &str) -> VerifiedPrimal {
        VerifiedPrimal {
            name: name.to_string(),
            node_id: node.to_string(),
            family_id: "1894e909e454".to_string(),
            capabilities: vec!["test".to_string()],
            endpoint: Endpoint {
                endpoint_type: EndpointType::UnixSocket,
                address: format!("/tmp/{name}.sock"),
            },
            trust_level: TrustLevel::Verified,
            version: "1.0.0".to_string(),
        }
    }

    #[tokio::test]
    async fn test_registry_register_and_get() {
        let registry = Registry::new();
        let primal = create_test_primal("beardog", "node-alpha");

        registry.register(primal.clone()).await;

        let retrieved = registry.get("beardog", "node-alpha").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().primal.name, "beardog");
    }

    #[tokio::test]
    async fn test_registry_find_by_capability() {
        let registry = Registry::new();

        let primal1 = create_test_primal("beardog", "node-alpha");
        let primal2 = create_test_primal("songbird", "node-beta");

        registry.register(primal1).await;
        registry.register(primal2).await;

        let found = registry.find_by_capability("test").await;
        assert_eq!(found.len(), 2);
    }

    #[tokio::test]
    async fn test_registry_health_update() {
        let registry = Registry::new();
        let primal = create_test_primal("beardog", "node-alpha");

        registry.register(primal).await;
        registry.update_health("beardog", "node-alpha", false).await;

        let retrieved = registry.get("beardog", "node-alpha").await.unwrap();
        assert!(!retrieved.healthy);
        assert!(retrieved.last_health_check.is_some());
    }

    #[tokio::test]
    async fn test_registry_stats() {
        let registry = Registry::new();

        let primal1 = create_test_primal("beardog", "node-alpha");
        let primal2 = create_test_primal("songbird", "node-beta");

        registry.register(primal1).await;
        registry.register(primal2).await;

        let stats = registry.stats().await;
        assert_eq!(stats.total_primals, 2);
        assert_eq!(stats.healthy_primals, 2);
    }
}
