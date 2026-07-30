// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tower live validation: tower.health, tower.mesh_status, tower.enroll
//! topology + schema matching + tier distribution.

use super::graphs_dir;

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

    let beardog = nodes
        .iter()
        .find(|n| n["name"].as_str() == Some("beardog_health"))
        .unwrap();
    assert_eq!(beardog["by_capability"].as_str(), Some("security"));

    let songbird = nodes
        .iter()
        .find(|n| n["name"].as_str() == Some("songbird_health"))
        .unwrap();
    assert_eq!(songbird["by_capability"].as_str(), Some("discovery"));

    let skunkbat = nodes
        .iter()
        .find(|n| n["name"].as_str() == Some("skunkbat_health"))
        .unwrap();
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

    let mesh = nodes
        .iter()
        .find(|n| n["name"].as_str() == Some("mesh_peers"))
        .unwrap();
    assert_eq!(mesh["binary"].as_str(), Some("songbird"));
    assert_eq!(mesh["by_capability"].as_str(), Some("discovery"));
    let mesh_caps = mesh["capabilities"].as_array().expect("mesh capabilities");
    let mesh_cap_strs: Vec<&str> = mesh_caps.iter().filter_map(|c| c.as_str()).collect();
    assert!(mesh_cap_strs.contains(&"mesh.peers"));
    assert!(mesh_cap_strs.contains(&"mesh.health_check"));
    assert_eq!(mesh["required"].as_bool(), Some(true));

    let crypto = nodes
        .iter()
        .find(|n| n["name"].as_str() == Some("crypto_health"))
        .unwrap();
    assert_eq!(crypto["binary"].as_str(), Some("beardog"));
    assert_eq!(crypto["by_capability"].as_str(), Some("security"));
    let crypto_caps = crypto["capabilities"]
        .as_array()
        .expect("crypto capabilities");
    let crypto_cap_strs: Vec<&str> = crypto_caps.iter().filter_map(|c| c.as_str()).collect();
    assert!(crypto_cap_strs.contains(&"health.liveness"));
    assert!(crypto_cap_strs.contains(&"crypto.status"));
    assert_eq!(crypto["required"].as_bool(), Some(true));

    let threat = nodes
        .iter()
        .find(|n| n["name"].as_str() == Some("threat_posture"))
        .unwrap();
    assert_eq!(threat["binary"].as_str(), Some("skunkbat"));
    assert_eq!(threat["by_capability"].as_str(), Some("defense"));
    assert_eq!(
        threat["required"].as_bool(),
        Some(false),
        "threat_posture is optional — mesh_status reports without it"
    );

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

    let register = &nodes[1];
    let deps = register["depends_on"]
        .as_array()
        .expect("register depends_on");
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

    assert_eq!(tower_count, 8, "Tower: 8 signals");
    assert_eq!(node_count, 3, "Node: 3 signals");
    assert_eq!(nest_count, 9, "Nest: 9 signals");
    assert_eq!(meta_count, 5, "Meta: 5 signals");
    assert_eq!(braid_count, 2, "Braid: 2 signals");
    assert_eq!(
        tower_count + node_count + nest_count + meta_count + braid_count,
        27,
        "Total signal count"
    );
}
