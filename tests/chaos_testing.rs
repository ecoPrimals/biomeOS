//! Chaos Testing for BiomeOS
//!
//! Comprehensive chaos testing to verify system resilience under various failure conditions,
//! including network partitions, service failures, resource exhaustion, and recovery scenarios.

use anyhow::Result;
use biomeos_core::{
    config::{BiomeOSConfig, DiscoveryMethod, Environment},
    integration::live_service::LiveService,
    universal_biomeos_manager::{HealthStatus, PrimalInfo, UniversalBiomeOSManager},
};
use biomeos_types::{PrimalCapability, Health, PrimalType};
use serde_json::json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::{sleep, timeout, Instant};
use tracing_test::traced_test;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate, Times};

/// Chaos testing scenarios
#[derive(Debug, Clone)]
enum ChaosScenario {
    /// Network partition - services become unreachable
    NetworkPartition,
    /// Service degradation - services respond slowly
    ServiceDegradation,
    /// Intermittent failures - services fail randomly
    IntermittentFailures,
    /// Resource exhaustion - services report high resource usage
    ResourceExhaustion,
    /// Cascade failures - failures propagate through system
    CascadeFailures,
    /// Recovery testing - services come back online
    RecoveryTesting,
}

/// Chaos test configuration
struct ChaosTestConfig {
    scenario: ChaosScenario,
    duration: Duration,
    failure_rate: f64, // 0.0 to 1.0
    recovery_time: Duration,
}

/// Mock server that can simulate various failure modes
struct ChaosMockServer {
    server: MockServer,
    failure_rate: Arc<AtomicUsize>, // Percentage as integer (0-100)
    is_degraded: Arc<AtomicBool>,
    is_partitioned: Arc<AtomicBool>,
    request_count: Arc<AtomicUsize>,
}

impl ChaosMockServer {
    async fn new() -> Self {
        let server = MockServer::start().await;
        let failure_rate = Arc::new(AtomicUsize::new(0));
        let is_degraded = Arc::new(AtomicBool::new(false));
        let is_partitioned = Arc::new(AtomicBool::new(false));
        let request_count = Arc::new(AtomicUsize::new(0));

        Self {
            server,
            failure_rate,
            is_degraded,
            is_partitioned,
            request_count,
        }
    }

    /// Setup mock responses with chaos behavior
    async fn setup_chaos_responses(&self) {
        let failure_rate = Arc::clone(&self.failure_rate);
        let is_degraded = Arc::clone(&self.is_degraded);
        let is_partitioned = Arc::clone(&self.is_partitioned);
        let request_count = Arc::clone(&self.request_count);

        // Health endpoint with chaos behavior
        let health_failure_rate = Arc::clone(&failure_rate);
        let health_is_degraded = Arc::clone(&is_degraded);
        let health_is_partitioned = Arc::clone(&is_partitioned);
        let health_request_count = Arc::clone(&request_count);

        Mock::given(method("GET"))
            .and(path("/api/v1/health"))
            .respond_with_function(move |_req| {
                let count = health_request_count.fetch_add(1, Ordering::SeqCst);
                let failure_pct = health_failure_rate.load(Ordering::SeqCst);
                let is_degraded = health_is_degraded.load(Ordering::SeqCst);
                let is_partitioned = health_is_partitioned.load(Ordering::SeqCst);

                // Network partition simulation
                if is_partitioned {
                    return ResponseTemplate::new(0).set_delay(Duration::from_secs(30));
                    // Simulate network timeout
                }

                // Random failures based on failure rate
                if count % 100 < failure_pct {
                    return ResponseTemplate::new(500).set_body_json(json!({
                        "error": "Internal server error",
                        "chaos_simulation": true,
                        "request_count": count
                    }));
                }

                // Service degradation
                let delay = if is_degraded {
                    Duration::from_millis(2000 + (count % 3) * 1000) // 2-5 second delays
                } else {
                    Duration::from_millis(50 + (count % 10) * 10) // Normal response times
                };

                let status = if is_degraded && count % 3 == 0 {
                    "degraded"
                } else {
                    "healthy"
                };

                ResponseTemplate::new(200)
                    .set_body_json(json!({
                        "status": status,
                        "api_version": "1.0.0",
                        "capabilities": ["compute", "chaos-test"],
                        "uptime": format!("{}s", count * 10),
                        "request_count": count,
                        "chaos_mode": is_degraded
                    }))
                    .set_delay(delay)
            })
            .expect(1..)
            .mount(&self.server)
            .await;

        // Services endpoint with chaos behavior
        let services_failure_rate = Arc::clone(&failure_rate);
        let services_is_degraded = Arc::clone(&is_degraded);
        let services_is_partitioned = Arc::clone(&is_partitioned);

        Mock::given(method("GET"))
            .and(path("/api/v1/services"))
            .respond_with_function(move |_req| {
                let failure_pct = services_failure_rate.load(Ordering::SeqCst);
                let is_degraded = services_is_degraded.load(Ordering::SeqCst);
                let is_partitioned = services_is_partitioned.load(Ordering::SeqCst);

                if is_partitioned {
                    return ResponseTemplate::new(0).set_delay(Duration::from_secs(30));
                }

                let random_val = rand::random::<u32>() % 100;
                if random_val < failure_pct as u32 {
                    return ResponseTemplate::new(503).set_body_json(json!({
                        "error": "Service temporarily unavailable",
                        "chaos_mode": true
                    }));
                }

                let services = if is_degraded {
                    // Return fewer services during degradation
                    json!({
                        "services": [
                            {
                                "name": "surviving-service",
                                "type": "compute",
                                "endpoint": "http://localhost:8001",
                                "capabilities": ["compute"],
                                "health": "degraded",
                                "load": 0.9
                            }
                        ],
                        "total_services": 1,
                        "degraded": true
                    })
                } else {
                    json!({
                        "services": [
                            {
                                "name": "service-1",
                                "type": "compute",
                                "endpoint": "http://localhost:8001",
                                "capabilities": ["compute", "scale"],
                                "health": "healthy",
                                "load": 0.3
                            },
                            {
                                "name": "service-2",
                                "type": "orchestration",
                                "endpoint": "http://localhost:8002",
                                "capabilities": ["service_discovery", "routing"],
                                "health": "healthy",
                                "load": 0.2
                            },
                            {
                                "name": "service-3",
                                "type": "storage",
                                "endpoint": "http://localhost:8003",
                                "capabilities": ["storage", "backup"],
                                "health": "healthy",
                                "load": 0.1
                            }
                        ],
                        "total_services": 3
                    })
                };

                let delay = if is_degraded {
                    Duration::from_millis(1000 + random_val as u64 * 10)
                } else {
                    Duration::from_millis(100)
                };

                ResponseTemplate::new(200)
                    .set_body_json(services)
                    .set_delay(delay)
            })
            .expect(1..)
            .mount(&self.server)
            .await;
    }

    fn set_failure_rate(&self, rate: f64) {
        self.failure_rate
            .store((rate * 100.0) as usize, Ordering::SeqCst);
    }

    fn set_degraded(&self, degraded: bool) {
        self.is_degraded.store(degraded, Ordering::SeqCst);
    }

    fn set_partitioned(&self, partitioned: bool) {
        self.is_partitioned.store(partitioned, Ordering::SeqCst);
    }

    fn get_request_count(&self) -> usize {
        self.request_count.load(Ordering::SeqCst)
    }

    fn uri(&self) -> String {
        self.server.uri()
    }
}

#[traced_test]
#[tokio::test]
async fn test_network_partition_resilience() -> Result<()> {
    let chaos_server = ChaosMockServer::new().await;
    chaos_server.setup_chaos_responses().await;

    // Configure manager to use chaos server
    let mut config = BiomeOSConfig::default();
    config.system.environment = Environment::Testing;
    config.primals.discovery.method = DiscoveryMethod::Registry {
        url: chaos_server.uri(),
    };
    config.primals.timeouts.discovery_timeout_ms = 3000; // Short timeout for testing

    let manager = UniversalBiomeOSManager::new(config);

    // Initial discovery should work
    let initial_services = manager
        .discover_registry(&format!("{}/api/v1/services", chaos_server.uri()))
        .await?;
    assert!(
        !initial_services.is_empty(),
        "Should discover services initially"
    );

    // Simulate network partition
    chaos_server.set_partitioned(true);

    // Discovery should fail gracefully during partition
    let start_time = Instant::now();
    let partitioned_result = timeout(
        Duration::from_secs(5),
        manager.discover_registry(&format!("{}/api/v1/services", chaos_server.uri())),
    )
    .await;

    let elapsed = start_time.elapsed();
    assert!(
        elapsed < Duration::from_secs(4),
        "Should timeout quickly during partition"
    );

    match partitioned_result {
        Ok(Ok(services)) => {
            // If successful, should return empty or cached results
            // This is acceptable behavior during network issues
        }
        Ok(Err(_)) => {
            // Network error is expected during partition
        }
        Err(_) => {
            // Timeout is expected and acceptable
        }
    }

    // Recovery: remove partition
    chaos_server.set_partitioned(false);

    // Give system time to recover
    sleep(Duration::from_millis(500)).await;

    // Should recover and work again
    let recovered_services = manager
        .discover_registry(&format!("{}/api/v1/services", chaos_server.uri()))
        .await?;
    assert!(
        !recovered_services.is_empty(),
        "Should recover after partition ends"
    );

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_intermittent_failures_resilience() -> Result<()> {
    let chaos_server = ChaosMockServer::new().await;
    chaos_server.setup_chaos_responses().await;

    // Set 30% failure rate
    chaos_server.set_failure_rate(0.3);

    let mut config = BiomeOSConfig::default();
    config.system.environment = Environment::Testing;
    config.primals.discovery.method = DiscoveryMethod::Registry {
        url: chaos_server.uri(),
    };

    let manager = UniversalBiomeOSManager::new(config);

    // Run multiple discovery attempts
    let mut success_count = 0;
    let mut failure_count = 0;
    let attempts = 20;

    for _i in 0..attempts {
        let result = timeout(
            Duration::from_secs(2),
            manager.discover_registry(&format!("{}/api/v1/services", chaos_server.uri())),
        )
        .await;

        match result {
            Ok(Ok(services)) => {
                if !services.is_empty() {
                    success_count += 1;
                }
            }
            Ok(Err(_)) => {
                failure_count += 1;
            }
            Err(_) => {
                failure_count += 1;
            }
        }

        // Small delay between attempts
        sleep(Duration::from_millis(100)).await;
    }

    // Should have some successes despite failures
    assert!(
        success_count > 0,
        "Should have some successful discoveries despite failures"
    );

    // Failure rate should be reasonable (not all requests failing)
    let total_responses = success_count + failure_count;
    let actual_failure_rate = failure_count as f64 / total_responses as f64;

    println!(
        "Actual failure rate: {:.2}% ({}/{} attempts)",
        actual_failure_rate * 100.0,
        failure_count,
        total_responses
    );

    // Should be resilient enough to handle intermittent failures
    assert!(
        success_count >= attempts / 4,
        "Should succeed at least 25% of the time"
    );

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_service_degradation_handling() -> Result<()> {
    let chaos_server = ChaosMockServer::new().await;
    chaos_server.setup_chaos_responses().await;

    // Enable degradation mode
    chaos_server.set_degraded(true);

    let mut config = BiomeOSConfig::default();
    config.system.environment = Environment::Testing;
    config.primals.timeouts.discovery_timeout_ms = 5000; // Longer timeout for degraded services

    let manager = UniversalBiomeOSManager::new(config);

    // Test discovery during degradation
    let start_time = Instant::now();
    let degraded_result = manager
        .discover_registry(&format!("{}/api/v1/services", chaos_server.uri()))
        .await;
    let elapsed = start_time.elapsed();

    match degraded_result {
        Ok(services) => {
            // Should eventually succeed despite slowness
            // May return fewer services during degradation
            println!(
                "Discovered {} services during degradation in {:?}",
                services.len(),
                elapsed
            );

            // Should take longer than normal but not timeout
            assert!(
                elapsed > Duration::from_millis(500),
                "Should be slower during degradation"
            );
            assert!(elapsed < Duration::from_secs(8), "Should not timeout");
        }
        Err(e) => {
            // Some failures are acceptable during severe degradation
            println!("Discovery failed during degradation: {}", e);
        }
    }

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_cascade_failure_simulation() -> Result<()> {
    // Create multiple chaos servers to simulate cascade failures
    let primary_server = ChaosMockServer::new().await;
    let secondary_server = ChaosMockServer::new().await;

    primary_server.setup_chaos_responses().await;
    secondary_server.setup_chaos_responses().await;

    let mut config = BiomeOSConfig::default();
    config.system.environment = Environment::Testing;

    let manager = UniversalBiomeOSManager::new(config);

    // Start with both services healthy
    let initial_primary = manager
        .discover_registry(&format!("{}/api/v1/services", primary_server.uri()))
        .await?;
    let initial_secondary = manager
        .discover_registry(&format!("{}/api/v1/services", secondary_server.uri()))
        .await?;

    assert!(!initial_primary.is_empty());
    assert!(!initial_secondary.is_empty());

    // Simulate cascade failure: primary fails, secondary degrades
    primary_server.set_failure_rate(1.0); // 100% failure
    secondary_server.set_degraded(true);

    // Test resilience during cascade failure
    let cascade_start = Instant::now();

    // Primary should fail
    let primary_result = timeout(
        Duration::from_secs(3),
        manager.discover_registry(&format!("{}/api/v1/services", primary_server.uri())),
    )
    .await;

    let primary_failed = match primary_result {
        Ok(Ok(services)) => services.is_empty(),
        Ok(Err(_)) => true,
        Err(_) => true,
    };

    assert!(primary_failed, "Primary service should fail during cascade");

    // Secondary should be degraded but potentially still work
    let secondary_result = timeout(
        Duration::from_secs(8),
        manager.discover_registry(&format!("{}/api/v1/services", secondary_server.uri())),
    )
    .await;

    match secondary_result {
        Ok(Ok(services)) => {
            // May return fewer services or degraded status
            println!(
                "Secondary service returned {} services during cascade",
                services.len()
            );
        }
        Ok(Err(_)) => {
            println!("Secondary service failed during cascade (acceptable)");
        }
        Err(_) => {
            println!("Secondary service timed out during cascade");
        }
    }

    let cascade_duration = cascade_start.elapsed();
    println!("Cascade failure handled in {:?}", cascade_duration);

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_recovery_after_failures() -> Result<()> {
    let chaos_server = ChaosMockServer::new().await;
    chaos_server.setup_chaos_responses().await;

    let mut config = BiomeOSConfig::default();
    config.system.environment = Environment::Testing;

    let manager = UniversalBiomeOSManager::new(config);

    // Start healthy
    let initial_services = manager
        .discover_registry(&format!("{}/api/v1/services", chaos_server.uri()))
        .await?;
    assert!(!initial_services.is_empty(), "Should start healthy");

    // Introduce failures
    chaos_server.set_failure_rate(0.8); // 80% failure rate
    chaos_server.set_degraded(true);

    // System should struggle during failures
    let mut failure_period_attempts = 0;
    let mut failure_period_successes = 0;

    for _i in 0..10 {
        let result = timeout(
            Duration::from_secs(2),
            manager.discover_registry(&format!("{}/api/v1/services", chaos_server.uri())),
        )
        .await;

        failure_period_attempts += 1;
        if result.is_ok() {
            if let Ok(Ok(services)) = result {
                if !services.is_empty() {
                    failure_period_successes += 1;
                }
            }
        }

        sleep(Duration::from_millis(100)).await;
    }

    println!(
        "During failure period: {}/{} successes",
        failure_period_successes, failure_period_attempts
    );

    // Recovery: restore healthy state
    chaos_server.set_failure_rate(0.0);
    chaos_server.set_degraded(false);

    // Allow time for recovery
    sleep(Duration::from_millis(500)).await;

    // System should recover
    let recovery_start = Instant::now();
    let mut recovery_attempts = 0;
    let mut recovery_successes = 0;

    for _i in 0..10 {
        let result = manager
            .discover_registry(&format!("{}/api/v1/services", chaos_server.uri()))
            .await;
        recovery_attempts += 1;

        if let Ok(services) = result {
            if !services.is_empty() {
                recovery_successes += 1;
            }
        }

        sleep(Duration::from_millis(100)).await;
    }

    let recovery_duration = recovery_start.elapsed();
    println!(
        "Recovery period: {}/{} successes in {:?}",
        recovery_successes, recovery_attempts, recovery_duration
    );

    // Should recover to high success rate
    assert!(
        recovery_successes >= recovery_attempts * 8 / 10,
        "Should recover to at least 80% success rate after failures end"
    );

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_concurrent_chaos_resilience() -> Result<()> {
    let chaos_server = ChaosMockServer::new().await;
    chaos_server.setup_chaos_responses().await;

    // Set moderate failure rate
    chaos_server.set_failure_rate(0.4);

    let mut config = BiomeOSConfig::default();
    config.system.environment = Environment::Testing;

    let manager = UniversalBiomeOSManager::new(config);

    // Run many concurrent discovery operations during chaos
    let concurrent_tasks = 20;
    let mut handles = Vec::new();

    for i in 0..concurrent_tasks {
        let manager_ref = &manager;
        let server_uri = chaos_server.uri();

        handles.push(tokio::spawn(async move {
            let mut local_successes = 0;
            let mut local_failures = 0;

            // Each task tries multiple times
            for _attempt in 0..5 {
                let result = timeout(
                    Duration::from_secs(3),
                    manager_ref.discover_registry(&format!("{}/api/v1/services", server_uri)),
                )
                .await;

                match result {
                    Ok(Ok(services)) if !services.is_empty() => local_successes += 1,
                    _ => local_failures += 1,
                }

                sleep(Duration::from_millis(50)).await;
            }

            (i, local_successes, local_failures)
        }));
    }

    // Collect results
    let mut total_successes = 0;
    let mut total_failures = 0;

    for handle in handles {
        let (task_id, successes, failures) = timeout(Duration::from_secs(30), handle).await??;
        total_successes += successes;
        total_failures += failures;

        println!(
            "Task {}: {}/{} successes",
            task_id,
            successes,
            successes + failures
        );
    }

    let total_attempts = total_successes + total_failures;
    let success_rate = total_successes as f64 / total_attempts as f64;

    println!(
        "Overall concurrent chaos test: {:.1}% success rate ({}/{})",
        success_rate * 100.0,
        total_successes,
        total_attempts
    );

    // Should handle concurrent load reasonably well despite chaos
    assert!(
        success_rate > 0.3,
        "Should succeed at least 30% of the time under concurrent chaos"
    );
    assert!(total_attempts > 50, "Should have attempted many operations");

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_health_monitoring_under_chaos() -> Result<()> {
    let chaos_server = ChaosMockServer::new().await;
    chaos_server.setup_chaos_responses().await;

    // Set degradation mode
    chaos_server.set_degraded(true);
    chaos_server.set_failure_rate(0.2);

    let mut config = BiomeOSConfig::default();
    config.system.environment = Environment::Testing;

    let mut live_service = LiveService::new(config).await?;
    live_service.start().await?;

    // Register some test primals pointing to chaos server
    let manager = &live_service.universal_manager;

    let chaos_primal = PrimalInfo {
        id: "chaos-test-service".to_string(),
        primal_type: PrimalType::new("chaos", "test", "1.0.0"),
        capabilities: vec![PrimalCapability::custom("chaos", "Chaos testing")],
        health: Health::Healthy,
        endpoint: format!("{}/api/v1/health", chaos_server.uri()),
    };

    manager
        .register_primal("chaos-test".to_string(), chaos_primal)
        .await?;

    // Monitor health during chaos
    let monitoring_duration = Duration::from_secs(5);
    let monitoring_start = Instant::now();
    let mut health_checks = Vec::new();

    while monitoring_start.elapsed() < monitoring_duration {
        let system_health = manager.get_system_health().await;
        health_checks.push((monitoring_start.elapsed(), system_health));

        sleep(Duration::from_millis(500)).await;
    }

    // Analyze health monitoring results
    assert!(
        !health_checks.is_empty(),
        "Should have collected health data"
    );

    let healthy_checks = health_checks
        .iter()
        .filter(|(_, health)| matches!(health.overall_status, HealthStatus::Healthy))
        .count();

    let degraded_checks = health_checks
        .iter()
        .filter(|(_, health)| {
            matches!(
                health.overall_status,
                HealthStatus::Degraded | HealthStatus::Warning
            )
        })
        .count();

    println!(
        "Health monitoring during chaos: {} healthy, {} degraded, {} total",
        healthy_checks,
        degraded_checks,
        health_checks.len()
    );

    // System should detect degradation or maintain basic functionality
    assert!(
        healthy_checks + degraded_checks == health_checks.len(),
        "All health checks should return valid status"
    );

    // Resource usage should remain within bounds even during chaos
    for (elapsed, health) in &health_checks {
        let usage = &health.resource_usage;
        assert!(
            usage.cpu_usage_percent >= 0.0 && usage.cpu_usage_percent <= 800.0,
            "CPU usage should be valid at {:?}",
            elapsed
        );
        assert!(
            usage.memory_usage_percent >= 0.0 && usage.memory_usage_percent <= 100.0,
            "Memory usage should be valid at {:?}",
            elapsed
        );
    }

    Ok(())
}
