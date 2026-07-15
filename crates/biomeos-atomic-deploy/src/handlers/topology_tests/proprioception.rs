// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::helpers::make_handler;
use super::super::super::*;
use crate::neural_router::NeuralRouter;
use std::sync::Arc;

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
async fn get_proprioception_status_bucket() {
    let router = Arc::new(NeuralRouter::new("topo-cov-fam4"));
    let exec = Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new()));
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
async fn get_proprioception_empty_primals_yields_low_health() {
    let router = Arc::new(NeuralRouter::new("empty-proprio"));
    let exec = Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new()));
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
    let exec = Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new()));
    let tmp = tempfile::tempdir().expect("tempdir");
    let h = TopologyHandler::new("deg-fam", router, exec, tmp.path());
    let v = h.get_proprioception().await.expect("proprio");
    assert_eq!(v["health"]["status"], "degraded");
    let pct = v["health"]["percentage"].as_f64().expect("pct");
    assert!(pct > 50.0 && pct < 80.0, "expected ~66%, got {pct}");
}

#[tokio::test]
async fn motor_coordination_requires_two_or_more_primals() {
    let router = Arc::new(NeuralRouter::new("motor-fam"));
    router
        .register_capability_unix("x", "p1", "/tmp/p1-motor-fam.sock", "t")
        .await
        .expect("r1");
    let exec = Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new()));
    let tmp = tempfile::tempdir().expect("tempdir");
    let h = TopologyHandler::new("motor-fam", router.clone(), exec, tmp.path());
    let v = h.get_proprioception().await.expect("p");
    assert_eq!(v["motor"]["can_coordinate_primals"], false);

    router
        .register_capability_unix("y", "p2", "/tmp/p2-motor-fam.sock", "t")
        .await
        .expect("r2");
    let exec2 = Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new()));
    let h2 = TopologyHandler::new("motor-fam", router, exec2, tmp.path());
    let v2 = h2.get_proprioception().await.expect("p2");
    assert_eq!(v2["motor"]["can_coordinate_primals"], true);
}
