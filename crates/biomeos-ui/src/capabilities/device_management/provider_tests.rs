//! Tests for DeviceManagementProvider
//!
//! Extracted from provider.rs to keep file under 1000 lines.

use super::provider::{DeviceManagementProvider, ProviderCache};
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
