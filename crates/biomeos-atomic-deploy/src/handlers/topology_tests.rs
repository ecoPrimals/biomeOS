// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::*;
use crate::handlers::graph::ExecutionStatus;
use crate::neural_router::NeuralRouter;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

fn make_handler(
    family_id: &str,
    router: Arc<NeuralRouter>,
    graphs_dir: impl Into<PathBuf>,
) -> TopologyHandler {
    let executions = Arc::new(RwLock::new(HashMap::new()));
    TopologyHandler::new(family_id, router, executions, graphs_dir)
}

#[tokio::test]
async fn test_topology_handler_creation() {
    let router = Arc::new(NeuralRouter::new("test-family"));
    let handler = make_handler("test-family", router, "/tmp");

    let result = handler
        .get_proprioception()
        .await
        .expect("get_proprioception");
    assert_eq!(result["family_id"], "test-family");
}

// =========================================================================
// Topology node/edge types and serialization
// =========================================================================

#[tokio::test]
async fn test_topology_get_response_structure() {
    let router = Arc::new(NeuralRouter::new("test-family"));
    let temp = tempfile::tempdir().expect("temp dir");
    let handler = make_handler("test-family", router, temp.path().to_path_buf());

    let result = handler.get().await.expect("topology.get");

    assert!(
        result.get("primals").is_some(),
        "Response must have primals"
    );
    assert!(
        result.get("connections").is_some(),
        "Response must have connections"
    );
    assert!(
        result.get("timestamp").is_some(),
        "Response must have timestamp"
    );

    let primals = result["primals"].as_array().expect("primals is array");
    let connections = result["connections"]
        .as_array()
        .expect("connections is array");

    for p in primals {
        assert!(p.get("id").is_some(), "Primal must have id");
        assert!(
            p.get("primal_type").is_some(),
            "Primal must have primal_type"
        );
        assert!(
            p.get("socket_path").is_some(),
            "Primal must have socket_path"
        );
        assert!(p.get("health").is_some(), "Primal must have health");
        assert!(
            p.get("capabilities").is_some(),
            "Primal must have capabilities"
        );
    }

    for c in connections {
        assert!(c.get("from").is_some(), "Connection must have from");
        assert!(c.get("to").is_some(), "Connection must have to");
        assert_eq!(
            c["connection_type"].as_str(),
            Some("security-provider"),
            "Connection type"
        );
    }
}

#[tokio::test]
async fn test_topology_get_with_registered_capabilities() {
    let router = Arc::new(NeuralRouter::new("test-family"));
    router
        .register_capability_unix(
            "security",
            "beardog",
            "/tmp/beardog-test-family.sock",
            "test",
        )
        .await
        .expect("register security");
    router
        .register_capability_unix(
            "discovery",
            "songbird",
            "/tmp/songbird-test-family.sock",
            "test",
        )
        .await
        .expect("register discovery");

    let temp = tempfile::tempdir().expect("temp dir");
    let handler = make_handler("test-family", router, temp.path().to_path_buf());

    let result = handler.get().await.expect("topology.get");
    let primals = result["primals"].as_array().expect("primals");
    let connections = result["connections"].as_array().expect("connections");

    assert!(
        primals.len() >= 2,
        "Should discover beardog and songbird from registry, got {}",
        primals.len()
    );

    let primal_ids: Vec<&str> = primals
        .iter()
        .map(|p| p["id"].as_str().unwrap_or(""))
        .collect();
    assert!(
        primal_ids.contains(&"beardog-test-family"),
        "Should have beardog, got {primal_ids:?}"
    );
    assert!(
        primal_ids.contains(&"songbird-test-family"),
        "Should have songbird, got {primal_ids:?}"
    );

    if !connections.is_empty() {
        let conn = &connections[0];
        assert_eq!(conn["connection_type"].as_str(), Some("security-provider"));
        assert!(conn["from"].as_str().is_some());
        assert!(conn["to"].as_str().is_some());
    }
}

// =========================================================================
// Topology construction and query logic
// =========================================================================

#[tokio::test]
async fn test_get_primals_response_format() {
    let router = Arc::new(NeuralRouter::new("test-family"));
    let temp = tempfile::tempdir().expect("temp dir");
    let handler = make_handler("test-family", router, temp.path().to_path_buf());

    let result = handler.get_primals().await.expect("get_primals");

    assert_eq!(result["family_id"], "test-family");
    assert!(result.get("timestamp").is_some());
    assert!(result.get("primals").is_some());
    assert!(result.get("count").is_some());

    let count = result["count"].as_u64().expect("count is number");
    let primals = result["primals"].as_array().expect("primals is array");
    assert_eq!(count as usize, primals.len());
}

#[tokio::test]
async fn test_get_proprioception_health_levels() {
    let router = Arc::new(NeuralRouter::new("test-family"));
    let temp = tempfile::tempdir().expect("temp dir");
    let handler = make_handler("test-family", router, temp.path().to_path_buf());

    let result = handler
        .get_proprioception()
        .await
        .expect("get_proprioception");

    assert!(result.get("health").is_some());
    let health = &result["health"];
    assert!(health.get("percentage").is_some());
    assert!(health.get("status").is_some());

    let status = health["status"].as_str().expect("status is string");
    assert!(
        ["healthy", "degraded", "critical"].contains(&status),
        "status must be healthy/degraded/critical, got {status}"
    );

    assert!(result.get("self_awareness").is_some());
    assert!(result.get("motor").is_some());
    assert!(result.get("sensory").is_some());
}

#[tokio::test]
async fn test_get_proprioception_with_full_capabilities() {
    let router = Arc::new(NeuralRouter::new("test-family"));
    router
        .register_capability_unix("security", "beardog", "/tmp/beardog-test.sock", "test")
        .await
        .expect("register");
    router
        .register_capability_unix("discovery", "songbird", "/tmp/songbird-test.sock", "test")
        .await
        .expect("register");
    router
        .register_capability_unix("compute", "toadstool", "/tmp/toadstool-test.sock", "test")
        .await
        .expect("register");

    let temp = tempfile::tempdir().expect("temp dir");
    let handler = make_handler("test-family", router, temp.path().to_path_buf());

    let result = handler
        .get_proprioception()
        .await
        .expect("get_proprioception");

    let sa = &result["self_awareness"];
    assert_eq!(sa["has_security"], true);
    assert_eq!(sa["has_discovery"], true);
    assert_eq!(sa["has_compute"], true);

    assert_eq!(result["health"]["percentage"], 100.0);
    assert_eq!(result["health"]["status"], "healthy");
    assert_eq!(result["confidence"], 100.0);
}

#[tokio::test]
async fn test_get_proprioception_capability_via_primal_type() {
    let router = Arc::new(NeuralRouter::new("test-family"));
    router
        .register_capability_unix("other", "beardog", "/tmp/beardog-test.sock", "test")
        .await
        .expect("register");

    let temp = tempfile::tempdir().expect("temp dir");
    let handler = make_handler("test-family", router, temp.path().to_path_buf());

    let result = handler
        .get_proprioception()
        .await
        .expect("get_proprioception");

    let sa = &result["self_awareness"];
    assert_eq!(
        sa["has_security"], true,
        "beardog provides security via taxonomy"
    );
}

#[tokio::test]
async fn test_get_metrics_response_structure() {
    let router = Arc::new(NeuralRouter::new("test-family"));
    let temp = tempfile::tempdir().expect("temp dir");
    let handler = make_handler("test-family", router, temp.path().to_path_buf());

    let result = handler.get_metrics().await.expect("get_metrics");

    assert!(result.get("timestamp").is_some());
    assert!(result.get("system").is_some());
    assert!(result.get("neural_api").is_some());

    let system = &result["system"];
    assert!(system.get("cpu_percent").is_some());
    assert!(system.get("memory_used_mb").is_some());
    assert!(system.get("memory_total_mb").is_some());
    assert!(system.get("memory_percent").is_some());
    assert!(system.get("uptime_seconds").is_some());

    let neural = &result["neural_api"];
    assert_eq!(neural["family_id"], "test-family");
    assert!(neural.get("active_primals").is_some());
    assert!(neural.get("graphs_available").is_some());
    assert!(neural.get("active_executions").is_some());
}

#[tokio::test]
async fn test_get_metrics_with_graphs_dir() {
    let router = Arc::new(NeuralRouter::new("test-family"));
    let temp = tempfile::tempdir().expect("temp dir");
    let graph_file = temp.path().join("test.toml");
    std::fs::write(
        &graph_file,
        r#"
[graph]
id = "test"
version = "1.0"
description = "Test"

[[nodes]]
id = "node1"
"#,
    )
    .expect("write graph");
    let handler = make_handler("test-family", router, temp.path().to_path_buf());

    let result = handler.get_metrics().await.expect("get_metrics");
    assert_eq!(result["neural_api"]["graphs_available"], 1);
}

#[tokio::test]
async fn test_get_metrics_with_nonexistent_graphs_dir() {
    let router = Arc::new(NeuralRouter::new("test-family"));
    let handler = make_handler("test-family", router, "/nonexistent/path/12345");

    let result = handler.get_metrics().await.expect("get_metrics");
    assert_eq!(result["neural_api"]["graphs_available"], 0);
}

#[tokio::test]
async fn test_get_metrics_active_executions_count() {
    let router = Arc::new(NeuralRouter::new("test-family"));
    let temp = tempfile::tempdir().expect("temp dir");
    let executions = Arc::new(RwLock::new(HashMap::from([(
        "exec-1".to_string(),
        ExecutionStatus {
            execution_id: "exec-1".to_string(),
            state: "running".to_string(),
            current_phase: Some(1),
            total_phases: 2,
            completed_nodes: vec![],
            failed_nodes: vec![],
            duration_ms: 100,
            error: None,
        },
    )])));
    let handler = TopologyHandler::new("test-family", router, executions, temp.path());

    let result = handler.get_metrics().await.expect("get_metrics");
    assert_eq!(result["neural_api"]["active_executions"], 1);
}

// =========================================================================
// Additional coverage (originally in topology_tests.rs sibling file)
// =========================================================================

#[test]
fn get_socket_directories_non_empty() {
    let dirs = TopologyHandler::get_socket_directories();
    assert!(
        !dirs.is_empty(),
        "expected at least legacy /tmp fallback or runtime path"
    );
}

#[tokio::test]
async fn get_primals_has_family_timestamp_and_count() {
    let router = Arc::new(NeuralRouter::new("topo-cov-fam"));
    let exec = Arc::new(RwLock::new(HashMap::<String, ExecutionStatus>::new()));
    let tmp = tempfile::tempdir().expect("tempdir");
    let h = TopologyHandler::new("topo-cov-fam", router, exec, tmp.path());
    let v = h.get_primals().await.expect("get_primals");
    assert_eq!(v["family_id"], "topo-cov-fam");
    assert!(v["timestamp"].as_str().is_some());
    assert_eq!(v["count"], v["primals"].as_array().map_or(0, |a| a.len()));
}

#[tokio::test]
async fn get_topology_timestamp_rfc3339() {
    let router = Arc::new(NeuralRouter::new("topo-cov-fam2"));
    let exec = Arc::new(RwLock::new(HashMap::new()));
    let tmp = tempfile::tempdir().expect("tempdir");
    let h = TopologyHandler::new("topo-cov-fam2", router, exec, tmp.path());
    let v = h.get().await.expect("get");
    let ts = v["timestamp"].as_str().expect("timestamp");
    assert!(ts.contains('T') || ts.contains('t'), "rfc3339-ish: {ts}");
}

#[tokio::test]
async fn get_metrics_has_neural_api_block() {
    let router = Arc::new(NeuralRouter::new("topo-cov-fam3"));
    let exec = Arc::new(RwLock::new(HashMap::new()));
    let tmp = tempfile::tempdir().expect("tempdir");
    let h = TopologyHandler::new("topo-cov-fam3", router, exec, tmp.path());
    let v = h.get_metrics().await.expect("metrics");
    assert!(v.get("system").is_some());
    assert_eq!(v["neural_api"]["family_id"], "topo-cov-fam3");
    assert!(v["neural_api"].get("active_primals").is_some());
}

#[tokio::test]
async fn get_proprioception_status_bucket() {
    let router = Arc::new(NeuralRouter::new("topo-cov-fam4"));
    let exec = Arc::new(RwLock::new(HashMap::new()));
    let tmp = tempfile::tempdir().expect("tempdir");
    let h = TopologyHandler::new("topo-cov-fam4", router, exec, tmp.path());
    let v = h.get_proprioception().await.expect("proprio");
    let status = v["health"]["status"].as_str().unwrap();
    assert!(
        matches!(status, "healthy" | "degraded" | "critical"),
        "{status}"
    );
}

#[tokio::test]
async fn handler_new_stores_family_id() {
    let router = Arc::new(NeuralRouter::new("fid"));
    let exec = Arc::new(RwLock::new(HashMap::new()));
    let h = TopologyHandler::new("my-family-id", router, exec, "/tmp");
    let v = h.get_proprioception().await.expect("p");
    assert_eq!(v["family_id"], "my-family-id");
}

#[tokio::test]
async fn get_topology_has_primals_and_connections_arrays() {
    let router = Arc::new(NeuralRouter::new("topo-struct"));
    let exec = Arc::new(RwLock::new(HashMap::new()));
    let tmp = tempfile::tempdir().expect("tempdir");
    let h = TopologyHandler::new("topo-struct", router, exec, tmp.path());
    let v = h.get().await.expect("get");
    assert!(v["primals"].is_array());
    assert!(v["connections"].is_array());
}

#[tokio::test]
async fn get_proprioception_empty_primals_yields_low_health() {
    let router = Arc::new(NeuralRouter::new("empty-proprio"));
    let exec = Arc::new(RwLock::new(HashMap::new()));
    let tmp = tempfile::tempdir().expect("tempdir");
    let h = TopologyHandler::new("empty-proprio", router, exec, tmp.path());
    let v = h.get_proprioception().await.expect("proprio");
    let pct = v["health"]["percentage"].as_f64().unwrap();
    assert!(
        pct < 80.0,
        "expected sub-healthy without capabilities, got {pct}"
    );
}

#[tokio::test]
async fn get_metrics_timestamp_and_system_keys() {
    let router = Arc::new(NeuralRouter::new("metrics-keys"));
    let exec = Arc::new(RwLock::new(HashMap::new()));
    let tmp = tempfile::tempdir().expect("tempdir");
    let h = TopologyHandler::new("metrics-keys", router, exec, tmp.path());
    let v = h.get_metrics().await.expect("metrics");
    assert!(v["timestamp"].as_str().is_some());
    let sys = v["system"].as_object().unwrap();
    assert!(sys.contains_key("cpu_percent"));
    assert!(sys.contains_key("uptime_seconds"));
}

#[tokio::test]
async fn get_primals_count_matches_array_len_with_registry() {
    let router = Arc::new(NeuralRouter::new("cnt-fam"));
    router
        .register_capability_unix("compute", "toad", "/tmp/toad-cnt.sock", "test")
        .await
        .expect("register");
    let exec = Arc::new(RwLock::new(HashMap::new()));
    let tmp = tempfile::tempdir().expect("tempdir");
    let h = TopologyHandler::new("cnt-fam", router, exec, tmp.path());
    let v = h.get_primals().await.expect("primals");
    let arr = v["primals"].as_array().unwrap();
    assert_eq!(v["count"], arr.len());
}

#[test]
fn socket_directories_non_empty_and_biomeos_or_legacy_tmp() {
    let dirs = TopologyHandler::get_socket_directories();
    assert!(
        !dirs.is_empty(),
        "expected at least one socket directory: {dirs:?}"
    );
    assert!(
        dirs.iter().any(|p| p == std::path::Path::new("/tmp"))
            || dirs.iter().any(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n == "biomeos")
            }),
        "expected /tmp fallback or a biomeos runtime dir, got {dirs:?}"
    );
}

#[tokio::test]
async fn proprioception_two_of_three_capabilities_is_degraded() {
    let router = Arc::new(NeuralRouter::new("deg-fam"));
    router
        .register_capability_unix("security", "a", "/tmp/a-deg-fam.sock", "t")
        .await
        .expect("reg");
    router
        .register_capability_unix("discovery", "b", "/tmp/b-deg-fam.sock", "t")
        .await
        .expect("reg");
    let exec = Arc::new(RwLock::new(HashMap::new()));
    let tmp = tempfile::tempdir().expect("tempdir");
    let h = TopologyHandler::new("deg-fam", router, exec, tmp.path());
    let v = h.get_proprioception().await.expect("proprio");
    assert_eq!(v["health"]["status"], "degraded");
    let pct = v["health"]["percentage"].as_f64().expect("pct");
    assert!(pct > 50.0 && pct < 80.0, "expected ~66%, got {pct}");
}

#[tokio::test]
async fn topology_connections_empty_without_discovery_and_security_pair() {
    let router = Arc::new(NeuralRouter::new("solo-fam"));
    router
        .register_capability_unix("security", "solo", "/tmp/solo-solo-fam.sock", "t")
        .await
        .expect("reg");
    let exec = Arc::new(RwLock::new(HashMap::new()));
    let tmp = tempfile::tempdir().expect("tempdir");
    let h = TopologyHandler::new("solo-fam", router, exec, tmp.path());
    let top = h.get().await.expect("get");
    let conns = top["connections"].as_array().expect("connections");
    assert!(
        conns.is_empty(),
        "infer_connections needs discovery + security pair, got {conns:?}"
    );
}

#[tokio::test]
async fn get_primals_includes_registry_entries_with_expected_ids() {
    let router = Arc::new(NeuralRouter::new("id-fam"));
    router
        .register_capability_unix("compute", "toad", "/tmp/toad-id-fam.sock", "t")
        .await
        .expect("reg");
    let exec = Arc::new(RwLock::new(HashMap::new()));
    let tmp = tempfile::tempdir().expect("tempdir");
    let h = TopologyHandler::new("id-fam", router, exec, tmp.path());
    let v = h.get_primals().await.expect("primals");
    let arr = v["primals"].as_array().expect("arr");
    assert!(
        arr.iter().any(|p| p["id"] == "toad-id-fam"),
        "expected registry primal id, got {arr:?}"
    );
}

#[tokio::test]
async fn motor_coordination_requires_two_or_more_primals() {
    let router = Arc::new(NeuralRouter::new("motor-fam"));
    router
        .register_capability_unix("x", "p1", "/tmp/p1-motor-fam.sock", "t")
        .await
        .expect("r1");
    let exec = Arc::new(RwLock::new(HashMap::new()));
    let tmp = tempfile::tempdir().expect("tempdir");
    let h = TopologyHandler::new("motor-fam", router.clone(), exec, tmp.path());
    let v = h.get_proprioception().await.expect("p");
    assert_eq!(v["motor"]["can_coordinate_primals"], false);

    router
        .register_capability_unix("y", "p2", "/tmp/p2-motor-fam.sock", "t")
        .await
        .expect("r2");
    let exec2 = Arc::new(RwLock::new(HashMap::new()));
    let h2 = TopologyHandler::new("motor-fam", router, exec2, tmp.path());
    let v2 = h2.get_proprioception().await.expect("p2");
    assert_eq!(v2["motor"]["can_coordinate_primals"], true);
}
