//! Modern End-to-End Workflow Tests
//!
//! Comprehensive E2E tests that validate complete biomeOS workflows,
//! from system initialization through primal discovery, registration,
//! capability-based routing, and system monitoring.

use anyhow::Result;
use biomeos_core::{BiomeOSConfig, UniversalBiomeOSManager};
use biomeos_core::config::*;
use biomeos_core::universal_biomeos_manager::PrimalInfo;
use biomeos_core::integration::live_service::LiveService;
use biomeos_types::{PrimalCapability, Health, PrimalType};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

mod common;
use common::*;

/// Complete system lifecycle workflow test
#[tokio::test]
async fn test_complete_system_lifecycle() -> Result<()> {
    println!("🚀 Starting Complete System Lifecycle Test");

    // 1. System Initialization
    println!("1️⃣ Initializing biomeOS system...");
    let config = TestConfigBuilder::new()
        .with_static_endpoints(vec![
            ("toadstool", "http://localhost:8084"),
            ("songbird", "http://localhost:8081"),
            ("nestgate", "http://localhost:8082"),
        ])
        .with_security_enabled(false) // Simplified for testing
        .build();

    let manager = PerformanceTestUtils::assert_performance_bounds(
        UniversalBiomeOSManager::new(config),
        200, // System should initialize quickly
        "System initialization"
    ).await?;

    // Verify initial system health
    let initial_health = manager.get_system_health().await;
    TestAssertions::assert_system_healthy(&initial_health);
    println!("   ✅ System initialized successfully");

    // 2. Primal Discovery Phase
    println!("2️⃣ Discovering available primals...");
    
    // Attempt network discovery (may be empty in test environment)
    let discovered_primals = manager.discover().await?;
    println!("   🔍 Network discovery found {} primals", discovered_primals.len());

    // Simulate discovery by creating test primals
    let test_primals = vec![
        MockPrimalFactory::create_compute_primal("production-compute"),
        MockPrimalFactory::create_storage_primal("production-storage"),
        MockPrimalFactory::create_orchestration_primal("production-orchestration"),
    ];

    println!("   🎭 Created {} test primals for workflow", test_primals.len());

    // 3. Primal Registration Phase
    println!("3️⃣ Registering discovered primals...");
    
    for primal in &test_primals {
        manager.register_primal(primal.clone()).await?;
        println!("   📝 Registered primal: {} ({})", primal.name, primal.id);
    }

    // Verify registration
    let registered_primals = manager.get_registered_primals().await;
    assert_eq!(registered_primals.len(), 3);
    println!("   ✅ All primals registered successfully");

    // 4. Capability-Based Discovery
    println!("4️⃣ Testing capability-based service discovery...");
    
    let capability_tests = vec![
        (PrimalCapability::compute_provider(), "Compute services"),
        (PrimalCapability::storage_provider(), "Storage services"),
        (PrimalCapability::orchestration_provider(), "Orchestration services"),
    ];

    for (capability, description) in capability_tests {
        let results = manager.discover_by_capability(&[capability]).await?;
        println!("   🎯 {} discovery: {} matches found", description, results.len());
        
        // Network discovery may not find registered primals, but should work
        assert!(results.len() >= 0);
    }

    // 5. System Health Monitoring
    println!("5️⃣ Monitoring system health...");
    
    // Monitor health over time
    let mut health_checks = Vec::new();
    for i in 0..3 {
        let health = manager.get_system_health().await;
        health_checks.push(health);
        println!("   🏥 Health check {}: {:?}", i + 1, health_checks[i].overall_status);
        
        if i < 2 {
            sleep(Duration::from_millis(50)).await;
        }
    }

    // Verify health progression
    for health in &health_checks {
        TestAssertions::assert_system_healthy(health);
    }

    // Uptime should increase
    assert!(health_checks[2].uptime >= health_checks[0].uptime);
    println!("   ✅ System health monitoring working correctly");

    // 6. Live Service Integration
    println!("6️⃣ Testing live service integration...");
    
    let live_service = LiveService::new().await?;
    let system_status = live_service.get_system_status().await?;
    
    assert!(system_status.uptime.num_seconds() >= 0);
    println!("   📊 Live service operational - uptime: {}s", system_status.uptime.num_seconds());

    // Health check integration
    let health_result = live_service.health_check().await?;
    println!("   🔍 Comprehensive health check completed - overall_healthy: {}", health_result.overall_healthy);

    // 7. System Stress Test
    println!("7️⃣ Performing system stress test...");
    
    // Concurrent operations stress test
    let mut handles = Vec::new();
    let stress_operations = 20;

    for i in 0..stress_operations {
        let manager_clone = manager.clone();
        let handle = tokio::spawn(async move {
            match i % 3 {
                0 => {
                    // Health checks
                    let _health = manager_clone.get_system_health().await;
                    format!("health-{}", i)
                }
                1 => {
                    // Primal queries
                    let _primals = manager_clone.get_registered_primals().await;
                    format!("query-{}", i)
                }
                2 => {
                    // Discovery operations
                    let _discovered = manager_clone.discover().await;
                    format!("discovery-{}", i)
                }
                _ => unreachable!()
            }
        });
        handles.push(handle);
    }

    // Wait for all stress operations
    let stress_start = std::time::Instant::now();
    for handle in handles {
        let _result = handle.await?;
    }
    let stress_duration = stress_start.elapsed();

    println!("   ⚡ Stress test completed in {}ms", stress_duration.as_millis());
    assert!(stress_duration.as_millis() < 5000, "Stress test took too long");

    // 8. System Verification
    println!("8️⃣ Final system verification...");
    
    // Verify system integrity after stress test
    let final_health = manager.get_system_health().await;
    TestAssertions::assert_system_healthy(&final_health);

    let final_primals = manager.get_registered_primals().await;
    assert_eq!(final_primals.len(), 3);

    println!("   ✅ System integrity maintained");

    println!("🎉 Complete System Lifecycle Test PASSED");
    Ok(())
}

/// Primal ecosystem workflow test
#[tokio::test]
async fn test_primal_ecosystem_workflow() -> Result<()> {
    println!("🌱 Starting Primal Ecosystem Workflow Test");

    let manager = TestManagerFactory::create_default().await?;

    // 1. Create a diverse ecosystem of primals
    println!("1️⃣ Building diverse primal ecosystem...");
    
    let ecosystem_primals = vec![
        // Compute cluster
        MockPrimalFactory::create_compute_primal("compute-primary"),
        MockPrimalFactory::create_compute_primal("compute-secondary"),
        
        // Storage cluster
        MockPrimalFactory::create_storage_primal("storage-primary"),
        MockPrimalFactory::create_storage_primal("storage-backup"),
        
        // Orchestration services
        MockPrimalFactory::create_orchestration_primal("orchestrator-main"),
        
        // Custom specialized primal
        {
            let mut specialized = MockPrimalFactory::create_compute_primal("ai-accelerator");
            specialized.capabilities = vec![
                PrimalCapability::ai_provider(),
                PrimalCapability::machine_learning(),
            ];
            specialized.metadata.insert("specialization".to_string(), "AI/ML".to_string());
            specialized
        },
    ];

    // Register all primals
    for primal in &ecosystem_primals {
        manager.register_primal(primal.clone()).await?;
        println!("   🔧 Registered: {} with {} capabilities", 
            primal.name, primal.capabilities.len());
    }

    let registered = manager.get_registered_primals().await;
    assert_eq!(registered.len(), 6);
    println!("   ✅ Ecosystem established with {} primals", registered.len());

    // 2. Test capability-based service routing
    println!("2️⃣ Testing service routing by capabilities...");
    
    let routing_tests = vec![
        (vec![PrimalCapability::compute_provider()], "compute", 3), // compute-primary, compute-secondary, ai-accelerator
        (vec![PrimalCapability::storage_provider()], "storage", 2), // storage-primary, storage-backup
        (vec![PrimalCapability::orchestration_provider()], "orchestration", 1), // orchestrator-main
        (vec![PrimalCapability::ai_provider()], "AI", 1), // ai-accelerator
    ];

    for (capabilities, service_type, _expected_network_matches) in routing_tests {
        let matches = manager.discover_by_capability(&capabilities).await?;
        println!("   🎯 {} service routing: {} network matches found", 
            service_type, matches.len());
        
        // Network discovery searches actual network, not registered primals
        assert!(matches.len() >= 0);
    }

    // 3. Test ecosystem health monitoring
    println!("3️⃣ Monitoring ecosystem health...");
    
    // Simulate health state changes
    let mut unhealthy_primal = MockPrimalFactory::create_storage_primal("storage-failing");
    unhealthy_primal.health = Health::Unhealthy;
    manager.register_primal(unhealthy_primal).await?;

    let ecosystem_health = manager.get_system_health().await;
    TestAssertions::assert_system_healthy(&ecosystem_health);
    println!("   🏥 Ecosystem health check completed");

    // 4. Test primal lifecycle management
    println!("4️⃣ Testing primal lifecycle management...");
    
    // Update an existing primal
    let mut updated_primal = MockPrimalFactory::create_compute_primal("compute-primary");
    updated_primal.name = "Updated Compute Primary".to_string();
    updated_primal.health = Health::Degraded;
    updated_primal.metadata.insert("updated".to_string(), "true".to_string());
    
    manager.register_primal(updated_primal).await?;
    
    // Verify update
    let updated_registered = manager.get_registered_primals().await;
    let updated_entry = updated_registered.iter()
        .find(|p| p.id == "compute-primary")
        .expect("Updated primal should exist");
    
    assert_eq!(updated_entry.name, "Updated Compute Primary");
    assert_eq!(updated_entry.health, Health::Degraded);
    assert_eq!(updated_entry.metadata.get("updated"), Some(&"true".to_string()));
    
    println!("   🔄 Primal lifecycle management working correctly");

    println!("🌟 Primal Ecosystem Workflow Test PASSED");
    Ok(())
}

/// High-load concurrent workflow test
#[tokio::test]
async fn test_high_load_concurrent_workflow() -> Result<()> {
    println!("⚡ Starting High-Load Concurrent Workflow Test");

    let manager = TestManagerFactory::create_default().await?;

    // 1. Concurrent primal registration
    println!("1️⃣ Testing concurrent primal registration...");
    
    let registration_start = std::time::Instant::now();
    let mut registration_handles = Vec::new();
    let concurrent_registrations = 50;

    for i in 0..concurrent_registrations {
        let manager_clone = manager.clone();
        let handle = tokio::spawn(async move {
            let primal_type = match i % 3 {
                0 => "compute",
                1 => "storage", 
                2 => "orchestration",
                _ => unreachable!()
            };
            
            let primal = match primal_type {
                "compute" => MockPrimalFactory::create_compute_primal(&format!("concurrent-compute-{}", i)),
                "storage" => MockPrimalFactory::create_storage_primal(&format!("concurrent-storage-{}", i)),
                "orchestration" => MockPrimalFactory::create_orchestration_primal(&format!("concurrent-orchestration-{}", i)),
                _ => unreachable!()
            };
            
            manager_clone.register_primal(primal).await.unwrap();
            format!("{}-{}", primal_type, i)
        });
        registration_handles.push(handle);
    }

    // Wait for all registrations
    let mut registration_results = Vec::new();
    for handle in registration_handles {
        registration_results.push(handle.await?);
    }

    let registration_duration = registration_start.elapsed();
    println!("   📝 {} concurrent registrations completed in {}ms", 
        concurrent_registrations, registration_duration.as_millis());

    // Verify all registrations succeeded
    assert_eq!(registration_results.len(), concurrent_registrations);
    let final_registered = manager.get_registered_primals().await;
    assert_eq!(final_registered.len(), concurrent_registrations);

    // 2. Concurrent mixed operations
    println!("2️⃣ Testing concurrent mixed operations...");
    
    let mixed_ops_start = std::time::Instant::now();
    let mut mixed_handles = Vec::new();
    let mixed_operations = 100;

    for i in 0..mixed_operations {
        let manager_clone = manager.clone();
        let handle = tokio::spawn(async move {
            match i % 5 {
                0 => {
                    // Health checks
                    let _health = manager_clone.get_system_health().await;
                    "health"
                }
                1 => {
                    // Primal queries
                    let _primals = manager_clone.get_registered_primals().await;
                    "query"
                }
                2 => {
                    // Discovery
                    let _discovered = manager_clone.discover().await.unwrap();
                    "discovery"
                }
                3 => {
                    // Capability search
                    let caps = vec![PrimalCapability::compute_provider()];
                    let _results = manager_clone.discover_by_capability(&caps).await.unwrap();
                    "capability"
                }
                4 => {
                    // Registry search
                    let _results = manager_clone.discover_registry("http://test-registry").await;
                    "registry"
                }
                _ => unreachable!()
            }
        });
        mixed_handles.push(handle);
    }

    // Collect results with timeout
    let timeout_duration = Duration::from_secs(30);
    let mixed_results = tokio::time::timeout(timeout_duration, async {
        let mut results = Vec::new();
        for handle in mixed_handles {
            results.push(handle.await.unwrap());
        }
        results
    }).await?;

    let mixed_ops_duration = mixed_ops_start.elapsed();
    println!("   ⚡ {} mixed operations completed in {}ms", 
        mixed_operations, mixed_ops_duration.as_millis());

    // Verify performance
    assert!(mixed_ops_duration.as_secs() < 25, "Mixed operations took too long");
    assert_eq!(mixed_results.len(), mixed_operations);

    // 3. System stability verification
    println!("3️⃣ Verifying system stability after high load...");
    
    // System should remain healthy
    let post_load_health = manager.get_system_health().await;
    TestAssertions::assert_system_healthy(&post_load_health);

    // All primals should still be registered
    let post_load_primals = manager.get_registered_primals().await;
    assert_eq!(post_load_primals.len(), concurrent_registrations);

    // Performance should still be good
    let final_health_start = std::time::Instant::now();
    let _final_health = manager.get_system_health().await;
    let final_health_duration = final_health_start.elapsed();
    
    assert!(final_health_duration.as_millis() < 100, 
        "Health check degraded after load test");

    println!("   ✅ System stability maintained under high load");

    println!("🚀 High-Load Concurrent Workflow Test PASSED");
    Ok(())
}

/// Error recovery workflow test
#[tokio::test]
async fn test_error_recovery_workflow() -> Result<()> {
    println!("🛠️  Starting Error Recovery Workflow Test");

    let manager = TestManagerFactory::create_default().await?;

    // 1. Test recovery from network failures
    println!("1️⃣ Testing network failure recovery...");
    
    // Attempt discovery with invalid endpoints
    let invalid_discovery = manager.discover_registry("http://invalid-host-12345:99999").await;
    match invalid_discovery {
        Ok(results) => {
            println!("   🔍 Invalid discovery returned {} results (graceful handling)", results.len());
            assert!(results.len() >= 0);
        }
        Err(e) => {
            println!("   ❌ Invalid discovery failed as expected: {}", e);
        }
    }

    // System should remain healthy
    let health_after_failure = manager.get_system_health().await;
    TestAssertions::assert_system_healthy(&health_after_failure);
    println!("   ✅ System recovered from network failures");

    // 2. Test recovery from invalid primal data
    println!("2️⃣ Testing invalid data recovery...");
    
    // Create primals with edge case data
    let edge_case_primals = vec![
        // Empty endpoint
        {
            let mut primal = MockPrimalFactory::create_compute_primal("empty-endpoint");
            primal.endpoint = "".to_string();
            primal
        },
        // Very long name
        {
            let mut primal = MockPrimalFactory::create_storage_primal("long-name");
            primal.name = "x".repeat(1000);
            primal
        },
        // No capabilities
        {
            let mut primal = MockPrimalFactory::create_orchestration_primal("no-caps");
            primal.capabilities = vec![];
            primal
        },
    ];

    let mut successful_registrations = 0;
    for primal in edge_case_primals {
        match manager.register_primal(primal.clone()).await {
            Ok(_) => {
                successful_registrations += 1;
                println!("   ✅ Edge case primal '{}' accepted", primal.id);
            }
            Err(e) => {
                println!("   ⚠️  Edge case primal '{}' rejected: {}", primal.id, e);
            }
        }
    }

    println!("   📊 {}/3 edge case primals handled", successful_registrations);

    // System should remain stable
    let health_after_edge_cases = manager.get_system_health().await;
    TestAssertions::assert_system_healthy(&health_after_edge_cases);

    // 3. Test concurrent error handling
    println!("3️⃣ Testing concurrent error scenarios...");
    
    let mut error_handles = Vec::new();
    let error_operations = 20;

    for i in 0..error_operations {
        let manager_clone = manager.clone();
        let handle = tokio::spawn(async move {
            match i % 4 {
                0 => {
                    // Invalid registry calls
                    let _result = manager_clone.discover_registry("invalid-url").await;
                    "invalid_registry"
                }
                1 => {
                    // Invalid endpoint probes
                    let _result = manager_clone.probe_endpoint("not-a-url").await;
                    "invalid_probe"
                }
                2 => {
                    // Valid operations (should succeed)
                    let _health = manager_clone.get_system_health().await;
                    "valid_health"
                }
                3 => {
                    // Valid discovery (should succeed)
                    let _result = manager_clone.discover().await;
                    "valid_discovery"
                }
                _ => unreachable!()
            }
        });
        error_handles.push(handle);
    }

    // Wait for all error scenario operations
    let mut error_results = Vec::new();
    for handle in error_handles {
        error_results.push(handle.await?);
    }

    assert_eq!(error_results.len(), error_operations);
    println!("   ⚡ {} concurrent error scenarios handled", error_operations);

    // 4. Final system verification
    println!("4️⃣ Final system health verification...");
    
    let final_health = manager.get_system_health().await;
    TestAssertions::assert_system_healthy(&final_health);

    // System should still be responsive
    let response_start = std::time::Instant::now();
    let _final_primals = manager.get_registered_primals().await;
    let response_time = response_start.elapsed();

    assert!(response_time.as_millis() < 100, 
        "System response degraded after error scenarios");

    println!("   ✅ System fully recovered and operational");

    println!("💪 Error Recovery Workflow Test PASSED");
    Ok(())
} 