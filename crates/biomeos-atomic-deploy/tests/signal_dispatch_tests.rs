// SPDX-License-Identifier: AGPL-3.0-or-later
//
//! Signal dispatch integration tests.
//!
//! Validates that the composition collapse layer correctly maps atomic
//! signals to graph paths, loads all 16 signal graphs, and intercepts
//! signal-tier capability calls.

use std::path::PathBuf;

fn graphs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../graphs")
}

#[test]
fn all_16_signal_graphs_exist() {
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
fn list_signal_graphs_finds_all_16() {
    use biomeos_atomic_deploy::handlers::signal::list_signal_graphs;

    let signals = list_signal_graphs(&graphs_dir());
    assert_eq!(
        signals.len(),
        16,
        "Expected 16 signal graphs, found {}",
        signals.len()
    );

    let names: Vec<&str> = signals.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"tower.publish"));
    assert!(names.contains(&"nest.store"));
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
    assert_eq!(tools_arr.len(), 16, "Expected 16 tool definitions");
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
