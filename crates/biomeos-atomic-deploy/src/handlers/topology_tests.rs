// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Additional topology handler coverage (sibling to `topology.rs`).

#![expect(clippy::unwrap_used, reason = "test")]

use super::topology::TopologyHandler;
use crate::handlers::graph::ExecutionStatus;
use crate::neural_router::NeuralRouter;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

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
