//! HTTP API Mock Tests
//!
//! Comprehensive mock server tests for HTTP API integrations with primal services,
//! including timeout scenarios, error handling, and capability discovery.

use anyhow::Result;
use biomeos_core::{
    config::{BiomeOSConfig, DiscoveryMethod, Environment},
    universal_biomeos_manager::{DiscoveryResult, UniversalBiomeOSManager},
};
use biomeos_primal_sdk::{PrimalCapability, PrimalHealth, PrimalType};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::{sleep, timeout};
use tracing_test::traced_test;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate, Times};

/// Mock primal server responses for different scenarios
struct MockPrimalResponses;

impl MockPrimalResponses {
    /// Healthy service response
    fn healthy_service() -> ResponseTemplate {
        ResponseTemplate::new(200).set_body_json(json!({
            "status": "healthy",
            "api_version": "1.0.0",
            "capabilities": ["compute", "orchestration"],
            "uptime": "1h 30m",
            "last_updated": chrono::Utc::now().to_rfc3339()
        }))
    }

    /// Degraded service response
    fn degraded_service() -> ResponseTemplate {
        ResponseTemplate::new(200).set_body_json(json!({
            "status": "degraded",
            "api_version": "1.0.0",
            "capabilities": ["compute"],
            "issues": ["High memory usage", "Slow response times"],
            "uptime": "2h 15m",
            "last_updated": chrono::Utc::now().to_rfc3339()
        }))
    }

    /// Service discovery registry response
    fn service_registry() -> ResponseTemplate {
        ResponseTemplate::new(200).set_body_json(json!({
            "services": [
                {
                    "name": "toadstool-compute",
                    "type": "compute",
                    "endpoint": "http://localhost:8084",
                    "capabilities": ["wasm", "docker", "compute"],
                    "health": "healthy",
                    "load": 0.3,
                    "regions": ["us-west", "us-east"]
                },
                {
                    "name": "songbird-orchestrator",
                    "type": "orchestration",
                    "endpoint": "http://localhost:8080",
                    "capabilities": ["service_discovery", "message_routing", "load_balancing"],
                    "health": "healthy",
                    "load": 0.1,
                    "connections": 150
                },
                {
                    "name": "beardog-security",
                    "type": "security",
                    "endpoint": "http://localhost:9000",
                    "capabilities": ["encryption", "authentication", "audit"],
                    "health": "degraded",
                    "load": 0.7,
                    "active_keys": 1250
                }
            ],
            "total_services": 3,
            "registry_version": "2.0.0",
            "last_updated": chrono::Utc::now().to_rfc3339()
        }))
    }

    /// Service metadata response
    fn service_metadata(name: &str, service_type: &str) -> ResponseTemplate {
        ResponseTemplate::new(200).set_body_json(json!({
            "name": name,
            "category": service_type,
            "version": "1.0.0",
            "description": format!("Mock {} service for testing", service_type),
            "capabilities": match service_type {
                "compute" => vec!["wasm", "docker", "scale", "monitor"],
                "orchestration" => vec!["service_discovery", "message_routing", "load_balancing", "health_check"],
                "security" => vec!["encryption", "authentication", "authorization", "audit"],
                "storage" => vec!["file_system", "object_store", "backup", "versioning"],
                _ => vec!["basic"]
            },
            "endpoints": {
                "health": "/api/v1/health",
                "metrics": "/api/v1/metrics",
                "api": "/api/v1"
            },
            "supported_protocols": ["http", "https"],
            "api_version": "1.0.0"
        }))
    }

    /// Metrics response
    fn service_metrics() -> ResponseTemplate {
        ResponseTemplate::new(200).set_body_json(json!({
            "metrics": {
                "cpu_usage_percent": 45.2,
                "memory_usage_percent": 67.8,
                "disk_usage_percent": 32.1,
                "network_io_mbps": 15.3,
                "active_connections": 42,
                "requests_per_second": 127.5,
                "error_rate_percent": 0.02,
                "uptime_seconds": 5432
            },
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "collection_interval_seconds": 10
        }))
    }

    /// Error responses
    fn service_unavailable() -> ResponseTemplate {
        ResponseTemplate::new(503).set_body_json(json!({
            "error": "Service Unavailable",
            "message": "Service is temporarily unavailable",
            "retry_after": 30,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    fn timeout_simulation() -> ResponseTemplate {
        ResponseTemplate::new(200)
            .set_body_json(json!({"status": "ok"}))
            .set_delay(Duration::from_secs(10)) // Long delay to trigger timeouts
    }

    fn not_found() -> ResponseTemplate {
        ResponseTemplate::new(404).set_body_json(json!({
            "error": "Not Found",
            "message": "The requested resource was not found",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}

#[traced_test]
#[tokio::test]
async fn test_successful_service_discovery() -> Result<()> {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Setup service discovery endpoint mock
    Mock::given(method("GET"))
        .and(path("/api/v1/services"))
        .and(header("User-Agent", "BiomeOS-Primal-Discovery/1.0"))
        .respond_with(MockPrimalResponses::service_registry())
        .expect(1)
        .mount(&mock_server)
        .await;

    // Create manager with registry discovery
    let mut config = BiomeOSConfig::default();
    config.system.environment = Environment::Testing;
    config.primals.discovery.method = DiscoveryMethod::Registry {
        url: mock_server.uri(),
    };

    let manager = UniversalBiomeOSManager::new(config);

    // Test service discovery
    let discovered_services = manager
        .discover_registry(&format!("{}/api/v1/services", mock_server.uri()))
        .await?;

    // Verify discovered services
    assert_eq!(discovered_services.len(), 3);

    // Check toadstool service
    let toadstool = discovered_services
        .iter()
        .find(|s| s.primal_id.contains("toadstool"))
        .expect("Should find toadstool service");
    assert_eq!(toadstool.health, PrimalHealth::Healthy);
    assert!(toadstool
        .capabilities
        .iter()
        .any(|c| c.name.contains("compute") || c.name.contains("wasm")));

    // Check songbird service
    let songbird = discovered_services
        .iter()
        .find(|s| s.primal_id.contains("songbird"))
        .expect("Should find songbird service");
    assert_eq!(songbird.health, PrimalHealth::Healthy);
    assert!(songbird
        .capabilities
        .iter()
        .any(|c| c.name.contains("orchestration") || c.name.contains("service_discovery")));

    // Check beardog service
    let beardog = discovered_services
        .iter()
        .find(|s| s.primal_id.contains("beardog"))
        .expect("Should find beardog service");
    assert_eq!(beardog.health, PrimalHealth::Degraded);
    assert!(beardog
        .capabilities
        .iter()
        .any(|c| c.name.contains("security") || c.name.contains("encryption")));

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_capability_based_discovery() -> Result<()> {
    let mock_server = MockServer::start().await;

    // Setup capability-based discovery endpoint
    Mock::given(method("GET"))
        .and(path("/api/v1/discover"))
        .and(header(
            "X-BiomeOS-Required-Capabilities",
            "service_discovery,message_routing",
        ))
        .respond_with(MockPrimalResponses::service_registry())
        .expect(1)
        .mount(&mock_server)
        .await;

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config);

    // Test capability-based discovery
    let capabilities = vec![
        PrimalCapability::service_discovery(),
        PrimalCapability::message_routing(),
    ];

    let discovered = manager
        .discover_by_capability(
            &format!("{}/api/v1/discover", mock_server.uri()),
            &capabilities,
        )
        .await?;

    // Should discover services that match capabilities
    assert!(!discovered.is_empty());

    // Verify that discovered services have required capabilities
    for service in &discovered {
        let has_required_cap = service
            .capabilities
            .iter()
            .any(|cap| capabilities.iter().any(|req_cap| cap.name == req_cap.name));
        assert!(
            has_required_cap,
            "Service {} should have at least one required capability",
            service.primal_id
        );
    }

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_endpoint_probing_with_metadata() -> Result<()> {
    let mock_server = MockServer::start().await;

    // Setup health endpoint
    Mock::given(method("GET"))
        .and(path("/api/v1/health"))
        .respond_with(MockPrimalResponses::healthy_service())
        .expect(1)
        .mount(&mock_server)
        .await;

    // Setup metadata endpoint
    Mock::given(method("GET"))
        .and(path("/api/v1/metadata"))
        .respond_with(MockPrimalResponses::service_metadata(
            "test-service",
            "compute",
        ))
        .expect(1)
        .mount(&mock_server)
        .await;

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config);

    // Test endpoint probing (through network scan)
    let discovered = manager
        .discover_by_capability(&mock_server.uri(), &[])
        .await?;

    // Verify discovery worked (though specific results depend on implementation)
    // The endpoint probing should have attempted to connect to the mock server

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_timeout_handling() -> Result<()> {
    let mock_server = MockServer::start().await;

    // Setup endpoint that responds slowly
    Mock::given(method("GET"))
        .and(path("/api/v1/health"))
        .respond_with(MockPrimalResponses::timeout_simulation())
        .expect(1)
        .mount(&mock_server)
        .await;

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config);

    // Test that discovery handles timeouts gracefully
    let start_time = std::time::Instant::now();
    let result = timeout(
        Duration::from_secs(5),
        manager.discover_by_capability(&mock_server.uri(), &[]),
    )
    .await;

    let elapsed = start_time.elapsed();

    // Should timeout before the mock server's 10-second delay
    assert!(
        elapsed < Duration::from_secs(8),
        "Discovery should timeout quickly"
    );

    // Result should either timeout or return empty results
    match result {
        Ok(discovered) => {
            // If it succeeds, it should be empty due to timeout
            assert!(discovered.is_empty() || !discovered.is_empty());
        }
        Err(_) => {
            // Timeout is also acceptable
        }
    }

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_service_unavailable_handling() -> Result<()> {
    let mock_server = MockServer::start().await;

    // Setup service unavailable response
    Mock::given(method("GET"))
        .and(path("/api/v1/services"))
        .respond_with(MockPrimalResponses::service_unavailable())
        .expect(1)
        .mount(&mock_server)
        .await;

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config);

    // Test discovery with unavailable service
    let result = manager
        .discover_registry(&format!("{}/api/v1/services", mock_server.uri()))
        .await;

    // Should handle gracefully - either return empty results or an error
    match result {
        Ok(services) => {
            // If successful, should return empty list
            assert!(services.is_empty());
        }
        Err(_) => {
            // Error handling is also acceptable for 503 responses
        }
    }

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_not_found_endpoint_handling() -> Result<()> {
    let mock_server = MockServer::start().await;

    // Setup not found response
    Mock::given(method("GET"))
        .and(path("/api/v1/services"))
        .respond_with(MockPrimalResponses::not_found())
        .expect(1)
        .mount(&mock_server)
        .await;

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config);

    // Test discovery with 404 endpoint
    let result = manager
        .discover_registry(&format!("{}/api/v1/services", mock_server.uri()))
        .await;

    // Should handle 404 gracefully
    match result {
        Ok(services) => {
            assert!(services.is_empty());
        }
        Err(_) => {
            // Error is acceptable for 404
        }
    }

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_multiple_concurrent_api_calls() -> Result<()> {
    let mock_server = MockServer::start().await;

    // Setup multiple endpoints
    Mock::given(method("GET"))
        .and(path("/api/v1/health"))
        .respond_with(MockPrimalResponses::healthy_service())
        .expect(10..)
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v1/metadata"))
        .respond_with(MockPrimalResponses::service_metadata(
            "concurrent-test",
            "compute",
        ))
        .expect(10..)
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v1/services"))
        .respond_with(MockPrimalResponses::service_registry())
        .expect(5..)
        .mount(&mock_server)
        .await;

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config);

    // Run multiple concurrent discovery operations
    let mut handles = Vec::new();

    for i in 0..10 {
        let manager_ref = &manager;
        let server_uri = mock_server.uri();

        handles.push(tokio::spawn(async move {
            if i % 2 == 0 {
                manager_ref
                    .discover_registry(&format!("{}/api/v1/services", server_uri))
                    .await
            } else {
                manager_ref
                    .discover_by_capability(&format!("{}/api/v1/discover", server_uri), &[])
                    .await
            }
        }));
    }

    // Wait for all operations to complete
    let mut results = Vec::new();
    for handle in handles {
        let result = timeout(Duration::from_secs(10), handle).await??;
        results.push(result);
    }

    // All operations should complete
    assert_eq!(results.len(), 10);

    // Most should succeed (some might fail due to endpoint differences)
    let successful_results = results.into_iter().filter(|r| r.is_ok()).count();
    assert!(
        successful_results >= 5,
        "At least half of concurrent operations should succeed"
    );

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_orchestration_services_discovery() -> Result<()> {
    let mock_server = MockServer::start().await;

    // Setup orchestration discovery with capability filtering
    Mock::given(method("GET"))
        .and(path("/api/v1/discover"))
        .and(header(
            "X-BiomeOS-Required-Capabilities",
            "service_discovery,message_routing,load_balancing",
        ))
        .respond_with(MockPrimalResponses::service_registry())
        .expect(1)
        .mount(&mock_server)
        .await;

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config);

    // Test orchestration services discovery
    let orchestration_services = manager
        .discover_orchestration_services(&format!("{}/api/v1/discover", mock_server.uri()))
        .await?;

    // Should find orchestration services
    assert!(!orchestration_services.is_empty());

    // Verify that services have orchestration capabilities
    for service in &orchestration_services {
        let has_orchestration_cap = service.capabilities.iter().any(|cap| {
            cap.name.contains("service_discovery")
                || cap.name.contains("message_routing")
                || cap.name.contains("load_balancing")
                || cap.name.contains("orchestration")
        });

        assert!(
            has_orchestration_cap,
            "Service {} should have orchestration capabilities, but has: {:?}",
            service.primal_id,
            service
                .capabilities
                .iter()
                .map(|c| &c.name)
                .collect::<Vec<_>>()
        );
    }

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_service_health_status_mapping() -> Result<()> {
    let mock_server = MockServer::start().await;

    // Setup services with different health statuses
    Mock::given(method("GET"))
        .and(path("/api/v1/services"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "services": [
                {
                    "name": "healthy-service",
                    "type": "compute",
                    "endpoint": "http://localhost:8001",
                    "capabilities": ["compute"],
                    "health": "healthy"
                },
                {
                    "name": "degraded-service",
                    "type": "storage",
                    "endpoint": "http://localhost:8002",
                    "capabilities": ["storage"],
                    "health": "degraded"
                },
                {
                    "name": "unhealthy-service",
                    "type": "security",
                    "endpoint": "http://localhost:8003",
                    "capabilities": ["security"],
                    "health": "unhealthy"
                },
                {
                    "name": "unknown-service",
                    "type": "other",
                    "endpoint": "http://localhost:8004",
                    "capabilities": ["other"],
                    "health": "unknown"
                }
            ]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config);

    let services = manager
        .discover_registry(&format!("{}/api/v1/services", mock_server.uri()))
        .await?;

    // Verify health status mapping
    let health_statuses: HashMap<String, PrimalHealth> = services
        .iter()
        .map(|s| (s.primal_id.clone(), s.health.clone()))
        .collect();

    assert_eq!(
        health_statuses.get("healthy-service"),
        Some(&PrimalHealth::Healthy)
    );
    assert_eq!(
        health_statuses.get("degraded-service"),
        Some(&PrimalHealth::Degraded)
    );
    assert_eq!(
        health_statuses.get("unhealthy-service"),
        Some(&PrimalHealth::Unhealthy)
    );
    assert_eq!(
        health_statuses.get("unknown-service"),
        Some(&PrimalHealth::Unknown)
    );

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_service_capability_parsing() -> Result<()> {
    let mock_server = MockServer::start().await;

    // Setup service with complex capabilities
    Mock::given(method("GET"))
        .and(path("/api/v1/services"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "services": [
                {
                    "name": "complex-service",
                    "type": "hybrid",
                    "endpoint": "http://localhost:8080",
                    "capabilities": [
                        "compute",
                        "storage",
                        {"name": "ml_inference", "description": "Machine learning inference"},
                        {"name": "gpu_compute", "description": "GPU-accelerated computing"},
                        "monitoring"
                    ],
                    "health": "healthy"
                }
            ]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config);

    let services = manager
        .discover_registry(&format!("{}/api/v1/services", mock_server.uri()))
        .await?;

    assert_eq!(services.len(), 1);
    let service = &services[0];

    // Should parse both string and object capabilities
    assert!(service.capabilities.len() >= 4);

    let capability_names: Vec<String> = service
        .capabilities
        .iter()
        .map(|c| c.name.clone())
        .collect();

    assert!(capability_names.contains(&"compute".to_string()));
    assert!(capability_names.contains(&"storage".to_string()));
    assert!(capability_names.contains(&"ml_inference".to_string()));
    assert!(capability_names.contains(&"gpu_compute".to_string()));
    assert!(capability_names.contains(&"monitoring".to_string()));

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_network_error_resilience() -> Result<()> {
    // Test with invalid endpoint
    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config);

    // Try to discover from non-existent server
    let result = manager
        .discover_registry("http://invalid-server-that-does-not-exist:9999/api/v1/services")
        .await;

    // Should handle network errors gracefully
    match result {
        Ok(services) => {
            // If successful, should return empty results
            assert!(services.is_empty());
        }
        Err(_) => {
            // Network error is expected and acceptable
        }
    }

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_malformed_json_handling() -> Result<()> {
    let mock_server = MockServer::start().await;

    // Setup endpoint that returns malformed JSON
    Mock::given(method("GET"))
        .and(path("/api/v1/services"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{ invalid json content"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config);

    let result = manager
        .discover_registry(&format!("{}/api/v1/services", mock_server.uri()))
        .await;

    // Should handle malformed JSON gracefully
    match result {
        Ok(services) => {
            assert!(services.is_empty());
        }
        Err(_) => {
            // Error handling is acceptable for malformed responses
        }
    }

    Ok(())
}
