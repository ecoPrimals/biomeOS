// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::capability_registry::*;
use crate::Capability;
use biomeos_types::PrimalId;
use std::collections::HashMap;

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
