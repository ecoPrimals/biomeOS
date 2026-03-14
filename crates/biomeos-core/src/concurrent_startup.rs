// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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
        let mut graph = DependencyGraph {
            provides: HashMap::new(),
            requires: HashMap::new(),
            capability_providers: HashMap::new(),
        };

        for primal in primals {
            let id = primal.id().clone();
            let provided = primal.provides();
            let required = primal.requires();

            graph
                .provides
                .insert(id.clone(), provided.iter().map(|c| c.to_string()).collect());
            graph
                .requires
                .insert(id.clone(), required.iter().map(|c| c.to_string()).collect());

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
                                    .map(|provider| started.contains(provider))
                                    .unwrap_or(false)
                            }) || req.is_empty()
                        }
                    }
                })
                .cloned()
                .collect();

            if wave.is_empty() {
                // No progress - circular dependency or missing capability
                let unmet: Vec<_> = remaining
                    .iter()
                    .map(|id| {
                        let required = self
                            .requires
                            .get(id)
                            .expect("remaining IDs must be in requires map");
                        let missing: Vec<_> = required
                            .iter()
                            .filter(|cap| {
                                !self
                                    .capability_providers
                                    .get(*cap)
                                    .map(|p| started.contains(p))
                                    .unwrap_or(false)
                            })
                            .cloned()
                            .collect();
                        (id.clone(), missing)
                    })
                    .collect();

                anyhow::bail!("Circular dependency or missing capabilities: {unmet:?}");
            }

            // Sort wave for deterministic ordering (by display string)
            wave.sort_by_key(|a| a.to_string());

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
        for result in results.into_iter() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::Capability;
    use crate::discovery_modern::HealthStatus;
    use async_trait::async_trait;
    use biomeos_types::error::BiomeResult;
    use biomeos_types::identifiers::Endpoint;

    // ── Mock primal for DependencyGraph::build tests ──────────────────
    struct MockPrimal {
        id: PrimalId,
        provides: Vec<Capability>,
        requires: Vec<Capability>,
    }

    impl MockPrimal {
        fn new(name: &str, provides: Vec<Capability>, requires: Vec<Capability>) -> Self {
            Self {
                id: PrimalId::new(name).expect("valid primal name"),
                provides,
                requires,
            }
        }
    }

    #[async_trait]
    impl ManagedPrimal for MockPrimal {
        fn id(&self) -> &PrimalId {
            &self.id
        }
        fn provides(&self) -> &[Capability] {
            &self.provides
        }
        fn requires(&self) -> &[Capability] {
            &self.requires
        }
        async fn endpoint(&self) -> Option<Endpoint> {
            None
        }
        async fn start(&self) -> BiomeResult<()> {
            Ok(())
        }
        async fn stop(&self) -> BiomeResult<()> {
            Ok(())
        }
        async fn health_check(&self) -> BiomeResult<HealthStatus> {
            Ok(HealthStatus::Healthy)
        }
    }

    // ── Helper ────────────────────────────────────────────────────────
    fn pid(name: &str) -> PrimalId {
        PrimalId::new(name).expect("valid primal id")
    }

    // ── topological_waves: empty graph ────────────────────────────────
    #[test]
    fn test_empty_graph() {
        let graph = DependencyGraph {
            provides: HashMap::new(),
            requires: HashMap::new(),
            capability_providers: HashMap::new(),
        };

        let waves = graph
            .topological_waves()
            .expect("empty graph should succeed");
        assert_eq!(waves.len(), 0);
    }

    // ── topological_waves: all independent → single wave ─────────────
    #[test]
    fn test_single_wave_independent_primals() {
        let mut provides = HashMap::new();
        provides.insert(pid("alpha"), HashSet::from(["cap-a".into()]));
        provides.insert(pid("beta"), HashSet::from(["cap-b".into()]));
        provides.insert(pid("gamma"), HashSet::from(["cap-c".into()]));

        let mut requires = HashMap::new();
        requires.insert(pid("alpha"), HashSet::new());
        requires.insert(pid("beta"), HashSet::new());
        requires.insert(pid("gamma"), HashSet::new());

        let graph = DependencyGraph {
            provides,
            requires,
            capability_providers: HashMap::new(),
        };

        let waves = graph.topological_waves().expect("should resolve");
        assert_eq!(waves.len(), 1, "all independent → 1 wave");
        assert_eq!(waves[0].len(), 3);
    }

    // ── topological_waves: linear chain A→B→C ────────────────────────
    #[test]
    fn test_linear_dependency_chain() {
        // A provides "security", B requires "security" and provides "discovery",
        // C requires "discovery"
        let a = pid("beardog");
        let b = pid("songbird");
        let c = pid("toadstool");

        let mut provides = HashMap::new();
        provides.insert(a.clone(), HashSet::from(["security".into()]));
        provides.insert(b.clone(), HashSet::from(["discovery".into()]));
        provides.insert(c.clone(), HashSet::new());

        let mut requires = HashMap::new();
        requires.insert(a.clone(), HashSet::new());
        requires.insert(b.clone(), HashSet::from(["security".into()]));
        requires.insert(c.clone(), HashSet::from(["discovery".into()]));

        let mut cap_providers = HashMap::new();
        cap_providers.insert("security".into(), a.clone());
        cap_providers.insert("discovery".into(), b.clone());

        let graph = DependencyGraph {
            provides,
            requires,
            capability_providers: cap_providers,
        };

        let waves = graph.topological_waves().expect("should resolve");
        assert_eq!(waves.len(), 3, "linear chain produces 3 waves");
        assert_eq!(waves[0], vec![a]);
        assert_eq!(waves[1], vec![b]);
        assert_eq!(waves[2], vec![c]);
    }

    // ── topological_waves: diamond dependency ─────────────────────────
    #[test]
    fn test_diamond_dependency() {
        // A provides "security"
        // B requires "security", provides "discovery"
        // C requires "security", provides "storage"
        // D requires "discovery" + "storage"
        let a = pid("a-primal");
        let b = pid("b-primal");
        let c = pid("c-primal");
        let d = pid("d-primal");

        let mut provides = HashMap::new();
        provides.insert(a.clone(), HashSet::from(["security".into()]));
        provides.insert(b.clone(), HashSet::from(["discovery".into()]));
        provides.insert(c.clone(), HashSet::from(["storage".into()]));
        provides.insert(d.clone(), HashSet::new());

        let mut requires = HashMap::new();
        requires.insert(a.clone(), HashSet::new());
        requires.insert(b.clone(), HashSet::from(["security".into()]));
        requires.insert(c.clone(), HashSet::from(["security".into()]));
        requires.insert(
            d.clone(),
            HashSet::from(["discovery".into(), "storage".into()]),
        );

        let mut cap_providers = HashMap::new();
        cap_providers.insert("security".into(), a.clone());
        cap_providers.insert("discovery".into(), b.clone());
        cap_providers.insert("storage".into(), c.clone());

        let graph = DependencyGraph {
            provides,
            requires,
            capability_providers: cap_providers,
        };

        let waves = graph.topological_waves().expect("should resolve");
        assert_eq!(waves.len(), 3, "diamond produces 3 waves");
        // Wave 0: A (no deps)
        assert_eq!(waves[0], vec![a]);
        // Wave 1: B and C (both depend only on A), sorted alphabetically
        assert_eq!(waves[1].len(), 2);
        assert_eq!(waves[1][0], b);
        assert_eq!(waves[1][1], c);
        // Wave 2: D (depends on B and C)
        assert_eq!(waves[2], vec![d]);
    }

    // ── topological_waves: circular dependency → error ────────────────
    #[test]
    fn test_circular_dependency_error() {
        let a = pid("alpha");
        let b = pid("beta");

        let mut provides = HashMap::new();
        provides.insert(a.clone(), HashSet::from(["cap-a".into()]));
        provides.insert(b.clone(), HashSet::from(["cap-b".into()]));

        let mut requires = HashMap::new();
        requires.insert(a.clone(), HashSet::from(["cap-b".into()]));
        requires.insert(b.clone(), HashSet::from(["cap-a".into()]));

        let mut cap_providers = HashMap::new();
        cap_providers.insert("cap-a".into(), a.clone());
        cap_providers.insert("cap-b".into(), b.clone());

        let graph = DependencyGraph {
            provides,
            requires,
            capability_providers: cap_providers,
        };

        let result = graph.topological_waves();
        assert!(result.is_err(), "circular deps must fail");
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("Circular dependency") || err_msg.contains("missing capabilities"),
            "error should mention circular dependency, got: {err_msg}"
        );
    }

    // ── topological_waves: missing capability provider → error ────────
    #[test]
    fn test_missing_capability_provider_error() {
        let a = pid("lonely");

        let mut provides = HashMap::new();
        provides.insert(a.clone(), HashSet::new());

        let mut requires = HashMap::new();
        // Requires a capability nobody provides
        requires.insert(a.clone(), HashSet::from(["nonexistent-cap".into()]));

        let graph = DependencyGraph {
            provides,
            requires,
            capability_providers: HashMap::new(),
        };

        let result = graph.topological_waves();
        assert!(result.is_err(), "missing provider must fail");
    }

    // ── topological_waves: deterministic wave ordering ─────────────────
    #[test]
    fn test_wave_ordering_is_deterministic() {
        // Three independent primals should appear sorted within their wave
        let mut provides = HashMap::new();
        provides.insert(pid("zebra"), HashSet::new());
        provides.insert(pid("apple"), HashSet::new());
        provides.insert(pid("mango"), HashSet::new());

        let mut requires = HashMap::new();
        requires.insert(pid("zebra"), HashSet::new());
        requires.insert(pid("apple"), HashSet::new());
        requires.insert(pid("mango"), HashSet::new());

        let graph = DependencyGraph {
            provides,
            requires,
            capability_providers: HashMap::new(),
        };

        let waves = graph.topological_waves().expect("should resolve");
        assert_eq!(waves.len(), 1);
        assert_eq!(waves[0][0], pid("apple"));
        assert_eq!(waves[0][1], pid("mango"));
        assert_eq!(waves[0][2], pid("zebra"));
    }

    // ── topological_waves: primal with empty requirement set ──────────
    #[test]
    fn test_primal_with_empty_requirement_set() {
        let a = pid("core");

        let mut provides = HashMap::new();
        provides.insert(a.clone(), HashSet::from(["security".into()]));

        let mut requires = HashMap::new();
        requires.insert(a.clone(), HashSet::new()); // Explicit empty set

        let graph = DependencyGraph {
            provides,
            requires,
            capability_providers: HashMap::new(),
        };

        let waves = graph.topological_waves().expect("should resolve");
        assert_eq!(waves.len(), 1);
        assert_eq!(waves[0], vec![a]);
    }

    // ── topological_waves: primal with no entry in requires map ───────
    #[test]
    fn test_primal_not_in_requires_map() {
        // If a primal is in `provides` but not in `requires`, it has no
        // requirements and should be in the first wave.
        let a = pid("orphan");

        let mut provides = HashMap::new();
        provides.insert(a.clone(), HashSet::from(["cap-x".into()]));

        let graph = DependencyGraph {
            provides,
            requires: HashMap::new(), // deliberately absent
            capability_providers: HashMap::new(),
        };

        let waves = graph.topological_waves().expect("should resolve");
        assert_eq!(waves.len(), 1);
        assert_eq!(waves[0], vec![a]);
    }

    // ── topological_waves: wide fan-out (many depend on one) ──────────
    #[test]
    fn test_wide_fanout_dependency() {
        let root = pid("root-primal");

        let mut provides = HashMap::new();
        provides.insert(root.clone(), HashSet::from(["security".into()]));

        let mut requires = HashMap::new();
        requires.insert(root.clone(), HashSet::new());

        let mut cap_providers = HashMap::new();
        cap_providers.insert("security".into(), root.clone());

        // 5 primals all depending on "security"
        for i in 0..5 {
            let leaf = pid(&format!("leaf-{i}"));
            provides.insert(leaf.clone(), HashSet::new());
            requires.insert(leaf.clone(), HashSet::from(["security".into()]));
        }

        let graph = DependencyGraph {
            provides,
            requires,
            capability_providers: cap_providers,
        };

        let waves = graph.topological_waves().expect("should resolve");
        assert_eq!(waves.len(), 2, "root + fan-out = 2 waves");
        assert_eq!(waves[0].len(), 1);
        assert_eq!(waves[0][0], root);
        assert_eq!(waves[1].len(), 5, "all leaves in wave 2");
    }

    // ── DependencyGraph::build: basic construction ────────────────────
    #[test]
    fn test_build_empty() {
        let primals: Vec<Arc<dyn ManagedPrimal>> = vec![];
        let graph = DependencyGraph::build(&primals).expect("empty build should succeed");
        assert!(graph.provides.is_empty());
        assert!(graph.requires.is_empty());
        assert!(graph.capability_providers.is_empty());
    }

    #[test]
    fn test_build_single_primal_no_deps() {
        let primals: Vec<Arc<dyn ManagedPrimal>> = vec![Arc::new(MockPrimal::new(
            "solo",
            vec![Capability::Security],
            vec![],
        ))];
        let graph = DependencyGraph::build(&primals).expect("should build");
        assert_eq!(graph.provides.len(), 1);
        assert!(graph.provides[&pid("solo")].contains("security"));
        assert_eq!(graph.requires.len(), 1);
        assert!(graph.requires[&pid("solo")].is_empty());
        assert_eq!(graph.capability_providers.len(), 1);
        assert_eq!(graph.capability_providers["security"], pid("solo"));
    }

    #[test]
    fn test_build_with_dependencies() {
        let primals: Vec<Arc<dyn ManagedPrimal>> = vec![
            Arc::new(MockPrimal::new(
                "beardog",
                vec![Capability::Security],
                vec![],
            )),
            Arc::new(MockPrimal::new(
                "songbird",
                vec![Capability::Discovery],
                vec![Capability::Security],
            )),
        ];

        let graph = DependencyGraph::build(&primals).expect("should build");
        assert_eq!(graph.provides.len(), 2);
        assert_eq!(graph.requires.len(), 2);
        assert!(graph.requires[&pid("songbird")].contains("security"));
        assert_eq!(graph.capability_providers["security"], pid("beardog"));
        assert_eq!(graph.capability_providers["discovery"], pid("songbird"));
    }

    #[test]
    fn test_build_multiple_capabilities() {
        let primals: Vec<Arc<dyn ManagedPrimal>> = vec![Arc::new(MockPrimal::new(
            "multi-cap",
            vec![Capability::Security, Capability::Storage, Capability::AI],
            vec![Capability::Discovery],
        ))];

        let graph = DependencyGraph::build(&primals).expect("should build");
        let provided = &graph.provides[&pid("multi-cap")];
        assert_eq!(provided.len(), 3);
        assert!(provided.contains("security"));
        assert!(provided.contains("storage"));
        assert!(provided.contains("ai"));

        let required = &graph.requires[&pid("multi-cap")];
        assert_eq!(required.len(), 1);
        assert!(required.contains("discovery"));
    }

    #[test]
    fn test_build_custom_capability() {
        let primals: Vec<Arc<dyn ManagedPrimal>> = vec![Arc::new(MockPrimal::new(
            "custom-svc",
            vec![Capability::Custom("my-extension".into())],
            vec![],
        ))];

        let graph = DependencyGraph::build(&primals).expect("should build");
        assert!(graph
            .capability_providers
            .contains_key("custom:my-extension"));
    }

    // ── build + topological_waves integration ─────────────────────────
    #[test]
    fn test_build_then_waves_linear() {
        let primals: Vec<Arc<dyn ManagedPrimal>> = vec![
            Arc::new(MockPrimal::new(
                "beardog",
                vec![Capability::Security],
                vec![],
            )),
            Arc::new(MockPrimal::new(
                "songbird",
                vec![Capability::Discovery],
                vec![Capability::Security],
            )),
            Arc::new(MockPrimal::new(
                "nestgate",
                vec![Capability::Storage],
                vec![Capability::Discovery],
            )),
        ];

        let graph = DependencyGraph::build(&primals).expect("should build");
        let waves = graph.topological_waves().expect("should resolve");

        assert_eq!(waves.len(), 3);
        assert_eq!(waves[0], vec![pid("beardog")]);
        assert_eq!(waves[1], vec![pid("songbird")]);
        assert_eq!(waves[2], vec![pid("nestgate")]);
    }

    #[test]
    fn test_build_then_waves_parallel() {
        // beardog provides security, songbird provides discovery
        // toadstool and nestgate both require only security → same wave
        let primals: Vec<Arc<dyn ManagedPrimal>> = vec![
            Arc::new(MockPrimal::new(
                "beardog",
                vec![Capability::Security],
                vec![],
            )),
            Arc::new(MockPrimal::new(
                "nestgate",
                vec![Capability::Storage],
                vec![Capability::Security],
            )),
            Arc::new(MockPrimal::new(
                "toadstool",
                vec![Capability::Compute],
                vec![Capability::Security],
            )),
        ];

        let graph = DependencyGraph::build(&primals).expect("should build");
        let waves = graph.topological_waves().expect("should resolve");

        assert_eq!(waves.len(), 2);
        assert_eq!(waves[0], vec![pid("beardog")]);
        assert_eq!(waves[1].len(), 2);
        // Sorted alphabetically
        assert_eq!(waves[1][0], pid("nestgate"));
        assert_eq!(waves[1][1], pid("toadstool"));
    }

    #[test]
    fn test_build_then_waves_circular() {
        let primals: Vec<Arc<dyn ManagedPrimal>> = vec![
            Arc::new(MockPrimal::new(
                "alpha",
                vec![Capability::Security],
                vec![Capability::Discovery],
            )),
            Arc::new(MockPrimal::new(
                "beta",
                vec![Capability::Discovery],
                vec![Capability::Security],
            )),
        ];

        let graph = DependencyGraph::build(&primals).expect("should build");
        let result = graph.topological_waves();
        assert!(result.is_err(), "circular deps via build must fail");
    }

    // ── DependencyGraph clone ─────────────────────────────────────────
    #[test]
    fn test_dependency_graph_clone() {
        let mut provides = HashMap::new();
        provides.insert(pid("a"), HashSet::from(["cap".into()]));

        let graph = DependencyGraph {
            provides,
            requires: HashMap::new(),
            capability_providers: HashMap::from([("cap".into(), pid("a"))]),
        };

        let cloned = graph.clone();
        assert_eq!(cloned.provides.len(), graph.provides.len());
        assert_eq!(cloned.capability_providers["cap"], pid("a"));
    }

    // ── DependencyGraph debug ─────────────────────────────────────────
    #[test]
    fn test_dependency_graph_debug() {
        let graph = DependencyGraph {
            provides: HashMap::new(),
            requires: HashMap::new(),
            capability_providers: HashMap::new(),
        };
        let debug_str = format!("{graph:?}");
        assert!(debug_str.contains("DependencyGraph"));
    }

    // ── Multiple providers of same capability (last-write-wins) ───────
    #[test]
    fn test_duplicate_capability_providers() {
        // Two primals both provide "security" — last one wins in the map
        let primals: Vec<Arc<dyn ManagedPrimal>> = vec![
            Arc::new(MockPrimal::new(
                "beardog",
                vec![Capability::Security],
                vec![],
            )),
            Arc::new(MockPrimal::new(
                "beardog-backup",
                vec![Capability::Security],
                vec![],
            )),
        ];

        let graph = DependencyGraph::build(&primals).expect("should build despite duplicate");
        // Both should be tracked in provides
        assert_eq!(graph.provides.len(), 2);
        // capability_providers will have the last writer
        assert!(graph.capability_providers.contains_key("security"));
    }

    // ── Deep chain: 5 levels ──────────────────────────────────────────
    #[test]
    fn test_deep_chain_five_levels() {
        let primals: Vec<Arc<dyn ManagedPrimal>> = vec![
            Arc::new(MockPrimal::new(
                "level-0",
                vec![Capability::Custom("l0".into())],
                vec![],
            )),
            Arc::new(MockPrimal::new(
                "level-1",
                vec![Capability::Custom("l1".into())],
                vec![Capability::Custom("l0".into())],
            )),
            Arc::new(MockPrimal::new(
                "level-2",
                vec![Capability::Custom("l2".into())],
                vec![Capability::Custom("l1".into())],
            )),
            Arc::new(MockPrimal::new(
                "level-3",
                vec![Capability::Custom("l3".into())],
                vec![Capability::Custom("l2".into())],
            )),
            Arc::new(MockPrimal::new(
                "level-4",
                vec![],
                vec![Capability::Custom("l3".into())],
            )),
        ];

        let graph = DependencyGraph::build(&primals).expect("should build");
        let waves = graph.topological_waves().expect("should resolve");

        assert_eq!(waves.len(), 5, "5-level chain → 5 waves");
        assert_eq!(waves[0], vec![pid("level-0")]);
        assert_eq!(waves[1], vec![pid("level-1")]);
        assert_eq!(waves[2], vec![pid("level-2")]);
        assert_eq!(waves[3], vec![pid("level-3")]);
        assert_eq!(waves[4], vec![pid("level-4")]);
    }

    // ── Self-dependency → circular error ──────────────────────────────
    #[test]
    fn test_self_dependency_error() {
        let primals: Vec<Arc<dyn ManagedPrimal>> = vec![Arc::new(MockPrimal::new(
            "narcissist",
            vec![Capability::Security],
            vec![Capability::Security],
        ))];

        let graph = DependencyGraph::build(&primals).expect("build should work");
        // Narcissist provides security AND requires it — it depends on itself.
        // Since it provides the cap, the provider has "started" check will look
        // for whether "narcissist" is already in the started set. On the first
        // wave it won't be, so it should still be schedulable (self-provided).
        // Actually: capability_providers maps "security" -> "narcissist",
        // and started.contains("narcissist") is false in wave 0, so it gets
        // stuck. This is correctly detected as a circular dep.
        let result = graph.topological_waves();
        assert!(result.is_err(), "self-dependency should be detected");
    }

    // ── start_in_waves integration ────────────────────────────────────────

    #[tokio::test]
    async fn test_start_in_waves_linear_chain() {
        use crate::primal_orchestrator::{PrimalHealthMonitor, PrimalOrchestrator};
        use crate::retry::RetryPolicy;
        use std::time::Duration;

        #[allow(clippy::unwrap_used)]
        let primals: Vec<Arc<dyn ManagedPrimal>> = vec![
            Arc::new(MockPrimal::new(
                "beardog",
                vec![Capability::Security],
                vec![],
            )),
            Arc::new(MockPrimal::new(
                "songbird",
                vec![Capability::Discovery],
                vec![Capability::Security],
            )),
            Arc::new(MockPrimal::new(
                "nestgate",
                vec![Capability::Storage],
                vec![Capability::Discovery],
            )),
        ];

        let monitor = Arc::new(PrimalHealthMonitor::builder().build());
        let retry = RetryPolicy::exponential(1, Duration::from_millis(10));
        let orchestrator = Arc::new(PrimalOrchestrator::new(monitor, retry));

        for p in &primals {
            orchestrator.register(Arc::clone(p)).await;
        }

        let result = start_in_waves(&orchestrator, primals).await;
        assert!(result.is_ok(), "start_in_waves should succeed: {result:?}");
    }

    #[tokio::test]
    async fn test_start_in_waves_single_primal() {
        use crate::primal_orchestrator::{PrimalHealthMonitor, PrimalOrchestrator};
        use crate::retry::RetryPolicy;
        use std::time::Duration;

        #[allow(clippy::unwrap_used)]
        let primals: Vec<Arc<dyn ManagedPrimal>> = vec![Arc::new(MockPrimal::new(
            "solo",
            vec![Capability::Security],
            vec![],
        ))];

        let monitor = Arc::new(PrimalHealthMonitor::builder().build());
        let retry = RetryPolicy::exponential(1, Duration::from_millis(10));
        let orchestrator = Arc::new(PrimalOrchestrator::new(monitor, retry));

        orchestrator.register(Arc::clone(&primals[0])).await;

        let result = start_in_waves(&orchestrator, primals).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_start_in_waves_parallel_wave() {
        use crate::primal_orchestrator::{PrimalHealthMonitor, PrimalOrchestrator};
        use crate::retry::RetryPolicy;
        use std::time::Duration;

        #[allow(clippy::unwrap_used)]
        let primals: Vec<Arc<dyn ManagedPrimal>> = vec![
            Arc::new(MockPrimal::new(
                "beardog",
                vec![Capability::Security],
                vec![],
            )),
            Arc::new(MockPrimal::new(
                "nestgate",
                vec![Capability::Storage],
                vec![Capability::Security],
            )),
            Arc::new(MockPrimal::new(
                "toadstool",
                vec![Capability::Compute],
                vec![Capability::Security],
            )),
        ];

        let monitor = Arc::new(PrimalHealthMonitor::builder().build());
        let retry = RetryPolicy::exponential(1, Duration::from_millis(10));
        let orchestrator = Arc::new(PrimalOrchestrator::new(monitor, retry));

        for p in &primals {
            orchestrator.register(Arc::clone(p)).await;
        }

        let result = start_in_waves(&orchestrator, primals).await;
        assert!(result.is_ok());
    }
}
