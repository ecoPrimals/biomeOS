// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::graph::*;

#[test]
fn test_resolve_composition_explicit_nucleus_in_metadata_toml() {
    let toml_str = r#"
            [graph]
            id = "comp-explicit"
            name = "Explicit"
            version = "1.0.0"

            [graph.metadata]
            composition = "nucleus"

            [[graph.nodes]]
            id = "a"
            name = "A"
            capability = "crypto.sign"
        "#;
    let graph: DeploymentGraph = toml::from_str(toml_str).unwrap();
    assert_eq!(graph.resolve_composition(), AtomicComposition::Nucleus);
}

#[test]
fn test_resolve_composition_infers_tower() {
    let toml_str = r#"
            [graph]
            id = "tower-infer"
            name = "Tower"
            version = "1.0.0"

            [[graph.nodes]]
            id = "a"
            name = "A"
            capability = "discovery.ping"

            [[graph.nodes]]
            id = "b"
            name = "B"
            capability = "crypto.blake3_hash"
        "#;
    let graph: DeploymentGraph = toml::from_str(toml_str).unwrap();
    assert_eq!(graph.resolve_composition(), AtomicComposition::Tower);
}

#[test]
fn test_resolve_composition_infers_node_from_compute() {
    let toml_str = r#"
            [graph]
            id = "node-infer"
            name = "Node"
            version = "1.0.0"

            [[graph.nodes]]
            id = "work"
            name = "Work"
            capability = "ml.compute.forward"
        "#;
    let graph: DeploymentGraph = toml::from_str(toml_str).unwrap();
    assert_eq!(graph.resolve_composition(), AtomicComposition::Node);
}

#[test]
fn test_resolve_composition_infers_node_from_gpu() {
    let toml_str = r#"
            [graph]
            id = "gpu-infer"
            name = "GPU"
            version = "1.0.0"

            [[graph.nodes]]
            id = "g"
            name = "G"
            capability = "render.gpu.submit"
        "#;
    let graph: DeploymentGraph = toml::from_str(toml_str).unwrap();
    assert_eq!(graph.resolve_composition(), AtomicComposition::Node);
}

#[test]
fn test_resolve_composition_infers_nest_from_storage() {
    let toml_str = r#"
            [graph]
            id = "nest-infer"
            name = "Nest"
            version = "1.0.0"

            [[graph.nodes]]
            id = "store"
            name = "Store"
            capability = "nest.storage.put"
        "#;
    let graph: DeploymentGraph = toml::from_str(toml_str).unwrap();
    assert_eq!(graph.resolve_composition(), AtomicComposition::Nest);
}

#[test]
fn test_resolve_composition_infers_nest_from_persistence() {
    let toml_str = r#"
            [graph]
            id = "persist-infer"
            name = "Persist"
            version = "1.0.0"

            [[graph.nodes]]
            id = "p"
            name = "P"
            capability = "db.persistence.snapshot"
        "#;
    let graph: DeploymentGraph = toml::from_str(toml_str).unwrap();
    assert_eq!(graph.resolve_composition(), AtomicComposition::Nest);
}

#[test]
fn test_resolve_composition_infers_nucleus_from_ai_segment() {
    let toml_str = r#"
            [graph]
            id = "ai-infer"
            name = "AI"
            version = "1.0.0"

            [[graph.nodes]]
            id = "brain"
            name = "Brain"
            capability = "squirrel.ai.embed"
        "#;
    let graph: DeploymentGraph = toml::from_str(toml_str).unwrap();
    assert_eq!(graph.resolve_composition(), AtomicComposition::Nucleus);
}

#[test]
fn test_resolve_composition_infers_nucleus_from_orchestration_substring() {
    let toml_str = r#"
            [graph]
            id = "orch-infer"
            name = "Orch"
            version = "1.0.0"

            [[graph.nodes]]
            id = "o"
            name = "O"
            capability = "workflow.orchestration.start"
        "#;
    let graph: DeploymentGraph = toml::from_str(toml_str).unwrap();
    assert_eq!(graph.resolve_composition(), AtomicComposition::Nucleus);
}

#[test]
fn test_resolve_composition_nucleus_overrides_nest_and_node_signals() {
    let toml_str = r#"
            [graph]
            id = "full-stack"
            name = "Full"
            version = "1.0.0"

            [[graph.nodes]]
            id = "x"
            name = "X"
            capability = "nest.storage.get"

            [[graph.nodes]]
            id = "y"
            name = "Y"
            capability = "batch.compute.run"

            [[graph.nodes]]
            id = "z"
            name = "Z"
            capability = "agent.ai.reason"
        "#;
    let graph: DeploymentGraph = toml::from_str(toml_str).unwrap();
    assert_eq!(graph.resolve_composition(), AtomicComposition::Nucleus);
}

#[test]
fn test_resolve_composition_nest_takes_priority_over_compute_only() {
    let toml_str = r#"
            [graph]
            id = "nest-vs-node"
            name = "Both"
            version = "1.0.0"

            [[graph.nodes]]
            id = "s"
            name = "S"
            capability = "block.storage.write"

            [[graph.nodes]]
            id = "c"
            name = "C"
            capability = "task.compute.execute"
        "#;
    let graph: DeploymentGraph = toml::from_str(toml_str).unwrap();
    assert_eq!(graph.resolve_composition(), AtomicComposition::Nest);
}

#[test]
fn test_resolve_composition_from_operation_dependencies() {
    let toml_str = r#"
            [graph]
            id = "deps-only"
            name = "Deps"
            version = "1.0.0"

            [[graph.nodes]]
            id = "n"
            name = "N"
            operation_dependencies = ["pool.gpu.alloc"]
        "#;
    let graph: DeploymentGraph = toml::from_str(toml_str).unwrap();
    assert_eq!(graph.resolve_composition(), AtomicComposition::Node);
}

#[test]
fn test_atomic_composition_serde_roundtrip() {
    let c = AtomicComposition::Nest;
    let json = serde_json::to_string(&c).unwrap();
    assert_eq!(json, "\"nest\"");
    let back: AtomicComposition = serde_json::from_str(&json).unwrap();
    assert_eq!(back, AtomicComposition::Nest);
}
