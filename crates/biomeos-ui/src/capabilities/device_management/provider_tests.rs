// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Tests for DeviceManagementProvider
//!
//! Extracted from provider.rs to keep file under 1000 lines.

use super::provider::{DeviceManagementProvider, ProviderCache};
use super::templates::{node_template, tower_template};
use super::types::*;

#[test]
fn test_provider_creation() {
    let provider = DeviceManagementProvider::new("/tmp/test.sock");
    assert_eq!(provider.socket_path, "/tmp/test.sock");
}

#[test]
fn test_provider_with_different_paths() {
    let xdg_provider = DeviceManagementProvider::new("/run/user/1000/biomeos.sock");
    assert!(xdg_provider.socket_path.contains("run/user"));

    let tmp_provider = DeviceManagementProvider::new("/tmp/biomeos.sock");
    assert!(tmp_provider.socket_path.contains("tmp"));
}

// Note: Template tests are now in templates.rs module

#[test]
fn test_cache_default() {
    let cache = ProviderCache::default();
    assert!(cache.devices.is_empty());
    assert!(cache.primals.is_empty());
    assert!(cache.templates.is_empty());
    assert!(cache.last_update.is_none());
}

#[tokio::test]
async fn test_provider_cache_initialization() {
    let provider = DeviceManagementProvider::new("/tmp/test.sock");

    // Cache should be initialized empty
    let cache = provider.cache.read().await;
    assert!(cache.devices.is_empty());
    assert!(cache.primals.is_empty());
    assert!(cache.templates.is_empty());
    assert!(cache.last_update.is_none());
}

#[tokio::test]
async fn test_provider_cache_isolation() {
    let provider1 = DeviceManagementProvider::new("/tmp/test1.sock");
    let provider2 = DeviceManagementProvider::new("/tmp/test2.sock");

    // Modify provider1's cache with a minimal template
    provider1.cache.write().await.templates = vec![NicheTemplate {
        id: "test".to_string(),
        name: "Test".to_string(),
        description: "Test".to_string(),
        required_primals: vec![],
        optional_primals: vec![],
        estimated_resources: ResourceRequirements {
            cpu_cores: 1,
            memory_mb: 512,
            storage_gb: 1,
            gpu_required: false,
            network_bandwidth_mbps: 10,
        },
        metadata: serde_json::json!({}),
    }];

    // Provider2's cache should still be empty (isolation)
    assert!(provider2.cache.read().await.templates.is_empty());
}

#[test]
fn test_types_serialization_roundtrip() {
    // ResourceRequirements, PrimalRole, and NicheTemplate all serialize/deserialize correctly
    let res = ResourceRequirements {
        cpu_cores: 4,
        memory_mb: 2048,
        storage_gb: 10,
        gpu_required: true,
        network_bandwidth_mbps: 100,
    };
    let r: ResourceRequirements =
        serde_json::from_str(&serde_json::to_string(&res).unwrap()).unwrap();
    assert_eq!((r.cpu_cores, r.gpu_required), (4, true));

    let role = PrimalRole {
        role: "security".into(),
        capabilities: vec!["crypto".into()],
        min_health: 0.9,
        metadata: serde_json::json!({}),
    };
    let pr: PrimalRole = serde_json::from_str(&serde_json::to_string(&role).unwrap()).unwrap();
    assert_eq!(pr.role.as_str(), "security");

    let tmpl = NicheTemplate {
        id: "t".into(),
        name: "T".into(),
        description: "T".into(),
        required_primals: vec![role],
        optional_primals: vec![],
        estimated_resources: res,
        metadata: serde_json::json!({}),
    };
    let nt: NicheTemplate = serde_json::from_str(&serde_json::to_string(&tmpl).unwrap()).unwrap();
    assert_eq!(nt.required_primals.len(), 1);
}

#[test]
fn test_get_builtin_templates() {
    let templates = DeviceManagementProvider::get_builtin_templates();

    assert_eq!(templates.len(), 2);
    let ids: Vec<_> = templates.iter().map(|t| t.id.as_str()).collect();
    assert!(ids.contains(&"tower"));
    assert!(ids.contains(&"node"));

    let tower = templates
        .iter()
        .find(|t| t.id == "tower")
        .expect("tower template");
    assert!(!tower.estimated_resources.gpu_required);
    assert_eq!(tower.required_primals.len(), 2);

    let node = templates
        .iter()
        .find(|t| t.id == "node")
        .expect("node template");
    assert!(node.estimated_resources.gpu_required);
    assert_eq!(node.required_primals.len(), 3);
}

#[tokio::test]
async fn test_validate_niche_tower_template() {
    let provider = DeviceManagementProvider::new("/tmp/test.sock");
    let template = tower_template();

    let result = provider
        .validate_niche(&template)
        .await
        .expect("validate_niche should succeed");

    assert!(
        result.valid || !result.errors.is_empty(),
        "Validation should return structured result"
    );
}

#[tokio::test]
async fn test_validate_niche_node_template() {
    let provider = DeviceManagementProvider::new("/tmp/test.sock");
    let template = node_template();

    let result = provider
        .validate_niche(&template)
        .await
        .expect("validate_niche should succeed");

    if !result.valid {
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("GPU") || e.contains("primal")),
            "Node template validation should report GPU or primal requirements: {:?}",
            result.errors
        );
    }
}

#[tokio::test]
async fn test_deploy_niche_no_orchestration() {
    let provider = DeviceManagementProvider::new("/tmp/test.sock");

    let result = provider
        .deploy_niche(serde_json::json!({
            "template_id": "tower",
            "family_id": "test"
        }))
        .await;

    assert!(
        result.is_err(),
        "deploy_niche should fail when no orchestration available"
    );
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("orchestration") || err.to_string().contains("capability"),
        "Expected orchestration-related error, got: {err}"
    );
}

#[tokio::test]
async fn test_assign_device_no_registry() {
    let provider = DeviceManagementProvider::new("/tmp/test.sock");

    let result = provider
        .assign_device("gpu-0".to_string(), "toadstool-1".to_string())
        .await;

    assert!(
        result.is_err(),
        "assign_device should fail when registry not available"
    );
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("registry")
            || msg.contains("assign_device")
            || msg.contains("failed")
            || msg.contains("JSON-RPC")
            || msg.contains("available"),
        "Expected registry-related error, got: {msg}"
    );
}

#[tokio::test]
async fn test_provider_start() {
    let provider = DeviceManagementProvider::new("/tmp/nonexistent-biomeos-start-test.sock");

    let result = provider.start().await;

    assert!(
        result.is_ok(),
        "start() should succeed even when registry unavailable (graceful degradation)"
    );
}

#[tokio::test]
async fn test_get_niche_templates_returns_builtin_when_no_storage() {
    let provider = DeviceManagementProvider::new("/tmp/test.sock");

    let templates = provider
        .get_niche_templates()
        .await
        .expect("get_niche_templates should succeed");

    assert!(
        !templates.is_empty(),
        "Should return built-in templates when storage provider unavailable"
    );
    assert_eq!(templates.len(), 2);
}

#[tokio::test]
async fn test_get_devices_succeeds() {
    let provider = DeviceManagementProvider::new("/tmp/test.sock");

    let devices = provider
        .get_devices()
        .await
        .expect("get_devices should succeed");

    assert!(
        !devices.is_empty(),
        "get_devices should discover devices on Linux (CPU from /proc/cpuinfo, etc.)"
    );
}

#[tokio::test]
async fn test_get_primals_includes_biomeos() {
    let provider = DeviceManagementProvider::new("/tmp/test.sock");

    let primals = provider
        .get_primals()
        .await
        .expect("get_primals should succeed");

    let biomeos = primals.iter().find(|p| p.id == "biomeos");
    assert!(
        biomeos.is_some(),
        "get_primals should always include biomeOS itself"
    );
}

#[tokio::test]
async fn test_validate_niche_required_primal_missing() {
    let provider = DeviceManagementProvider::new("/tmp/test.sock");
    let template = NicheTemplate {
        id: "custom".to_string(),
        name: "Custom".to_string(),
        description: "Requires non-existent primal".to_string(),
        required_primals: vec![PrimalRole {
            role: "quantum-compute".to_string(),
            capabilities: vec![
                "quantum-computing".to_string(),
                "fusion-reactor".to_string(),
            ],
            min_health: 0.9,
            metadata: serde_json::json!({}),
        }],
        optional_primals: vec![],
        estimated_resources: ResourceRequirements {
            cpu_cores: 1,
            memory_mb: 512,
            storage_gb: 1,
            gpu_required: false,
            network_bandwidth_mbps: 10,
        },
        metadata: serde_json::json!({}),
    };

    let result = provider
        .validate_niche(&template)
        .await
        .expect("validate_niche should succeed");

    assert!(
        !result.valid,
        "Template with missing required primal should be invalid"
    );
    assert!(
        result.errors.iter().any(|e| e.contains("quantum-compute")),
        "Should report missing required role: {:?}",
        result.errors
    );
}

#[tokio::test]
async fn test_validate_niche_optional_primal_missing_warning() {
    let provider = DeviceManagementProvider::new("/tmp/test.sock");
    let template = NicheTemplate {
        id: "custom".to_string(),
        name: "Custom".to_string(),
        description: "Optional primal missing".to_string(),
        required_primals: vec![],
        optional_primals: vec![PrimalRole {
            role: "optional-ai".to_string(),
            capabilities: vec!["neural-interface".to_string()],
            min_health: 0.5,
            metadata: serde_json::json!({}),
        }],
        estimated_resources: ResourceRequirements {
            cpu_cores: 1,
            memory_mb: 512,
            storage_gb: 1,
            gpu_required: false,
            network_bandwidth_mbps: 10,
        },
        metadata: serde_json::json!({}),
    };

    let result = provider
        .validate_niche(&template)
        .await
        .expect("validate_niche should succeed");

    assert!(
        result.valid,
        "Missing optional primal should not fail validation"
    );
    assert!(
        result.warnings.iter().any(|w| w.contains("optional-ai")),
        "Should warn about missing optional role: {:?}",
        result.warnings
    );
}

#[tokio::test]
async fn test_validate_niche_node_template_returns_structured_result() {
    let provider = DeviceManagementProvider::new("/tmp/test.sock");
    let templates = provider.get_niche_templates().await.unwrap();
    let node_template = templates
        .iter()
        .find(|t| t.id == "node")
        .expect("node template exists")
        .clone();

    let result = provider
        .validate_niche(&node_template)
        .await
        .expect("validate_niche should succeed");

    assert!(
        !result.valid || result.errors.is_empty(),
        "Node template validation should return coherent result"
    );
    assert!(
        result.errors.iter().any(|e| {
            e.contains("GPU") || e.contains("gpu") || e.contains("primal") || e.contains("role")
        }) || result.valid,
        "When invalid, should report GPU or primal requirements: {:?}",
        result.errors
    );
}

#[tokio::test]
async fn test_provider_cache_updated_after_get_devices() {
    let provider = DeviceManagementProvider::new("/tmp/test.sock");
    let _ = provider
        .get_devices()
        .await
        .expect("get_devices should succeed");

    let cache = provider.cache.read().await;
    assert!(
        !cache.devices.is_empty() || cache.last_update.is_some(),
        "Cache should be updated after get_devices"
    );
}

#[tokio::test]
async fn test_provider_cache_updated_after_get_primals() {
    let provider = DeviceManagementProvider::new("/tmp/test.sock");
    let _ = provider
        .get_primals()
        .await
        .expect("get_primals should succeed");

    let cache = provider.cache.read().await;
    assert!(
        !cache.primals.is_empty(),
        "Cache should contain primals after get_primals"
    );
    assert!(cache.last_update.is_some());
}
