// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Primal Management Operations
//!
//! Handles primal registration, management, and status tracking.

use super::core::{PrimalInfo, UniversalBiomeOSManager};
use anyhow::Result;

/// Compute primal statistics from a list of primals (testable pure function)
pub(crate) fn compute_primal_statistics(
    primals: impl IntoIterator<Item = PrimalInfo>,
) -> PrimalStatistics {
    let mut total = 0;
    let mut healthy = 0;
    let mut degraded = 0;
    let mut unhealthy = 0;
    let mut unknown = 0;
    let mut by_type = std::collections::HashMap::new();

    for primal in primals {
        total += 1;
        match primal.health {
            biomeos_types::Health::Healthy => healthy += 1,
            biomeos_types::Health::Degraded { .. } => degraded += 1,
            biomeos_types::Health::Unhealthy { .. } => unhealthy += 1,
            _ => unknown += 1,
        }
        *by_type.entry(primal.primal_type.name.clone()).or_insert(0) += 1;
    }

    PrimalStatistics {
        total,
        healthy,
        degraded,
        unhealthy,
        unknown,
        by_type,
    }
}

impl UniversalBiomeOSManager {
    /// Register a primal with the manager
    pub async fn register_primal(&self, primal_info: PrimalInfo) -> Result<()> {
        let mut registry = self.registered_primals.write().await;
        registry.insert(primal_info.id.clone(), primal_info.clone());
        tracing::info!(
            "📝 Registered primal: {} ({})",
            primal_info.name,
            primal_info.id
        );
        Ok(())
    }

    /// Get all registered primals
    pub async fn get_registered_primals(&self) -> Vec<PrimalInfo> {
        let registry = self.registered_primals.read().await;
        registry.values().cloned().collect()
    }

    /// Get a specific primal by ID
    pub async fn get_primal(&self, id: &str) -> Option<PrimalInfo> {
        let registry = self.registered_primals.read().await;
        registry.get(id).cloned()
    }

    /// Update primal information
    pub async fn update_primal(&self, id: &str, primal_info: PrimalInfo) -> Result<()> {
        let mut registry = self.registered_primals.write().await;
        if registry.contains_key(id) {
            registry.insert(id.to_string(), primal_info);
            tracing::info!("🔄 Updated primal: {}", id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Primal not found: {id}"))
        }
    }

    /// Unregister a primal
    pub async fn unregister_primal(&self, id: &str) -> Result<PrimalInfo> {
        let mut registry = self.registered_primals.write().await;
        match registry.remove(id) {
            Some(primal) => {
                tracing::info!("🗑️ Unregistered primal: {} ({})", primal.name, id);
                Ok(primal)
            }
            None => Err(anyhow::anyhow!("Primal not found: {id}")),
        }
    }

    /// Get primals by type
    pub async fn get_primals_by_type(
        &self,
        primal_type: &biomeos_primal_sdk::PrimalType,
    ) -> Vec<PrimalInfo> {
        let registry = self.registered_primals.read().await;
        registry
            .values()
            .filter(|primal| &primal.primal_type == primal_type)
            .cloned()
            .collect()
    }

    /// Get healthy primals only
    pub async fn get_healthy_primals(&self) -> Vec<PrimalInfo> {
        let registry = self.registered_primals.read().await;
        registry
            .values()
            .filter(|primal| matches!(primal.health, biomeos_types::Health::Healthy))
            .cloned()
            .collect()
    }

    /// Get primal count
    pub async fn get_primal_count(&self) -> usize {
        let registry = self.registered_primals.read().await;
        registry.len()
    }

    /// Check if primal is registered
    pub async fn is_primal_registered(&self, id: &str) -> bool {
        let registry = self.registered_primals.read().await;
        registry.contains_key(id)
    }

    /// Update primal health status
    pub async fn update_primal_health(
        &self,
        id: &str,
        health: biomeos_types::Health,
    ) -> Result<()> {
        let mut registry = self.registered_primals.write().await;
        if let Some(primal) = registry.get_mut(id) {
            primal.health = health;
            primal.last_seen = chrono::Utc::now();
            tracing::debug!("🏥 Updated health for primal {}: {:?}", id, primal.health);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Primal not found: {id}"))
        }
    }

    /// Get primals with specific capabilities
    pub async fn get_primals_with_capabilities(
        &self,
        capabilities: &[biomeos_primal_sdk::PrimalCapability],
    ) -> Vec<PrimalInfo> {
        let registry = self.registered_primals.read().await;
        registry
            .values()
            .filter(|primal| {
                // Check if this primal has the required capabilities

                primal.capabilities.iter().any(|primal_cap| {
                    capabilities.iter().any(|required_cap| {
                        primal_cap.category == required_cap.category
                            && primal_cap.name == required_cap.name
                    })
                })
            })
            .cloned()
            .collect()
    }

    /// Get primal statistics
    pub async fn get_primal_statistics(&self) -> PrimalStatistics {
        let registry = self.registered_primals.read().await;
        compute_primal_statistics(registry.values().cloned())
    }

    /// Clear all registered primals (useful for testing)
    pub async fn clear_all_primals(&self) -> Result<()> {
        let mut registry = self.registered_primals.write().await;
        let count = registry.len();
        registry.clear();
        tracing::info!("🧹 Cleared {} registered primals", count);
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use biomeos_types::PrimalType;
    use chrono::Utc;

    fn test_primal(id: &str, name: &str, health: biomeos_types::Health) -> PrimalInfo {
        PrimalInfo {
            id: id.to_string(),
            name: name.to_string(),
            primal_type: PrimalType::from_discovered("test", name, "1.0.0"),
            endpoint: format!("unix:/tmp/{id}"),
            capabilities: vec![],
            health,
            last_seen: Utc::now(),
            discovered_at: Utc::now(),
            metadata: Default::default(),
        }
    }

    #[test]
    fn test_compute_primal_statistics() {
        let primals = vec![
            test_primal("1", "a", biomeos_types::Health::Healthy),
            test_primal("2", "b", biomeos_types::Health::Healthy),
            test_primal(
                "3",
                "c",
                biomeos_types::Health::Degraded {
                    issues: vec![],
                    impact_score: None,
                },
            ),
        ];
        let stats = compute_primal_statistics(primals);
        assert_eq!(stats.total, 3);
        assert_eq!(stats.healthy, 2);
        assert_eq!(stats.degraded, 1);
        assert_eq!(stats.unhealthy, 0);
        assert_eq!(stats.unknown, 0);
    }

    #[test]
    fn test_compute_primal_statistics_empty() {
        let stats = compute_primal_statistics(vec![]);
        assert_eq!(stats.total, 0);
        assert_eq!(stats.healthy, 0);
    }
}

/// Statistics about registered primals
#[derive(Debug, Clone)]
pub struct PrimalStatistics {
    /// Total number of registered primals
    pub total: usize,
    /// Number of healthy primals
    pub healthy: usize,
    /// Number of degraded primals
    pub degraded: usize,
    /// Number of unhealthy primals
    pub unhealthy: usize,
    /// Number of primals in unknown state
    pub unknown: usize,
    /// Count of primals grouped by type
    pub by_type: std::collections::HashMap<String, usize>,
}
