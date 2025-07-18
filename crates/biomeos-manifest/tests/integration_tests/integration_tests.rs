//! Generic integration tests for eco-primals
//!
//! This module provides configurable integration tests that can work with
//! any primal types, replacing hardcoded primal-specific tests.

use biomeos_manifest::*;
use serde_json::json;
use crate::integration_tests::common::*;
use crate::integration_tests::primal_configs::*;

/// Test basic integration between any two primals
pub fn test_basic_primal_integration(
    orchestrator_name: &str,
    orchestrator_type: &PrimalType,
    storage_name: &str,
    storage_type: &PrimalType,
    specialization: BiomeSpecialization,
) {
    let manifest = TestScenarios::two_primal_integration(
        orchestrator_name,
        *orchestrator_type,
        storage_name,
        *storage_type,
        specialization,
    ).build();

    // Validate the integration
    let orchestrator = manifest.primals.get(orchestrator_name).unwrap();
    let storage = manifest.primals.get(storage_name).unwrap();

    // Check that storage depends on orchestrator
    assertions::assert_primal_dependencies(storage, &[orchestrator_name.to_string()]);

    // Check that both primals are enabled
    assertions::assert_primal_basic_config(orchestrator, *orchestrator_type, 1);
    assertions::assert_primal_basic_config(storage, *storage_type, 2);

    // Check priority ordering (orchestrator starts first)
    assert!(orchestrator.priority < storage.priority);

    // Check that storage is configured to use orchestrator for discovery
    let storage_config = storage.config.as_ref().unwrap();
    let discovery_config = &storage_config["discovery"];
    assert!(discovery_config.is_object(), "Storage should have discovery configuration");

    // Check that both have proper networking configuration
    assertions::assert_primal_networking(orchestrator, Some(&[8500, 8501, 8502]));
    assertions::assert_primal_networking(storage, Some(&[9000, 9001, 9002]));

    let storage_net = storage.networking.as_ref().unwrap();
    assert_eq!(storage_net.discovery.as_ref().unwrap().method, "service_mesh");
}

/// Test gaming tournament integration with any primals
pub fn test_gaming_tournament_integration(
    orchestrator_name: &str,
    orchestrator_type: &PrimalType,
    storage_name: &str,
    storage_type: &PrimalType,
) {
    let manifest = TestScenarios::gaming_tournament(
        orchestrator_name,
        *orchestrator_type,
        storage_name,
        *storage_type,
    ).build();

    // Validate basic structure
    let orchestrator = manifest.primals.get(orchestrator_name).unwrap();
    let storage = manifest.primals.get(storage_name).unwrap();

    // Check gaming-specific configuration
    assertions::assert_primal_basic_config(orchestrator, *orchestrator_type, 1);
    assertions::assert_primal_basic_config(storage, *storage_type, 2);

    // Check gaming-specific features
    let orchestrator_config = orchestrator.config.as_ref().unwrap();
    assert_eq!(orchestrator_config["topology"], "ring");
    assert_eq!(orchestrator_config["tournament"]["match_making"], true);
    assert_eq!(orchestrator_config["tournament"]["player_balancing"], true);
    assert_eq!(orchestrator_config["tournament"]["server_allocation"], true);

    // Check storage is configured for gaming
    let storage_config = storage.config.as_ref().unwrap();
    assert_eq!(storage_config["consistency"], "strong");
    assert!(storage_config["features"].as_array().unwrap().contains(&json!("low_latency")));

    // Check resources are gaming-optimized
    let orchestrator_resources = orchestrator.resources.as_ref().unwrap();
    assert_eq!(orchestrator_resources.cpu.as_ref().unwrap().max_cores, Some(6.0));
    assert_eq!(orchestrator_resources.memory.as_ref().unwrap().max_mb, Some(12288));

    // Check networking has gaming-specific ports
    assertions::assert_primal_networking(orchestrator, Some(&[8500, 8501, 8502, 8503]));
    assertions::assert_primal_networking(storage, Some(&[9000, 9001, 9002]));

    // Check specialization
    assert_eq!(manifest.metadata.specialization, Some(BiomeSpecialization::GamingServer));

    // Check niche classification
    let niches = manifest.metadata.niches.as_ref().unwrap();
    assert_eq!(niches.primary, "gaming-tournament");
    assert!(niches.secondary.contains(&"esports".to_string()));
    assert!(niches.secondary.contains(&"competitive-gaming".to_string()));
}

/// Test advanced feature configurations
pub fn test_advanced_feature_integration(
    orchestrator_name: &str,
    orchestrator_type: PrimalType,
    storage_name: &str,
    storage_type: PrimalType,
) {
    let manifest = TestScenarios::gaming_tournament(
        orchestrator_name,
        orchestrator_type,
        storage_name,
        storage_type,
    ).build();

    // Validate dependencies
    let dependencies = manifest.dependencies.as_ref().unwrap();
    assertions::assert_dependency_config(dependencies, 3, 1, 0); // gaming has 3 required deps

    // Check required dependencies
    let orchestrator_dep = dependencies
        .requires
        .iter()
        .find(|d| d.name == format!("{}-runtime", orchestrator_name))
        .unwrap();
    assert!(!orchestrator_dep.optional);
    assert_eq!(orchestrator_dep.version, Some(">=1.0.0".to_string()));

    let storage_dep = dependencies
        .requires
        .iter()
        .find(|d| d.name == format!("{}-runtime", storage_name))
        .unwrap();
    assert!(!storage_dep.optional);
    assert_eq!(storage_dep.version, Some(">=1.0.0".to_string()));

    let gaming_dep = dependencies
        .requires
        .iter()
        .find(|d| d.name == "gaming-engine")
        .unwrap();
    assert!(!gaming_dep.optional);
    assert_eq!(gaming_dep.version, Some(">=3.0.0".to_string()));

    // Check feature configurations
    let cross_region_feature = dependencies
        .features
        .get("cross-region-replication")
        .unwrap();
    assertions::assert_feature_config(
        cross_region_feature,
        &[format!("{}-runtime", storage_name)],
        &["replication-service".to_string()],
        false,
    );

    let load_balancing_feature = dependencies
        .features
        .get("advanced-load-balancing")
        .unwrap();
    assertions::assert_feature_config(
        load_balancing_feature,
        &[format!("{}-runtime", orchestrator_name)],
        &["load-balancer".to_string()],
        true,
    );
}

/// Test large-scale integration with multiple primals
pub fn test_multi_primal_integration(
    primals: Vec<(String, PrimalType, u32)>, // (name, type, priority)
    dependencies: Vec<(String, Vec<String>)>, // (name, depends_on)
    specialization: BiomeSpecialization,
) {
    let mut manifest_builder = TestManifestBuilder::new("multi-primal-integration")
        .with_metadata(
            "1.0.0",
            "Multi-primal integration test",
            specialization,
        )
        .with_tags(vec![
            "integration".to_string(),
            "multi-primal".to_string(),
            "eco-primals".to_string(),
        ])
        .with_attribution("biomeOS Test Suite", "MIT");

    // Add all primals
    let primal_count = primals.len();
    for (name, primal_type, priority) in &primals {
        let deps = dependencies
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, deps)| deps.clone())
            .unwrap_or_default();

        let primal = PrimalBuilder::new(name, *primal_type)
            .with_priority_and_deps(*priority, deps)
            .with_version_and_timeout("1.0.0", "30s")
            .with_config(json!({
                "mode": "distributed",
                "features": ["service_discovery", "health_monitoring"]
            }))
            .with_networking(
                NetworkingBuilder::new()
                    .with_ports(vec![8000 + *priority as u16])
                    .with_host("0.0.0.0")
                    .build()
            )
            .with_resources(ResourcePreset::Standard.to_resources());

        manifest_builder = manifest_builder.with_primal(primal);
    }

    let manifest = manifest_builder.build();
    let creation_time = start_time.elapsed();

    result.add_info(&format!("Large manifest creation took: {:?}", creation_time));

    // Validate all primals are present
    assert_eq!(manifest.primals.len(), primal_count);

    // Validate dependency chain
    for (name, expected_deps) in dependencies {
        let primal = manifest.primals.get(&name).unwrap();
        assertions::assert_primal_dependencies(primal, &expected_deps);
    }

    // Validate priorities are ordered correctly
    let mut primal_priorities: Vec<_> = manifest.primals.values()
        .map(|p| p.priority)
        .collect();
    primal_priorities.sort();
    
    for i in 1..primal_priorities.len() {
        assert!(primal_priorities[i-1] <= primal_priorities[i], 
            "Priorities should be ordered: {} <= {}", 
            primal_priorities[i-1], primal_priorities[i]);
    }
}

/// Test fault tolerance and recovery scenarios
pub fn test_fault_tolerance_integration(
    orchestrator_name: &str,
    orchestrator_type: PrimalType,
    storage_name: &str,
    storage_type: PrimalType,
) {
    let manifest = TestScenarios::two_primal_integration(
        orchestrator_name,
        orchestrator_type,
        storage_name,
        storage_type,
        BiomeSpecialization::DataCenter,
    ).build();

    // Validate fault tolerance configurations
    let orchestrator = manifest.primals.get(orchestrator_name).unwrap();
    let storage = manifest.primals.get(storage_name).unwrap();

    // Check orchestrator has mesh topology for redundancy
    let orchestrator_config = orchestrator.config.as_ref().unwrap();
    assert_eq!(orchestrator_config["topology"], "mesh");

    // Check storage has proper replication
    let storage_config = storage.config.as_ref().unwrap();
    assert_eq!(storage_config["replication_factor"], 3);
    assert_eq!(storage_config["consistency"], "eventual");

    // Check timeout configurations
    assert_eq!(orchestrator.startup_timeout, Some("30s".to_string()));
    assert_eq!(storage.startup_timeout, Some("60s".to_string()));

    // Check resource limits for stability
    let orchestrator_resources = orchestrator.resources.as_ref().unwrap();
    assert!(orchestrator_resources.cpu.is_some());
    assert!(orchestrator_resources.memory.is_some());
    assert!(orchestrator_resources.storage.is_some());

    let storage_resources = storage.resources.as_ref().unwrap();
    assert!(storage_resources.cpu.is_some());
    assert!(storage_resources.memory.is_some());
    assert!(storage_resources.storage.is_some());

    // Check networking redundancy
    let orchestrator_net = orchestrator.networking.as_ref().unwrap();
    assert!(orchestrator_net.ports.as_ref().unwrap().len() >= 3);

    let storage_net = storage.networking.as_ref().unwrap();
    assert!(storage_net.ports.as_ref().unwrap().len() >= 3);
}

/// Test cross-region deployment scenarios
pub fn test_cross_region_integration(
    orchestrator_name: &str,
    orchestrator_type: PrimalType,
    storage_name: &str,
    storage_type: PrimalType,
) {
    let orchestrator = PrimalConfigFactory::create_orchestrator(
        orchestrator_name,
        orchestrator_type,
        1,
        PrimalPreset::ServiceMeshOrchestrator {
            topology: "multi-region".to_string(),
            features: vec![
                "service_discovery".to_string(),
                "load_balancing".to_string(),
                "cross_region_routing".to_string(),
                "global_consensus".to_string(),
            ],
            discovery_method: "consul".to_string(),
            datacenter: "global".to_string(),
        },
        ResourcePreset::HighPerformance,
        NetworkingPreset::ServiceMesh {
            ports: vec![8500, 8501, 8502, 8600],
            discovery_method: "consul".to_string(),
        },
    );

    let storage = PrimalConfigFactory::create_storage(
        storage_name,
        storage_type,
        2,
        vec![orchestrator_name.to_string()],
        PrimalPreset::DistributedStorage {
            storage_type: "object".to_string(),
            replication_factor: 5, // Higher replication for cross-region
            consistency: "strong".to_string(),
            features: vec![
                "encryption".to_string(),
                "compression".to_string(),
                "cross_region_replication".to_string(),
                "conflict_resolution".to_string(),
            ],
        },
        ResourcePreset::HighPerformance,
        NetworkingPreset::Storage {
            ports: vec![9000, 9001, 9002, 9003],
            service_name: format!("{}-global-storage", storage_name),
        },
    );

    let manifest = TestManifestBuilder::new("cross-region-integration")
        .with_metadata(
            "1.0.0",
            "Cross-region integration test",
            BiomeSpecialization::DataCenter,
        )
        .with_tags(vec![
            "integration".to_string(),
            "cross-region".to_string(),
            "global".to_string(),
        ])
        .with_attribution("biomeOS Global Team", "MIT")
        .with_primal(orchestrator)
        .with_primal(storage)
        .build();

    // Validate cross-region configurations
    let orchestrator_spec = manifest.primals.get(orchestrator_name).unwrap();
    let storage_spec = manifest.primals.get(storage_name).unwrap();

    // Check orchestrator has cross-region features
    let orchestrator_config = orchestrator_spec.config.as_ref().unwrap();
    assert_eq!(orchestrator_config["topology"], "multi-region");
    assert_eq!(orchestrator_config["datacenter"], "global");
    assert!(orchestrator_config["features"].as_array().unwrap()
        .contains(&json!("cross_region_routing")));

    // Check storage has cross-region replication
    let storage_config = storage_spec.config.as_ref().unwrap();
    assert_eq!(storage_config["replication_factor"], 5);
    assert_eq!(storage_config["consistency"], "strong");
    assert!(storage_config["features"].as_array().unwrap()
        .contains(&json!("cross_region_replication")));

    // Check enhanced networking for cross-region
    let orchestrator_net = orchestrator_spec.networking.as_ref().unwrap();
    assert!(orchestrator_net.ports.as_ref().unwrap().contains(&8600));

    let storage_net = storage_spec.networking.as_ref().unwrap();
    assert!(storage_net.ports.as_ref().unwrap().contains(&9003));
}

/// Test performance optimization scenarios
pub fn test_performance_optimization_integration(
    orchestrator_name: &str,
    orchestrator_type: PrimalType,
    storage_name: &str,
    storage_type: PrimalType,
) {
    let orchestrator = PrimalConfigFactory::create_orchestrator(
        orchestrator_name,
        orchestrator_type,
        1,
        PrimalPreset::ServiceMeshOrchestrator {
            topology: "mesh".to_string(),
            features: vec![
                "service_discovery".to_string(),
                "intelligent_load_balancing".to_string(),
                "performance_monitoring".to_string(),
                "auto_scaling".to_string(),
            ],
            discovery_method: "consul".to_string(),
            datacenter: "performance".to_string(),
        },
        ResourcePreset::HighPerformance,
        NetworkingPreset::ServiceMesh {
            ports: vec![8500, 8501, 8502],
            discovery_method: "consul".to_string(),
        },
    );

    let storage = PrimalConfigFactory::create_storage(
        storage_name,
        storage_type,
        2,
        vec![orchestrator_name.to_string()],
        PrimalPreset::DistributedStorage {
            storage_type: "object".to_string(),
            replication_factor: 3,
            consistency: "eventual".to_string(),
            features: vec![
                "compression".to_string(),
                "deduplication".to_string(),
                "caching".to_string(),
                "performance_optimization".to_string(),
            ],
        },
        ResourcePreset::HighPerformance,
        NetworkingPreset::Storage {
            ports: vec![9000, 9001, 9002],
            service_name: format!("{}-performance-storage", storage_name),
        },
    );

    let manifest = TestManifestBuilder::new("performance-optimization-integration")
        .with_metadata(
            "1.0.0",
            "Performance optimization integration test",
            BiomeSpecialization::DataCenter,
        )
        .with_tags(vec![
            "integration".to_string(),
            "performance".to_string(),
            "optimization".to_string(),
        ])
        .with_attribution("biomeOS Performance Team", "MIT")
        .with_primal(orchestrator)
        .with_primal(storage)
        .build();

    // Validate performance configurations
    let orchestrator_spec = manifest.primals.get(orchestrator_name).unwrap();
    let storage_spec = manifest.primals.get(storage_name).unwrap();

    // Check high-performance resources
    let orchestrator_resources = orchestrator_spec.resources.as_ref().unwrap();
    assert_eq!(orchestrator_resources.cpu.as_ref().unwrap().max_cores, Some(4.0));
    assert_eq!(orchestrator_resources.memory.as_ref().unwrap().max_mb, Some(8192));
    assert_eq!(orchestrator_resources.storage.as_ref().unwrap().storage_type, Some("nvme".to_string()));

    let storage_resources = storage_spec.resources.as_ref().unwrap();
    assert_eq!(storage_resources.cpu.as_ref().unwrap().max_cores, Some(4.0));
    assert_eq!(storage_resources.memory.as_ref().unwrap().max_mb, Some(8192));
    assert_eq!(storage_resources.storage.as_ref().unwrap().storage_type, Some("nvme".to_string()));

    // Check performance features
    let orchestrator_config = orchestrator_spec.config.as_ref().unwrap();
    assert!(orchestrator_config["features"].as_array().unwrap()
        .contains(&json!("intelligent_load_balancing")));
    assert!(orchestrator_config["features"].as_array().unwrap()
        .contains(&json!("performance_monitoring")));

    let storage_config = storage_spec.config.as_ref().unwrap();
    assert!(storage_config["features"].as_array().unwrap()
        .contains(&json!("caching")));
    assert!(storage_config["features"].as_array().unwrap()
        .contains(&json!("performance_optimization")));
} 