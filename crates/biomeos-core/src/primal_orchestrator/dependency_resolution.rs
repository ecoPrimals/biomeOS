// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Capability-based dependency ordering (topological sort).

use std::collections::HashMap;

use biomeos_types::error::{BiomeError, BiomeResult};
use biomeos_types::identifiers::PrimalId;

use crate::capabilities::Capability;

use super::orchestrator::PrimalOrchestrator;
use super::state::PrimalRecord;

pub(crate) fn resolve_capability_dependency_order(
    primals: &HashMap<PrimalId, PrimalRecord>,
) -> BiomeResult<Vec<PrimalId>> {
    let mut capability_providers: HashMap<Capability, Vec<PrimalId>> = HashMap::new();
    let mut primal_requirements: HashMap<PrimalId, Vec<Capability>> = HashMap::new();

    for (id, record) in primals {
        for cap in record.primal.provides() {
            capability_providers
                .entry(cap.clone())
                .or_default()
                .push(id.clone());
        }

        primal_requirements.insert(id.clone(), record.primal.requires().to_vec());
    }

    let mut in_degree: HashMap<PrimalId, usize> = HashMap::new();
    let mut graph: HashMap<PrimalId, Vec<PrimalId>> = HashMap::new();

    for (consumer_id, required_caps) in &primal_requirements {
        in_degree.entry(consumer_id.clone()).or_insert(0);

        for required_cap in required_caps {
            if let Some(providers) = capability_providers.get(required_cap) {
                for provider_id in providers {
                    graph
                        .entry(provider_id.clone())
                        .or_default()
                        .push(consumer_id.clone());

                    *in_degree.entry(consumer_id.clone()).or_insert(0) += 1;
                }
            }
        }
    }

    let mut queue: Vec<PrimalId> = in_degree
        .iter()
        .filter(|&(_, &degree)| degree == 0)
        .map(|(id, _)| id.clone())
        .collect();

    let mut result = Vec::new();

    while let Some(id) = queue.pop() {
        result.push(id.clone());

        if let Some(neighbors) = graph.get(&id) {
            for neighbor in neighbors {
                if let Some(degree) = in_degree.get_mut(neighbor) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push(neighbor.clone());
                    }
                }
            }
        }
    }

    if result.len() != primals.len() {
        return Err(BiomeError::config_error(
            "Circular capability dependencies detected",
            Some("capability_deps"),
        ));
    }

    Ok(result)
}

impl PrimalOrchestrator {
    /// Resolve dependency order for startup (capability-based)
    pub async fn resolve_dependencies(&self) -> BiomeResult<Vec<PrimalId>> {
        let primals = self.primals.read().await;
        resolve_capability_dependency_order(&primals)
    }
}
