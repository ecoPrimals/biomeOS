// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Capability catalog and provider introspection.

use super::super::capability_heuristics;
use super::CapabilityHandler;
use anyhow::{Context, Result};
use serde_json::{Value, json};
impl CapabilityHandler {
    /// List all known capabilities with provider details and available operations.
    ///
    /// JSON-RPC method: `capabilities.list`
    ///
    /// Returns a rich response including:
    /// - Per-capability provider information (primal, socket, registration time)
    /// - Available operations (from the capability translation registry)
    /// - `cost_estimates` per operation (latency hints for pathway learning)
    /// - `operation_dependencies` DAG edges (prerequisite operations)
    /// - `domains` and `locality` metadata for ecosystem introspection
    /// - Total counts
    ///
    /// Extended with cost/dependency metadata absorbed from downstream primal
    /// `capability.list` schemas (AI, provenance, networking, crypto domains).
    pub async fn list(&self) -> Result<Value> {
        let capabilities = self.router.list_capabilities().await;
        let registry = self.translation_registry.read().await;

        let mut cap_entries: Vec<Value> = Vec::new();
        let mut all_domains: Vec<&String> = Vec::new();

        for (cap_name, providers) in &capabilities {
            let provider_list: Vec<Value> = providers
                .iter()
                .map(|p| {
                    json!({
                        "primal": p.primal_name,
                        "endpoint": p.endpoint.display_string(),
                        "source": p.source,
                        "registered_at": p.registered_at.to_rfc3339()
                    })
                })
                .collect();

            let translations = registry.list_translations(cap_name).unwrap_or_default();

            let operations: Vec<String> = translations
                .iter()
                .map(|(semantic, _actual)| semantic.clone())
                .collect();

            let cost_estimates: Vec<Value> = translations
                .iter()
                .map(|(semantic, _actual)| {
                    json!({
                        "operation": semantic,
                        "estimated_latency_ms": capability_heuristics::estimate_operation_latency(cap_name, semantic),
                        "requires_gpu": capability_heuristics::operation_requires_gpu(cap_name),
                    })
                })
                .collect();

            let operation_dependencies =
                capability_heuristics::build_operation_dependencies(cap_name, &operations);

            all_domains.push(cap_name);

            cap_entries.push(json!({
                "capability": cap_name,
                "providers": provider_list,
                "provider_count": provider_list.len(),
                "operations": operations,
                "operation_count": operations.len(),
                "cost_estimates": cost_estimates,
                "operation_dependencies": operation_dependencies,
                "locality": capability_heuristics::capability_locality(cap_name),
            }));
        }

        Ok(json!({
            "capabilities": all_domains,
            "details": cap_entries,
            "count": all_domains.len(),
            "domains": all_domains,
            "primal": biomeos_types::primal_names::BIOMEOS,
        }))
    }

    /// Get providers for a capability.
    ///
    /// JSON-RPC method: `capability.providers`
    pub async fn providers(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let capability = params["capability"]
            .as_str()
            .context("Missing 'capability' field")?;

        let providers = self
            .router
            .get_capability_providers(capability)
            .await
            .unwrap_or_default();

        Ok(json!({
            "capability": capability,
            "providers": providers.iter().map(|p| {
                json!({
                    "primal": p.primal_name,
                    "endpoint": p.endpoint.display_string(),
                    "source": p.source,
                    "registered_at": p.registered_at.to_rfc3339()
                })
            }).collect::<Vec<_>>(),
            "count": providers.len()
        }))
    }
}
