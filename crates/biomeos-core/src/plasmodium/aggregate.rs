// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Aggregate capabilities, models, and compute across gates into `CollectiveCapabilities`.

use std::collections::{HashMap, HashSet};

use super::types::{CollectiveCapabilities, GateInfo, ModelAvailability};

impl super::Plasmodium {
    /// Aggregate capabilities across all gates
    pub(super) fn aggregate_capabilities(gates: &[GateInfo]) -> CollectiveCapabilities {
        let mut all_gpus = Vec::new();
        let mut total_ram_gb = 0u64;
        let mut model_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut capability_set: HashSet<String> = HashSet::new();

        for gate in gates {
            if !gate.reachable {
                continue;
            }

            // GPUs
            for gpu in &gate.compute.gpus {
                all_gpus.push(gpu.clone());
            }

            // RAM
            total_ram_gb += gate.compute.ram_gb;

            // Models
            for model_id in &gate.models {
                model_map
                    .entry(model_id.clone())
                    .or_default()
                    .push(gate.gate_id.clone());
            }

            // Capabilities from primals (capability-based, not name-hardcoded)
            for primal in &gate.primals {
                if primal.healthy {
                    // Use the standard capability taxonomy to resolve primal capabilities
                    let caps =
                        biomeos_types::capability_taxonomy::capabilities_for_primal(&primal.name);
                    for cap in caps {
                        capability_set.insert(cap);
                    }
                }
            }
        }

        let models: Vec<ModelAvailability> = model_map
            .into_iter()
            .map(|(model_id, gates)| ModelAvailability {
                model_id,
                size_bytes: 0, // Would need model_cache lookup
                format: String::new(),
                gates,
            })
            .collect();

        let mut capabilities: Vec<String> = capability_set.into_iter().collect();
        capabilities.sort();

        CollectiveCapabilities {
            total_gpus: all_gpus.len(),
            gpus: all_gpus,
            total_ram_gb,
            total_models: models.len(),
            models,
            capabilities,
        }
    }
}
