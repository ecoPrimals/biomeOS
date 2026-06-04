// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Registry-backed capability resolution: exact match, domain prefix scan, category.
//!
//! Provider selection uses **L4 weighted routing**: when multiple providers
//! serve a capability, `select_primary` picks the highest-scoring candidate
//! via [`RoutingWeightTable::select_best`] (EWMA latency, error rate, affinity,
//! circuit breaker). Falls back to first registered provider when the weight
//! table has no data.

use anyhow::{Result, anyhow};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tracing::{debug, info};

use super::NeuralRouter;
use super::perceptron::build_candidate_features;
use super::types::{DiscoveredAtomic, DiscoveredPrimal, RegisteredCapability};

const SHADOW_LOG_DISPATCH_LIMIT: u64 = 1000;

impl NeuralRouter {
    /// Select the primary provider from candidates using routing weights.
    ///
    /// Returns the index into `providers` of the best candidate. When the
    /// weight table has no observations for any candidate, returns 0 (first).
    ///
    /// During the first [`SHADOW_LOG_DISPATCH_LIMIT`] multi-provider dispatches,
    /// logs both the weighted choice and the legacy first-match choice at INFO
    /// level for A/B validation.
    pub(crate) async fn select_primary(&self, capability: &str, providers: &[RegisteredCapability]) -> usize {
        if providers.len() <= 1 {
            return 0;
        }

        let candidates: Vec<Arc<str>> = providers.iter().map(|p| p.primal_name.clone()).collect();
        let weights = self.routing_weights.read().await;

        let weighted_idx = if let Some(best) = weights.select_best(capability, &candidates) {
            candidates.iter().position(|c| c == best).unwrap_or(0)
        } else {
            0
        };

        if weighted_idx != 0 {
            self.weighted_disagreement_counter.fetch_add(1, Ordering::Relaxed);
        }

        let dispatch_n = self.weighted_dispatch_counter.fetch_add(1, Ordering::Relaxed);
        if dispatch_n < SHADOW_LOG_DISPATCH_LIMIT {
            let first_match = &providers[0].primal_name;
            let weighted = &providers[weighted_idx].primal_name;
            let score = weights
                .select_best(capability, &candidates)
                .and_then(|best| {
                    let key = (Arc::from(capability), best.clone());
                    weights.score_for(&key)
                });
            if weighted_idx != 0 {
                info!(
                    "L4 shadow [{}/{}]: {} weighted={} first_match={} score={:.3}",
                    dispatch_n + 1,
                    SHADOW_LOG_DISPATCH_LIMIT,
                    capability,
                    weighted,
                    first_match,
                    score.unwrap_or(0.0)
                );
            } else {
                debug!(
                    "L4 shadow [{}/{}]: {} selected={} (matches first_match)",
                    dispatch_n + 1,
                    SHADOW_LOG_DISPATCH_LIMIT,
                    capability,
                    weighted,
                );
            }

            let n = dispatch_n + 1;
            if n == 100 || n == 500 || n == SHADOW_LOG_DISPATCH_LIMIT {
                let disagreements = self.weighted_disagreement_counter.load(Ordering::Relaxed);
                let rate = if n > 0 {
                    (disagreements as f64 / n as f64) * 100.0
                } else {
                    0.0
                };
                info!(
                    "L4 shadow milestone [{}/{}]: {} disagreements ({:.1}% divergence)",
                    n, SHADOW_LOG_DISPATCH_LIMIT, disagreements, rate
                );
            }
        }

        // L5 perceptron shadow: run perceptron alongside L4, log disagreements.
        // Uses remote ml.mlp_infer via barraCuda when available, falls back to local.
        if let Some(ref perceptron) = self.perceptron {
            let gate_load = self
                .utilization_tracker
                .read()
                .await
                .summary()
                .tracked_methods as f32
                / 100.0;
            let features =
                build_candidate_features(capability, &candidates, &weights, gate_load);
            if perceptron.has_remote_infer() {
                perceptron
                    .shadow_compare_remote(weighted_idx, &features, capability)
                    .await;
            } else {
                perceptron.shadow_compare(weighted_idx, &features, capability);
            }
        }

        weighted_idx
    }

    /// Look up a capability in the registry; returns `None` on miss.
    pub(crate) async fn try_registry_lookup(&self, capability: &str) -> Option<DiscoveredAtomic> {
        let providers = self.get_capability_providers(capability).await?;
        if providers.is_empty() {
            return None;
        }

        let primary_idx = self.select_primary(capability, &providers).await;
        let primary = &providers[primary_idx];
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
    /// returns the best-scoring provider, deduplicating by primal name.
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

        drop(registry);

        let primary_idx = self.select_primary(domain, &unique_providers).await;
        let primary = &unique_providers[primary_idx];
        info!(
            "   ✅ Prefix match: '{}' → {} (via '{prefix}*' scan)",
            domain, primary.primal_name
        );

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

        drop(registry);

        let primary_idx = self.select_primary(capability, &matching_providers).await;
        let primary = &matching_providers[primary_idx];
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
