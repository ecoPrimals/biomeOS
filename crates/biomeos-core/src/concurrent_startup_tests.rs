// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

//! Unit tests for concurrent wave-based primal startup.
//!
//! Extracted from concurrent_startup.rs to keep main module under 1000 LOC.

use crate::capabilities::Capability;
use crate::concurrent_startup::{DependencyGraph, start_in_waves};
use crate::discovery_modern::HealthStatus;
use crate::primal_orchestrator::{ManagedPrimal, PrimalHealthMonitor, PrimalOrchestrator};
use crate::retry::RetryPolicy;
use biomeos_types::PrimalId;
use biomeos_types::error::BiomeResult;
use biomeos_types::identifiers::Endpoint;
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

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
    fn endpoint(&self) -> Pin<Box<dyn Future<Output = Option<Endpoint>> + Send + '_>> {
        Box::pin(async move { None })
    }
    fn start(&self) -> Pin<Box<dyn Future<Output = BiomeResult<()>> + Send + '_>> {
        Box::pin(async move { Ok(()) })
    }
    fn stop(&self) -> Pin<Box<dyn Future<Output = BiomeResult<()>> + Send + '_>> {
        Box::pin(async move { Ok(()) })
    }
    fn health_check(&self) -> Pin<Box<dyn Future<Output = BiomeResult<HealthStatus>> + Send + '_>> {
        Box::pin(async move { Ok(HealthStatus::Healthy) })
    }
}

fn pid(name: &str) -> PrimalId {
    PrimalId::new(name).expect("valid primal id")
}

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

#[test]
fn test_linear_dependency_chain() {
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

#[test]
fn test_diamond_dependency() {
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
    assert_eq!(waves[0], vec![a]);
    assert_eq!(waves[1].len(), 2);
    assert_eq!(waves[1][0], b);
    assert_eq!(waves[1][1], c);
    assert_eq!(waves[2], vec![d]);
}

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
    cap_providers.insert("cap-a".into(), a);
    cap_providers.insert("cap-b".into(), b);

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

#[test]
fn test_missing_capability_provider_error() {
    let a = pid("lonely");

    let mut provides = HashMap::new();
    provides.insert(a.clone(), HashSet::new());

    let mut requires = HashMap::new();
    requires.insert(a, HashSet::from(["nonexistent-cap".into()]));

    let graph = DependencyGraph {
        provides,
        requires,
        capability_providers: HashMap::new(),
    };

    let result = graph.topological_waves();
    assert!(result.is_err(), "missing provider must fail");
}

#[test]
fn test_wave_ordering_is_deterministic() {
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

#[test]
fn test_primal_with_empty_requirement_set() {
    let a = pid("core");

    let mut provides = HashMap::new();
    provides.insert(a.clone(), HashSet::from(["security".into()]));

    let mut requires = HashMap::new();
    requires.insert(a.clone(), HashSet::new());

    let graph = DependencyGraph {
        provides,
        requires,
        capability_providers: HashMap::new(),
    };

    let waves = graph.topological_waves().expect("should resolve");
    assert_eq!(waves.len(), 1);
    assert_eq!(waves[0], vec![a]);
}

#[test]
fn test_primal_not_in_requires_map() {
    let a = pid("orphan");

    let mut provides = HashMap::new();
    provides.insert(a.clone(), HashSet::from(["cap-x".into()]));

    let graph = DependencyGraph {
        provides,
        requires: HashMap::new(),
        capability_providers: HashMap::new(),
    };

    let waves = graph.topological_waves().expect("should resolve");
    assert_eq!(waves.len(), 1);
    assert_eq!(waves[0], vec![a]);
}

#[test]
fn test_wide_fanout_dependency() {
    let root = pid("root-primal");

    let mut provides = HashMap::new();
    provides.insert(root.clone(), HashSet::from(["security".into()]));

    let mut requires = HashMap::new();
    requires.insert(root.clone(), HashSet::new());

    let mut cap_providers = HashMap::new();
    cap_providers.insert("security".into(), root.clone());

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
    assert!(
        graph
            .capability_providers
            .contains_key("custom:my-extension")
    );
}

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

#[test]
fn test_duplicate_capability_providers() {
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
    assert_eq!(graph.provides.len(), 2);
    assert!(graph.capability_providers.contains_key("security"));
}

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

#[test]
fn test_self_dependency_error() {
    let primals: Vec<Arc<dyn ManagedPrimal>> = vec![Arc::new(MockPrimal::new(
        "narcissist",
        vec![Capability::Security],
        vec![Capability::Security],
    ))];

    let graph = DependencyGraph::build(&primals).expect("build should work");
    let result = graph.topological_waves();
    assert!(result.is_err(), "self-dependency should be detected");
}

#[tokio::test]
async fn test_start_in_waves_linear_chain() {
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
