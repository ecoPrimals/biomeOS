//! Generic integration tests for eco-primals
//!
//! This test suite provides configurable integration tests that can work with
//! any primal types, replacing the hardcoded songbird_nestgate_integration_tests.rs
//! with a flexible, maintainable framework.
//!
//! # Features
//!
//! - **Configurable**: Tests work with any primal types, not hardcoded to specific primals
//! - **Comprehensive**: Covers basic integration, gaming, performance, and cross-format scenarios
//! - **Maintainable**: Uses builder patterns and configurable presets
//! - **Reusable**: Common test utilities and validation functions

use biomeos_manifest::*;

mod integration_tests;
use integration_tests::*;

/// Test basic integration between any two primals
#[test]
fn test_basic_primal_integration() {
    // Test with Songbird + NestGate (original scenario)
    let result = convenience::test_songbird_nestgate_integration();
    assert!(!result.has_errors(), "Songbird+NestGate integration failed: {}", result.summary());

    // Test with different primal combinations
    let result = convenience::test_orchestrator_storage_integration(
        "orchestrator",
        PrimalType::Songbird,
        "storage",
        PrimalType::NestGate,
        BiomeSpecialization::DataCenter,
    );
    assert!(!result.has_errors(), "Generic orchestrator+storage integration failed: {}", result.summary());
}

/// Test gaming tournament integration with configurable primals
#[test]
fn test_gaming_tournament_integration() {
    // Test gaming scenario with Songbird + NestGate
    let result = convenience::test_songbird_nestgate_gaming_integration();
    assert!(!result.has_errors(), "Songbird+NestGate gaming integration failed: {}", result.summary());

    // Test gaming scenario with any primals
    let result = IntegrationTestSuite::run_gaming_integration_test(
        "gaming-orchestrator",
        PrimalType::Songbird,
        "gaming-storage",
        PrimalType::NestGate,
    );
    assert!(!result.has_errors(), "Generic gaming integration failed: {}", result.summary());
}

/// Test advanced feature configurations
#[test]
fn test_advanced_feature_integration() {
    // Test advanced features with configurable primals
    integration_tests::test_advanced_feature_integration(
        "orchestrator",
        PrimalType::Songbird,
        "storage",
        PrimalType::NestGate,
    );
}

/// Test large-scale integration with multiple primals
#[test]
fn test_multi_primal_integration() {
    // Test complete ecosystem integration
    let result = convenience::test_cross_primal_ecosystem();
    assert!(!result.has_errors(), "Cross-primal ecosystem integration failed: {}", result.summary());

    // Test custom multi-primal configuration
    let primals = vec![
        ("orchestrator".to_string(), PrimalType::Songbird, 1),
        ("storage".to_string(), PrimalType::NestGate, 2),
        ("analytics".to_string(), PrimalType::Squirrel, 3),
    ];

    let dependencies = vec![
        ("storage".to_string(), vec!["orchestrator".to_string()]),
        ("analytics".to_string(), vec!["orchestrator".to_string(), "storage".to_string()]),
    ];

    integration_tests::test_multi_primal_integration(
        primals,
        dependencies,
        BiomeSpecialization::DataCenter,
    );
}

/// Test fault tolerance and recovery scenarios
#[test]
fn test_fault_tolerance_integration() {
    // Test fault tolerance with different primal combinations
    integration_tests::test_fault_tolerance_integration(
        "orchestrator",
        PrimalType::Songbird,
        "storage",
        PrimalType::NestGate,
    );

    integration_tests::test_fault_tolerance_integration(
        "service-mesh",
        PrimalType::Songbird,
        "distributed-storage",
        PrimalType::NestGate,
    );
}

/// Test cross-region deployment scenarios
#[test]
fn test_cross_region_integration() {
    // Test cross-region deployment with configurable primals
    integration_tests::test_cross_region_integration(
        "global-orchestrator",
        PrimalType::Songbird,
        "global-storage",
        PrimalType::NestGate,
    );
}

/// Test performance optimization scenarios
#[test]
fn test_performance_optimization_integration() {
    // Test performance optimization with configurable primals
    let result = IntegrationTestSuite::run_performance_integration_test(
        "performance-orchestrator",
        PrimalType::Songbird,
        "performance-storage",
        PrimalType::NestGate,
    );
    assert!(!result.has_errors(), "Performance optimization integration failed: {}", result.summary());
}

/// Test YAML serialization compatibility
#[test]
fn test_yaml_serialization_compatibility() {
    // Test YAML serialization with Songbird + NestGate
    let result = SerializationTester::test_yaml_serialization_compatibility(
        "songbird",
        PrimalType::Songbird,
        "nestgate",
        PrimalType::NestGate,
        BiomeSpecialization::DataCenter,
    );
    assert!(!result.has_errors(), "YAML serialization test failed: {}", result.summary());

    // Test YAML serialization with other primal combinations
    let result = SerializationTester::test_yaml_serialization_compatibility(
        "orchestrator",
        PrimalType::Songbird,
        "storage",
        PrimalType::NestGate,
        BiomeSpecialization::GamingServer,
    );
    assert!(!result.has_errors(), "YAML serialization test (gaming) failed: {}", result.summary());
}

/// Test JSON serialization compatibility
#[test]
fn test_json_serialization_compatibility() {
    // Test JSON serialization with configurable primals
    let result = SerializationTester::test_json_serialization_compatibility(
        "orchestrator",
        PrimalType::Songbird,
        "storage",
        PrimalType::NestGate,
        BiomeSpecialization::DataCenter,
    );
    assert!(!result.has_errors(), "JSON serialization test failed: {}", result.summary());
}

/// Test gaming-specific serialization
#[test]
fn test_gaming_serialization() {
    // Test gaming serialization with configurable primals
    let result = SerializationTester::test_gaming_serialization(
        "gaming-orchestrator",
        PrimalType::Songbird,
        "gaming-storage",
        PrimalType::NestGate,
    );
    assert!(!result.has_errors(), "Gaming serialization test failed: {}", result.summary());
}

/// Test cross-format compatibility
#[test]
fn test_cross_format_compatibility() {
    // Test cross-format compatibility with configurable primals
    let result = IntegrationTestSuite::run_cross_format_compatibility_test(
        "orchestrator",
        PrimalType::Songbird,
        "storage",
        PrimalType::NestGate,
    );
    assert!(!result.has_errors(), "Cross-format compatibility test failed: {}", result.summary());
}

/// Test large manifest serialization performance
#[test]
fn test_large_manifest_serialization() {
    // Test large manifest with many primals
    let primals = vec![
        ("songbird".to_string(), PrimalType::Songbird, 1),
        ("nestgate".to_string(), PrimalType::NestGate, 2),
        ("squirrel".to_string(), PrimalType::Squirrel, 3),
        ("beardog".to_string(), PrimalType::BearDog, 4),
        ("toadstool".to_string(), PrimalType::Toadstool, 5),
    ];

    let dependencies = vec![
        ("nestgate".to_string(), vec!["songbird".to_string()]),
        ("squirrel".to_string(), vec!["songbird".to_string()]),
        ("beardog".to_string(), vec!["songbird".to_string(), "nestgate".to_string()]),
        ("toadstool".to_string(), vec!["songbird".to_string()]),
    ];

    let result = SerializationTester::test_large_manifest_serialization(primals, dependencies);
    assert!(!result.has_errors(), "Large manifest serialization test failed: {}", result.summary());
}

/// Test configuration validation
#[test]
fn test_configuration_validation() {
    // Create test manifest with configurable primals
    let manifest = TestScenarios::two_primal_integration(
        "orchestrator",
        PrimalType::Songbird,
        "storage",
        PrimalType::NestGate,
        BiomeSpecialization::DataCenter,
    ).build();

    // Test comprehensive validation
    let result = IntegrationValidator::validate_primal_integration(
        &manifest,
        "orchestrator",
        "storage",
        PrimalType::Songbird,
        PrimalType::NestGate,
    );
    assert!(!result.has_errors(), "Configuration validation failed: {}", result.summary());

    // Test metadata validation
    let metadata_result = IntegrationValidator::validate_manifest_metadata(&manifest);
    assert!(!metadata_result.has_errors(), "Metadata validation failed: {}", metadata_result.summary());

    // Test dependency validation
    if let Some(deps) = &manifest.dependencies {
        let dep_result = IntegrationValidator::validate_dependency_config(deps);
        assert!(!dep_result.has_errors(), "Dependency validation failed: {}", dep_result.summary());
    }
}

/// Test gaming-specific validation
#[test]
fn test_gaming_configuration_validation() {
    // Create gaming manifest with configurable primals
    let manifest = TestScenarios::gaming_tournament(
        "gaming-orchestrator",
        PrimalType::Songbird,
        "gaming-storage",
        PrimalType::NestGate,
    ).build();

    // Test gaming-specific validation
    let result = IntegrationValidator::validate_gaming_configuration(&manifest, "gaming-orchestrator");
    assert!(!result.has_errors(), "Gaming configuration validation failed: {}", result.summary());
}

/// Test data center scenario validation
#[test]
fn test_data_center_scenario_validation() {
    // Create data center manifest with configurable primals
    let manifest = TestScenarios::two_primal_integration(
        "dc-orchestrator",
        PrimalType::Songbird,
        "dc-storage",
        PrimalType::NestGate,
        BiomeSpecialization::DataCenter,
    ).build();

    // Test data center scenario validation
    let result = ScenarioValidators::validate_data_center_scenario(&manifest, "dc-orchestrator", "dc-storage");
    assert!(!result.has_errors(), "Data center scenario validation failed: {}", result.summary());
}

/// Test with custom primal configurations
#[test]
fn test_custom_primal_configurations() {
    // Test with custom orchestrator configuration
    let orchestrator = PrimalConfigFactory::create_orchestrator(
        "custom-orchestrator",
        PrimalType::Songbird,
        1,
        PrimalPreset::ServiceMeshOrchestrator {
            topology: "star".to_string(),
            features: vec![
                "custom_feature_1".to_string(),
                "custom_feature_2".to_string(),
            ],
            discovery_method: "etcd".to_string(),
            datacenter: "custom-dc".to_string(),
        },
        ResourcePreset::HighPerformance,
        NetworkingPreset::ServiceMesh {
            ports: vec![8000, 8001, 8002],
            discovery_method: "etcd".to_string(),
        },
    );

    // Test with custom storage configuration
    let storage = PrimalConfigFactory::create_storage(
        "custom-storage",
        PrimalType::NestGate,
        2,
        vec!["custom-orchestrator".to_string()],
        PrimalPreset::DistributedStorage {
            storage_type: "block".to_string(),
            replication_factor: 5,
            consistency: "strong".to_string(),
            features: vec![
                "custom_storage_feature".to_string(),
                "advanced_encryption".to_string(),
            ],
        },
        ResourcePreset::StorageOptimized,
        NetworkingPreset::Storage {
            ports: vec![9000, 9001, 9002],
            service_name: "custom-storage-service".to_string(),
        },
    );

    // Build and test custom manifest
    let manifest = TestManifestBuilder::new("custom-integration-test")
        .with_metadata(
            "1.0.0",
            "Custom primal integration test",
            BiomeSpecialization::DataCenter,
        )
        .with_tags(vec!["custom".to_string(), "integration".to_string()])
        .with_attribution("biomeOS Custom Test Suite", "MIT")
        .with_primal(orchestrator)
        .with_primal(storage)
        .build();

    // Validate custom configuration
    let result = IntegrationValidator::validate_primal_integration(
        &manifest,
        "custom-orchestrator",
        "custom-storage",
        PrimalType::Songbird,
        PrimalType::NestGate,
    );
    assert!(!result.has_errors(), "Custom configuration validation failed: {}", result.summary());
}

/// Test with different primal type combinations
#[test]
fn test_different_primal_combinations() {
    // Test different primal type combinations to ensure flexibility
    let test_cases = vec![
        ("songbird", PrimalType::Songbird, "nestgate", PrimalType::NestGate),
        ("orchestrator", PrimalType::Squirrel, "storage", PrimalType::BearDog),
        ("mesh", PrimalType::Toadstool, "store", PrimalType::Songbird),
    ];

    for (orch_name, orch_type, stor_name, stor_type) in test_cases {
        let result = IntegrationTestSuite::run_complete_integration_test(
            orch_name,
            orch_type,
            stor_name,
            stor_type,
            BiomeSpecialization::DataCenter,
        );
        assert!(!result.has_errors(), 
            "Integration test failed for {} ({:?}) + {} ({:?}): {}", 
            orch_name, orch_type, stor_name, stor_type, result.summary());
    }
}

/// Test configuration helpers
#[test]
fn test_configuration_helpers() {
    // Test configuration helpers for different scenarios
    let configs = vec![
        config::create_test_config(
            PrimalType::Songbird,
            PrimalType::NestGate,
            BiomeSpecialization::DataCenter,
        ),
        config::create_test_config(
            PrimalType::Squirrel,
            PrimalType::BearDog,
            BiomeSpecialization::GamingServer,
        ),
        config::create_test_config(
            PrimalType::Toadstool,
            PrimalType::Songbird,
            BiomeSpecialization::DataCenter,
        ),
    ];

    for config in configs {
        let result = config.run_complete_test();
        assert!(!result.has_errors(), "Config helper test failed: {}", result.summary());
    }
}

/// Demonstrate the flexibility of the new framework
#[test]
fn test_framework_flexibility() {
    // This test demonstrates how the new framework can easily test
    // any primal combination without hardcoded dependencies

    // Test 1: Original hardcoded scenario (Songbird + NestGate)
    let original_result = convenience::test_songbird_nestgate_integration();
    assert!(!original_result.has_errors(), "Original scenario failed");

    // Test 2: Different primal types
    let different_result = convenience::test_orchestrator_storage_integration(
        "analytics",
        PrimalType::Squirrel,
        "security",
        PrimalType::BearDog,
        BiomeSpecialization::DataCenter,
    );
    assert!(!different_result.has_errors(), "Different primal types failed");

    // Test 3: Different specializations
    let gaming_result = convenience::test_orchestrator_storage_integration(
        "game-server",
        PrimalType::Toadstool,
        "game-storage",
        PrimalType::NestGate,
        BiomeSpecialization::GamingServer,
    );
    assert!(!gaming_result.has_errors(), "Gaming specialization failed");

    // Test 4: Custom names with same types
    let custom_result = convenience::test_orchestrator_storage_integration(
        "my-custom-orchestrator",
        PrimalType::Songbird,
        "my-custom-storage",
        PrimalType::NestGate,
        BiomeSpecialization::DataCenter,
    );
    assert!(!custom_result.has_errors(), "Custom names failed");

    println!("Framework flexibility test passed - all scenarios work!");
} 