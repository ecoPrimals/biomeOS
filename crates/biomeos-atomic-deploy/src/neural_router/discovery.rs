// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Capability-based primal discovery

use anyhow::{Result, anyhow};
use std::sync::Arc;
use tracing::{info, warn};

use crate::capability_domains::capability_to_provider_fallback;

use super::NeuralRouter;
use super::types::DiscoveredAtomic;

impl NeuralRouter {
    /// Discover primal(s) by capability
    ///
    /// Resolution order:
    /// 1. Exact key lookup in capability registry
    /// 2. Lazy socket rescan (BM-04) + retry exact lookup
    /// 3. Domain prefix matching — `"dag"` finds `"dag.session.create"` etc.
    /// 4. Composite atomic discovery (Tower, Nest, Node)
    /// 5. Category-based discovery (security, ai, math, ...)
    /// 6. `capability_domains.rs` compiled-in fallback table
    pub async fn discover_capability(&self, capability: &str) -> Result<DiscoveredAtomic> {
        info!("🔍 Discovering capability: {}", capability);

        // 1. Exact key lookup
        if let Some(result) = self.try_registry_lookup(capability).await {
            return Ok(result);
        }

        // 2. BM-04 fix: lazy rescan on first miss
        let new_caps = self.lazy_rescan_sockets().await;
        if new_caps > 0 {
            if let Some(result) = self.try_registry_lookup(capability).await {
                return Ok(result);
            }
        }

        // 3. Domain prefix matching — handles capability.call domain routing
        if let Some(result) = self.try_prefix_lookup(capability).await {
            return Ok(result);
        }

        // 4 + 5. Composite atomics and category discovery
        warn!("   ⚠️  Capability not in registry, trying capability category discovery");
        match capability {
            "secure_http" | "http.request" | "http.post" | "http.get" => {
                return self.discover_tower_atomic().await;
            }
            "secure_storage" => return self.discover_nest_atomic().await,
            "secure_compute" => return self.discover_node_atomic().await,
            "crypto_sign" | "crypto.sign" | "crypto" | "security" | "encryption" | "discovery"
            | "ai" | "ai.routing" | "ai.text_generation" | "ai.coordination" | "math"
            | "tensor" | "stats" | "noise" | "activation" | "rng" | "shader" | "wgsl" | "spirv"
            | "compute" | "workload" | "orchestration" => {
                return self.discover_by_capability_category(capability).await;
            }
            _ => {}
        }

        // 6. capability_domains.rs fallback — uses the compiled-in domain table
        //    to resolve capability → primal name, then finds the primal by socket.
        if let Some(provider) = capability_to_provider_fallback(capability) {
            info!(
                "   ⚠️  Using domain fallback: '{}' → '{}'",
                capability, provider
            );
            let primal = self.find_primal_by_socket(provider).await?;
            let endpoint = primal.endpoint.clone();
            return Ok(DiscoveredAtomic {
                capability: Arc::from(capability),
                primals: vec![primal],
                atomic_type: None,
                primary_endpoint: endpoint,
            });
        }

        Err(anyhow!(
            "Capability '{}' not registered. Available: {:?}",
            capability,
            self.capability_registry
                .read()
                .await
                .keys()
                .collect::<Vec<_>>()
        ))
    }
}

#[cfg(test)]
#[path = "discovery_tests.rs"]
mod tests;
