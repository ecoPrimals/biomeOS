// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

// =============================================================================
// Integration Tests - Real Niche Files with Graph References
// NOTE: These tests require specific graph files that may not exist.
// They are ignored by default until niches are updated to new graph format.
// =============================================================================

use biomeos_manifest::niche::NicheManifest;
use std::path::Path;

#[test]
#[ignore = "Requires tower_deploy.toml - niches need graph format update"]
fn test_parse_tower_niche() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let niche_path = workspace_root.join("niches/tower.toml");

    let manifest = NicheManifest::from_file(&niche_path).expect("Failed to parse tower.toml");

    assert_eq!(manifest.niche.name, "tower");
    assert_eq!(manifest.niche.niche_type, "communication");
    assert_eq!(manifest.niche.architecture, "vertical");

    // Should have 2 primals (Songbird + BearDog)
    assert_eq!(manifest.primals.len(), 2);

    // Should have 3 graphs (deploy, health_check, shutdown)
    assert_eq!(manifest.graphs.len(), 3);

    // Should have a default graph
    assert!(manifest.get_default_graph().is_some());
    let default = manifest.get_default_graph().unwrap();
    assert_eq!(default.name, "deploy");

    // Verify capabilities
    assert!(manifest.provides_capability("discovery"));
    assert!(manifest.provides_capability("encryption"));
    assert!(manifest.provides_capability("federation"));
}

#[test]
#[ignore = "Requires node_deploy.toml - niches need graph format update"]
fn test_parse_compute_node_niche() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let niche_path = workspace_root.join("niches/compute-node.toml");

    let manifest =
        NicheManifest::from_file(&niche_path).expect("Failed to parse compute-node.toml");

    assert_eq!(manifest.niche.name, "compute-node");
    assert_eq!(manifest.niche.niche_type, "compute");

    // Has optional primal (BearDog)
    let optional_count = manifest.primals.iter().filter(|p| p.optional).count();
    assert_eq!(optional_count, 1);
}

#[test]
#[ignore = "Requires nest_deploy.toml - niches need graph format update"]
fn test_parse_nest_niche() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let niche_path = workspace_root.join("niches/nest.toml");

    let manifest = NicheManifest::from_file(&niche_path).expect("Failed to parse nest.toml");

    assert_eq!(manifest.niche.name, "nest");
    assert_eq!(manifest.niche.niche_type, "data");

    // Should have 3 primals (NestGate + BearDog + Songbird)
    assert_eq!(manifest.primals.len(), 3);

    // BearDog is mandatory for nests
    let mandatory_count = manifest.primals.iter().filter(|p| !p.optional).count();
    assert_eq!(mandatory_count, 3);
}

#[test]
#[ignore = "Requires node_deploy.toml - niches need graph format update"]
fn test_backward_compatibility_no_graphs() {
    // Compute and nest niches don't have graphs yet (backward compatible!)
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let niche_path = workspace_root.join("niches/compute-node.toml");

    let manifest =
        NicheManifest::from_file(&niche_path).expect("Failed to parse compute-node.toml");

    // No graphs is perfectly valid!
    assert_eq!(manifest.graphs.len(), 0);
    assert!(manifest.get_default_graph().is_none());
}

#[test]
#[ignore = "Requires tower_deploy.toml - niches need graph format update"]
fn test_get_graph_by_name() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let niche_path = workspace_root.join("niches/tower.toml");

    let manifest = NicheManifest::from_file(&niche_path).expect("Failed to parse tower.toml");

    // Get specific graphs by name
    let deploy = manifest.get_graph("deploy");
    assert!(deploy.is_some());

    let health_check = manifest.get_graph("health_check");
    assert!(health_check.is_some());

    let shutdown = manifest.get_graph("shutdown");
    assert!(shutdown.is_some());

    // Non-existent graph
    let missing = manifest.get_graph("nonexistent");
    assert!(missing.is_none());
}
