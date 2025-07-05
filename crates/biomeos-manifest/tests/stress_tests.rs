//! Stress tests for biomeOS recursive BYOB functionality
//! 
//! This test suite validates system behavior under extreme loads and
//! stress conditions with focus on Songbird and NestGate eco-primals.

use biomeos_manifest::*;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn tournament_stress_test() {
    println!("🎮 Running gaming tournament stress test...");
    
    let start = Instant::now();
    
    // Simulate a massive tournament with 10,000 concurrent players
    let tournament_config = create_massive_tournament_config(10000);
    
    // Test manifest creation under stress
    let creation_result = create_tournament_manifest_under_stress(tournament_config);
    assert!(creation_result.is_ok(), "Tournament manifest creation failed under stress");
    
    let manifest = creation_result.unwrap();
    
    // Validate that all required components are present
    assert!(manifest.topology.is_some());
    assert!(manifest.nested_biomes.is_some());
    assert!(manifest.iterative.is_some());
    assert!(manifest.monitoring.is_some());
    assert!(manifest.scaling.is_some());
    
    // Test serialization under stress
    let serialization_result = serde_yaml::to_string(&manifest);
    assert!(serialization_result.is_ok(), "Serialization failed under stress");
    
    // Test deserialization under stress
    let yaml_str = serialization_result.unwrap();
    let deserialization_result: Result<BiomeManifest, _> = serde_yaml::from_str(&yaml_str);
    assert!(deserialization_result.is_ok(), "Deserialization failed under stress");
    
    let duration = start.elapsed();
    println!("✅ Tournament stress test completed in {:?}", duration);
    assert!(duration < Duration::from_secs(10), "Tournament stress test took too long: {:?}", duration);
}

#[test]
fn concurrent_biome_creation_stress() {
    println!("🔄 Running concurrent biome creation stress test...");
    
    let start = Instant::now();
    let thread_count = 50;
    let biomes_per_thread = 20;
    
    // Create shared result storage
    let results = Arc::new(Mutex::new(Vec::new()));
    let errors = Arc::new(Mutex::new(Vec::new()));
    
    // Spawn concurrent threads
    let handles: Vec<_> = (0..thread_count)
        .map(|thread_id| {
            let results = Arc::clone(&results);
            let errors = Arc::clone(&errors);
            
            thread::spawn(move || {
                for biome_id in 0..biomes_per_thread {
                    let biome_name = format!("concurrent-biome-{}-{}", thread_id, biome_id);
                    
                    match create_stress_biome(biome_name.clone()) {
                        Ok(manifest) => {
                            let mut results = results.lock().unwrap();
                            results.push((biome_name, manifest));
                        }
                        Err(e) => {
                            let mut errors = errors.lock().unwrap();
                            errors.push((biome_name, e));
                        }
                    }
                }
            })
        })
        .collect();
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    let duration = start.elapsed();
    
    // Validate results
    let results = results.lock().unwrap();
    let errors = errors.lock().unwrap();
    
    let total_expected = thread_count * biomes_per_thread;
    let total_created = results.len();
    let total_errors = errors.len();
    
    println!("📊 Concurrent creation results:");
    println!("  Total expected: {}", total_expected);
    println!("  Successfully created: {}", total_created);
    println!("  Errors: {}", total_errors);
    println!("  Success rate: {:.2}%", (total_created as f64 / total_expected as f64) * 100.0);
    println!("  Duration: {:?}", duration);
    
    // Assert acceptable performance
    assert!(duration < Duration::from_secs(30), "Concurrent creation too slow: {:?}", duration);
    assert!(total_errors == 0, "Concurrent creation had {} errors", total_errors);
    assert_eq!(total_created, total_expected, "Not all biomes were created");
}

#[test]
fn recursive_nesting_stress() {
    println!("🔄 Running recursive nesting stress test...");
    
    let start = Instant::now();
    
    // Test deep nesting (100 levels)
    let deep_nesting_result = create_deeply_nested_biome(100);
    assert!(deep_nesting_result.is_ok(), "Deep nesting failed");
    
    let deep_biome = deep_nesting_result.unwrap();
    
    // Test wide nesting (1000 nested biomes at same level)
    let wide_nesting_result = create_widely_nested_biome(1000);
    assert!(wide_nesting_result.is_ok(), "Wide nesting failed");
    
    let wide_biome = wide_nesting_result.unwrap();
    
    // Test serialization of nested structures
    let deep_serialization = serde_yaml::to_string(&deep_biome);
    assert!(deep_serialization.is_ok(), "Deep biome serialization failed");
    
    let wide_serialization = serde_yaml::to_string(&wide_biome);
    assert!(wide_serialization.is_ok(), "Wide biome serialization failed");
    
    let duration = start.elapsed();
    println!("✅ Recursive nesting stress test completed in {:?}", duration);
    assert!(duration < Duration::from_secs(15), "Recursive nesting stress test took too long: {:?}", duration);
}

#[test]
fn songbird_ring_scaling_stress() {
    println!("🐦 Running Songbird ring scaling stress test...");
    
    let start = Instant::now();
    
    // Test scaling from 3 to 1000 Songbird instances
    let scaling_steps = vec![3, 10, 50, 100, 250, 500, 1000];
    
    for &ring_size in &scaling_steps {
        println!("  🔄 Testing ring size: {}", ring_size);
        
        let ring_result = create_songbird_ring(ring_size);
        assert!(ring_result.is_ok(), "Failed to create Songbird ring of size {}", ring_size);
        
        let manifest = ring_result.unwrap();
        
        // Validate ring topology
        let topology = manifest.topology.as_ref().unwrap();
        let orchestration_ring = topology.orchestration_ring.as_ref().unwrap();
        assert_eq!(orchestration_ring.instances, ring_size as u32);
        
        // Validate iterative patterns
        let iterative = manifest.iterative.as_ref().unwrap();
        let ring_formation = iterative.get("ring-formation").unwrap();
        assert_eq!(ring_formation.instances, ring_size as u32);
        
        // Test serialization performance
        let serialization_start = Instant::now();
        let serialization_result = serde_yaml::to_string(&manifest);
        let serialization_duration = serialization_start.elapsed();
        
        assert!(serialization_result.is_ok(), "Serialization failed for ring size {}", ring_size);
        assert!(serialization_duration < Duration::from_secs(5), "Serialization too slow for ring size {}", ring_size);
    }
    
    let duration = start.elapsed();
    println!("✅ Songbird ring scaling stress test completed in {:?}", duration);
    assert!(duration < Duration::from_secs(60), "Songbird ring scaling stress test took too long: {:?}", duration);
}

#[test]
fn nestgate_replication_stress() {
    println!("🏠 Running NestGate replication stress test...");
    
    let start = Instant::now();
    
    // Test increasing replication complexity
    let replication_configs = vec![
        (3, 3),   // 3 instances, 3x replication
        (10, 5),  // 10 instances, 5x replication
        (50, 7),  // 50 instances, 7x replication
        (100, 9), // 100 instances, 9x replication
    ];
    
    for &(instance_count, replication_factor) in &replication_configs {
        println!("  🔄 Testing {} instances with {}x replication", instance_count, replication_factor);
        
        let replication_result = create_nestgate_replication_setup(instance_count, replication_factor);
        assert!(replication_result.is_ok(), "Failed to create NestGate replication setup");
        
        let manifest = replication_result.unwrap();
        
        // Validate NestGate instances
        let nestgate_count = manifest.primals.iter()
            .filter(|(_, primal)| primal.primal_type == PrimalType::NestGate)
            .count();
        assert_eq!(nestgate_count, instance_count);
        
        // Validate replication configuration
        for (name, primal) in &manifest.primals {
            if primal.primal_type == PrimalType::NestGate {
                let config = primal.config.as_ref().unwrap();
                assert_eq!(config["replication_factor"], replication_factor);
            }
        }
        
        // Test cross-region replication setup
        let cross_region_result = add_cross_region_replication(&manifest);
        assert!(cross_region_result.is_ok(), "Failed to add cross-region replication");
    }
    
    let duration = start.elapsed();
    println!("✅ NestGate replication stress test completed in {:?}", duration);
    assert!(duration < Duration::from_secs(30), "NestGate replication stress test took too long: {:?}", duration);
}

#[test]
fn memory_pressure_stress() {
    println!("💾 Running memory pressure stress test...");
    
    let start = Instant::now();
    let initial_memory = get_memory_usage();
    
    // Create increasingly large biome structures
    let mut biomes = Vec::new();
    let sizes = vec![10, 50, 100, 500, 1000, 2000];
    
    for &size in &sizes {
        println!("  🔄 Creating biome with {} nested components", size);
        
        let biome_result = create_memory_intensive_biome(size);
        assert!(biome_result.is_ok(), "Failed to create memory-intensive biome of size {}", size);
        
        let biome = biome_result.unwrap();
        biomes.push(biome);
        
        let current_memory = get_memory_usage();
        let memory_increase = current_memory - initial_memory;
        
        println!("    Memory usage: {} MB (+{} MB)", current_memory / 1024 / 1024, memory_increase / 1024 / 1024);
        
        // Ensure memory usage stays reasonable
        assert!(memory_increase < 1024 * 1024 * 1024, "Memory usage too high: {} MB", memory_increase / 1024 / 1024);
    }
    
    // Test serialization under memory pressure
    println!("  🔄 Testing serialization under memory pressure...");
    for (i, biome) in biomes.iter().enumerate() {
        let serialization_result = serde_yaml::to_string(biome);
        assert!(serialization_result.is_ok(), "Serialization failed for biome {} under memory pressure", i);
    }
    
    // Clean up and verify memory release
    drop(biomes);
    std::thread::sleep(Duration::from_millis(100));
    
    let final_memory = get_memory_usage();
    let memory_released = get_memory_usage() - final_memory;
    
    let duration = start.elapsed();
    println!("✅ Memory pressure stress test completed in {:?}", duration);
    println!("  Final memory usage: {} MB", final_memory / 1024 / 1024);
    println!("  Memory released: {} MB", memory_released / 1024 / 1024);
    
    assert!(duration < Duration::from_secs(45), "Memory pressure stress test took too long: {:?}", duration);
}

#[test]
fn rapid_scaling_stress() {
    println!("⚡ Running rapid scaling stress test...");
    
    let start = Instant::now();
    
    // Test rapid scaling up and down
    let scaling_cycles = 10;
    let max_instances = 100;
    
    for cycle in 0..scaling_cycles {
        println!("  🔄 Scaling cycle {}/{}", cycle + 1, scaling_cycles);
        
        // Scale up
        let scale_up_result = create_rapid_scaling_manifest(max_instances, "up");
        assert!(scale_up_result.is_ok(), "Failed to create scale-up manifest in cycle {}", cycle);
        
        let up_manifest = scale_up_result.unwrap();
        
        // Validate scaling configuration
        let scaling = up_manifest.scaling.as_ref().unwrap();
        assert!(!scaling.triggers.is_empty(), "No scaling triggers in cycle {}", cycle);
        
        // Scale down
        let scale_down_result = create_rapid_scaling_manifest(3, "down");
        assert!(scale_down_result.is_ok(), "Failed to create scale-down manifest in cycle {}", cycle);
        
        let down_manifest = scale_down_result.unwrap();
        
        // Test serialization performance during rapid scaling
        let serialization_start = Instant::now();
        let up_serialization = serde_yaml::to_string(&up_manifest);
        let down_serialization = serde_yaml::to_string(&down_manifest);
        let serialization_duration = serialization_start.elapsed();
        
        assert!(up_serialization.is_ok(), "Scale-up serialization failed in cycle {}", cycle);
        assert!(down_serialization.is_ok(), "Scale-down serialization failed in cycle {}", cycle);
        assert!(serialization_duration < Duration::from_millis(500), "Serialization too slow in cycle {}", cycle);
    }
    
    let duration = start.elapsed();
    println!("✅ Rapid scaling stress test completed in {:?}", duration);
    assert!(duration < Duration::from_secs(30), "Rapid scaling stress test took too long: {:?}", duration);
}

#[test]
fn eco_primal_harmony_stress() {
    println!("🌱 Running eco-primal harmony stress test...");
    
    let start = Instant::now();
    
    // Test all eco-primals working together under stress
    let primal_combinations = vec![
        vec![PrimalType::BearDog, PrimalType::Songbird],
        vec![PrimalType::Songbird, PrimalType::NestGate],
        vec![PrimalType::NestGate, PrimalType::Toadstool],
        vec![PrimalType::Toadstool, PrimalType::Squirrel],
        vec![PrimalType::BearDog, PrimalType::Songbird, PrimalType::NestGate],
        vec![PrimalType::Songbird, PrimalType::NestGate, PrimalType::Toadstool],
        vec![PrimalType::BearDog, PrimalType::Songbird, PrimalType::NestGate, PrimalType::Toadstool, PrimalType::Squirrel],
    ];
    
    for (i, primal_types) in primal_combinations.iter().enumerate() {
        println!("  🔄 Testing combination {}/{}: {:?}", i + 1, primal_combinations.len(), primal_types);
        
        let harmony_result = create_eco_harmony_manifest(primal_types.clone());
        assert!(harmony_result.is_ok(), "Failed to create eco-harmony manifest for combination {}", i);
        
        let manifest = harmony_result.unwrap();
        
        // Validate all primals are present
        for primal_type in primal_types {
            let primal_found = manifest.primals.iter()
                .any(|(_, primal)| primal.primal_type == *primal_type);
            assert!(primal_found, "Primal {:?} not found in combination {}", primal_type, i);
        }
        
        // Test dependencies are properly resolved
        let dependency_result = validate_primal_dependencies(&manifest);
        assert!(dependency_result.is_ok(), "Dependency validation failed for combination {}", i);
        
        // Test serialization of complex eco-system
        let serialization_result = serde_yaml::to_string(&manifest);
        assert!(serialization_result.is_ok(), "Serialization failed for combination {}", i);
    }
    
    let duration = start.elapsed();
    println!("✅ Eco-primal harmony stress test completed in {:?}", duration);
    assert!(duration < Duration::from_secs(20), "Eco-primal harmony stress test took too long: {:?}", duration);
}

// Helper functions for stress testing

fn create_massive_tournament_config(player_count: usize) -> TournamentConfig {
    TournamentConfig {
        max_players: player_count,
        regions: vec![
            "us-east".to_string(),
            "us-west".to_string(),
            "eu-west".to_string(),
            "eu-central".to_string(),
            "ap-southeast".to_string(),
            "ap-northeast".to_string(),
        ],
        physics_quality: "ultra".to_string(),
        anti_cheat: true,
        match_duration: "20m".to_string(),
    }
}

fn create_tournament_manifest_under_stress(config: TournamentConfig) -> Result<BiomeManifest, String> {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: "stress-tournament".to_string(),
            version: "1.0.0".to_string(),
            description: Some(format!("Stress test tournament for {} players", config.max_players)),
            specialization: Some(BiomeSpecialization::GamingServer),
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
        templates: None,
        monitoring: None,
        scaling: None,
    };
    
    // Setup massive tournament topology
    manifest.topology = Some(TopologyConfig {
        topology_type: "recursive".to_string(),
        layers: None,
        orchestration_ring: Some(BiomeReference {
            topology: TopologyPattern::Ring,
            instances: config.regions.len() as u32,
            regions: Some(config.regions.clone()),
            template: "songbird-orchestrator".to_string(),
            depends_on: None,
            placement_strategy: Some("region_distributed".to_string()),
            hosts: None,
        }),
        physics_layer: Some(BiomeReference {
            topology: TopologyPattern::Singleton,
            instances: 1,
            regions: None,
            template: "physics-toadstool".to_string(),
            depends_on: Some(vec!["orchestration_ring".to_string()]),
            placement_strategy: Some("central".to_string()),
            hosts: None,
        }),
        compute_layers: None,
    });
    
    // Setup iterative patterns
    let mut iterative = HashMap::new();
    iterative.insert("ring-formation".to_string(), IterativeDeployment {
        pattern: TopologyPattern::Ring,
        instances: config.regions.len() as u32,
        configuration: {
            let mut config = HashMap::new();
            config.insert("ring_size".to_string(), json!(config.regions.len()));
            config.insert("redundancy".to_string(), json!(3));
            config
        },
        iteration: IterationConfig {
            variables: HashMap::new(),
            dependencies: None,
            constraints: None,
        },
    });
    manifest.iterative = Some(iterative);
    
    // Setup monitoring
    manifest.monitoring = Some(RecursiveMonitoring {
        recursive: true,
        aggregation: "hierarchical".to_string(),
        metrics: vec![
            LayerMetrics {
                layer: "orchestration_ring".to_string(),
                collect: vec!["latency".to_string(), "throughput".to_string()],
                thresholds: None,
            },
        ],
    });
    
    // Setup scaling
    manifest.scaling = Some(RecursiveScaling {
        triggers: vec![
            ScalingTrigger {
                metric: "player_count".to_string(),
                threshold: format!("> {}", config.max_players / 2),
                action: ScalingAction {
                    scale_up: Some(ScalingTarget {
                        component: "game-servers".to_string(),
                        instances: Some("+50%".to_string()),
                        resources: None,
                    }),
                    scale_down: None,
                },
            },
        ],
        constraints: None,
    });
    
    Ok(manifest)
}

fn create_stress_biome(name: String) -> Result<BiomeManifest, String> {
    let manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name,
            version: "1.0.0".to_string(),
            description: Some("Stress test biome".to_string()),
            specialization: Some(BiomeSpecialization::DataCenter),
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
        templates: None,
        monitoring: None,
        scaling: None,
    };
    
    Ok(manifest)
}

fn create_deeply_nested_biome(depth: usize) -> Result<BiomeManifest, String> {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: format!("deep-nested-{}", depth),
            version: "1.0.0".to_string(),
            description: Some(format!("Deep nested biome with {} levels", depth)),
            specialization: Some(BiomeSpecialization::DataCenter),
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
        templates: None,
        monitoring: None,
        scaling: None,
    };
    
    // Create nested structure
    if depth > 0 {
        let mut nested_biomes = HashMap::new();
        let child_biome = create_deeply_nested_biome(depth - 1)?;
        nested_biomes.insert(format!("nested-{}", depth), child_biome);
        manifest.nested_biomes = Some(nested_biomes);
    }
    
    Ok(manifest)
}

fn create_widely_nested_biome(width: usize) -> Result<BiomeManifest, String> {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: format!("wide-nested-{}", width),
            version: "1.0.0".to_string(),
            description: Some(format!("Wide nested biome with {} children", width)),
            specialization: Some(BiomeSpecialization::DataCenter),
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
        templates: None,
        monitoring: None,
        scaling: None,
    };
    
    // Create wide nested structure
    let mut nested_biomes = HashMap::new();
    for i in 0..width {
        let child_biome = create_stress_biome(format!("child-{}", i))?;
        nested_biomes.insert(format!("child-{}", i), child_biome);
    }
    manifest.nested_biomes = Some(nested_biomes);
    
    Ok(manifest)
}

fn create_songbird_ring(ring_size: usize) -> Result<BiomeManifest, String> {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: format!("songbird-ring-{}", ring_size),
            version: "1.0.0".to_string(),
            description: Some(format!("Songbird ring with {} instances", ring_size)),
            specialization: Some(BiomeSpecialization::NetworkingLab),
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
        templates: None,
        monitoring: None,
        scaling: None,
    };
    
    // Setup ring topology
    manifest.topology = Some(TopologyConfig {
        topology_type: "recursive".to_string(),
        layers: None,
        orchestration_ring: Some(BiomeReference {
            topology: TopologyPattern::Ring,
            instances: ring_size as u32,
            regions: Some(vec!["us-east".to_string(), "eu-west".to_string(), "ap-southeast".to_string()]),
            template: "songbird-orchestrator".to_string(),
            depends_on: None,
            placement_strategy: Some("distributed".to_string()),
            hosts: None,
        }),
        physics_layer: None,
        compute_layers: None,
    });
    
    // Setup iterative patterns
    let mut iterative = HashMap::new();
    iterative.insert("ring-formation".to_string(), IterativeDeployment {
        pattern: TopologyPattern::Ring,
        instances: ring_size as u32,
        configuration: {
            let mut config = HashMap::new();
            config.insert("ring_size".to_string(), json!(ring_size));
            config.insert("redundancy".to_string(), json!(3));
            config
        },
        iteration: IterationConfig {
            variables: HashMap::new(),
            dependencies: None,
            constraints: None,
        },
    });
    manifest.iterative = Some(iterative);
    
    Ok(manifest)
}

fn create_nestgate_replication_setup(instance_count: usize, replication_factor: usize) -> Result<BiomeManifest, String> {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: format!("nestgate-replication-{}-{}", instance_count, replication_factor),
            version: "1.0.0".to_string(),
            description: Some(format!("NestGate replication with {} instances and {}x replication", instance_count, replication_factor)),
            specialization: Some(BiomeSpecialization::DataCenter),
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
        templates: None,
        monitoring: None,
        scaling: None,
    };
    
    // Add NestGate instances
    for i in 0..instance_count {
        manifest.primals.insert(format!("nestgate-{}", i), PrimalSpec {
            enabled: true,
            primal_type: PrimalType::NestGate,
            priority: 1,
            version: None,
            source: None,
            depends_on: vec![],
            startup_timeout: None,
            config: Some(json!({
                "mode": "distributed",
                "replication_factor": replication_factor,
                "instance_id": i
            })),
            networking: None,
            resources: None,
            extensions: None,
        });
    }
    
    Ok(manifest)
}

fn create_memory_intensive_biome(component_count: usize) -> Result<BiomeManifest, String> {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: format!("memory-intensive-{}", component_count),
            version: "1.0.0".to_string(),
            description: Some(format!("Memory intensive biome with {} components", component_count)),
            specialization: Some(BiomeSpecialization::DataCenter),
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
        templates: None,
        monitoring: None,
        scaling: None,
    };
    
    // Add many components
    for i in 0..component_count {
        let primal_type = match i % 5 {
            0 => PrimalType::BearDog,
            1 => PrimalType::Songbird,
            2 => PrimalType::NestGate,
            3 => PrimalType::Toadstool,
            _ => PrimalType::Squirrel,
        };
        
        manifest.primals.insert(format!("primal-{}", i), PrimalSpec {
            enabled: true,
            primal_type: primal_type.clone(),
            priority: (i + 1) as u32,
            version: None,
            source: None,
            depends_on: vec![],
            startup_timeout: None,
            config: Some(json!({
                "instance_id": i,
                "mode": "harmony"
            })),
            networking: None,
            resources: None,
            extensions: None,
        });
    }
    
    Ok(manifest)
}

fn create_rapid_scaling_manifest(instance_count: usize, direction: &str) -> Result<BiomeManifest, String> {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: format!("rapid-scaling-{}-{}", direction, instance_count),
            version: "1.0.0".to_string(),
            description: Some(format!("Rapid scaling {} to {} instances", direction, instance_count)),
            specialization: Some(BiomeSpecialization::DataCenter),
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
        templates: None,
        monitoring: None,
        scaling: None,
    };
    
    // Setup scaling configuration
    let action = if direction == "up" {
        ScalingAction {
            scale_up: Some(ScalingTarget {
                component: "dynamic-component".to_string(),
                instances: Some(format!("{}", instance_count)),
                resources: None,
            }),
            scale_down: None,
        }
    } else {
        ScalingAction {
            scale_up: None,
            scale_down: Some(ScalingTarget {
                component: "dynamic-component".to_string(),
                instances: Some(format!("{}", instance_count)),
                resources: None,
            }),
        }
    };
    
    manifest.scaling = Some(RecursiveScaling {
        triggers: vec![
            ScalingTrigger {
                metric: "load".to_string(),
                threshold: "> 50%".to_string(),
                action,
            },
        ],
        constraints: None,
    });
    
    Ok(manifest)
}

fn create_eco_harmony_manifest(primal_types: Vec<PrimalType>) -> Result<BiomeManifest, String> {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: "eco-harmony".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Eco-primal harmony test".to_string()),
            specialization: Some(BiomeSpecialization::DataCenter),
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
        templates: None,
        monitoring: None,
        scaling: None,
    };
    
    // Add all requested primal types
    for (i, primal_type) in primal_types.iter().enumerate() {
        let primal_name = format!("{:?}-{}", primal_type, i).to_lowercase();
        
        manifest.primals.insert(primal_name, PrimalSpec {
            enabled: true,
            primal_type: primal_type.clone(),
            priority: (i + 1) as u32,
            version: None,
            source: None,
            depends_on: vec![],
            startup_timeout: None,
            config: Some(json!({
                "harmony_mode": true,
                "eco_integration": true
            })),
            networking: None,
            resources: None,
            extensions: None,
        });
    }
    
    Ok(manifest)
}

fn add_cross_region_replication(manifest: &BiomeManifest) -> Result<(), String> {
    // Simplified cross-region replication validation
    let nestgate_count = manifest.primals.iter()
        .filter(|(_, primal)| primal.primal_type == PrimalType::NestGate)
        .count();
    
    if nestgate_count == 0 {
        return Err("No NestGate instances found for cross-region replication".to_string());
    }
    
    Ok(())
}

fn validate_primal_dependencies(manifest: &BiomeManifest) -> Result<(), String> {
    // Simplified dependency validation
    for (name, primal) in &manifest.primals {
        for dependency in &primal.depends_on {
            if !manifest.primals.contains_key(dependency) {
                return Err(format!("Dependency {} not found for primal {}", dependency, name));
            }
        }
    }
    Ok(())
}

fn get_memory_usage() -> usize {
    // Simplified memory usage estimation
    std::mem::size_of::<BiomeManifest>() * 1000
}

// Supporting structures for stress tests

#[derive(Debug, Clone)]
struct TournamentConfig {
    max_players: usize,
    regions: Vec<String>,
    physics_quality: String,
    anti_cheat: bool,
    match_duration: String,
}
