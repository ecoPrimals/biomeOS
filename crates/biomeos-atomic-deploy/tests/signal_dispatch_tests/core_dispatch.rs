// SPDX-License-Identifier: AGPL-3.0-or-later

//! Core signal dispatch: graph existence, TOML parsing, path resolution,
//! tier recognition, listing, and metadata validation.

use super::graphs_dir;

#[test]
fn all_30_signal_graphs_exist() {
    let dir = graphs_dir().join("signals");
    assert!(dir.exists(), "graphs/signals/ directory not found");

    let expected = [
        // Tower (8)
        "tower_publish",
        "tower_authenticate",
        "tower_discover",
        "tower_health",
        "tower_bootstrap",
        "tower_enroll",
        "tower_key_rotate",
        "tower_mesh_status",
        // Node (3)
        "node_compute",
        "node_discover_hardware",
        "node_dispatch",
        // Nest (12: 9 core + 3 data federation)
        "nest_store",
        "nest_commit",
        "nest_retrieve",
        "nest_sync",
        "nest_ingest_spore",
        "nest_ingest_dataset",
        "nest_emit_spore",
        "nest_verify",
        "nest_federate",
        "nest_declare_dataset",
        "nest_acquire_file",
        "nest_complete_dataset",
        // Braid (2)
        "braid_partial_update",
        "braid_complete",
        // Meta (5)
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
fn list_signal_graphs_finds_all_30() {
    use biomeos_atomic_deploy::handlers::signal::list_signal_graphs;

    let signals = list_signal_graphs(&graphs_dir());
    assert_eq!(
        signals.len(),
        30,
        "Expected 30 signal graphs, found {}",
        signals.len()
    );

    let names: Vec<&str> = signals.iter().map(|s| s.name.as_str()).collect();
    // Tower (8)
    assert!(names.contains(&"tower.publish"));
    assert!(names.contains(&"tower.authenticate"));
    assert!(names.contains(&"tower.discover"));
    assert!(names.contains(&"tower.health"));
    assert!(names.contains(&"tower.bootstrap"));
    assert!(names.contains(&"tower.enroll"));
    assert!(names.contains(&"tower.key_rotate"));
    assert!(names.contains(&"tower.mesh_status"));
    // Node (3)
    assert!(names.contains(&"node.compute"));
    assert!(names.contains(&"node.discover_hardware"));
    assert!(names.contains(&"node.dispatch"));
    // Nest (12: 9 core + 3 data federation)
    assert!(names.contains(&"nest.store"));
    assert!(names.contains(&"nest.commit"));
    assert!(names.contains(&"nest.retrieve"));
    assert!(names.contains(&"nest.sync"));
    assert!(names.contains(&"nest.ingest_spore"));
    assert!(names.contains(&"nest.ingest_dataset"));
    assert!(names.contains(&"nest.emit_spore"));
    assert!(names.contains(&"nest.verify"));
    assert!(names.contains(&"nest.federate"));
    assert!(names.contains(&"nest.declare_dataset"));
    assert!(names.contains(&"nest.acquire_file"));
    assert!(names.contains(&"nest.complete_dataset"));
    // Braid (2)
    assert!(names.contains(&"braid.partial_update"));
    assert!(names.contains(&"braid.complete"));
    // Meta (5)
    assert!(names.contains(&"meta.observe"));
    assert!(names.contains(&"meta.intent"));
    assert!(names.contains(&"meta.render"));
    assert!(names.contains(&"meta.health"));
    assert!(names.contains(&"meta.deploy"));
}

#[test]
fn signal_schema_loads() {
    use biomeos_atomic_deploy::handlers::signal::load_signal_schema;

    let result = load_signal_schema(&graphs_dir());
    assert!(result.is_ok(), "signal_tools.toml should load: {result:?}");

    let schema = result.unwrap();
    let tools = schema.get("tools").expect("schema should have 'tools' key");
    let tools_arr = tools.as_array().expect("'tools' should be an array");
    assert_eq!(tools_arr.len(), 30, "Expected 30 tool definitions");
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
