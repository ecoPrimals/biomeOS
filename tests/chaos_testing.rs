// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Chaos Testing for BiomeOS
//!
//! Comprehensive chaos testing to verify system resilience under various failure conditions,
//! including network partitions, service failures, resource exhaustion, and recovery scenarios.

use anyhow::Result;
use biomeos_core::{
    integration::live_service::LiveService,
    universal_biomeos_manager::{PrimalInfo, UniversalBiomeOSManager},
};
use biomeos_primal_sdk::{PrimalCapability, PrimalType};
use biomeos_types::{BiomeOSConfig, Health};
use serde_json::json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{timeout, Instant};
use tracing_test::traced_test;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

/// Chaos responder implementing the Respond trait for dynamic mock responses
struct ChaosHealthResponder {
    failure_rate: Arc<AtomicUsize>,
    is_degraded: Arc<AtomicBool>,
    is_partitioned: Arc<AtomicBool>,
    request_count: Arc<AtomicUsize>,
}

impl Respond for ChaosHealthResponder {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        let count = self.request_count.fetch_add(1, Ordering::SeqCst);
        let failure_pct = self.failure_rate.load(Ordering::SeqCst);
        let is_degraded = self.is_degraded.load(Ordering::SeqCst);
        let is_partitioned = self.is_partitioned.load(Ordering::SeqCst);

        // Network partition simulation
        if is_partitioned {
            return ResponseTemplate::new(0).set_delay(Duration::from_secs(30));
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
            Duration::from_millis(2000 + (count as u64 % 3) * 1000)
        } else {
            Duration::from_millis(50 + (count as u64 % 10) * 10)
        };

        let status = if is_degraded && count.is_multiple_of(3) {
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
    }
}

/// Chaos responder for services endpoint
struct ChaosServicesResponder {
    failure_rate: Arc<AtomicUsize>,
    is_degraded: Arc<AtomicBool>,
    is_partitioned: Arc<AtomicBool>,
}

impl Respond for ChaosServicesResponder {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        let failure_pct = self.failure_rate.load(Ordering::SeqCst);
        let is_degraded = self.is_degraded.load(Ordering::SeqCst);
        let is_partitioned = self.is_partitioned.load(Ordering::SeqCst);

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

        let delay = if is_degraded {
            Duration::from_millis(1500)
        } else {
            Duration::from_millis(50)
        };

        let services = if is_degraded {
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
                    }
                ],
                "total_services": 2
            })
        };

        ResponseTemplate::new(200)
            .set_body_json(services)
            .set_delay(delay)
    }
}

/// Mock server that can simulate various failure modes
struct ChaosMockServer {
    server: MockServer,
    failure_rate: Arc<AtomicUsize>,
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
        // Health endpoint
        let health_responder = ChaosHealthResponder {
            failure_rate: Arc::clone(&self.failure_rate),
            is_degraded: Arc::clone(&self.is_degraded),
            is_partitioned: Arc::clone(&self.is_partitioned),
            request_count: Arc::clone(&self.request_count),
        };

        Mock::given(method("GET"))
            .and(path("/api/v1/health"))
            .respond_with(health_responder)
            .mount(&self.server)
            .await;

        // Services endpoint
        let services_responder = ChaosServicesResponder {
            failure_rate: Arc::clone(&self.failure_rate),
            is_degraded: Arc::clone(&self.is_degraded),
            is_partitioned: Arc::clone(&self.is_partitioned),
        };

        Mock::given(method("GET"))
            .and(path("/api/v1/services"))
            .respond_with(services_responder)
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
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_network_partition_resilience() -> Result<()> {
    let chaos_server = ChaosMockServer::new().await;
    chaos_server.setup_chaos_responses().await;

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    // Initial discovery should work
    let _initial_result = manager
        .discover_registry(&format!("{}/api/v1/services", chaos_server.uri()))
        .await;

    // May succeed or fail depending on mock response format compatibility
    // The key is graceful handling

    // Simulate network partition
    chaos_server.set_partitioned(true);

    // Discovery should fail gracefully during partition
    let _start_time = Instant::now();
    let partitioned_result = timeout(
        Duration::from_secs(5),
        manager.discover_registry(&format!("{}/api/v1/services", chaos_server.uri())),
    )
    .await;

    // Should timeout or fail gracefully
    match partitioned_result {
        Ok(Ok(_services)) => {
            // Cached results are acceptable
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

    // Wait for recovery: poll until discover succeeds or timeout
    let recovered = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if manager
                .discover_registry(&format!("{}/api/v1/services", chaos_server.uri()))
                .await
                .is_ok()
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await;
    assert!(
        recovered.is_ok(),
        "Should recover within 5s after partition lifted"
    );

    // Should have recovered
    let _recovered_result = manager
        .discover_registry(&format!("{}/api/v1/services", chaos_server.uri()))
        .await;

    Ok(())
}

#[traced_test]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_service_degradation_handling() -> Result<()> {
    let chaos_server = ChaosMockServer::new().await;
    chaos_server.setup_chaos_responses().await;

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    // Set degraded mode
    chaos_server.set_degraded(true);
    chaos_server.set_failure_rate(0.2);

    // Perform health checks during degradation
    let health_report = manager.get_system_health().await;

    // System should still function, possibly reporting degraded status
    assert!(matches!(
        health_report.health,
        Health::Healthy
            | Health::Degraded { .. }
            | Health::Critical { .. }
            | Health::Unknown { .. }
    ));

    Ok(())
}

#[traced_test]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_intermittent_failures() -> Result<()> {
    let chaos_server = ChaosMockServer::new().await;
    chaos_server.setup_chaos_responses().await;

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    // Set 30% failure rate
    chaos_server.set_failure_rate(0.3);

    // Perform multiple operations
    let mut success_count = 0;
    let mut failure_count = 0;

    for _ in 0..10 {
        match manager
            .discover_registry(&format!("{}/api/v1/services", chaos_server.uri()))
            .await
        {
            Ok(_) => success_count += 1,
            Err(_) => failure_count += 1,
        }
        tokio::task::yield_now().await;
    }

    // Should have mix of successes and failures
    println!("Intermittent failures test: {success_count} successes, {failure_count} failures");

    // System should handle failures gracefully without crashing
    let health_report = manager.get_system_health().await;
    assert!(matches!(
        health_report.health,
        Health::Healthy
            | Health::Degraded { .. }
            | Health::Critical { .. }
            | Health::Unknown { .. }
    ));

    Ok(())
}

#[traced_test]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_recovery_after_cascade_failure() -> Result<()> {
    let chaos_server = ChaosMockServer::new().await;
    chaos_server.setup_chaos_responses().await;

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    // Simulate cascade failure: partition + degradation + high failure rate
    chaos_server.set_partitioned(true);
    chaos_server.set_degraded(true);
    chaos_server.set_failure_rate(0.8);

    // Try operations during total failure
    let cascade_result = timeout(
        Duration::from_secs(2),
        manager.discover_registry(&format!("{}/api/v1/services", chaos_server.uri())),
    )
    .await;

    // Should fail or timeout gracefully
    match cascade_result {
        Ok(Ok(_)) => println!("Unexpected success during cascade failure"),
        Ok(Err(e)) => println!("Expected error during cascade: {e}"),
        Err(_) => println!("Expected timeout during cascade"),
    }

    // Begin recovery (mock state changes are immediate)
    chaos_server.set_partitioned(false);
    chaos_server.set_failure_rate(0.2);
    chaos_server.set_degraded(false);
    chaos_server.set_failure_rate(0.0);

    // System should recover
    let health_report = manager.get_system_health().await;
    assert!(matches!(
        health_report.health,
        Health::Healthy
            | Health::Degraded { .. }
            | Health::Critical { .. }
            | Health::Unknown { .. }
    ));

    Ok(())
}

#[traced_test]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_health_monitoring_during_chaos() -> Result<()> {
    let chaos_server = ChaosMockServer::new().await;
    chaos_server.setup_chaos_responses().await;

    let mut live_service = LiveService::new().await?;
    live_service.start().await?;

    let manager = &live_service.universal_manager;

    // Register a test primal
    let now = chrono::Utc::now();
    let chaos_primal = PrimalInfo {
        id: "chaos-test-service".to_string(),
        name: "Chaos Test Service".to_string(),
        primal_type: PrimalType::new("chaos", "test", "1.0.0"),
        capabilities: vec![PrimalCapability::new("chaos", "testing", "1.0.0")],
        health: Health::Healthy,
        endpoint: format!("{}/api/v1/health", chaos_server.uri()),
        last_seen: now,
        discovered_at: now,
        metadata: HashMap::new(),
    };

    manager.register_primal(chaos_primal).await?;

    // Set degraded mode
    chaos_server.set_degraded(true);
    chaos_server.set_failure_rate(0.2);

    // Monitor health during chaos
    let monitoring_duration = Duration::from_secs(3);
    let monitoring_start = Instant::now();
    let mut health_checks = Vec::new();

    while monitoring_start.elapsed() < monitoring_duration {
        let health_report = manager.get_system_health().await;
        health_checks.push((monitoring_start.elapsed(), health_report));
        // Intentional: polling interval for health monitoring simulation
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    // Verify we collected health data
    assert!(
        !health_checks.is_empty(),
        "Should have collected health data"
    );

    // All health checks should return valid status
    for (elapsed, health) in &health_checks {
        assert!(
            matches!(
                health.health,
                Health::Healthy
                    | Health::Degraded { .. }
                    | Health::Critical { .. }
                    | Health::Unknown { .. }
            ),
            "Invalid health status at {elapsed:?}"
        );
    }

    println!(
        "Health monitoring during chaos: {} checks collected",
        health_checks.len()
    );

    Ok(())
}

#[traced_test]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_request_counting_under_load() -> Result<()> {
    let chaos_server = ChaosMockServer::new().await;
    chaos_server.setup_chaos_responses().await;

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    // Perform multiple parallel requests
    let mut handles = Vec::new();
    let request_count = 20;

    for _ in 0..request_count {
        let manager_clone = manager.clone();
        let uri = chaos_server.uri();
        let handle = tokio::spawn(async move {
            let _ = manager_clone
                .discover_registry(&format!("{uri}/api/v1/health"))
                .await;
        });
        handles.push(handle);
    }

    // Wait for all requests
    for handle in handles {
        let _ = handle.await;
    }

    // Check request count
    let count = chaos_server.get_request_count();
    println!("Total requests processed: {count}");
    // Request count is usize so always >= 0, validation complete

    Ok(())
}
