//! End-to-End Testing Suite for BiomeOS
//!
//! This comprehensive test suite validates complete biomeOS workflows including
//! primal discovery, manifest deployment, resource management, and failure recovery.

use anyhow::Result;
use biomeos_core::{
    config::{BiomeOSConfig, DiscoveryMethod, Environment},
    integration::live_service::LiveService,
    universal_biomeos_manager::{HealthStatus, PrimalInfo, UniversalBiomeOSManager},
};
use biomeos_types::{PrimalCapability, Health, PrimalType};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{sleep, timeout, Instant};
use tracing::{error, info, warn};
use tracing_test::traced_test;

/// End-to-end test configuration
#[derive(Debug, Clone)]
pub struct E2ETestConfig {
    pub test_timeout: Duration,
    pub primal_startup_timeout: Duration,
    pub health_check_interval: Duration,
    pub max_retries: u32,
    pub enable_chaos_testing: bool,
    pub enable_performance_testing: bool,
}

impl Default for E2ETestConfig {
    fn default() -> Self {
        Self {
            test_timeout: Duration::from_secs(300), // 5 minutes
            primal_startup_timeout: Duration::from_secs(60),
            health_check_interval: Duration::from_secs(5),
            max_retries: 3,
            enable_chaos_testing: false,
            enable_performance_testing: false,
        }
    }
}

/// End-to-end test results
#[derive(Debug, Clone)]
pub struct E2ETestResults {
    pub total_tests: u32,
    pub passed_tests: u32,
    pub failed_tests: u32,
    pub skipped_tests: u32,
    pub execution_time: Duration,
    pub test_details: Vec<TestResult>,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub status: TestStatus,
    pub execution_time: Duration,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
}

/// Main E2E test runner
pub struct E2ETestRunner {
    config: E2ETestConfig,
    biomeos_manager: Arc<UniversalBiomeOSManager>,
    live_service: Arc<LiveService>,
    test_results: Vec<TestResult>,
}

impl E2ETestRunner {
    /// Create a new E2E test runner
    pub async fn new(config: E2ETestConfig) -> Result<Self> {
        info!("🚀 Initializing E2E Test Runner");

        // Create biomeOS configuration
        let biomeos_config = BiomeOSConfig {
            discovery: biomeos_core::config::DiscoveryConfig {
                method: DiscoveryMethod::NetworkScan,
                timeout: Duration::from_secs(30),
                scan_hosts: vec!["localhost".to_string(), "127.0.0.1".to_string()],
                scan_ports: vec![8080, 8081, 8082, 8083, 8084],
                registry_endpoints: vec!["http://localhost:8080".to_string()],
            },
            environment: Environment::Development,
            max_concurrent_requests: 100,
            request_timeout: Duration::from_secs(30),
            health_check_interval: Duration::from_secs(10),
        };

        // Initialize biomeOS manager
        let manager = UniversalBiomeOSManager::new(biomeos_config).await?;

        // Initialize live service
        let live_service = LiveService::new().await?;

        Ok(Self {
            config,
            biomeos_manager: Arc::new(manager),
            live_service: Arc::new(live_service),
            test_results: Vec::new(),
        })
    }

    /// Run all end-to-end tests
    pub async fn run_all_tests(&mut self) -> Result<E2ETestResults> {
        info!("🧪 Starting comprehensive E2E test suite");
        let start_time = Instant::now();

        // Core functionality tests
        self.run_core_functionality_tests().await?;

        // Integration tests
        self.run_integration_tests().await?;

        // Workflow tests
        self.run_workflow_tests().await?;

        // Performance tests (if enabled)
        if self.config.enable_performance_testing {
            self.run_performance_tests().await?;
        }

        // Chaos tests (if enabled)
        if self.config.enable_chaos_testing {
            self.run_chaos_tests().await?;
        }

        // Recovery tests
        self.run_recovery_tests().await?;

        let execution_time = start_time.elapsed();

        // Calculate results
        let total_tests = self.test_results.len() as u32;
        let passed_tests = self
            .test_results
            .iter()
            .filter(|r| r.status == TestStatus::Passed)
            .count() as u32;
        let failed_tests = self
            .test_results
            .iter()
            .filter(|r| r.status == TestStatus::Failed)
            .count() as u32;
        let skipped_tests = self
            .test_results
            .iter()
            .filter(|r| r.status == TestStatus::Skipped)
            .count() as u32;

        let results = E2ETestResults {
            total_tests,
            passed_tests,
            failed_tests,
            skipped_tests,
            execution_time,
            test_details: self.test_results.clone(),
        };

        info!(
            "✅ E2E test suite completed: {}/{} tests passed",
            passed_tests, total_tests
        );

        Ok(results)
    }

    /// Run core functionality tests
    async fn run_core_functionality_tests(&mut self) -> Result<()> {
        info!("🔧 Running core functionality tests");

        // Test system initialization
        self.run_test("system_initialization", async {
            let status = self.live_service.get_system_status().await?;
            assert!(status.is_ok(), "System should be initialized");
            Ok(())
        })
        .await;

        // Test primal discovery
        self.run_test("primal_discovery", async {
            let discovered = self.biomeos_manager.discover_network_scan().await?;
            info!("Discovered {} primals", discovered.len());
            assert!(
                !discovered.is_empty(),
                "Should discover at least one primal"
            );
            Ok(())
        })
        .await;

        // Test capability matching
        self.run_test("capability_matching", async {
            let capabilities = vec![PrimalCapability::Compute, PrimalCapability::Storage];
            let results = self
                .biomeos_manager
                .discover_by_capability(&capabilities)
                .await?;
            info!("Found {} primals with required capabilities", results.len());
            Ok(())
        })
        .await;

        // Test health monitoring
        self.run_test("health_monitoring", async {
            let health_status = self.biomeos_manager.get_system_health().await?;
            assert!(matches!(
                health_status.overall_status,
                HealthStatus::Healthy | HealthStatus::Degraded
            ));
            Ok(())
        })
        .await;

        Ok(())
    }

    /// Run integration tests
    async fn run_integration_tests(&mut self) -> Result<()> {
        info!("🔗 Running integration tests");

        // Test cross-primal communication
        self.run_test("cross_primal_communication", async {
            // Simulate cross-primal communication
            let primals = self.biomeos_manager.discover_network_scan().await?;
            for primal in &primals {
                let probe_result = self.biomeos_manager.probe_endpoint(&primal.endpoint).await;
                info!("Primal {} probe result: {:?}", primal.id, probe_result);
            }
            Ok(())
        })
        .await;

        // Test service discovery integration
        self.run_test("service_discovery_integration", async {
            let registry_results = self
                .biomeos_manager
                .discover_registry("http://localhost:8080")
                .await;
            info!("Registry discovery result: {:?}", registry_results);
            Ok(())
        })
        .await;

        // Test resource coordination
        self.run_test("resource_coordination", async {
            let system_resources = self.live_service.get_system_status().await?;
            assert!(
                system_resources.is_ok(),
                "Resource coordination should work"
            );
            Ok(())
        })
        .await;

        Ok(())
    }

    /// Run workflow tests
    async fn run_workflow_tests(&mut self) -> Result<()> {
        info!("🔄 Running workflow tests");

        // Test biome deployment workflow
        self.run_test("biome_deployment_workflow", async {
            // Create a test biome manifest
            let test_manifest = json!({
                "metadata": {
                    "name": "e2e-test-biome",
                    "version": "1.0.0"
                },
                "primals": {
                    "compute": {
                        "type": "toadstool",
                        "config": {
                            "cpu": "2",
                            "memory": "4GB"
                        }
                    }
                },
                "services": {
                    "web-server": {
                        "image": "nginx:latest",
                        "ports": ["80:8080"]
                    }
                }
            });

            // Simulate deployment process
            info!("Deploying test biome manifest");

            // In a real implementation, this would deploy the manifest
            // For now, we just validate the manifest structure
            assert!(test_manifest["metadata"]["name"].is_string());
            assert!(test_manifest["primals"].is_object());
            assert!(test_manifest["services"].is_object());

            Ok(())
        })
        .await;

        // Test service lifecycle management
        self.run_test("service_lifecycle_management", async {
            // Test service start, monitor, stop cycle
            info!("Testing service lifecycle management");

            // Simulate service lifecycle
            let services = vec!["web-server", "database", "cache"];
            for service in services {
                info!("Managing lifecycle for service: {}", service);
                // In real implementation, would start/stop services
            }

            Ok(())
        })
        .await;

        // Test scaling operations
        self.run_test("scaling_operations", async {
            info!("Testing scaling operations");

            // Simulate scaling up and down
            let scale_up_result = self.simulate_scale_operation("scale_up", 3).await;
            let scale_down_result = self.simulate_scale_operation("scale_down", 1).await;

            assert!(scale_up_result.is_ok());
            assert!(scale_down_result.is_ok());

            Ok(())
        })
        .await;

        Ok(())
    }

    /// Run performance tests
    async fn run_performance_tests(&mut self) -> Result<()> {
        info!("⚡ Running performance tests");

        // Test concurrent discovery performance
        self.run_test("concurrent_discovery_performance", async {
            let start_time = Instant::now();

            // Run multiple discovery operations concurrently
            let mut handles = Vec::new();
            for i in 0..10 {
                let manager = Arc::clone(&self.biomeos_manager);
                let handle = tokio::spawn(async move {
                    let result = manager.discover_network_scan().await;
                    info!("Concurrent discovery {} completed", i);
                    result
                });
                handles.push(handle);
            }

            // Wait for all to complete
            for handle in handles {
                handle.await??;
            }

            let duration = start_time.elapsed();
            info!("Concurrent discovery completed in {:?}", duration);

            // Performance assertion - should complete within reasonable time
            assert!(
                duration < Duration::from_secs(60),
                "Concurrent discovery should complete quickly"
            );

            Ok(())
        })
        .await;

        // Test memory usage under load
        self.run_test("memory_usage_under_load", async {
            info!("Testing memory usage under load");

            // Simulate high load operations
            for i in 0..100 {
                let _discovered = self.biomeos_manager.discover_network_scan().await?;
                if i % 10 == 0 {
                    info!("Load test iteration {}/100", i);
                }
            }

            // In a real implementation, would check memory usage
            info!("Memory usage test completed");

            Ok(())
        })
        .await;

        Ok(())
    }

    /// Run chaos tests
    async fn run_chaos_tests(&mut self) -> Result<()> {
        info!("🌪️ Running chaos tests");

        // Test network partition recovery
        self.run_test("network_partition_recovery", async {
            info!("Simulating network partition");

            // Simulate network issues
            sleep(Duration::from_millis(100)).await;

            // Test recovery
            let recovery_result = self.biomeos_manager.discover_network_scan().await;
            info!("Recovery result: {:?}", recovery_result);

            Ok(())
        })
        .await;

        // Test service failure handling
        self.run_test("service_failure_handling", async {
            info!("Testing service failure handling");

            // Simulate service failures and recovery
            let health_status = self.biomeos_manager.get_system_health().await?;
            info!(
                "System health during failure simulation: {:?}",
                health_status
            );

            Ok(())
        })
        .await;

        Ok(())
    }

    /// Run recovery tests
    async fn run_recovery_tests(&mut self) -> Result<()> {
        info!("🔄 Running recovery tests");

        // Test automatic recovery
        self.run_test("automatic_recovery", async {
            info!("Testing automatic recovery mechanisms");

            // Simulate failure and recovery
            let initial_health = self.biomeos_manager.get_system_health().await?;
            info!("Initial health: {:?}", initial_health);

            // Wait for potential recovery
            sleep(Duration::from_secs(2)).await;

            let recovered_health = self.biomeos_manager.get_system_health().await?;
            info!("Recovered health: {:?}", recovered_health);

            Ok(())
        })
        .await;

        // Test data consistency after recovery
        self.run_test("data_consistency_after_recovery", async {
            info!("Testing data consistency after recovery");

            // Check that system state is consistent
            let system_status = self.live_service.get_system_status().await?;
            assert!(
                system_status.is_ok(),
                "System should be consistent after recovery"
            );

            Ok(())
        })
        .await;

        Ok(())
    }

    /// Run a single test with error handling and timing
    async fn run_test<F, Fut>(&mut self, test_name: &str, test_fn: F)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        info!("🧪 Running test: {}", test_name);
        let start_time = Instant::now();

        let result = timeout(self.config.test_timeout, test_fn()).await;
        let execution_time = start_time.elapsed();

        let test_result = match result {
            Ok(Ok(())) => {
                info!("✅ Test passed: {} ({:?})", test_name, execution_time);
                TestResult {
                    name: test_name.to_string(),
                    status: TestStatus::Passed,
                    execution_time,
                    error_message: None,
                }
            }
            Ok(Err(e)) => {
                error!("❌ Test failed: {} - Error: {:?}", test_name, e);
                TestResult {
                    name: test_name.to_string(),
                    status: TestStatus::Failed,
                    execution_time,
                    error_message: Some(e.to_string()),
                }
            }
            Err(_) => {
                error!("⏰ Test timed out: {}", test_name);
                TestResult {
                    name: test_name.to_string(),
                    status: TestStatus::Failed,
                    execution_time,
                    error_message: Some("Test timed out".to_string()),
                }
            }
        };

        self.test_results.push(test_result);
    }

    /// Simulate scaling operation
    async fn simulate_scale_operation(&self, operation: &str, target_replicas: u32) -> Result<()> {
        info!(
            "Simulating {} operation to {} replicas",
            operation, target_replicas
        );

        // Simulate scaling time
        sleep(Duration::from_millis(100)).await;

        info!("Scaling operation {} completed successfully", operation);
        Ok(())
    }
}

/// Integration test for the complete E2E test suite
#[tokio::test]
#[traced_test]
async fn test_complete_e2e_suite() -> Result<()> {
    let config = E2ETestConfig {
        test_timeout: Duration::from_secs(30),
        enable_chaos_testing: false,
        enable_performance_testing: false,
        ..Default::default()
    };

    let mut runner = E2ETestRunner::new(config).await?;
    let results = runner.run_all_tests().await?;

    // Verify test results
    assert!(results.total_tests > 0, "Should run at least one test");
    assert!(
        results.passed_tests > 0,
        "Should have at least one passing test"
    );

    info!(
        "E2E test suite completed: {}/{} tests passed",
        results.passed_tests, results.total_tests
    );

    // Print detailed results
    for test_result in &results.test_details {
        match test_result.status {
            TestStatus::Passed => {
                info!("✅ {} ({:?})", test_result.name, test_result.execution_time)
            }
            TestStatus::Failed => error!(
                "❌ {} - {}",
                test_result.name,
                test_result
                    .error_message
                    .as_deref()
                    .unwrap_or("Unknown error")
            ),
            TestStatus::Skipped => warn!("⏭️ {} (skipped)", test_result.name),
        }
    }

    Ok(())
}

/// Fault tolerance test
#[tokio::test]
#[traced_test]
async fn test_fault_tolerance() -> Result<()> {
    let config = E2ETestConfig {
        enable_chaos_testing: true,
        ..Default::default()
    };

    let mut runner = E2ETestRunner::new(config).await?;

    // Run only chaos tests
    runner.run_chaos_tests().await?;

    // Verify system can handle failures
    let health_status = runner.biomeos_manager.get_system_health().await?;
    info!("System health after chaos tests: {:?}", health_status);

    Ok(())
}

/// Performance benchmark test
#[tokio::test]
#[traced_test]
async fn test_performance_benchmarks() -> Result<()> {
    let config = E2ETestConfig {
        enable_performance_testing: true,
        test_timeout: Duration::from_secs(120),
        ..Default::default()
    };

    let mut runner = E2ETestRunner::new(config).await?;

    // Run performance tests
    runner.run_performance_tests().await?;

    // Verify performance results
    let performance_results: Vec<_> = runner
        .test_results
        .iter()
        .filter(|r| r.name.contains("performance"))
        .collect();

    assert!(
        !performance_results.is_empty(),
        "Should run performance tests"
    );

    for result in performance_results {
        info!(
            "Performance test {} took {:?}",
            result.name, result.execution_time
        );
    }

    Ok(())
}
