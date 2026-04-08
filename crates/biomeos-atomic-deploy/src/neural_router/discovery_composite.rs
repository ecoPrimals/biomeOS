// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Composite atomic discovery (Tower, Nest, Node) and capability → primal helpers.

use anyhow::{Context, Result, anyhow};
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::capability_domains::capability_to_provider_fallback;

use super::NeuralRouter;
use super::types::{AtomicType, DiscoveredAtomic, DiscoveredPrimal};

impl NeuralRouter {
    /// Discover Tower Atomic (security + discovery capabilities)
    pub(crate) async fn discover_tower_atomic(&self) -> Result<DiscoveredAtomic> {
        debug!("   Discovering Tower Atomic (security + discovery capabilities)");

        let security_primal = self
            .find_primal_by_capability("security")
            .await
            .context("Tower Atomic requires a primal with 'security' capability")?;

        let discovery_primal = self
            .find_primal_by_capability("discovery")
            .await
            .context("Tower Atomic requires a primal with 'discovery' capability")?;

        if !security_primal.healthy || !discovery_primal.healthy {
            warn!(
                "   ⚠️  Tower Atomic unhealthy: security={}, discovery={}",
                security_primal.healthy, discovery_primal.healthy
            );
        }

        info!(
            "   ✅ Tower Atomic discovered: {} (security) + {} (discovery)",
            security_primal.name, discovery_primal.name
        );

        let primary = discovery_primal.endpoint.clone();
        Ok(DiscoveredAtomic {
            capability: Arc::from("secure_http"),
            primals: vec![security_primal, discovery_primal],
            atomic_type: Some(AtomicType::Tower),
            primary_endpoint: primary,
        })
    }

    /// Discover Nest Atomic (Tower + storage capability)
    pub(crate) async fn discover_nest_atomic(&self) -> Result<DiscoveredAtomic> {
        debug!("   Discovering Nest Atomic (Tower + storage capability)");

        let tower = self.discover_tower_atomic().await?;

        let storage_primal = self
            .find_primal_by_capability("storage")
            .await
            .context("Nest Atomic requires a primal with 'storage' capability")?;

        let primary = storage_primal.endpoint.clone();
        let mut primals = tower.primals;
        primals.push(storage_primal);

        info!(
            "   ✅ Nest Atomic discovered: Tower + {} (storage)",
            primals.last().map_or("?", |p| p.name.as_ref())
        );

        Ok(DiscoveredAtomic {
            capability: Arc::from("secure_storage"),
            primals,
            atomic_type: Some(AtomicType::Nest),
            primary_endpoint: primary,
        })
    }

    /// Discover Node Atomic (Tower + compute capability)
    pub(crate) async fn discover_node_atomic(&self) -> Result<DiscoveredAtomic> {
        debug!("   Discovering Node Atomic (Tower + compute capability)");

        let tower = self.discover_tower_atomic().await?;

        let compute_primal = self
            .find_primal_by_capability("compute")
            .await
            .context("Node Atomic requires a primal with 'compute' capability")?;

        let primary = compute_primal.endpoint.clone();
        let mut primals = tower.primals;
        primals.push(compute_primal);

        info!(
            "   ✅ Node Atomic discovered: Tower + {} (compute)",
            primals.last().map_or("?", |p| p.name.as_ref())
        );

        Ok(DiscoveredAtomic {
            capability: Arc::from("secure_compute"),
            primals,
            atomic_type: Some(AtomicType::Node),
            primary_endpoint: primary,
        })
    }

    /// Find primal by capability
    pub(crate) async fn find_primal_by_capability(
        &self,
        capability: &str,
    ) -> Result<DiscoveredPrimal> {
        let registry = self.capability_registry.read().await;

        if let Some(providers) = registry.get(capability) {
            if let Some(provider) = providers.first() {
                debug!(
                    "   📖 Registry hit: {} provides '{}'",
                    provider.primal_name, capability
                );

                let healthy = self.quick_health_check(&provider.endpoint).await;

                return Ok(DiscoveredPrimal {
                    name: provider.primal_name.clone(),
                    endpoint: provider.endpoint.clone(),
                    capabilities: vec![capability.to_string()],
                    healthy,
                    last_check: chrono::Utc::now(),
                });
            }
        }

        let fallback_primal = capability_to_provider_fallback(capability);

        if let Some(primal) = fallback_primal {
            debug!(
                "   ⚠️  Registry miss: using fallback mapping {} → {}",
                capability, primal
            );
            self.find_primal_by_socket(primal).await
        } else {
            Err(anyhow!(
                "No primal found for capability '{capability}'. Register a provider or check the capability name."
            ))
        }
    }
}
