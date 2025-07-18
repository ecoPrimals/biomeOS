//! Replication and eco-primal harmony stress tests

use super::common::*;
use biomeos_manifest::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn nestgate_replication_stress() {
    println!("🏠 Running NestGate replication stress test...");

    let start = Instant::now();
    let replica_counts = vec![5, 10, 25, 50];

    for replica_count in replica_counts {
        println!("Testing {} replicas", replica_count);
        
        let replication_result = create_nestgate_replication_setup(replica_count);
        assert!(
            replication_result.is_ok(),
            "NestGate replication setup failed for {} replicas: {:?}",
            replica_count,
            replication_result.err()
        );

        let manifest = replication_result.unwrap();
        
        // Validate replication structure
        assert!(manifest.nested_biomes.is_some());
        let nested_biomes = manifest.nested_biomes.as_ref().unwrap();
        assert_eq!(nested_biomes.len(), replica_count);
        
        // Validate NestGate-specific configuration
        assert!(manifest.primals.contains_key("nestgate"));
        
        // Validate cross-region replication
        let cross_region_result = add_cross_region_replication(&manifest);
        assert!(
            cross_region_result.is_ok(),
            "Cross-region replication failed: {:?}",
            cross_region_result.err()
        );
        
        println!("✅ {} replicas validated successfully", replica_count);
    }

    let duration = start.elapsed();
    println!("🏁 NestGate replication stress test completed in {:?}", duration);

    // Performance assertions
    assert!(
        duration.as_secs() < 180,
        "NestGate replication took too long: {:?}",
        duration
    );
}

#[test]
fn eco_primal_harmony_stress() {
    println!("🌿 Running eco-primal harmony stress test...");

    let start = Instant::now();
    let primal_combinations = vec![
        vec![PrimalType::Songbird, PrimalType::NestGate],
        vec![PrimalType::Songbird, PrimalType::NestGate, PrimalType::Toadstool],
        vec![PrimalType::Songbird, PrimalType::NestGate, PrimalType::Toadstool, PrimalType::Beardog],
        vec![PrimalType::Songbird, PrimalType::NestGate, PrimalType::Toadstool, PrimalType::Beardog, PrimalType::Squirrel],
    ];

    for primal_types in primal_combinations {
        println!("Testing harmony with {} primals: {:?}", primal_types.len(), primal_types);
        
        let harmony_result = create_eco_harmony_manifest(primal_types.clone());
        assert!(
            harmony_result.is_ok(),
            "Eco-primal harmony failed for {:?}: {:?}",
            primal_types,
            harmony_result.err()
        );

        let manifest = harmony_result.unwrap();
        
        // Validate primal presence
        assert_eq!(manifest.primals.len(), primal_types.len());
        
        // Validate primal dependencies
        let dependency_result = validate_primal_dependencies(&manifest);
        assert!(
            dependency_result.is_ok(),
            "Primal dependency validation failed: {:?}",
            dependency_result.err()
        );
        
        println!("✅ Harmony with {} primals validated successfully", primal_types.len());
    }

    let duration = start.elapsed();
    println!("🏁 Eco-primal harmony stress test completed in {:?}", duration);

    // Performance assertions
    assert!(
        duration.as_secs() < 120,
        "Eco-primal harmony took too long: {:?}",
        duration
    );
}

/// Create a NestGate replication setup for testing
pub fn create_nestgate_replication_setup(
    replica_count: usize,
) -> Result<BiomeManifest, String> {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: format!("nestgate-replication-{}", replica_count),
            version: "1.0.0".to_string(),
            description: Some(format!("NestGate replication with {} replicas", replica_count)),
            specialization: Some(BiomeSpecialization::DataStorage),
            tags: None,
            author: None,
            created: None,
            repository: None,
            license: None,
            created_by: None,
            forked_from: None,
            sharing: None,
            niches: None,
            template: None,
            custom: None,
        },
        sources: SourceConfig::default(),
        primals: HashMap::new(),
        services: HashMap::new(),
        mycorrhiza: MycorrhizaConfig::default(),
        volumes: HashMap::new(),
        networks: HashMap::new(),
        networking: None,
        security: None,
        resources: None,
        schedules: HashMap::new(),
        environments: HashMap::new(),
        dependencies: None,
        extensions: None,
        biomes: None,
        nested_biomes: None,
        topology: None,
        iterative: None,
        monitoring: None,
        scaling: None,
        custom: None,
    };

    // Add NestGate primal
    let nestgate_primal = PrimalSpec {
        primal_type: "nestgate".to_string(),
        version: "1.0.0".to_string(),
        source: None,
        resources: Some(ResourceRequirements {
            cpu: Some("1000m".to_string()),
            memory: Some("2Gi".to_string()),
            storage: Some("10Gi".to_string()),
            network: Some("10Gbps".to_string()),
            gpu: None,
            custom: None,
        }),
        health_check: None,
        config: None,
        dependencies: None,
        networks: None,
        volumes: None,
        environment: None,
        scaling: Some(ScalingConfig {
            min_replicas: Some(replica_count as u32),
            max_replicas: Some((replica_count * 2) as u32),
            target_cpu_utilization: Some(70),
            target_memory_utilization: Some(80),
            scale_up_threshold: Some(80),
            scale_down_threshold: Some(40),
            scale_up_delay: Some("1m".to_string()),
            scale_down_delay: Some("5m".to_string()),
            custom: None,
        }),
        security: None,
        monitoring: None,
        custom: None,
    };

    manifest.primals.insert("nestgate".to_string(), nestgate_primal);

    // Create replica nodes as nested biomes
    let mut nested_biomes = HashMap::new();
    for i in 0..replica_count {
        let replica_manifest = BiomeManifest {
            api_version: "v1".to_string(),
            kind: "Biome".to_string(),
            metadata: ManifestMetadata {
                name: format!("nestgate-replica-{}", i),
                version: "1.0.0".to_string(),
                description: Some(format!("NestGate replica node {}", i)),
                specialization: Some(BiomeSpecialization::DataStorage),
                tags: None,
                author: None,
                created: None,
                repository: None,
                license: None,
                created_by: None,
                forked_from: None,
                sharing: None,
                niches: None,
                template: None,
                custom: None,
            },
            sources: SourceConfig::default(),
            primals: HashMap::new(),
            services: HashMap::new(),
            mycorrhiza: MycorrhizaConfig::default(),
            volumes: HashMap::new(),
            networks: HashMap::new(),
            networking: None,
            security: None,
            resources: None,
            schedules: HashMap::new(),
            environments: HashMap::new(),
            dependencies: None,
            extensions: None,
            biomes: None,
            nested_biomes: None,
            topology: None,
            iterative: None,
            monitoring: None,
            scaling: None,
            custom: None,
        };
        nested_biomes.insert(format!("replica-{}", i), replica_manifest);
    }
    manifest.nested_biomes = Some(nested_biomes);

    // Add topology configuration for replication
    manifest.topology = Some(TopologyConfig {
        deployment_strategy: Some("replication".to_string()),
        load_balancing: Some("consistent_hash".to_string()),
        failover: Some("automatic_failover".to_string()),
        custom: None,
    });

    Ok(manifest)
}

/// Create an eco-primal harmony manifest for testing
pub fn create_eco_harmony_manifest(primal_types: Vec<PrimalType>) -> Result<BiomeManifest, String> {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: "eco-harmony".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Eco-primal harmony test".to_string()),
            specialization: Some(BiomeSpecialization::GenericCompute),
            tags: None,
            author: None,
            created: None,
            repository: None,
            license: None,
            created_by: None,
            forked_from: None,
            sharing: None,
            niches: None,
            template: None,
            custom: None,
        },
        sources: SourceConfig::default(),
        primals: HashMap::new(),
        services: HashMap::new(),
        mycorrhiza: MycorrhizaConfig::default(),
        volumes: HashMap::new(),
        networks: HashMap::new(),
        networking: None,
        security: None,
        resources: None,
        schedules: HashMap::new(),
        environments: HashMap::new(),
        dependencies: None,
        extensions: None,
        biomes: None,
        nested_biomes: None,
        topology: None,
        iterative: None,
        monitoring: None,
        scaling: None,
        custom: None,
    };

    // Add each primal type
    for primal_type in primal_types {
        let primal_name = format!("{:?}", primal_type).to_lowercase();
        let primal_spec = match primal_type {
            PrimalType::Songbird => create_songbird_primal(),
            PrimalType::NestGate => create_nestgate_primal(),
            PrimalType::Toadstool => create_toadstool_primal(),
            PrimalType::Beardog => create_beardog_primal(),
            PrimalType::Squirrel => create_squirrel_primal(),
        };
        
        manifest.primals.insert(primal_name, primal_spec);
    }

    Ok(manifest)
}

/// Create a Songbird primal specification
fn create_songbird_primal() -> PrimalSpec {
    PrimalSpec {
        primal_type: "songbird".to_string(),
        version: "1.0.0".to_string(),
        source: None,
        resources: Some(ResourceRequirements {
            cpu: Some("500m".to_string()),
            memory: Some("1Gi".to_string()),
            storage: Some("5Gi".to_string()),
            network: Some("1Gbps".to_string()),
            gpu: None,
            custom: None,
        }),
        health_check: None,
        config: None,
        dependencies: None,
        networks: None,
        volumes: None,
        environment: None,
        scaling: None,
        security: None,
        monitoring: None,
        custom: None,
    }
}

/// Create a NestGate primal specification
fn create_nestgate_primal() -> PrimalSpec {
    PrimalSpec {
        primal_type: "nestgate".to_string(),
        version: "1.0.0".to_string(),
        source: None,
        resources: Some(ResourceRequirements {
            cpu: Some("1000m".to_string()),
            memory: Some("2Gi".to_string()),
            storage: Some("10Gi".to_string()),
            network: Some("10Gbps".to_string()),
            gpu: None,
            custom: None,
        }),
        health_check: None,
        config: None,
        dependencies: None,
        networks: None,
        volumes: None,
        environment: None,
        scaling: None,
        security: None,
        monitoring: None,
        custom: None,
    }
}

/// Create a Toadstool primal specification
fn create_toadstool_primal() -> PrimalSpec {
    PrimalSpec {
        primal_type: "toadstool".to_string(),
        version: "1.0.0".to_string(),
        source: None,
        resources: Some(ResourceRequirements {
            cpu: Some("750m".to_string()),
            memory: Some("1.5Gi".to_string()),
            storage: Some("8Gi".to_string()),
            network: Some("5Gbps".to_string()),
            gpu: None,
            custom: None,
        }),
        health_check: None,
        config: None,
        dependencies: None,
        networks: None,
        volumes: None,
        environment: None,
        scaling: None,
        security: None,
        monitoring: None,
        custom: None,
    }
}

/// Create a Beardog primal specification
fn create_beardog_primal() -> PrimalSpec {
    PrimalSpec {
        primal_type: "beardog".to_string(),
        version: "1.0.0".to_string(),
        source: None,
        resources: Some(ResourceRequirements {
            cpu: Some("2000m".to_string()),
            memory: Some("4Gi".to_string()),
            storage: Some("20Gi".to_string()),
            network: Some("10Gbps".to_string()),
            gpu: None,
            custom: None,
        }),
        health_check: None,
        config: None,
        dependencies: None,
        networks: None,
        volumes: None,
        environment: None,
        scaling: None,
        security: None,
        monitoring: None,
        custom: None,
    }
}

/// Create a Squirrel primal specification
fn create_squirrel_primal() -> PrimalSpec {
    PrimalSpec {
        primal_type: "squirrel".to_string(),
        version: "1.0.0".to_string(),
        source: None,
        resources: Some(ResourceRequirements {
            cpu: Some("250m".to_string()),
            memory: Some("512Mi".to_string()),
            storage: Some("2Gi".to_string()),
            network: Some("1Gbps".to_string()),
            gpu: None,
            custom: None,
        }),
        health_check: None,
        config: None,
        dependencies: None,
        networks: None,
        volumes: None,
        environment: None,
        scaling: None,
        security: None,
        monitoring: None,
        custom: None,
    }
} 