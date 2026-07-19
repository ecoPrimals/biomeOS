// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::super::*;
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
    let exec = Arc::new(RwLock::new(HashMap::<
        String,
        crate::handlers::graph::ExecutionStatus,
    >::new()));
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
