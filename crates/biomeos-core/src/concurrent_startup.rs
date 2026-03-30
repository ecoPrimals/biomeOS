// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

// Concurrent Wave-Based Primal Startup
//
// Starts primals in parallel waves based on dependency resolution

use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::primal_orchestrator::{ManagedPrimal, PrimalOrchestrator};
use biomeos_types::PrimalId;

/// Dependency graph for primals
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Primal ID -> set of capability names it provides
    pub provides: HashMap<PrimalId, HashSet<String>>,

    /// Primal ID -> set of capability names it requires
    pub requires: HashMap<PrimalId, HashSet<String>>,

    /// Capability name -> primal ID that provides it
    pub capability_providers: HashMap<String, PrimalId>,
}

impl DependencyGraph {
    /// Build dependency graph from primals
    pub fn build(primals: &[Arc<dyn ManagedPrimal>]) -> Result<Self> {
        let mut graph = Self {
            provides: HashMap::new(),
            requires: HashMap::new(),
            capability_providers: HashMap::new(),
        };

        for primal in primals {
            let id = primal.id().clone();
            let provided = primal.provides();
            let required = primal.requires();

            graph.provides.insert(
                id.clone(),
                provided
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect(),
            );
            graph.requires.insert(
                id.clone(),
                required
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect(),
            );

            // Map capabilities to providers
            for cap in provided {
                let cap_str = cap.to_string();
                if let Some(existing) = graph.capability_providers.get(&cap_str) {
                    warn!(
                        "⚠️  Capability '{}' provided by multiple primals: {} and {}",
                        cap_str, existing, id
                    );
                }
                graph.capability_providers.insert(cap_str, id.clone());
            }
        }

        Ok(graph)
    }

    /// Topological sort into waves (primals that can start in parallel)
    pub fn topological_waves(&self) -> Result<Vec<Vec<PrimalId>>> {
        let mut waves = Vec::new();
        let mut started = HashSet::new();
        let mut remaining: HashSet<_> = self.provides.keys().cloned().collect();

        while !remaining.is_empty() {
            // Find primals with all dependencies satisfied
            let mut wave: Vec<PrimalId> = remaining
                .iter()
                .filter(|id| {
                    // Get requirements, or empty set if not present
                    let required = self.requires.get(*id);
                    match required {
                        None => true, // No requirements means ready to start
                        Some(req) => {
                            req.iter().all(|cap| {
                                // Check if capability provider has started
                                self.capability_providers
                                    .get(cap)
                                    .is_some_and(|provider| started.contains(provider))
                            }) || req.is_empty()
                        }
                    }
                })
                .cloned()
                .collect();

            if wave.is_empty() {
                // No progress - circular dependency or missing capability
                let empty_requires: HashSet<String> = HashSet::new();
                let unmet: Vec<_> = remaining
                    .iter()
                    .map(|id| {
                        let required = self.requires.get(id).unwrap_or(&empty_requires);
                        let missing: Vec<_> = required
                            .iter()
                            .filter(|cap| {
                                !self
                                    .capability_providers
                                    .get(*cap)
                                    .is_some_and(|p| started.contains(p))
                            })
                            .cloned()
                            .collect();
                        (id.clone(), missing)
                    })
                    .collect();

                anyhow::bail!("Circular dependency or missing capabilities: {unmet:?}");
            }

            // Sort wave for deterministic ordering (by display string)
            wave.sort_by_key(std::string::ToString::to_string);

            // Mark as started
            for id in &wave {
                started.insert(id.clone());
                remaining.remove(id);
            }

            waves.push(wave);
        }

        Ok(waves)
    }
}

/// Start primals concurrently in dependency-ordered waves
pub async fn start_in_waves(
    orchestrator: &Arc<PrimalOrchestrator>,
    primals: Vec<Arc<dyn ManagedPrimal>>,
) -> Result<()> {
    info!("🌊 Starting primals with concurrent wave-based orchestration");

    // Build dependency graph
    let graph = DependencyGraph::build(&primals)?;

    // Get startup waves
    let waves = graph
        .topological_waves()
        .context("Failed to resolve dependencies")?;

    info!("📋 Resolved {} startup waves", waves.len());
    for (i, wave) in waves.iter().enumerate() {
        debug!("   Wave {}: {} primals", i + 1, wave.len());
    }

    // Start each wave concurrently
    for (wave_num, wave) in waves.iter().enumerate() {
        info!(
            "🌊 Starting wave {} ({} primals in parallel)",
            wave_num + 1,
            wave.len()
        );

        // Collect primals for this wave
        let wave_primals: Vec<_> = wave
            .iter()
            .filter_map(|id| primals.iter().find(|p| p.id() == id).cloned())
            .collect();

        // Start all primals in this wave concurrently
        let mut tasks = Vec::new();
        for primal in wave_primals {
            let orch = Arc::clone(orchestrator);
            let id = primal.id().clone();

            let task = tokio::spawn(async move {
                debug!("🚀 Starting primal: {}", id);
                orch.start_primal(&id).await?;
                debug!("✅ Primal started: {}", id);
                Ok::<_, anyhow::Error>(())
            });

            tasks.push(task);
        }

        // Wait for all tasks in this wave to complete
        let results = futures::future::join_all(tasks).await;

        // Check for errors
        for result in results {
            match result {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    anyhow::bail!("Failed to start primal in wave {}: {}", wave_num + 1, e);
                }
                Err(e) => {
                    anyhow::bail!("Task panicked in wave {}: {}", wave_num + 1, e);
                }
            }
        }

        info!("✅ Wave {} complete", wave_num + 1);
    }

    info!("🎉 All primals started successfully!");
    Ok(())
}
