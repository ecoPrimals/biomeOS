// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Phase 0: Graph Validation — verify deployment graphs parse and have correct
//! topology (runs without live primals, safe for CI).

use biomeos_atomic_deploy::neural_graph::Graph;

use crate::graphs_dir;

#[test]
fn test_loamspine_deploy_graph_parses() {
    let path = graphs_dir().join("loamspine_deploy.toml");
    let graph = Graph::from_toml_file(&path).expect("loamspine_deploy.toml should parse");

    assert_eq!(graph.id, "loamspine_deploy");
    assert_eq!(graph.nodes.len(), 5);

    let ops: Vec<&str> = graph
        .nodes
        .iter()
        .filter_map(|n| n.operation.as_ref().map(|o| o.name.as_str()))
        .collect();
    assert_eq!(
        ops,
        vec![
            "health_check",
            "health_check",
            "primal.launch",
            "register_capabilities",
            "health_check"
        ]
    );
}

#[test]
fn test_rhizocrypt_deploy_graph_parses() {
    let path = graphs_dir().join("rhizocrypt_deploy.toml");
    let graph = Graph::from_toml_file(&path).expect("rhizocrypt_deploy.toml should parse");

    assert_eq!(graph.id, "rhizocrypt_deploy");
    assert_eq!(graph.nodes.len(), 5);

    let start_node = graph
        .nodes
        .iter()
        .find(|n| n.id == "start-rhizocrypt")
        .expect("start-rhizocrypt node");
    let op = start_node.operation.as_ref().expect("operation");
    assert_eq!(op.name, "primal.launch");
    assert!(
        op.environment.is_some(),
        "Environment vars should be present"
    );
    let env = op.environment.as_ref().unwrap();
    assert_eq!(env.get("RHIZOCRYPT_RPC_PORT").unwrap(), "9400");
}

#[test]
fn test_sweetgrass_deploy_graph_parses() {
    let path = graphs_dir().join("sweetgrass_deploy.toml");
    let graph = Graph::from_toml_file(&path).expect("sweetgrass_deploy.toml should parse");

    assert_eq!(graph.id, "sweetgrass_deploy");
    assert_eq!(graph.nodes.len(), 5);

    let reg_node = graph
        .nodes
        .iter()
        .find(|n| n.id == "register-capabilities")
        .expect("register-capabilities node");
    assert_eq!(
        reg_node.capabilities,
        vec![
            "attribution",
            "braid",
            "provenance",
            "contribution",
            "privacy"
        ]
    );
}

#[test]
fn test_provenance_trio_deploy_graph_parses() {
    let path = graphs_dir().join("provenance_trio_deploy.toml");
    let graph = Graph::from_toml_file(&path).expect("provenance_trio_deploy.toml should parse");

    assert_eq!(graph.id, "provenance_trio_deploy");
    assert_eq!(graph.nodes.len(), 11);

    let launch_nodes: Vec<&str> = graph
        .nodes
        .iter()
        .filter(|n| {
            n.operation
                .as_ref()
                .is_some_and(|o| o.name == "primal.launch")
        })
        .map(|n| n.id.as_str())
        .collect();
    assert_eq!(
        launch_nodes,
        vec!["start-loamspine", "start-rhizocrypt", "start-sweetgrass"]
    );
}

#[test]
fn test_provenance_trio_dependency_order() {
    let path = graphs_dir().join("provenance_trio_deploy.toml");
    let graph = Graph::from_toml_file(&path).unwrap();

    let find_deps = |id: &str| -> Vec<String> {
        graph
            .nodes
            .iter()
            .find(|n| n.id == id)
            .map(|n| n.depends_on.clone())
            .unwrap_or_default()
    };

    assert_eq!(find_deps("start-loamspine"), vec!["verify-songbird"]);
    assert_eq!(find_deps("start-rhizocrypt"), vec!["health-loamspine"]);
    assert_eq!(find_deps("start-sweetgrass"), vec!["health-loamspine"]);
}

#[test]
fn test_rootpulse_commit_graph_parses() {
    let path = graphs_dir().join("rootpulse_commit.toml");
    let graph = Graph::from_toml_file(&path).expect("rootpulse_commit.toml should parse");

    assert_eq!(graph.id, "rootpulse_commit");

    let op_names: Vec<&str> = graph
        .nodes
        .iter()
        .filter_map(|n| n.operation.as_ref().map(|o| o.name.as_str()))
        .collect();

    assert!(
        op_names.contains(&"rpc_call"),
        "Should contain rpc_call nodes"
    );
    assert!(
        op_names.contains(&"capability_call"),
        "Should contain capability_call nodes"
    );
}

#[test]
fn test_provenance_pipeline_graph_parses() {
    let path = graphs_dir().join("provenance_pipeline.toml");
    let graph = Graph::from_toml_file(&path).expect("provenance_pipeline.toml should parse");

    assert_eq!(graph.id, "provenance_pipeline");
    assert!(
        graph.nodes.len() >= 4,
        "Pipeline should have at least 4 nodes"
    );
}

#[test]
fn test_all_deployment_graphs_have_environment_on_launch_nodes() {
    let deploy_graphs = [
        "loamspine_deploy.toml",
        "rhizocrypt_deploy.toml",
        "sweetgrass_deploy.toml",
        "provenance_trio_deploy.toml",
    ];

    for graph_name in &deploy_graphs {
        let path = graphs_dir().join(graph_name);
        let graph = Graph::from_toml_file(&path)
            .unwrap_or_else(|e| panic!("{graph_name} should parse: {e}"));

        for node in &graph.nodes {
            if let Some(ref op) = node.operation {
                if op.name == "primal.launch" {
                    assert!(
                        op.environment.is_some(),
                        "{}: launch node '{}' missing environment",
                        graph_name,
                        node.id
                    );
                }
            }
        }
    }
}

#[test]
fn test_provenance_trio_topological_order() {
    let path = graphs_dir().join("provenance_trio_deploy.toml");
    let graph = Graph::from_toml_file(&path).unwrap();

    let node_index = |id: &str| -> usize {
        graph
            .nodes
            .iter()
            .position(|n| n.id == id)
            .unwrap_or_else(|| panic!("Node '{id}' not found"))
    };

    let deps_of = |id: &str| -> &Vec<String> { &graph.nodes[node_index(id)].depends_on };

    assert!(
        deps_of("start-rhizocrypt").contains(&"health-loamspine".to_string()),
        "rhizoCrypt should depend on health-loamspine"
    );

    assert!(
        deps_of("start-sweetgrass").contains(&"health-loamspine".to_string()),
        "sweetGrass should depend on health-loamspine"
    );

    assert!(
        deps_of("health-loamspine").contains(&"start-loamspine".to_string()),
        "health-loamspine should depend on start-loamspine"
    );

    assert!(
        deps_of("start-loamspine").contains(&"verify-songbird".to_string()),
        "start-loamspine should depend on verify-songbird"
    );
}
