//! Integration tests for biomeos-cli discovery commands
//!
//! Tests service discovery functionality with mock primals.

use anyhow::Result;
use biomeos_test_utils::MockPrimal;

#[tokio::test]
async fn test_discovery_capability_based() -> Result<()> {
    // Test capability-based discovery
    let mock = MockPrimal::builder("capability-test")
        .port(0)
        .capability("compute")
        .capability("storage")
        .build()
        .start()
        .await?;

    // Query capabilities
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/v1/capabilities", mock.url()))
        .send()
        .await?;

    assert!(response.status().is_success());
    
    let capabilities: Vec<String> = response.json().await?;
    assert_eq!(capabilities.len(), 2);
    assert!(capabilities.contains(&"compute".to_string()));
    assert!(capabilities.contains(&"storage".to_string()));

    mock.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_discovery_multiple_services() -> Result<()> {
    // Discover multiple services with different capabilities
    let mocks: Vec<MockPrimal> = vec![
        MockPrimal::builder("service-compute")
            .port(0)
            .capability("compute")
            .build()
            .start()
            .await?,
        MockPrimal::builder("service-storage")
            .port(0)
            .capability("storage")
            .build()
            .start()
            .await?,
        MockPrimal::builder("service-network")
            .port(0)
            .capability("network")
            .build()
            .start()
            .await?,
    ];

    // Each service should be discoverable
    let client = reqwest::Client::new();
    for mock in &mocks {
        let response = client
            .get(format!("{}/api/v1/capabilities", mock.url()))
            .send()
            .await?;
        assert!(response.status().is_success());
    }

    // Cleanup
    for mock in mocks {
        mock.stop().await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_discovery_endpoint_probe() -> Result<()> {
    // Test probing a specific endpoint
    let mock = MockPrimal::builder("probe-test")
        .port(0)
        .capability("api")
        .build()
        .start()
        .await?;

    // Probe the endpoint
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/health", mock.url()))
        .send()
        .await?;

    assert!(response.status().is_success());

    mock.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_discovery_detailed_info() -> Result<()> {
    // Test getting detailed service information
    let mock = MockPrimal::builder("detailed-discovery")
        .port(0)
        .capability("compute")
        .capability("storage")
        .capability("network")
        .build()
        .start()
        .await?;

    // Get health (basic info)
    let client = reqwest::Client::new();
    let health_response = client
        .get(format!("{}/health", mock.url()))
        .send()
        .await?;

    assert!(health_response.status().is_success());

    // Get capabilities (detailed info)
    let caps_response = client
        .get(format!("{}/api/v1/capabilities", mock.url()))
        .send()
        .await?;

    assert!(caps_response.status().is_success());
    
    let capabilities: Vec<String> = caps_response.json().await?;
    assert_eq!(capabilities.len(), 3);

    mock.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_discovery_registry_based() -> Result<()> {
    // Test registry-based discovery pattern
    // Registry would aggregate multiple services
    let registry = MockPrimal::builder("registry")
        .port(0)
        .capability("registry")
        .capability("discovery")
        .build()
        .start()
        .await?;

    // Registry should be queryable
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/health", registry.url()))
        .send()
        .await?;

    assert!(response.status().is_success());

    registry.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_discovery_no_services() -> Result<()> {
    // Test behavior when no services are available
    // This tests graceful handling of empty discovery
    
    // Try to connect to non-existent service
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(100))
        .build()?;

    let result = client
        .get("http://localhost:9999/health")
        .send()
        .await;

    // Should fail to connect (no service running)
    assert!(result.is_err());

    Ok(())
}

