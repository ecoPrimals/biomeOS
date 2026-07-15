// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::pid;
use crate::concurrent_startup::DependencyGraph;
use std::collections::{HashMap, HashSet};

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
