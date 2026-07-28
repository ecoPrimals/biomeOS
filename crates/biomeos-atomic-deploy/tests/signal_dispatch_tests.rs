// SPDX-License-Identifier: AGPL-3.0-or-later
//
//! Signal dispatch integration tests.
//!
//! Validates that the composition collapse layer correctly maps atomic
//! signals to graph paths, loads all 26 signal graphs, and intercepts
//! signal-tier capability calls.

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use std::path::PathBuf;

fn graphs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../graphs")
}

#[test]
fn all_26_signal_graphs_exist() {
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
        // Nest (8)
        "nest_store",
        "nest_commit",
        "nest_retrieve",
        "nest_sync",
        "nest_ingest_spore",
        "nest_emit_spore",
        "nest_verify",
        "nest_federate",
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
fn list_signal_graphs_finds_all_26() {
    use biomeos_atomic_deploy::handlers::signal::list_signal_graphs;

    let signals = list_signal_graphs(&graphs_dir());
    assert_eq!(
        signals.len(),
        26,
        "Expected 26 signal graphs, found {}",
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
    // Nest (8)
    assert!(names.contains(&"nest.store"));
    assert!(names.contains(&"nest.commit"));
    assert!(names.contains(&"nest.retrieve"));
    assert!(names.contains(&"nest.sync"));
    assert!(names.contains(&"nest.ingest_spore"));
    assert!(names.contains(&"nest.emit_spore"));
    assert!(names.contains(&"nest.verify"));
    assert!(names.contains(&"nest.federate"));
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
    assert_eq!(tools_arr.len(), 26, "Expected 26 tool definitions");
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
    for signal in ["store", "commit", "retrieve", "sync", "ingest_spore", "emit_spore", "verify", "federate"] {
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
    for signal in ["publish", "authenticate", "discover", "health", "bootstrap", "enroll", "key_rotate", "mesh_status"] {
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

// ─── Tower Live Validation ─────────────────────────────────────────────
//
// These tests validate that tower.health and tower.mesh_status are
// structurally correct for live dispatch to online gates. They verify
// topology, coordination mode, security posture, and primal coverage.

#[test]
fn tower_health_graph_validates_for_live_dispatch() {
    let path = graphs_dir().join("signals/tower_health.toml");
    let content = std::fs::read_to_string(&path).expect("read tower_health.toml");
    let parsed: toml::Value = toml::from_str(&content).expect("parse tower_health.toml");

    let graph = parsed.get("graph").expect("missing [graph]");
    assert_eq!(graph["id"].as_str(), Some("tower_health"));
    assert_eq!(graph["signal_tier"].as_str(), Some("tower"));
    assert_eq!(graph["signal_name"].as_str(), Some("health"));
    assert_eq!(
        graph["coordination"].as_str(),
        Some("parallel"),
        "tower.health must be parallel — all 3 primals checked simultaneously"
    );

    let metadata = graph.get("metadata").expect("missing [graph.metadata]");
    assert_eq!(
        metadata["security_model"].as_str(),
        Some("btsp_enforced"),
        "tower.health requires BTSP — live gate communication must be authenticated"
    );
    assert_eq!(
        metadata["transport"].as_str(),
        Some("platform_native"),
        "tower.health uses platform_native transport (UDS on Linux, named pipes on Windows)"
    );
    assert_eq!(
        metadata["particle"].as_str(),
        Some("electron"),
        "tower signals use electron particle type"
    );

    let nodes = graph["nodes"].as_array().expect("nodes array");
    assert_eq!(
        nodes.len(),
        3,
        "tower.health: bearDog + songBird + skunkBat"
    );

    let node_names: Vec<&str> = nodes
        .iter()
        .map(|n| n["name"].as_str().unwrap_or(""))
        .collect();
    assert!(node_names.contains(&"beardog_health"));
    assert!(node_names.contains(&"songbird_health"));
    assert!(node_names.contains(&"skunkbat_health"));

    let binaries: Vec<&str> = nodes
        .iter()
        .map(|n| n["binary"].as_str().unwrap_or(""))
        .collect();
    assert!(binaries.contains(&"beardog"));
    assert!(binaries.contains(&"songbird"));
    assert!(binaries.contains(&"skunkbat"));

    // All nodes parallel (order=1), all required, none spawned
    for node in nodes {
        assert_eq!(
            node["order"].as_integer(),
            Some(1),
            "all tower.health nodes execute in parallel (order=1)"
        );
        assert_eq!(
            node["required"].as_bool(),
            Some(true),
            "all Tower primals are required for health — degraded Tower is unhealthy"
        );
        assert_eq!(
            node["spawn"].as_bool(),
            Some(false),
            "tower.health connects to existing primals, never spawns"
        );
        assert!(
            node.get("capabilities").is_some(),
            "each node must declare capabilities for capability-based routing"
        );
    }

    // Verify capability-based routing (not hardcoded endpoints)
    let beardog = nodes.iter().find(|n| n["name"].as_str() == Some("beardog_health")).unwrap();
    assert_eq!(beardog["by_capability"].as_str(), Some("security"));

    let songbird = nodes.iter().find(|n| n["name"].as_str() == Some("songbird_health")).unwrap();
    assert_eq!(songbird["by_capability"].as_str(), Some("discovery"));

    let skunkbat = nodes.iter().find(|n| n["name"].as_str() == Some("skunkbat_health")).unwrap();
    assert_eq!(skunkbat["by_capability"].as_str(), Some("defense"));
}

#[test]
fn tower_mesh_status_graph_validates_for_live_dispatch() {
    let path = graphs_dir().join("signals/tower_mesh_status.toml");
    let content = std::fs::read_to_string(&path).expect("read tower_mesh_status.toml");
    let parsed: toml::Value = toml::from_str(&content).expect("parse tower_mesh_status.toml");

    let graph = parsed.get("graph").expect("missing [graph]");
    assert_eq!(graph["id"].as_str(), Some("tower_mesh_status"));
    assert_eq!(graph["signal_tier"].as_str(), Some("tower"));
    assert_eq!(graph["signal_name"].as_str(), Some("mesh_status"));
    assert_eq!(
        graph["coordination"].as_str(),
        Some("parallel"),
        "tower.mesh_status must be parallel — each primal reports independently"
    );

    let metadata = graph.get("metadata").expect("missing [graph.metadata]");
    assert_eq!(
        metadata["security_model"].as_str(),
        Some("btsp_enforced"),
        "tower.mesh_status requires authenticated communication"
    );
    assert_eq!(
        metadata["transport"].as_str(),
        Some("platform_native"),
        "tower.mesh_status uses platform_native transport"
    );
    assert_eq!(metadata["particle"].as_str(), Some("electron"));

    let nodes = graph["nodes"].as_array().expect("nodes array");
    assert_eq!(
        nodes.len(),
        3,
        "tower.mesh_status: mesh_peers + crypto_health + threat_posture"
    );

    let node_names: Vec<&str> = nodes
        .iter()
        .map(|n| n["name"].as_str().unwrap_or(""))
        .collect();
    assert!(node_names.contains(&"mesh_peers"));
    assert!(node_names.contains(&"crypto_health"));
    assert!(node_names.contains(&"threat_posture"));

    // mesh_peers: songBird reports WireGuard mesh state
    let mesh = nodes.iter().find(|n| n["name"].as_str() == Some("mesh_peers")).unwrap();
    assert_eq!(mesh["binary"].as_str(), Some("songbird"));
    assert_eq!(mesh["by_capability"].as_str(), Some("discovery"));
    let mesh_caps = mesh["capabilities"].as_array().expect("mesh capabilities");
    let mesh_cap_strs: Vec<&str> = mesh_caps.iter().filter_map(|c| c.as_str()).collect();
    assert!(mesh_cap_strs.contains(&"mesh.peers"));
    assert!(mesh_cap_strs.contains(&"mesh.health_check"));
    assert_eq!(mesh["required"].as_bool(), Some(true));

    // crypto_health: bearDog reports key material + BTSP spine status
    let crypto = nodes.iter().find(|n| n["name"].as_str() == Some("crypto_health")).unwrap();
    assert_eq!(crypto["binary"].as_str(), Some("beardog"));
    assert_eq!(crypto["by_capability"].as_str(), Some("security"));
    let crypto_caps = crypto["capabilities"].as_array().expect("crypto capabilities");
    let crypto_cap_strs: Vec<&str> = crypto_caps.iter().filter_map(|c| c.as_str()).collect();
    assert!(crypto_cap_strs.contains(&"health.liveness"));
    assert!(crypto_cap_strs.contains(&"crypto.status"));
    assert_eq!(crypto["required"].as_bool(), Some(true));

    // threat_posture: skunkBat is optional — mesh_status degrades gracefully
    let threat = nodes.iter().find(|n| n["name"].as_str() == Some("threat_posture")).unwrap();
    assert_eq!(threat["binary"].as_str(), Some("skunkbat"));
    assert_eq!(threat["by_capability"].as_str(), Some("defense"));
    assert_eq!(
        threat["required"].as_bool(),
        Some(false),
        "threat_posture is optional — mesh_status reports without it"
    );

    // All parallel execution
    for node in nodes {
        assert_eq!(
            node["order"].as_integer(),
            Some(1),
            "all tower.mesh_status nodes execute in parallel"
        );
        assert_eq!(
            node["spawn"].as_bool(),
            Some(false),
            "tower.mesh_status connects to running primals"
        );
    }
}

#[test]
fn tower_enroll_graph_has_sequential_pipeline() {
    let path = graphs_dir().join("signals/tower_enroll.toml");
    let content = std::fs::read_to_string(&path).expect("read tower_enroll.toml");
    let parsed: toml::Value = toml::from_str(&content).expect("parse tower_enroll.toml");

    let graph = parsed.get("graph").expect("missing [graph]");
    assert_eq!(graph["signal_tier"].as_str(), Some("tower"));
    assert_eq!(graph["signal_name"].as_str(), Some("enroll"));
    assert_eq!(
        graph["coordination"].as_str(),
        Some("sequential"),
        "enrollment must be sequential: verify → register → audit"
    );

    let nodes = graph["nodes"].as_array().expect("nodes array");
    assert_eq!(nodes.len(), 3);

    let orders: Vec<i64> = nodes
        .iter()
        .filter_map(|n| n["order"].as_integer())
        .collect();
    assert_eq!(orders, [1, 2, 3], "strictly sequential ordering");

    let node_names: Vec<&str> = nodes
        .iter()
        .map(|n| n["name"].as_str().unwrap_or(""))
        .collect();
    assert_eq!(
        node_names,
        ["verify_proof", "register_peer", "audit_enrollment"],
        "enrollment pipeline: auth → mesh → audit"
    );

    // Dependency chain
    let register = &nodes[1];
    let deps = register["depends_on"].as_array().expect("register depends_on");
    assert!(deps.iter().any(|d| d.as_str() == Some("verify_proof")));

    let audit = &nodes[2];
    let deps = audit["depends_on"].as_array().expect("audit depends_on");
    assert!(deps.iter().any(|d| d.as_str() == Some("register_peer")));
}

#[test]
fn tower_signals_schema_matches_graphs() {
    use biomeos_atomic_deploy::handlers::signal::load_signal_schema;

    let dir = graphs_dir();
    let schema = load_signal_schema(&dir).expect("load signal_tools.toml");
    let tools = schema["tools"].as_array().expect("tools array");

    let tower_tools: Vec<&serde_json::Value> = tools
        .iter()
        .filter(|t| t["tier"].as_str() == Some("tower"))
        .collect();

    assert_eq!(
        tower_tools.len(),
        8,
        "signal_tools.toml should define all 8 Tower signals"
    );

    let tower_names: Vec<&str> = tower_tools
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    assert!(tower_names.contains(&"tower.health"));
    assert!(tower_names.contains(&"tower.mesh_status"));
    assert!(tower_names.contains(&"tower.enroll"));
    assert!(tower_names.contains(&"tower.key_rotate"));
    assert!(tower_names.contains(&"tower.publish"));
    assert!(tower_names.contains(&"tower.authenticate"));
    assert!(tower_names.contains(&"tower.discover"));
    assert!(tower_names.contains(&"tower.bootstrap"));

    // Verify tower.health and tower.mesh_status have parallel coordination
    for name in ["tower.health", "tower.mesh_status"] {
        let tool = tower_tools
            .iter()
            .find(|t| t["name"].as_str() == Some(name))
            .unwrap_or_else(|| panic!("missing {name} in schema"));
        assert_eq!(
            tool["coordination"].as_str(),
            Some("parallel"),
            "{name} must be parallel in schema"
        );
    }
}

#[test]
fn all_signal_graphs_have_consistent_tier_distribution() {
    use biomeos_atomic_deploy::handlers::signal::list_signal_graphs;

    let signals = list_signal_graphs(&graphs_dir());

    let tower_count = signals.iter().filter(|s| s.tier == "tower").count();
    let node_count = signals.iter().filter(|s| s.tier == "node").count();
    let nest_count = signals.iter().filter(|s| s.tier == "nest").count();
    let meta_count = signals.iter().filter(|s| s.tier == "meta").count();
    let braid_count = signals.iter().filter(|s| s.tier == "braid").count();

    assert_eq!(tower_count, 8, "Tower: 8 signals (publish, authenticate, discover, health, bootstrap, enroll, key_rotate, mesh_status)");
    assert_eq!(node_count, 3, "Node: 3 signals (compute, discover_hardware, dispatch)");
    assert_eq!(nest_count, 8, "Nest: 8 signals (store, commit, retrieve, sync, ingest_spore, emit_spore, verify, federate)");
    assert_eq!(meta_count, 5, "Meta: 5 signals (observe, intent, render, health, deploy)");
    assert_eq!(braid_count, 2, "Braid: 2 signals (partial_update, complete)");
    assert_eq!(
        tower_count + node_count + nest_count + meta_count + braid_count,
        26,
        "Total signal count"
    );
}
