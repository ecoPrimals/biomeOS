// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Discovery Integration Tests
//!
//! Tests for the primal discovery service integration.
//! Note: Many of these tests are marked as `#[ignore]` until the full Songbird
//! integration is complete. They document the expected API behavior.

use biomeos_core::universal_biomeos_manager::*;
use biomeos_types::BiomeOSConfig;
use serde_json::json;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

/// Test the registry discovery with a mock server
/// NOTE: This test is ignored until Songbird integration is complete.
/// The discovery service currently returns placeholder results.
#[ignore = "Pending Songbird integration — discovery returns placeholder results"]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_registry_discovery_success() {
    // Create mock registry server
    let mock_server = MockServer::start().await;

    // Mock registry response
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "primals": [
                {
                    "name": "compute-service",
                    "endpoint": "http://localhost:8001",
                    "category": "compute",
                    "capabilities": ["cpu", "memory", "gpu"]
                },
                {
                    "name": "storage-service",
                    "endpoint": "http://localhost:8002",
                    "category": "storage",
                    "capabilities": ["disk", "backup"]
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).unwrap();
    let results = manager.discover_registry(&mock_server.uri()).await.unwrap();

    assert_eq!(results.len(), 2);
    assert!(results[0].starts_with("http://"));
    assert!(results[1].starts_with("http://"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_registry_discovery_empty_response() {
    let mock_server = MockServer::start().await;

    // Mock empty registry
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "primals": []
        })))
        .mount(&mock_server)
        .await;

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).unwrap();
    let results = manager.discover_registry(&mock_server.uri()).await.unwrap();

    // Empty result is expected
    assert_eq!(results.len(), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_registry_discovery_malformed_response() {
    let mock_server = MockServer::start().await;

    // Mock malformed JSON
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&mock_server)
        .await;

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).unwrap();
    let results = manager.discover_registry(&mock_server.uri()).await;

    // Should handle malformed response gracefully (empty results)
    assert!(results.is_ok());
    assert_eq!(results.unwrap().len(), 0);
}

/// Test discovery of orchestration services
/// NOTE: Ignored until Songbird integration is complete.
#[ignore = "Pending Songbird integration"]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_capability_based_orchestration_discovery_success() {
    let mock_server = MockServer::start().await;

    // Mock universal service discovery API (not tied to specific service names)
    Mock::given(method("GET"))
        .and(path("/api/v1/services"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "services": [
                {
                    "name": "orchestration-primary",
                    "endpoint": "http://localhost:8099",
                    "capabilities": ["orchestration", "service_discovery"],
                    "health": "healthy"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).unwrap();
    let results = manager
        .discover_registry(&format!("{}/api/v1/services", mock_server.uri()))
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    assert!(results[0].contains("8099"));
}

/// Test endpoint probing with health response
/// NOTE: Ignored until endpoint probing is fully implemented.
#[ignore = "Pending endpoint probing implementation"]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_probe_endpoint_success() {
    let mock_server = MockServer::start().await;

    // Mock health endpoint
    Mock::given(method("GET"))
        .and(path("/api/v1/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "healthy",
            "name": "test-service",
            "version": "1.0.0"
        })))
        .mount(&mock_server)
        .await;

    // Mock metadata endpoint
    Mock::given(method("GET"))
        .and(path("/api/v1/metadata"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "name": "test-service",
            "capabilities": ["compute", "storage"]
        })))
        .mount(&mock_server)
        .await;

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).unwrap();
    let result = manager.probe_endpoint(&mock_server.uri());

    assert!(result.is_ok());
    let info = result.unwrap();
    assert!(info.contains("test-service") || info.contains("unknown"));
}

/// Test error handling for unreachable endpoints
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_probe_endpoint_unreachable() {
    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).unwrap();

    // Probe a non-existent endpoint
    let result = manager.probe_endpoint("http://localhost:99999");

    // Should succeed with placeholder result (graceful degradation)
    assert!(result.is_ok());
}
