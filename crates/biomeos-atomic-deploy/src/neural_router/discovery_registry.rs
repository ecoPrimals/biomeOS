// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Registry-backed capability resolution: exact match, domain prefix scan, category.

use anyhow::{Result, anyhow};
use std::sync::Arc;
use tracing::{debug, info};

use super::NeuralRouter;
use super::types::{DiscoveredAtomic, DiscoveredPrimal};

impl NeuralRouter {
    /// Look up a capability in the registry; returns `None` on miss.
    pub(crate) async fn try_registry_lookup(&self, capability: &str) -> Option<DiscoveredAtomic> {
        let providers = self.get_capability_providers(capability).await?;
        if providers.is_empty() {
            return None;
        }

        let primary = &providers[0];
        info!(
            "   ✅ Found in registry: {} → {}",
            capability, primary.primal_name
        );

        let mut primals = Vec::new();
        for provider in &providers {
            let healthy = Self::check_endpoint_health(&provider.endpoint).await;
            primals.push(DiscoveredPrimal {
                name: provider.primal_name.clone(),
                endpoint: provider.endpoint.clone(),
                capabilities: vec![capability.to_string()],
                healthy,
                last_check: chrono::Utc::now(),
            });
        }

        Some(DiscoveredAtomic {
            capability: Arc::from(capability),
            primals,
            atomic_type: None,
            primary_endpoint: primary.endpoint.clone(),
        })
    }

    /// Domain prefix matching: `"dag"` finds primals registered under `"dag.*"`.
    ///
    /// When `capability.call` receives `{ capability: "dag", operation: "session.create" }`,
    /// the exact key `"dag"` may not be registered — only `"dag.session.create"` etc.
    /// This method scans the registry for any key starting with `"{domain}."` and
    /// returns the first matching provider, deduplicating by primal name.
    pub(crate) async fn try_prefix_lookup(&self, domain: &str) -> Option<DiscoveredAtomic> {
        let prefix = format!("{domain}.");
        let registry = self.capability_registry.read().await;

        let mut seen = std::collections::HashSet::new();
        let mut unique_providers = Vec::new();
        for (key, providers) in registry.iter() {
            if key.starts_with(&prefix) {
                for p in providers {
                    if seen.insert(p.primal_name.clone()) {
                        unique_providers.push(p.clone());
                    }
                }
            }
        }

        if unique_providers.is_empty() {
            return None;
        }

        let primary = &unique_providers[0];
        info!(
            "   ✅ Prefix match: '{}' → {} (via '{prefix}*' scan)",
            domain, primary.primal_name
        );

        drop(registry);

        let mut primals = Vec::new();
        for provider in &unique_providers {
            let healthy = Self::check_endpoint_health(&provider.endpoint).await;
            primals.push(DiscoveredPrimal {
                name: provider.primal_name.clone(),
                endpoint: provider.endpoint.clone(),
                capabilities: vec![domain.to_string()],
                healthy,
                last_check: chrono::Utc::now(),
            });
        }

        Some(DiscoveredAtomic {
            capability: Arc::from(domain),
            primals,
            atomic_type: None,
            primary_endpoint: primary.endpoint.clone(),
        })
    }

    /// Discover primals by capability category
    pub(crate) async fn discover_by_capability_category(
        &self,
        capability: &str,
    ) -> Result<DiscoveredAtomic> {
        let category = match capability {
            "crypto_sign" | "crypto.sign" | "crypto" | "security" | "encryption" => "security",
            "discovery" => "discovery",
            "ai" | "ai.routing" | "ai.text_generation" | "ai.coordination" => "ai",
            "math" | "tensor" | "stats" | "noise" | "activation" | "rng" => "math",
            "shader" | "wgsl" | "spirv" => "shader",
            "compute" | "workload" | "orchestration" => "compute",
            _ => {
                return Err(anyhow!(
                    "Capability '{capability}' does not map to a known category"
                ));
            }
        };

        debug!(
            "   Mapping capability '{}' to category '{}'",
            capability, category
        );

        let registry = self.capability_registry.read().await;

        let mut matching_providers = Vec::new();
        for (registered_cap, providers) in registry.iter() {
            if registered_cap == category || registered_cap.starts_with(&format!("{category}.")) {
                matching_providers.extend(providers.iter().cloned());
            }
        }

        if matching_providers.is_empty() {
            return Err(anyhow!(
                "No primals found providing '{}' capability. Available capabilities: {:?}",
                category,
                registry.keys().collect::<Vec<_>>()
            ));
        }

        let primary = &matching_providers[0];
        info!(
            "   ✅ Found primal via capability category: {} → {} (provides {})",
            capability, primary.primal_name, category
        );

        let mut primals = Vec::new();
        for provider in &matching_providers {
            let healthy = Self::check_endpoint_health(&provider.endpoint).await;
            primals.push(DiscoveredPrimal {
                name: provider.primal_name.clone(),
                endpoint: provider.endpoint.clone(),
                capabilities: vec![category.to_string()],
                healthy,
                last_check: chrono::Utc::now(),
            });
        }

        Ok(DiscoveredAtomic {
            capability: Arc::from(capability),
            primals,
            atomic_type: None,
            primary_endpoint: primary.endpoint.clone(),
        })
    }
}
