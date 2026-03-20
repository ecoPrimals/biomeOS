// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! CapabilityRegistry Tests
//!
//! Extracted from capability_registry.rs to maintain files under 1000 lines.
//! Tests cover registration, discovery, heartbeats, unregistration, and edge cases.
//! Additional tests: `capability_registry_tests2.rs`.

use super::capability_registry::*;
use crate::Capability;
use biomeos_types::PrimalId;
use std::collections::HashMap;

#[tokio::test]
async fn test_register_and_get_provider() {
    let registry = CapabilityRegistry::new("test".to_string());

    let primal_id = PrimalId::new("beardog-localhost").unwrap();
    let params = RegisterParams {
        provides: vec![Capability::Security],
        requires: vec![],
        socket_path: Some("/tmp/beardog-test.sock".to_string()),
        http_endpoint: None,
        metadata: None,
    };

    registry.register(primal_id.clone(), params).await.unwrap();

    let provider = registry.get_provider(&Capability::Security).await.unwrap();
    assert!(provider.is_some());
    assert_eq!(provider.unwrap().id, primal_id);
}

#[tokio::test]
async fn test_register_multiple_capabilities() {
    let registry = CapabilityRegistry::new("test".to_string());

    let primal_id = PrimalId::new("beardog-localhost").unwrap();
    let params = RegisterParams {
        provides: vec![Capability::Security, Capability::Compute],
        requires: vec![],
        socket_path: Some("/tmp/beardog-test.sock".to_string()),
        http_endpoint: None,
        metadata: None,
    };

    registry.register(primal_id.clone(), params).await.unwrap();

    // Should find provider for both capabilities
    let security_provider = registry.get_provider(&Capability::Security).await.unwrap();
    assert!(security_provider.is_some());
    assert_eq!(security_provider.unwrap().id, primal_id);

    let compute_provider = registry.get_provider(&Capability::Compute).await.unwrap();
    assert!(compute_provider.is_some());
    assert_eq!(compute_provider.unwrap().id, primal_id);
}

#[tokio::test]
async fn test_register_with_metadata() {
    let registry = CapabilityRegistry::new("test".to_string());

    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), "1.0.0".to_string());
    metadata.insert("platform".to_string(), "linux".to_string());

    let primal_id = PrimalId::new("beardog-localhost").unwrap();
    let params = RegisterParams {
        provides: vec![Capability::Security],
        requires: vec![],
        socket_path: Some("/tmp/beardog-test.sock".to_string()),
        http_endpoint: None,
        metadata: Some(metadata.clone()),
    };

    registry.register(primal_id.clone(), params).await.unwrap();

    let provider = registry.get_provider(&Capability::Security).await.unwrap();
    assert!(provider.is_some());
    let info = provider.unwrap();
    assert_eq!(info.metadata.get("version"), Some(&"1.0.0".to_string()));
    assert_eq!(info.metadata.get("platform"), Some(&"linux".to_string()));
}

#[tokio::test]
async fn test_register_with_http_endpoint() {
    let registry = CapabilityRegistry::new("test".to_string());

    let primal_id = PrimalId::new("songbird-localhost").unwrap();
    let params = RegisterParams {
        provides: vec![Capability::Discovery],
        requires: vec![],
        socket_path: None,
        http_endpoint: Some("http://localhost:8080".to_string()),
        metadata: None,
    };

    registry.register(primal_id.clone(), params).await.unwrap();

    let provider = registry.get_provider(&Capability::Discovery).await.unwrap();
    assert!(provider.is_some());
    let info = provider.unwrap();
    assert_eq!(
        info.http_endpoint,
        Some("http://localhost:8080".to_string())
    );
}

#[tokio::test]
async fn test_register_with_requires() {
    let registry = CapabilityRegistry::new("test".to_string());

    let primal_id = PrimalId::new("test-primal").unwrap();
    let params = RegisterParams {
        provides: vec![Capability::Compute],
        requires: vec![Capability::Security, Capability::Storage],
        socket_path: Some("/tmp/test.sock".to_string()),
        http_endpoint: None,
        metadata: None,
    };

    registry.register(primal_id.clone(), params).await.unwrap();

    let provider = registry.get_provider(&Capability::Compute).await.unwrap();
    assert!(provider.is_some());
    let info = provider.unwrap();
    assert_eq!(info.requires.len(), 2);
    assert!(info.requires.contains(&Capability::Security));
    assert!(info.requires.contains(&Capability::Storage));
}

#[tokio::test]
async fn test_get_provider_nonexistent() {
    let registry = CapabilityRegistry::new("test".to_string());

    let provider = registry.get_provider(&Capability::Security).await.unwrap();
    assert!(provider.is_none());
}

#[tokio::test]
async fn test_list_primals_empty() {
    let registry = CapabilityRegistry::new("test".to_string());
    let primals = registry.list_primals().await;
    assert!(primals.is_empty());
}

#[tokio::test]
async fn test_list_primals_multiple() {
    let registry = CapabilityRegistry::new("test".to_string());

    let primal1_id = PrimalId::new("beardog-localhost").unwrap();
    let params1 = RegisterParams {
        provides: vec![Capability::Security],
        requires: vec![],
        socket_path: Some("/tmp/beardog.sock".to_string()),
        http_endpoint: None,
        metadata: None,
    };
    registry
        .register(primal1_id.clone(), params1)
        .await
        .unwrap();

    let primal2_id = PrimalId::new("songbird-localhost").unwrap();
    let params2 = RegisterParams {
        provides: vec![Capability::Discovery],
        requires: vec![],
        socket_path: Some("/tmp/songbird.sock".to_string()),
        http_endpoint: None,
        metadata: None,
    };
    registry
        .register(primal2_id.clone(), params2)
        .await
        .unwrap();

    let primals = registry.list_primals().await;
    assert_eq!(primals.len(), 2);
    assert!(primals.iter().any(|p| p.id == primal1_id));
    assert!(primals.iter().any(|p| p.id == primal2_id));
}

#[tokio::test]
async fn test_heartbeat() {
    let registry = CapabilityRegistry::new("test".to_string());

    let primal_id = PrimalId::new("beardog-localhost").unwrap();
    let params = RegisterParams {
        provides: vec![Capability::Security],
        requires: vec![],
        socket_path: Some("/tmp/beardog.sock".to_string()),
        http_endpoint: None,
        metadata: None,
    };

    registry.register(primal_id.clone(), params).await.unwrap();

    // Get initial heartbeat time
    let provider = registry
        .get_provider(&Capability::Security)
        .await
        .unwrap()
        .unwrap();
    let initial_heartbeat = provider.last_heartbeat;

    // Send heartbeat — updates last_heartbeat to Utc::now()
    registry.heartbeat(&primal_id).await.unwrap();

    // Verify heartbeat was updated (>= since chrono resolution may equal initial)
    let provider = registry
        .get_provider(&Capability::Security)
        .await
        .unwrap()
        .unwrap();
    assert!(provider.last_heartbeat >= initial_heartbeat);
}

#[tokio::test]
async fn test_heartbeat_nonexistent() {
    let registry = CapabilityRegistry::new("test".to_string());
    let primal_id = PrimalId::new("nonexistent").unwrap();

    let result = registry.heartbeat(&primal_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_unregister() {
    let registry = CapabilityRegistry::new("test".to_string());

    let primal_id = PrimalId::new("beardog-localhost").unwrap();
    let params = RegisterParams {
        provides: vec![Capability::Security],
        requires: vec![],
        socket_path: Some("/tmp/beardog-test.sock".to_string()),
        http_endpoint: None,
        metadata: None,
    };

    registry.register(primal_id.clone(), params).await.unwrap();
    registry.unregister(&primal_id).await.unwrap();

    let provider = registry.get_provider(&Capability::Security).await.unwrap();
    assert!(provider.is_none());
}

#[tokio::test]
async fn test_unregister_nonexistent() {
    let registry = CapabilityRegistry::new("test".to_string());
    let primal_id = PrimalId::new("nonexistent").unwrap();

    let result = registry.unregister(&primal_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_unregister_multiple_capabilities() {
    let registry = CapabilityRegistry::new("test".to_string());

    let primal_id = PrimalId::new("beardog-localhost").unwrap();
    let params = RegisterParams {
        provides: vec![Capability::Security, Capability::Compute],
        requires: vec![],
        socket_path: Some("/tmp/beardog.sock".to_string()),
        http_endpoint: None,
        metadata: None,
    };

    registry.register(primal_id.clone(), params).await.unwrap();

    // Verify both capabilities are registered
    assert!(
        registry
            .get_provider(&Capability::Security)
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        registry
            .get_provider(&Capability::Compute)
            .await
            .unwrap()
            .is_some()
    );

    // Unregister
    registry.unregister(&primal_id).await.unwrap();

    // Both capabilities should be removed
    assert!(
        registry
            .get_provider(&Capability::Security)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        registry
            .get_provider(&Capability::Compute)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn test_register_overwrite() {
    let registry = CapabilityRegistry::new("test".to_string());

    let primal_id = PrimalId::new("beardog-localhost").unwrap();

    // First registration
    let params1 = RegisterParams {
        provides: vec![Capability::Security],
        requires: vec![],
        socket_path: Some("/tmp/beardog-v1.sock".to_string()),
        http_endpoint: None,
        metadata: None,
    };
    registry.register(primal_id.clone(), params1).await.unwrap();

    // Second registration (overwrites)
    let params2 = RegisterParams {
        provides: vec![Capability::Security],
        requires: vec![],
        socket_path: Some("/tmp/beardog-v2.sock".to_string()),
        http_endpoint: None,
        metadata: None,
    };
    registry.register(primal_id.clone(), params2).await.unwrap();

    // Should have the new socket path
    let provider = registry
        .get_provider(&Capability::Security)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        provider.socket_path,
        Some("/tmp/beardog-v2.sock".to_string())
    );
}

#[tokio::test]
async fn test_multiple_providers_same_capability() {
    let registry = CapabilityRegistry::new("test".to_string());

    let primal1_id = PrimalId::new("beardog-1").unwrap();
    let params1 = RegisterParams {
        provides: vec![Capability::Security],
        requires: vec![],
        socket_path: Some("/tmp/beardog-1.sock".to_string()),
        http_endpoint: None,
        metadata: None,
    };
    registry
        .register(primal1_id.clone(), params1)
        .await
        .unwrap();

    let primal2_id = PrimalId::new("beardog-2").unwrap();
    let params2 = RegisterParams {
        provides: vec![Capability::Security],
        requires: vec![],
        socket_path: Some("/tmp/beardog-2.sock".to_string()),
        http_endpoint: None,
        metadata: None,
    };
    registry
        .register(primal2_id.clone(), params2)
        .await
        .unwrap();

    // get_provider returns the first registered provider
    let provider = registry.get_provider(&Capability::Security).await.unwrap();
    assert!(provider.is_some());
    // Should return one of the two (implementation-dependent)
    let provider_id = provider.unwrap().id;
    assert!(provider_id == primal1_id || provider_id == primal2_id);
}

#[tokio::test]
async fn test_registry_clone() {
    let registry1 = CapabilityRegistry::new("test".to_string());

    let primal_id = PrimalId::new("beardog-localhost").unwrap();
    let params = RegisterParams {
        provides: vec![Capability::Security],
        requires: vec![],
        socket_path: Some("/tmp/beardog.sock".to_string()),
        http_endpoint: None,
        metadata: None,
    };
    registry1.register(primal_id.clone(), params).await.unwrap();

    // Clone should share the same state
    let registry2 = registry1.clone();
    let provider = registry2.get_provider(&Capability::Security).await.unwrap();
    assert!(provider.is_some());
    assert_eq!(provider.unwrap().id, primal_id);
}

#[tokio::test]
async fn test_primal_info_timestamps() {
    let registry = CapabilityRegistry::new("test".to_string());

    let primal_id = PrimalId::new("beardog-localhost").unwrap();
    let params = RegisterParams {
        provides: vec![Capability::Security],
        requires: vec![],
        socket_path: Some("/tmp/beardog.sock".to_string()),
        http_endpoint: None,
        metadata: None,
    };

    let before_registration = chrono::Utc::now();
    registry.register(primal_id.clone(), params).await.unwrap();
    let after_registration = chrono::Utc::now();

    let provider = registry
        .get_provider(&Capability::Security)
        .await
        .unwrap()
        .unwrap();
    assert!(provider.registered_at >= before_registration);
    assert!(provider.registered_at <= after_registration);
    assert!(provider.last_heartbeat >= before_registration);
    assert!(provider.last_heartbeat <= after_registration);
}

#[tokio::test]
async fn test_empty_capabilities() {
    let registry = CapabilityRegistry::new("test".to_string());

    let primal_id = PrimalId::new("test-primal").unwrap();
    let params = RegisterParams {
        provides: vec![],
        requires: vec![],
        socket_path: Some("/tmp/test.sock".to_string()),
        http_endpoint: None,
        metadata: None,
    };

    registry.register(primal_id.clone(), params).await.unwrap();

    // Primal should be registered but not provide any capabilities
    let primals = registry.list_primals().await;
    assert_eq!(primals.len(), 1);
    assert_eq!(primals[0].id, primal_id);
    assert!(primals[0].provides.is_empty());
}

#[tokio::test]
async fn test_register_with_empty_metadata() {
    let registry = CapabilityRegistry::new("test".to_string());

    let primal_id = PrimalId::new("beardog-localhost").unwrap();
    let params = RegisterParams {
        provides: vec![Capability::Security],
        requires: vec![],
        socket_path: Some("/tmp/beardog.sock".to_string()),
        http_endpoint: None,
        metadata: Some(HashMap::new()),
    };

    registry.register(primal_id.clone(), params).await.unwrap();

    let provider = registry
        .get_provider(&Capability::Security)
        .await
        .unwrap()
        .unwrap();
    assert!(provider.metadata.is_empty());
}

#[tokio::test]
async fn test_register_with_none_metadata() {
    let registry = CapabilityRegistry::new("test".to_string());

    let primal_id = PrimalId::new("beardog-localhost").unwrap();
    let params = RegisterParams {
        provides: vec![Capability::Security],
        requires: vec![],
        socket_path: Some("/tmp/beardog.sock".to_string()),
        http_endpoint: None,
        metadata: None,
    };

    registry.register(primal_id.clone(), params).await.unwrap();

    let provider = registry
        .get_provider(&Capability::Security)
        .await
        .unwrap()
        .unwrap();
    assert!(provider.metadata.is_empty());
}

#[test]
fn test_registry_request_serialization() {
    let req = RegistryRequest::GetProvider {
        request_id: "req-1".to_string(),
        capability: Capability::Compute,
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let restored: RegistryRequest = serde_json::from_str(&json).expect("deserialize");
    match (&req, &restored) {
        (
            RegistryRequest::GetProvider {
                request_id: r1,
                capability: c1,
            },
            RegistryRequest::GetProvider {
                request_id: r2,
                capability: c2,
            },
        ) => {
            assert_eq!(r1, r2);
            assert_eq!(c1, c2);
        }
        _ => panic!("mismatch"),
    }
}

#[test]
fn test_registry_response_serialization() {
    let resp = RegistryResponse {
        request_id: "req-1".to_string(),
        status: ResponseStatus::Success,
        data: Some(serde_json::json!({"message": "ok"})),
        error: None,
    };
    let json = serde_json::to_value(&resp).expect("serialize");
    let restored: RegistryResponse = serde_json::from_value(json).expect("deserialize");
    assert_eq!(resp.request_id, restored.request_id);
    assert!(matches!(restored.status, ResponseStatus::Success));
}

#[test]
fn test_response_status_variants() {
    let statuses = [
        ResponseStatus::Success,
        ResponseStatus::Error,
        ResponseStatus::NotFound,
    ];
    for status in statuses {
        let json = serde_json::to_value(&status).expect("serialize");
        let restored: ResponseStatus = serde_json::from_value(json).expect("deserialize");
        assert!(matches!(
            (status, restored),
            (ResponseStatus::Success, ResponseStatus::Success)
                | (ResponseStatus::Error, ResponseStatus::Error)
                | (ResponseStatus::NotFound, ResponseStatus::NotFound)
        ));
    }
}

#[test]
fn test_register_params_serialization() {
    let params = RegisterParams {
        provides: vec![Capability::Security, Capability::Storage],
        requires: vec![Capability::Compute],
        socket_path: Some("/tmp/sock".to_string()),
        http_endpoint: None,
        metadata: Some({
            let mut m = HashMap::new();
            m.insert("key".to_string(), "value".to_string());
            m
        }),
    };
    let json = serde_json::to_value(&params).expect("serialize");
    let restored: RegisterParams = serde_json::from_value(json).expect("deserialize");
    assert_eq!(params.provides.len(), restored.provides.len());
    assert_eq!(params.requires.len(), restored.requires.len());
}

#[test]
fn test_registry_request_get_provider_roundtrip_json() {
    let req = RegistryRequest::GetProvider {
        request_id: "g1".to_string(),
        capability: Capability::Discovery,
    };
    let v = serde_json::to_value(&req).expect("to_value");
    let back: RegistryRequest = serde_json::from_value(v).expect("from_value");
    match back {
        RegistryRequest::GetProvider {
            request_id,
            capability,
        } => {
            assert_eq!(request_id, "g1");
            assert_eq!(capability, Capability::Discovery);
        }
        _ => panic!("expected GetProvider"),
    }
}

#[test]
fn test_primal_info_serialization_roundtrip() {
    let now = chrono::Utc::now();
    let info = PrimalInfo {
        id: PrimalId::new("beardog-localhost").unwrap(),
        provides: vec![Capability::Security, Capability::Compute],
        requires: vec![Capability::Storage],
        socket_path: Some("/tmp/beardog.sock".to_string()),
        http_endpoint: Some("http://localhost:8080".to_string()),
        metadata: {
            let mut m = HashMap::new();
            m.insert("version".to_string(), "1.0".to_string());
            m
        },
        registered_at: now,
        last_heartbeat: now,
    };
    let json = serde_json::to_value(&info).expect("serialize");
    let restored: PrimalInfo = serde_json::from_value(json).expect("deserialize");
    assert_eq!(restored.id, info.id);
    assert_eq!(restored.provides.len(), 2);
    assert_eq!(restored.requires.len(), 1);
    assert_eq!(restored.socket_path, info.socket_path);
}
