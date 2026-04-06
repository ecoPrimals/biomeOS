// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Integration tests for graph loading and validation.

use crate::loader::GraphLoader;

#[test]
fn test_load_livespore_create_graph_format() {
    // Test that we can parse the actual livespore_create.toml format
    let toml = r#"
        [graph]
        id = "livespore-create"
        name = "LiveSpore Creation"
        version = "1.0.0"
        description = "Create a new LiveSpore deployment"
        
        [graph.metadata]
        family_id = "1894e909e454"
        author = "biomeOS"
        created = "2026-01-27"
        category = "deployment"
        
        [graph.env]
        SPORE_TARGET = "${SPORE_TARGET:-/tmp/test-spore}"
        LINEAGE_MODE = "${LINEAGE_MODE:-genesis}"
        NODE_ID = "${NODE_ID:-spore-genesis}"
        
        [[graph.nodes]]
        id = "validate-target"
        name = "Validate Target Media"
        type = "capability"
        capability = "biomeos.validate"
        required = true
        order = 10
        
        [graph.nodes.config]
        action = "validate_writable_path"
        
        [graph.nodes.params]
        target_path = "${SPORE_TARGET}"
        min_space_mb = 500
        
        [[graph.nodes]]
        id = "create-structure"
        name = "Create Directory Structure"
        type = "capability"
        capability = "filesystem.create_directories"
        required = true
        order = 100
        depends_on = ["validate-target"]
        
        [graph.nodes.params]
        base_path = "${SPORE_TARGET}/biomeOS"
        directories = ["config", "graphs", "logs", "primals"]
        
        [[graph.nodes]]
        id = "generate-seed"
        name = "Generate Genetic Seed"
        type = "capability"
        capability = "crypto.generate_random"
        required = true
        order = 200
        depends_on = ["create-structure"]
        
        [graph.nodes.config]
        primal = "beardog"
        skip_if = "${LINEAGE_MODE} != genesis"
        
        [graph.nodes.params]
        length = 32
        output_path = "${SPORE_TARGET}/biomeOS/.genesis.seed"
        
        [graph.outputs]
        spore_root = "${SPORE_TARGET}/biomeOS"
        node_id = "${NODE_ID}"
    "#;

    let graph = GraphLoader::from_str(toml, None).expect("Failed to parse livespore format");

    // Verify basic structure
    assert_eq!(graph.id().as_str(), "livespore-create");
    assert_eq!(graph.name(), "LiveSpore Creation");
    assert_eq!(graph.nodes().len(), 3);

    // Verify topological order
    let ordered = graph.nodes_in_order();
    assert_eq!(ordered[0].id.as_str(), "validate-target");
    assert_eq!(ordered[1].id.as_str(), "create-structure");
    assert_eq!(ordered[2].id.as_str(), "generate-seed");

    // Verify env resolution
    assert!(
        graph
            .resolve_env("${SPORE_TARGET}/biomeOS")
            .contains("/biomeOS")
    );
}

#[test]
fn test_load_validation_graph_format() {
    let toml = r#"
        [graph]
        id = "livespore-validate"
        name = "LiveSpore Validation"
        version = "1.0.0"
        
        [graph.env]
        SPORE_TARGET = "/media/user/USB"
        UPDATE_MODE = "check"
        
        [[graph.nodes]]
        id = "discover-spore"
        name = "Discover LiveSpore"
        type = "capability"
        capability = "biomeos.discover"
        required = true
        order = 10
        
        [graph.nodes.params]
        target_path = "${SPORE_TARGET}"
        required_markers = [".family.seed", ".spore.json"]
        
        [[graph.nodes]]
        id = "validate-beardog"
        name = "Validate BearDog Binary"
        type = "capability"
        capability = "crypto.verify_checksum"
        required = true
        order = 100
        depends_on = ["discover-spore"]
        
        [graph.nodes.config]
        primal = "beardog"
        
        [graph.nodes.params]
        file_path = "${SPORE_TARGET}/biomeOS/primals/beardog"
        algorithm = "blake3"
    "#;

    let graph = GraphLoader::from_str(toml, None).expect("Failed to parse validation format");

    assert_eq!(graph.id().as_str(), "livespore-validate");
    assert_eq!(graph.nodes().len(), 2);

    // Check dependency resolution
    let ordered = graph.nodes_in_order();
    assert_eq!(ordered[0].id.as_str(), "discover-spore");
    assert_eq!(ordered[1].id.as_str(), "validate-beardog");
}

#[test]
fn test_graph_category() {
    use crate::graph::GraphCategory;

    let toml = r#"
        [graph]
        id = "test-graph"
        name = "Test"
        version = "1.0.0"
        
        [graph.metadata]
        category = "deployment"
    "#;

    let graph = GraphLoader::from_str(toml, None).unwrap();
    assert_eq!(
        graph.definition.metadata.category,
        Some(GraphCategory::Deployment)
    );
}

#[test]
fn test_node_params_types() {
    let toml = r#"
        [graph]
        id = "param-test"
        name = "Parameter Types Test"
        version = "1.0.0"
        
        [[graph.nodes]]
        id = "multi-params"
        name = "Multiple Param Types"
        capability = "test.capability"
        
        [graph.nodes.params]
        string_param = "hello"
        int_param = 42
        bool_param = true
        array_param = ["one", "two", "three"]
    "#;

    let graph = GraphLoader::from_str(toml, None).unwrap();
    let node = &graph.nodes()[0];

    assert_eq!(node.params.get_string("string_param"), Some("hello"));
    assert_eq!(node.params.get_i64("int_param"), Some(42));
    assert_eq!(node.params.get_bool("bool_param"), Some(true));
    assert!(node.params.get("array_param").unwrap().as_array().is_some());
}
