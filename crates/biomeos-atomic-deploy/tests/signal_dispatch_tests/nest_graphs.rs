// SPDX-License-Identifier: AGPL-3.0-or-later

//! Nest signal graph topology: nest_store, nest_sync pipelines and
//! path resolution for all tier signals.

use super::graphs_dir;

#[test]
fn nest_store_graph_has_provenance_pipeline() {
    let path = graphs_dir().join("signals/nest_store.toml");
    let content = std::fs::read_to_string(&path).expect("read nest_store.toml");
    let parsed: toml::Value = toml::from_str(&content).expect("parse nest_store.toml");

    let graph = parsed.get("graph").expect("missing [graph]");
    assert_eq!(
        graph["signal_tier"].as_str(),
        Some("nest"),
        "nest_store should be in 'nest' tier"
    );
    assert_eq!(
        graph["signal_name"].as_str(),
        Some("store"),
        "nest_store signal_name should be 'store'"
    );
    assert_eq!(
        graph["coordination"].as_str(),
        Some("sequential"),
        "nest_store should be sequential (provenance order matters)"
    );

    let nodes = graph["nodes"].as_array().expect("nodes array");
    assert_eq!(
        nodes.len(),
        4,
        "nest.store pipeline: store -> dag -> commit -> attribute"
    );

    let node_names: Vec<&str> = nodes
        .iter()
        .map(|n| n["name"].as_str().unwrap_or(""))
        .collect();
    assert_eq!(
        node_names,
        ["store_content", "dag_append", "commit", "attribute"],
        "nest.store pipeline order"
    );

    let binaries: Vec<&str> = nodes
        .iter()
        .map(|n| n["binary"].as_str().unwrap_or(""))
        .collect();
    assert_eq!(
        binaries,
        ["nestgate", "rhizocrypt", "loamspine", "sweetgrass"],
        "nest.store provenance trio + storage"
    );
}

#[test]
fn signal_graph_path_resolves_all_nest_signals() {
    use biomeos_atomic_deploy::handlers::signal::signal_graph_path;

    let dir = graphs_dir();
    for signal in [
        "store",
        "commit",
        "retrieve",
        "sync",
        "ingest_spore",
        "ingest_dataset",
        "emit_spore",
        "verify",
        "federate",
    ] {
        let path = signal_graph_path(&dir, "nest", signal);
        assert!(
            path.exists(),
            "nest.{signal} graph should exist at {}",
            path.display()
        );
    }
}

#[test]
fn signal_graph_path_resolves_all_tower_signals() {
    use biomeos_atomic_deploy::handlers::signal::signal_graph_path;

    let dir = graphs_dir();
    for signal in [
        "publish",
        "authenticate",
        "discover",
        "health",
        "bootstrap",
        "enroll",
        "key_rotate",
        "mesh_status",
    ] {
        let path = signal_graph_path(&dir, "tower", signal);
        assert!(
            path.exists(),
            "tower.{signal} graph should exist at {}",
            path.display()
        );
    }
}

#[test]
fn signal_graph_path_resolves_all_node_signals() {
    use biomeos_atomic_deploy::handlers::signal::signal_graph_path;

    let dir = graphs_dir();
    for signal in ["compute", "discover_hardware", "dispatch"] {
        let path = signal_graph_path(&dir, "node", signal);
        assert!(
            path.exists(),
            "node.{signal} graph should exist at {}",
            path.display()
        );
    }
}

#[test]
fn all_signal_tools_have_matching_graphs() {
    use biomeos_atomic_deploy::handlers::signal::load_signal_schema;

    let dir = graphs_dir();
    let schema = load_signal_schema(&dir).expect("load signal_tools.toml");
    let tools = schema["tools"].as_array().expect("tools array");

    for tool in tools {
        let name = tool["name"].as_str().expect("tool name");
        let graph_ref = tool["graph"].as_str().expect("tool graph path");
        let graph_path = dir.join("..").join(graph_ref);
        assert!(
            graph_path.exists(),
            "Tool '{}' references graph '{}' which does not exist at {}",
            name,
            graph_ref,
            graph_path.display()
        );
    }
}

#[test]
fn nest_sync_graph_has_cross_spring_pipeline() {
    let path = graphs_dir().join("signals/nest_sync.toml");
    let content = std::fs::read_to_string(&path).expect("read nest_sync.toml");
    let parsed: toml::Value = toml::from_str(&content).expect("parse nest_sync.toml");

    let graph = parsed.get("graph").expect("missing [graph]");
    assert_eq!(graph["signal_tier"].as_str(), Some("nest"));
    assert_eq!(graph["signal_name"].as_str(), Some("sync"));
    assert_eq!(graph["coordination"].as_str(), Some("sequential"));

    let metadata = graph.get("metadata").expect("missing [graph.metadata]");
    let fragments = metadata["fragments"].as_array().expect("fragments array");
    assert!(
        fragments
            .iter()
            .filter_map(|f| f.as_str())
            .any(|s| s == "cross_gate"),
        "nest.sync should declare cross_gate fragment for cross-spring exchange"
    );

    let nodes = graph["nodes"].as_array().expect("nodes array");
    assert_eq!(
        nodes.len(),
        6,
        "nest.sync pipeline: slice -> verify -> store -> sync_braid -> commit -> attribute"
    );

    let node_names: Vec<&str> = nodes
        .iter()
        .map(|n| n["name"].as_str().unwrap_or(""))
        .collect();
    assert_eq!(
        node_names,
        [
            "fetch_dag_slice",
            "verify_proof",
            "store_content",
            "sync_braid",
            "commit_sync",
            "attribute_sync"
        ],
        "nest.sync cross-spring pipeline order"
    );

    let binaries: Vec<&str> = nodes
        .iter()
        .map(|n| n["binary"].as_str().unwrap_or(""))
        .collect();
    assert_eq!(
        binaries,
        [
            "rhizocrypt",
            "rhizocrypt",
            "nestgate",
            "sweetgrass",
            "loamspine",
            "sweetgrass"
        ],
        "nest.sync uses full provenance trio with cross-gate dag fetch"
    );

    let fetch_node = &nodes[0];
    assert_eq!(
        fetch_node["gate"].as_str(),
        Some("remote_gate"),
        "fetch_dag_slice must target remote_gate for cross-spring DAG fetch"
    );

    for node in &nodes[1..] {
        assert!(
            node.get("gate").is_none(),
            "node '{}' should execute locally (no gate field)",
            node["name"].as_str().unwrap_or("?")
        );
    }

    let env = graph.get("env").expect("missing [graph.env]");
    assert!(
        env.get("remote_gate").is_some(),
        "graph.env must declare remote_gate for GateRegistry injection"
    );
}
