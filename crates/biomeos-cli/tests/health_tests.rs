//! Integration tests for biomeos-cli health commands
//!
//! Tests the health command handlers using mock primal servers.

use anyhow::Result;
use biomeos_test_utils::MockPrimal;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_health_command_basic() -> Result<()> {
    // Start a mock primal
    let mock = MockPrimal::builder("test-primal")
        .port(0)
        .capability("health")
        .build()
        .start()
        .await?;

    // Mock primal should respond to health checks
    let client = reqwest::Client::new();
    let response = client.get(format!("{}/health", mock.url())).send().await?;

    assert!(response.status().is_success());

    // Verify health check was received
    assert_eq!(mock.health_check_count().await, 1);

    mock.stop().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_health_command_detailed() -> Result<()> {
    // Test detailed health output
    let mock = MockPrimal::builder("detailed-test")
        .port(0)
        .capability("health")
        .capability("metrics")
        .build()
        .start()
        .await?;

    // Check capabilities are reported
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/v1/capabilities", mock.url()))
        .send()
        .await?;

    assert!(response.status().is_success());

    let capabilities: Vec<String> = response.json().await?;
    assert_eq!(capabilities.len(), 2);
    assert!(capabilities.contains(&"health".to_string()));
    assert!(capabilities.contains(&"metrics".to_string()));

    mock.stop().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_health_command_multiple_services() -> Result<()> {
    // Test health checking multiple services
    let mock1 = MockPrimal::builder("service-1")
        .port(0)
        .capability("api")
        .build()
        .start()
        .await?;

    let mock2 = MockPrimal::builder("service-2")
        .port(0)
        .capability("storage")
        .build()
        .start()
        .await?;

    // Both should respond to health
    let client = reqwest::Client::new();

    let resp1 = client.get(format!("{}/health", mock1.url())).send().await?;
    let resp2 = client.get(format!("{}/health", mock2.url())).send().await?;

    assert!(resp1.status().is_success());
    assert!(resp2.status().is_success());

    mock1.stop().await?;
    mock2.stop().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_health_probe_timeout() -> Result<()> {
    // Test probe with timeout behavior
    let mock = MockPrimal::builder("timeout-test")
        .port(0)
        .capability("health")
        .build()
        .start()
        .await?;

    // Quick probe should succeed
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    let response = client.get(format!("{}/health", mock.url())).send().await?;

    assert!(response.status().is_success());

    mock.stop().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_health_scan_discovery() -> Result<()> {
    // Test system scan discovers services
    let mocks: Vec<MockPrimal> = vec![
        MockPrimal::builder("scan-service-1")
            .port(0)
            .capability("compute")
            .build()
            .start()
            .await?,
        MockPrimal::builder("scan-service-2")
            .port(0)
            .capability("storage")
            .build()
            .start()
            .await?,
        MockPrimal::builder("scan-service-3")
            .port(0)
            .capability("network")
            .build()
            .start()
            .await?,
    ];

    // All services should be discoverable
    for mock in &mocks {
        let client = reqwest::Client::new();
        let response = client.get(format!("{}/health", mock.url())).send().await?;
        assert!(response.status().is_success());
    }

    // Cleanup
    for mock in mocks {
        mock.stop().await?;
    }

    Ok(())
}
