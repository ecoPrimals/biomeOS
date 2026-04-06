// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

use std::path::PathBuf;

use super::*;
use tempfile::TempDir;

// ═══════════════════════════════════════════════════════════════
// Helper
// ═══════════════════════════════════════════════════════════════

fn make_instance(name: &str, pid: u32) -> PrimalInstance {
    PrimalInstance {
        primal_name: name.to_string(),
        pid,
        socket_path: PathBuf::from(format!("/tmp/{name}.sock")),
        started_at: chrono::Utc::now(),
        child: None,
    }
}

// ═══════════════════════════════════════════════════════════════
// AtomicType tests
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_atomic_type_node_id() {
    assert_eq!(AtomicType::Tower.node_id(), "tower");
    assert_eq!(AtomicType::Node.node_id(), "node");
    assert_eq!(AtomicType::Nest.node_id(), "nest");
}

#[test]
fn test_atomic_type_required_primals() {
    assert_eq!(
        AtomicType::Tower.required_primals(),
        vec!["beardog-server", "songbird-orchestrator"]
    );
    assert_eq!(
        AtomicType::Node.required_primals(),
        vec!["beardog-server", "songbird-orchestrator", "toadstool"]
    );
    assert_eq!(
        AtomicType::Nest.required_primals(),
        vec!["beardog-server", "songbird-orchestrator", "nestgate"]
    );
}

#[test]
fn test_atomic_type_all_share_beardog_and_songbird() {
    // Verify every atomic type requires the base primals
    for atomic in [AtomicType::Tower, AtomicType::Node, AtomicType::Nest] {
        let primals = atomic.required_primals();
        assert!(
            primals.contains(&"beardog-server"),
            "{atomic:?} must require beardog-server"
        );
        assert!(
            primals.contains(&"songbird-orchestrator"),
            "{atomic:?} must require songbird-orchestrator"
        );
    }
}

#[test]
fn test_atomic_type_serialization_all_variants() {
    for (at, expected) in [
        (AtomicType::Tower, "\"Tower\""),
        (AtomicType::Node, "\"Node\""),
        (AtomicType::Nest, "\"Nest\""),
    ] {
        let json = serde_json::to_string(&at).expect("serialize");
        assert_eq!(json, expected);

        let restored: AtomicType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(at, restored);
    }
}

#[test]
fn test_atomic_type_equality_and_copy() {
    let a = AtomicType::Tower;
    let b = a; // Copy
    assert_eq!(a, b);

    let c = a; // Copy (AtomicType is Copy)
    assert_eq!(a, c);
}

#[test]
fn test_atomic_type_debug() {
    let dbg = format!("{:?}", AtomicType::Tower);
    assert_eq!(dbg, "Tower");
}

// ═══════════════════════════════════════════════════════════════
// DeploymentConfig tests
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_deployment_config_creation() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let seed_path = temp_dir.path().join("test.seed");

    let config = DeploymentConfig::test_config(seed_path.clone());

    assert_eq!(config.usb_seed_path, seed_path);
    assert_eq!(config.family_id, "1894e909e454");
    assert!(!config.neural_api_enabled);
    assert!(config.neural_api_endpoint.is_none());
    assert!(!config.deployment_batch.is_empty());
}

#[test]
fn test_deployment_config_serialization_roundtrip() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let config = DeploymentConfig::test_config(temp_dir.path().join("test.seed"));

    let json = serde_json::to_string(&config).expect("serialize");
    let deserialized: DeploymentConfig = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(config.family_id, deserialized.family_id);
    assert_eq!(config.deployment_batch, deserialized.deployment_batch);
    assert_eq!(config.neural_api_enabled, deserialized.neural_api_enabled);
    assert_eq!(config.neural_api_endpoint, deserialized.neural_api_endpoint);
    assert_eq!(config.usb_seed_path, deserialized.usb_seed_path);
}

#[test]
fn test_deployment_config_clone() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let config = DeploymentConfig::test_config(temp_dir.path().join("s.seed"));

    let cloned = config.clone();
    assert_eq!(config.family_id, cloned.family_id);
    assert_eq!(config.binary_dir, cloned.binary_dir);
}

#[test]
fn test_deployment_config_debug() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let config = DeploymentConfig::test_config(temp_dir.path().join("s.seed"));

    let dbg = format!("{config:?}");
    assert!(dbg.contains("DeploymentConfig"));
    assert!(dbg.contains("1894e909e454"));
}

#[test]
fn test_deployment_config_with_neural_api() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let mut config = DeploymentConfig::test_config(temp_dir.path().join("s.seed"));

    config.neural_api_enabled = true;
    config.neural_api_endpoint = Some("http://127.0.0.1:9000".into());

    let json = serde_json::to_string(&config).expect("serialize");
    assert!(json.contains("neural_api_enabled"));
    assert!(json.contains("127.0.0.1:9000"));

    let restored: DeploymentConfig = serde_json::from_str(&json).expect("deserialize");
    assert!(restored.neural_api_enabled);
    assert_eq!(
        restored.neural_api_endpoint.as_deref(),
        Some("http://127.0.0.1:9000")
    );
}

// ═══════════════════════════════════════════════════════════════
// DeploymentResult tests
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_deployment_result_new() {
    let result = DeploymentResult::new();
    assert!(result.tower.is_none());
    assert!(result.node.is_none());
    assert!(result.nest.is_none());
    assert_eq!(result.success_count, 0);
    assert!(result.errors.is_empty());
}

#[test]
fn test_deployment_result_is_success() {
    let mut result = DeploymentResult::new();
    assert!(!result.is_success());

    result.success_count = 2;
    assert!(!result.is_success()); // Need all 3

    result.success_count = 3;
    assert!(result.is_success());

    result.errors.push("test error".to_string());
    assert!(!result.is_success()); // Errors mean failure
}

#[test]
fn test_deployment_result_all_instances_empty() {
    let result = DeploymentResult::new();
    assert_eq!(result.all_instances().len(), 0);
}

#[test]
fn test_deployment_result_all_instances_tower_only() {
    let mut result = DeploymentResult::new();
    result.tower = Some(vec![make_instance("beardog-server", 1000)]);

    let instances = result.all_instances();
    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0].primal_name, "beardog-server");
}

#[test]
fn test_deployment_result_all_instances_full() {
    let mut result = DeploymentResult::new();
    result.tower = Some(vec![
        make_instance("beardog-server", 1000),
        make_instance("songbird-orchestrator", 1001),
    ]);
    result.node = Some(vec![
        make_instance("beardog-server", 2000),
        make_instance("songbird-orchestrator", 2001),
        make_instance("toadstool", 2002),
    ]);
    result.nest = Some(vec![
        make_instance("beardog-server", 3000),
        make_instance("songbird-orchestrator", 3001),
        make_instance("nestgate", 3002),
    ]);

    let instances = result.all_instances();
    assert_eq!(instances.len(), 8); // 2 + 3 + 3
}

#[test]
fn test_deployment_result_serialization_roundtrip() {
    let mut result = DeploymentResult::new();
    result.success_count = 2;
    result.errors.push("node failed".into());
    result.tower = Some(vec![make_instance("beardog", 100)]);

    let json = serde_json::to_string(&result).expect("serialize");
    let restored: DeploymentResult = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(restored.success_count, 2);
    assert_eq!(restored.errors.len(), 1);
    assert_eq!(restored.errors[0], "node failed");
    assert!(restored.tower.is_some());
    assert!(restored.node.is_none());
}

#[test]
fn test_deployment_result_clone() {
    let mut result = DeploymentResult::new();
    result.success_count = 1;
    result.tower = Some(vec![make_instance("beardog", 42)]);

    let cloned = result.clone();
    assert_eq!(cloned.success_count, 1);
    assert!(cloned.tower.is_some());
}

#[test]
fn test_deployment_result_debug() {
    let result = DeploymentResult::new();
    let dbg = format!("{result:?}");
    assert!(dbg.contains("DeploymentResult"));
}

// ═══════════════════════════════════════════════════════════════
// DeploymentOrchestrator tests
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_orchestrator_new_with_valid_dirs() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let binary_dir = temp_dir.path().join("bin");
    std::fs::create_dir(&binary_dir).expect("create binary dir");

    let mut config = DeploymentConfig::test_config(temp_dir.path().join("test.seed"));
    config.binary_dir = binary_dir;
    config.runtime_dir = temp_dir.path().join("runtime");

    let result = DeploymentOrchestrator::new(config);
    assert!(result.is_ok());

    // Runtime dir should have been created
    assert!(temp_dir.path().join("runtime").exists());
}

#[test]
fn test_orchestrator_new_missing_binary_dir() {
    let temp_dir = TempDir::new().expect("create temp dir");

    let mut config = DeploymentConfig::test_config(temp_dir.path().join("test.seed"));
    config.binary_dir = temp_dir.path().join("nonexistent_bin");
    config.runtime_dir = temp_dir.path().join("runtime");

    let result = DeploymentOrchestrator::new(config);
    assert!(result.is_err());
}

#[test]
fn test_derive_child_seed_path_construction() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let binary_dir = temp_dir.path().join("bin");
    std::fs::create_dir(&binary_dir).expect("create binary dir");

    let seed_path = temp_dir.path().join("test.seed");
    std::fs::write(&seed_path, vec![0u8; 32]).expect("write seed");

    let mut config = DeploymentConfig::test_config(seed_path);
    config.binary_dir = binary_dir;
    config.runtime_dir = temp_dir.path().join("runtime");

    let orch = DeploymentOrchestrator::new(config).expect("create orchestrator");

    // Verify expected child seed path structure
    let expected_path = orch
        .config
        .runtime_dir
        .join(format!(".family-tower-{}.seed", orch.config.family_id));
    assert!(
        expected_path
            .to_string_lossy()
            .contains(".family-tower-1894e909e454.seed"),
        "Expected path to contain family-tower pattern: {}",
        expected_path.display()
    );
}

#[tokio::test]
async fn test_deploy_atomic_missing_seed() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let binary_dir = temp_dir.path().join("bin");
    std::fs::create_dir(&binary_dir).expect("create binary dir");

    let mut config = DeploymentConfig::test_config(temp_dir.path().join("nonexistent.seed"));
    config.binary_dir = binary_dir;
    config.runtime_dir = temp_dir.path().join("runtime");

    let mut orch = DeploymentOrchestrator::new(config).expect("create orchestrator");

    let result = orch.deploy_atomic(AtomicType::Tower).await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("USB seed not found"),
        "unexpected error: {err_msg}"
    );
}

#[tokio::test]
async fn test_deploy_atomic_all_launches_fail_still_ok_empty_instances() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let binary_dir = temp_dir.path().join("bin");
    std::fs::create_dir(&binary_dir).expect("create binary dir");

    let seed_path = temp_dir.path().join("usb.seed");
    std::fs::write(&seed_path, vec![0u8; 32]).expect("32-byte seed");

    let mut config = DeploymentConfig::test_config(seed_path);
    config.binary_dir = binary_dir;
    config.runtime_dir = temp_dir.path().join("runtime");

    let mut orch = DeploymentOrchestrator::new(config).expect("create orchestrator");

    let result = orch.deploy_atomic(AtomicType::Tower).await;
    assert!(
        result.is_ok(),
        "degraded deploy with no binaries should return Ok(empty): {:?}",
        result
    );
    let instances = result.expect("ok");
    assert!(instances.is_empty());
}

#[tokio::test]
async fn test_deploy_all_three_phases_ok_when_launches_degraded() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let binary_dir = temp_dir.path().join("bin");
    std::fs::create_dir(&binary_dir).expect("create binary dir");

    let seed_path = temp_dir.path().join("usb.seed");
    std::fs::write(&seed_path, vec![0u8; 32]).expect("seed");

    let mut config = DeploymentConfig::test_config(seed_path);
    config.binary_dir = binary_dir;
    config.runtime_dir = temp_dir.path().join("runtime");

    let mut orch = DeploymentOrchestrator::new(config).expect("orchestrator");
    let report = orch.deploy_all().await.expect("deploy_all completes");
    assert_eq!(report.success_count, 3);
    assert!(report.errors.is_empty());
    assert!(report.tower.as_ref().map(|v| v.is_empty()).unwrap_or(false));
}

#[test]
fn test_atomic_type_required_primals_count() {
    assert_eq!(AtomicType::Tower.required_primals().len(), 2);
    assert_eq!(AtomicType::Node.required_primals().len(), 3);
    assert_eq!(AtomicType::Nest.required_primals().len(), 3);
}

#[test]
fn test_deployment_result_is_success_requires_no_errors() {
    let mut r = DeploymentResult::new();
    r.success_count = 3;
    r.errors.push("e".into());
    assert!(!r.is_success());
}

#[test]
fn test_deployment_config_neural_api_fields_roundtrip() {
    let temp_dir = TempDir::new().expect("temp");
    let mut c = DeploymentConfig::test_config(temp_dir.path().join("s.seed"));
    c.neural_api_enabled = true;
    c.neural_api_endpoint = Some("http://127.0.0.1:1".into());
    let j = serde_json::to_string(&c).expect("ser");
    let d: DeploymentConfig = serde_json::from_str(&j).expect("de");
    assert!(d.neural_api_enabled);
    assert_eq!(d.neural_api_endpoint.as_deref(), Some("http://127.0.0.1:1"));
}
