// SPDX-License-Identifier: AGPL-3.0-or-later
//
//! Signal dispatch integration tests.
//!
//! Validates that the composition collapse layer correctly maps atomic
//! signals to graph paths, loads all 19 signal graphs, and intercepts
//! signal-tier capability calls.

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use std::path::PathBuf;

fn graphs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../graphs")
}

#[test]
fn all_19_signal_graphs_exist() {
    let dir = graphs_dir().join("signals");
    assert!(dir.exists(), "graphs/signals/ directory not found");

    let expected = [
        "tower_publish",
        "tower_authenticate",
        "tower_discover",
        "tower_health",
        "tower_bootstrap",
        "node_compute",
        "nest_store",
        "nest_commit",
        "nest_retrieve",
        "nest_sync",
        "nest_ingest_spore",
        "nest_emit_spore",
        "braid_partial_update",
        "braid_complete",
        "meta_observe",
        "meta_intent",
        "meta_render",
        "meta_health",
        "meta_deploy",
    ];

    for name in &expected {
        let path = dir.join(format!("{name}.toml"));
        assert!(path.exists(), "Missing signal graph: {name}.toml");
    }
}

#[test]
fn signal_graphs_parse_as_valid_toml() {
    let dir = graphs_dir().join("signals");

    for entry in std::fs::read_dir(&dir).expect("read signals dir") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "toml") {
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
            let _: toml::Value = toml::from_str(&content)
                .unwrap_or_else(|e| panic!("parse {}: {e}", path.display()));
        }
    }
}

#[test]
fn signal_graph_path_resolution() {
    use biomeos_atomic_deploy::handlers::signal::signal_graph_path;

    let dir = graphs_dir();
    let path = signal_graph_path(&dir, "tower", "publish");
    assert_eq!(
        path,
        dir.join("signals/tower_publish.toml"),
        "signal_graph_path should map tier.signal to signals/tier_signal.toml"
    );
}

#[test]
fn is_signal_tier_recognizes_valid_tiers() {
    use biomeos_atomic_deploy::handlers::signal::is_signal_tier;

    assert!(is_signal_tier("tower"));
    assert!(is_signal_tier("node"));
    assert!(is_signal_tier("nest"));
    assert!(is_signal_tier("meta"));
    assert!(is_signal_tier("braid"));
    assert!(!is_signal_tier("crypto"));
    assert!(!is_signal_tier("security"));
    assert!(!is_signal_tier("orchestration"));
    assert!(!is_signal_tier(""));
}

#[test]
fn list_signal_graphs_finds_all_19() {
    use biomeos_atomic_deploy::handlers::signal::list_signal_graphs;

    let signals = list_signal_graphs(&graphs_dir());
    assert_eq!(
        signals.len(),
        19,
        "Expected 19 signal graphs, found {}",
        signals.len()
    );

    let names: Vec<&str> = signals.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"tower.publish"));
    assert!(names.contains(&"nest.store"));
    assert!(names.contains(&"nest.ingest_spore"));
    assert!(names.contains(&"nest.emit_spore"));
    assert!(names.contains(&"nest.sync"));
    assert!(names.contains(&"meta.deploy"));
    assert!(names.contains(&"tower.bootstrap"));
    assert!(names.contains(&"braid.partial_update"));
    assert!(names.contains(&"braid.complete"));
}

#[test]
fn signal_schema_loads() {
    use biomeos_atomic_deploy::handlers::signal::load_signal_schema;

    let result = load_signal_schema(&graphs_dir());
    assert!(result.is_ok(), "signal_tools.toml should load: {result:?}");

    let schema = result.unwrap();
    let tools = schema.get("tools").expect("schema should have 'tools' key");
    let tools_arr = tools.as_array().expect("'tools' should be an array");
    assert_eq!(tools_arr.len(), 19, "Expected 19 tool definitions");
}

#[test]
fn signal_graphs_have_required_metadata() {
    let dir = graphs_dir().join("signals");

    for entry in std::fs::read_dir(&dir).expect("read signals dir") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "toml") {
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
            let parsed: toml::Value = toml::from_str(&content)
                .unwrap_or_else(|e| panic!("parse {}: {e}", path.display()));

            let graph = parsed
                .get("graph")
                .unwrap_or_else(|| panic!("{}: missing [graph] section", path.display()));

            assert!(
                graph.get("name").is_some(),
                "{}: missing graph.name",
                path.display()
            );

            let nodes = graph
                .get("nodes")
                .or_else(|| parsed.get("nodes"))
                .and_then(|n| n.as_array());
            assert!(
                nodes.is_some_and(|n| !n.is_empty()),
                "{}: no nodes defined",
                path.display()
            );
        }
    }
}

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
    for signal in ["store", "commit", "retrieve", "sync"] {
        let path = signal_graph_path(&dir, "nest", signal);
        assert!(
            path.exists(),
            "nest.{signal} graph should exist at {}",
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

    // Wave 38: verify cross-gate wiring — fetch_dag_slice targets remote_gate
    let fetch_node = &nodes[0];
    assert_eq!(
        fetch_node["gate"].as_str(),
        Some("remote_gate"),
        "fetch_dag_slice must target remote_gate for cross-spring DAG fetch"
    );

    // Only fetch_dag_slice should have a gate — nodes 2-6 execute locally
    for node in &nodes[1..] {
        assert!(
            node.get("gate").is_none(),
            "node '{}' should execute locally (no gate field)",
            node["name"].as_str().unwrap_or("?")
        );
    }

    // Verify graph.env declares remote_gate placeholder
    let env = graph.get("env").expect("missing [graph.env]");
    assert!(
        env.get("remote_gate").is_some(),
        "graph.env must declare remote_gate for GateRegistry injection"
    );
}
