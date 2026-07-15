// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::{MockPrimal, pid};
use crate::capabilities::Capability;
use crate::concurrent_startup::DependencyGraph;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[test]
fn test_build_empty() {
    let primals: Vec<Arc<dyn crate::primal_orchestrator::ManagedPrimal>> = vec![];
    let graph = DependencyGraph::build(&primals).expect("empty build should succeed");
    assert!(graph.provides.is_empty());
    assert!(graph.requires.is_empty());
    assert!(graph.capability_providers.is_empty());
}

#[test]
fn test_build_single_primal_no_deps() {
    let primals: Vec<Arc<dyn crate::primal_orchestrator::ManagedPrimal>> = vec![Arc::new(
        MockPrimal::new("solo", vec![Capability::Security], vec![]),
    )];
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
    let primals: Vec<Arc<dyn crate::primal_orchestrator::ManagedPrimal>> = vec![
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
    let primals: Vec<Arc<dyn crate::primal_orchestrator::ManagedPrimal>> = vec![Arc::new(
        MockPrimal::new(
            "multi-cap",
            vec![Capability::Security, Capability::Storage, Capability::AI],
            vec![Capability::Discovery],
        ),
    )];

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
    let primals: Vec<Arc<dyn crate::primal_orchestrator::ManagedPrimal>> = vec![Arc::new(
        MockPrimal::new(
            "custom-svc",
            vec![Capability::Custom("my-extension".into())],
            vec![],
        ),
    )];

    let graph = DependencyGraph::build(&primals).expect("should build");
    assert!(
        graph
            .capability_providers
            .contains_key("custom:my-extension")
    );
}

#[test]
fn test_build_then_waves_linear() {
    let primals: Vec<Arc<dyn crate::primal_orchestrator::ManagedPrimal>> = vec![
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
    let primals: Vec<Arc<dyn crate::primal_orchestrator::ManagedPrimal>> = vec![
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
    let primals: Vec<Arc<dyn crate::primal_orchestrator::ManagedPrimal>> = vec![
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
    let primals: Vec<Arc<dyn crate::primal_orchestrator::ManagedPrimal>> = vec![
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
    let primals: Vec<Arc<dyn crate::primal_orchestrator::ManagedPrimal>> = vec![
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
    let primals: Vec<Arc<dyn crate::primal_orchestrator::ManagedPrimal>> = vec![Arc::new(
        MockPrimal::new(
            "narcissist",
            vec![Capability::Security],
            vec![Capability::Security],
        ),
    )];

    let graph = DependencyGraph::build(&primals).expect("build should work");
    let result = graph.topological_waves();
    assert!(result.is_err(), "self-dependency should be detected");
}
