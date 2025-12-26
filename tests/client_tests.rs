// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! Comprehensive tests for all primal clients
//!
//! These tests verify client functionality using mock servers,
//! ensuring proper error handling, request/response formats,
//! and capability-based discovery patterns.

use biomeos_core::clients::{
    beardog::BearDogClient, nestgate::NestGateClient, songbird::*, squirrel::SquirrelClient,
    toadstool::ToadStoolClient,
};
use biomeos_core::primal_client::PrimalClient;
use serde_json::json;
use wiremock::{
    matchers::{method, path, path_regex},
    Mock, MockServer, ResponseTemplate,
};

// ============================================================================
// Songbird Client Tests
// ============================================================================

#[tokio::test]
async fn test_songbird_health_check() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "healthy",
            "message": "Songbird is operational"
        })))
        .mount(&mock_server)
        .await;

    let client = SongbirdClient::new(mock_server.uri());
    let health = client.health_check().await.unwrap();

    assert!(health.healthy);
    assert_eq!(health.message, "Songbird is operational");
}

#[tokio::test]
async fn test_songbird_discover_by_capability() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/services/query/compute"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "service_id": "svc-001",
                "service_name": "toadstool-1",
                "endpoint": "http://localhost:8080",
                "capabilities": ["compute", "ai"],
                "metadata": {
                    "version": "1.0.0",
                    "tags": ["gpu", "ml"]
                }
            }
        ])))
        .mount(&mock_server)
        .await;

    let client = SongbirdClient::new(mock_server.uri());
    let services = client.discover_by_capability("compute").await.unwrap();

    assert_eq!(services.len(), 1);
    assert_eq!(services[0].service_name, "toadstool-1");
    assert_eq!(services[0].capabilities, vec!["compute", "ai"]);
}

#[tokio::test]
async fn test_songbird_register_service() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/services/register"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "service_id": "svc-new-123",
            "status": "registered"
        })))
        .mount(&mock_server)
        .await;

    let client = SongbirdClient::new(mock_server.uri());
    let registration = ServiceRegistration {
        service_name: "test-service".to_string(),
        capabilities: vec!["compute".to_string()],
        endpoint: "http://localhost:9999".to_string(),
        metadata: ServiceMetadata {
            version: "1.0.0".to_string(),
            location: None,
            tags: vec![],
        },
    };

    let service_id = client.register_service(&registration).await.unwrap();
    assert_eq!(service_id, "svc-new-123");
}

#[tokio::test]
async fn test_songbird_query_with_metadata() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/services/query/storage"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "service_id": "svc-001",
                "service_name": "storage-v1",
                "endpoint": "http://localhost:8001",
                "capabilities": ["storage"],
                "metadata": {
                    "version": "1.5.0",
                    "tags": ["ssd"]
                }
            },
            {
                "service_id": "svc-002",
                "service_name": "storage-v2",
                "endpoint": "http://localhost:8002",
                "capabilities": ["storage"],
                "metadata": {
                    "version": "2.0.0",
                    "tags": ["nvme"]
                }
            }
        ])))
        .mount(&mock_server)
        .await;

    let client = SongbirdClient::new(mock_server.uri());
    let v2_services = client
        .query_with_metadata("storage", |meta| meta.version.starts_with("2."))
        .await
        .unwrap();

    assert_eq!(v2_services.len(), 1);
    assert_eq!(v2_services[0].service_name, "storage-v2");
}

#[tokio::test]
async fn test_songbird_discover_by_location() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/services/all"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "service_id": "svc-ny",
                "service_name": "nyc-server",
                "endpoint": "http://nyc:8080",
                "capabilities": ["compute"],
                "metadata": {
                    "version": "1.0.0",
                    "location": {
                        "latitude": 40.7128,
                        "longitude": -74.0060
                    },
                    "tags": []
                }
            },
            {
                "service_id": "svc-sf",
                "service_name": "sf-server",
                "endpoint": "http://sf:8080",
                "capabilities": ["compute"],
                "metadata": {
                    "version": "1.0.0",
                    "location": {
                        "latitude": 37.7749,
                        "longitude": -122.4194
                    },
                    "tags": []
                }
            }
        ])))
        .mount(&mock_server)
        .await;

    let client = SongbirdClient::new(mock_server.uri());
    // Search within 100km of NYC
    let nearby = client
        .discover_by_location(40.7128, -74.0060, 100.0)
        .await
        .unwrap();

    assert_eq!(nearby.len(), 1);
    assert_eq!(nearby[0].service_name, "nyc-server");
}

// ============================================================================
// ToadStool Client Tests (Compute)
// ============================================================================

#[tokio::test]
async fn test_toadstool_health_check() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "healthy",
            "message": "ToadStool compute ready"
        })))
        .mount(&mock_server)
        .await;

    let client = ToadStoolClient::new(mock_server.uri());
    let health = client.health_check().await.unwrap();

    assert!(health.healthy);
    assert!(health.message.contains("ToadStool"));
}

#[tokio::test]
async fn test_toadstool_execute_job() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/compute/execute"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "job_id": "job-123",
            "status": "queued"
        })))
        .mount(&mock_server)
        .await;

    let client = ToadStoolClient::new(mock_server.uri());
    let response = client
        .request(
            "POST",
            "/api/v1/compute/execute",
            Some(json!({
                "task": "process_data"
            })),
        )
        .await
        .unwrap();

    assert_eq!(response["job_id"], "job-123");
}

#[tokio::test]
async fn test_toadstool_is_available() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "healthy"
        })))
        .mount(&mock_server)
        .await;

    let client = ToadStoolClient::new(mock_server.uri());
    assert!(client.is_available().await);
}

// ============================================================================
// NestGate Client Tests (Security/Auth)
// ============================================================================

#[tokio::test]
async fn test_nestgate_health_check() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "healthy",
            "message": "NestGate security operational"
        })))
        .mount(&mock_server)
        .await;

    let client = NestGateClient::new(mock_server.uri());
    let health = client.health_check().await.unwrap();

    assert!(health.healthy);
}

#[tokio::test]
async fn test_nestgate_authenticate() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/auth/authenticate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "token": "auth-token-xyz",
            "user_id": "user-123"
        })))
        .mount(&mock_server)
        .await;

    let client = NestGateClient::new(mock_server.uri());
    let response = client
        .request(
            "POST",
            "/api/v1/auth/authenticate",
            Some(json!({
                "username": "testuser"
            })),
        )
        .await
        .unwrap();

    assert_eq!(response["token"], "auth-token-xyz");
}

// ============================================================================
// BearDog Client Tests (Storage)
// ============================================================================

#[tokio::test]
async fn test_beardog_health_check() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "healthy",
            "message": "BearDog storage ready",
            "storage_available": true
        })))
        .mount(&mock_server)
        .await;

    let client = BearDogClient::new(mock_server.uri());
    let health = client.health_check().await.unwrap();

    assert!(health.healthy);
    assert!(health.message.contains("BearDog"));
}

#[tokio::test]
async fn test_beardog_store_data() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/storage/store"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object_id": "obj-456",
            "stored": true
        })))
        .mount(&mock_server)
        .await;

    let client = BearDogClient::new(mock_server.uri());
    let response = client
        .request(
            "POST",
            "/api/v1/storage/store",
            Some(json!({
                "data": "test_data"
            })),
        )
        .await
        .unwrap();

    assert_eq!(response["object_id"], "obj-456");
    assert_eq!(response["stored"], true);
}

// ============================================================================
// Squirrel Client Tests (Discovery Service)
// ============================================================================

#[tokio::test]
async fn test_squirrel_health_check() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "healthy",
            "message": "Squirrel discovery service ready"
        })))
        .mount(&mock_server)
        .await;

    let client = SquirrelClient::new(mock_server.uri());
    let health = client.health_check().await.unwrap();

    assert!(health.healthy);
}

#[tokio::test]
async fn test_squirrel_discover_services() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path_regex(r"/api/v1/discover.*"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "services": [
                {"name": "service1", "endpoint": "http://s1:8080"},
                {"name": "service2", "endpoint": "http://s2:8080"}
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = SquirrelClient::new(mock_server.uri());
    let response = client
        .request("GET", "/api/v1/discover", None)
        .await
        .unwrap();

    assert!(response["services"].is_array());
    assert_eq!(response["services"].as_array().unwrap().len(), 2);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_client_handles_404() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/nonexistent"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let client = SongbirdClient::new(mock_server.uri());
    let result = client.request("GET", "/nonexistent", None).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_client_handles_500() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/error"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let client = ToadStoolClient::new(mock_server.uri());
    let result = client.request("GET", "/error", None).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_client_handles_timeout() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/slow"))
        .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(60)))
        .mount(&mock_server)
        .await;

    let client = BearDogClient::new(mock_server.uri());
    let result = tokio::time::timeout(
        std::time::Duration::from_millis(100),
        client.request("GET", "/slow", None),
    )
    .await;

    assert!(result.is_err()); // Should timeout
}

// ============================================================================
// Capability-Based Discovery Tests
// ============================================================================

#[tokio::test]
async fn test_capability_discovery_no_hardcoding() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/services/query/ai"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "service_id": "ai-001",
                "service_name": "discovered-ai-service",
                "endpoint": "http://dynamic:7777",
                "capabilities": ["ai", "ml"],
                "metadata": {"version": "1.0.0", "tags": []}
            }
        ])))
        .mount(&mock_server)
        .await;

    let client = SongbirdClient::new(mock_server.uri());
    let services = client.discover_by_capability("ai").await.unwrap();

    // Verify we found services by capability, not hardcoded names
    assert_eq!(services.len(), 1);
    assert_eq!(services[0].service_name, "discovered-ai-service");
    assert!(services[0].capabilities.contains(&"ai".to_string()));
}

// ============================================================================
// Integration: Multiple Clients
// ============================================================================

#[tokio::test]
async fn test_multiple_clients_independent() {
    let mock_songbird = MockServer::start().await;
    let mock_toadstool = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "healthy"
        })))
        .mount(&mock_songbird)
        .await;

    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "healthy"
        })))
        .mount(&mock_toadstool)
        .await;

    let songbird = SongbirdClient::new(mock_songbird.uri());
    let toadstool = ToadStoolClient::new(mock_toadstool.uri());

    // Both clients should work independently
    assert!(songbird.is_available().await);
    assert!(toadstool.is_available().await);
}
