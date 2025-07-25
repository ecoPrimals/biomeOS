use biomeos_core::{universal_biomeos_manager::*, BiomeOSConfig};
use biomeos_primal_sdk::PrimalHealth;
use serde_json::json;
use wiremock::{
    matchers::{header, method, path},
    Mock, MockServer, ResponseTemplate,
};

/// Test the registry discovery with a mock server
#[tokio::test]
async fn test_registry_discovery_success() {
    // Create mock registry server
    let mock_server = MockServer::start().await;

    // Mock registry response
    Mock::given(method("GET"))
        .and(header("User-Agent", "BiomeOS-Universal-Manager/1.0"))
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

    // Create a manager to test discovery methods
    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config);

    // Access the discovery service methods through the manager's public API
    let results = manager.discover_registry(&mock_server.uri()).await.unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].id, "compute-service");
    assert_eq!(results[1].id, "storage-service");
    assert!(results[0].capabilities.len() >= 3);
    assert!(results[1].capabilities.len() >= 2);
}

#[tokio::test]
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
    let manager = UniversalBiomeOSManager::new(config);
    let results = manager.discover_registry(&mock_server.uri()).await.unwrap();

    assert!(results.is_empty());
}

#[tokio::test]
async fn test_registry_discovery_malformed_response() {
    let mock_server = MockServer::start().await;

    // Mock malformed JSON
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("invalid json"))
        .mount(&mock_server)
        .await;

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config);
    let results = manager.discover_registry(&mock_server.uri()).await.unwrap();

    // Should handle gracefully and return empty results
    assert!(results.is_empty());
}

#[tokio::test]
async fn test_capability_based_orchestration_discovery_success() {
    let mock_server = MockServer::start().await;

    // Mock universal service discovery API (not tied to specific service names)
    Mock::given(method("GET"))
        .and(path("/api/v1/discovery/services"))
        .and(header("User-Agent", "BiomeOS-Universal-Manager/1.0"))
        .and(header("X-BiomeOS-Discovery", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "services": [
                {
                    "name": "orchestration-service",
                    "endpoint": "http://localhost:9000",
                    "type": "orchestration",
                    "health": "healthy",
                    "capabilities": ["routing", "load-balancing", "service-mesh"]
                },
                {
                    "name": "security-service",
                    "endpoint": "http://localhost:9001",
                    "type": "security",
                    "health": "degraded",
                    "capabilities": ["authentication", "authorization"]
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config);
    let results = manager
        .discover_orchestration_services(&mock_server.uri())
        .await
        .unwrap();

    // Only the orchestration-service should match the required capabilities
    // (service_discovery, message_routing, load_balancing)
    // The security-service doesn't have these capabilities, so it's filtered out
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "orchestration-service");
    assert_eq!(results[0].health, PrimalHealth::Healthy);
}

#[tokio::test]
async fn test_probe_endpoint_success() {
    let mock_server = MockServer::start().await;

    // Mock health endpoint
    Mock::given(method("GET"))
        .and(path("/api/v1/health"))
        .and(header("User-Agent", "BiomeOS-Primal-Discovery/1.0"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    // Mock metadata endpoint
    Mock::given(method("GET"))
        .and(path("/api/v1/metadata"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "name": "test-service",
            "category": "testing",
            "capabilities": ["unit-test", "integration-test"]
        })))
        .mount(&mock_server)
        .await;

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config);
    let result = manager.probe_endpoint(&mock_server.uri()).await.unwrap();

    assert_eq!(result.name, "test-service");
    assert_eq!(result.category, "testing");
    assert_eq!(result.capabilities.len(), 2);
}

#[tokio::test]
async fn test_health_monitoring_integration() {
    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config);

    // Test system health retrieval
    let system_health = manager.get_system_health().await;

    // Verify basic structure
    assert!(matches!(
        system_health.overall_status,
        HealthStatus::Healthy
            | HealthStatus::Degraded
            | HealthStatus::Warning
            | HealthStatus::Critical
    ));

    // Verify uptime is positive
    assert!(system_health.uptime.num_seconds() > 0);

    // Verify we have some primal health data
    assert!(!system_health.primal_health.is_empty());

    // Verify resource usage is within reasonable bounds
    let resource = &system_health.resource_usage;
    assert!(resource.cpu_usage_percent >= 0.0);
    assert!(resource.memory_usage_percent >= 0.0);
    assert!(resource.disk_usage_percent >= 0.0);
    assert!(resource.network_usage_mbps >= 0.0);
}
