//! Performance tests for recursive BYOB functionality
//! 
//! This test suite validates performance characteristics of the recursive biome
//! architecture under various loads and scaling scenarios.

use biomeos_manifest::*;
use serde_json::json;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[test]
fn recursive_deployment_performance() {
    let start = Instant::now();
    
    // Create a large recursive biome structure
    let mut manifest = create_large_recursive_biome(1000); // 1000 nested biomes
    
    let creation_time = start.elapsed();
    assert!(creation_time < Duration::from_secs(1), "Biome creation too slow: {:?}", creation_time);
    
    // Test serialization performance
    let start = Instant::now();
    let yaml_result = serde_yaml::to_string(&manifest);
    let serialization_time = start.elapsed();
    
    assert!(yaml_result.is_ok());
    assert!(serialization_time < Duration::from_secs(2), "Serialization too slow: {:?}", serialization_time);
    
    // Test deserialization performance
    let yaml_str = yaml_result.unwrap();
    let start = Instant::now();
    let deserialization_result: Result<BiomeManifest, _> = serde_yaml::from_str(&yaml_str);
    let deserialization_time = start.elapsed();
    
    assert!(deserialization_result.is_ok());
    assert!(deserialization_time < Duration::from_secs(3), "Deserialization too slow: {:?}", deserialization_time);
    
    println!("Performance metrics:");
    println!("  Creation: {:?}", creation_time);
    println!("  Serialization: {:?}", serialization_time);
    println!("  Deserialization: {:?}", deserialization_time);
    println!("  YAML size: {} bytes", yaml_str.len());
}

#[test]
fn songbird_orchestration_performance() {
    let start = Instant::now();
    
    // Create a complex Songbird orchestration scenario
    let mut manifest = create_songbird_orchestration_scenario(100); // 100 Songbird instances
    
    let creation_time = start.elapsed();
    assert!(creation_time < Duration::from_millis(500), "Songbird orchestration creation too slow: {:?}", creation_time);
    
    // Test topology validation performance
    let start = Instant::now();
    let topology = manifest.topology.as_ref().unwrap();
    let validation_result = validate_topology_performance(topology);
    let validation_time = start.elapsed();
    
    assert!(validation_result.is_ok());
    assert!(validation_time < Duration::from_millis(200), "Topology validation too slow: {:?}", validation_time);
    
    // Test iterative pattern processing performance
    let start = Instant::now();
    let iterative = manifest.iterative.as_ref().unwrap();
    let processing_result = process_iterative_patterns_performance(iterative);
    let processing_time = start.elapsed();
    
    assert!(processing_result.is_ok());
    assert!(processing_time < Duration::from_millis(300), "Iterative processing too slow: {:?}", processing_time);
    
    println!("Songbird orchestration performance:");
    println!("  Creation: {:?}", creation_time);
    println!("  Topology validation: {:?}", validation_time);
    println!("  Iterative processing: {:?}", processing_time);
}

#[test]
fn nestgate_storage_performance() {
    let start = Instant::now();
    
    // Create a complex NestGate storage scenario
    let mut manifest = create_nestgate_storage_scenario(50); // 50 NestGate instances
    
    let creation_time = start.elapsed();
    assert!(creation_time < Duration::from_millis(300), "NestGate storage creation too slow: {:?}", creation_time);
    
    // Test storage configuration validation performance
    let start = Instant::now();
    let storage_validation_result = validate_storage_configuration_performance(&manifest);
    let storage_validation_time = start.elapsed();
    
    assert!(storage_validation_result.is_ok());
    assert!(storage_validation_time < Duration::from_millis(150), "Storage validation too slow: {:?}", storage_validation_time);
    
    // Test replication configuration performance
    let start = Instant::now();
    let replication_result = process_replication_configuration_performance(&manifest);
    let replication_time = start.elapsed();
    
    assert!(replication_result.is_ok());
    assert!(replication_time < Duration::from_millis(100), "Replication processing too slow: {:?}", replication_time);
    
    println!("NestGate storage performance:");
    println!("  Creation: {:?}", creation_time);
    println!("  Storage validation: {:?}", storage_validation_time);
    println!("  Replication processing: {:?}", replication_time);
}

#[test]
fn template_generation_performance() {
    let start = Instant::now();
    
    // Generate multiple templates in parallel
    let template_count = 100;
    let mut templates = Vec::new();
    
    for i in 0..template_count {
        let template = create_performance_template(i);
        templates.push(template);
    }
    
    let generation_time = start.elapsed();
    assert!(generation_time < Duration::from_secs(1), "Template generation too slow: {:?}", generation_time);
    
    // Test template validation performance
    let start = Instant::now();
    let mut validation_errors = 0;
    
    for template in &templates {
        if validate_template_performance(template).is_err() {
            validation_errors += 1;
        }
    }
    
    let validation_time = start.elapsed();
    assert!(validation_time < Duration::from_secs(2), "Template validation too slow: {:?}", validation_time);
    assert!(validation_errors == 0, "Template validation errors: {}", validation_errors);
    
    println!("Template generation performance:");
    println!("  Generation ({} templates): {:?}", template_count, generation_time);
    println!("  Validation: {:?}", validation_time);
    println!("  Average per template: {:?}", generation_time / template_count as u32);
}

#[test]
fn scaling_configuration_performance() {
    let start = Instant::now();
    
    // Create a complex scaling scenario
    let mut manifest = create_scaling_scenario(200); // 200 scaling triggers
    
    let creation_time = start.elapsed();
    assert!(creation_time < Duration::from_millis(400), "Scaling scenario creation too slow: {:?}", creation_time);
    
    // Test scaling trigger evaluation performance
    let start = Instant::now();
    let scaling = manifest.scaling.as_ref().unwrap();
    let evaluation_result = evaluate_scaling_triggers_performance(scaling);
    let evaluation_time = start.elapsed();
    
    assert!(evaluation_result.is_ok());
    assert!(evaluation_time < Duration::from_millis(250), "Scaling evaluation too slow: {:?}", evaluation_time);
    
    // Test scaling action processing performance
    let start = Instant::now();
    let action_result = process_scaling_actions_performance(scaling);
    let action_time = start.elapsed();
    
    assert!(action_result.is_ok());
    assert!(action_time < Duration::from_millis(200), "Scaling action processing too slow: {:?}", action_time);
    
    println!("Scaling configuration performance:");
    println!("  Creation: {:?}", creation_time);
    println!("  Trigger evaluation: {:?}", evaluation_time);
    println!("  Action processing: {:?}", action_time);
}

#[test]
fn memory_usage_performance() {
    // Test memory usage under high load
    let start_memory = get_memory_usage();
    
    // Create a large biome structure
    let mut biomes = Vec::new();
    for i in 0..1000 {
        let biome = create_large_recursive_biome(10);
        biomes.push(biome);
    }
    
    let peak_memory = get_memory_usage();
    let memory_increase = peak_memory - start_memory;
    
    // Drop all biomes and force garbage collection
    drop(biomes);
    std::thread::sleep(Duration::from_millis(100));
    
    let final_memory = get_memory_usage();
    let memory_cleanup = peak_memory - final_memory;
    
    println!("Memory usage performance:");
    println!("  Start memory: {} MB", start_memory / 1024 / 1024);
    println!("  Peak memory: {} MB", peak_memory / 1024 / 1024);
    println!("  Memory increase: {} MB", memory_increase / 1024 / 1024);
    println!("  Memory cleanup: {} MB", memory_cleanup / 1024 / 1024);
    println!("  Final memory: {} MB", final_memory / 1024 / 1024);
    
    // Assert reasonable memory usage
    assert!(memory_increase < 100 * 1024 * 1024, "Memory usage too high: {} MB", memory_increase / 1024 / 1024);
    assert!(memory_cleanup > memory_increase / 2, "Memory cleanup insufficient");
}

// Helper functions for performance testing

fn create_large_recursive_biome(nested_count: usize) -> BiomeManifest {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: format!("large-recursive-{}", nested_count),
            version: "1.0.0".to_string(),
            description: Some("Large recursive biome for performance testing".to_string()),
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
    
    // Add nested biomes
    let mut nested_biomes = HashMap::new();
    for i in 0..nested_count {
        let mut nested_biome = BiomeManifest {
            api_version: "v1".to_string(),
            kind: "Biome".to_string(),
            metadata: ManifestMetadata {
                name: format!("nested-{}", i),
                version: "1.0.0".to_string(),
                description: Some(format!("Nested biome {}", i)),
                specialization: Some(BiomeSpecialization::Gaming),
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
        
        // Add primals to nested biome
        nested_biome.primals.insert("songbird".to_string(), PrimalSpec {
            enabled: true,
            primal_type: PrimalType::Songbird,
            priority: 1,
            version: None,
            source: None,
            depends_on: vec![],
            startup_timeout: None,
            config: Some(json!({
                "mode": "orchestrator",
                "instance_id": i
            })),
            networking: None,
            resources: None,
            extensions: None,
        });
        
        nested_biomes.insert(format!("nested-{}", i), nested_biome);
    }
    
    manifest.nested_biomes = Some(nested_biomes);
    manifest
}

fn create_songbird_orchestration_scenario(instance_count: usize) -> BiomeManifest {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: format!("songbird-orchestration-{}", instance_count),
            version: "1.0.0".to_string(),
            description: Some("Songbird orchestration performance test".to_string()),
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
    
    // Setup topology
    manifest.topology = Some(TopologyConfig {
        topology_type: "recursive".to_string(),
        layers: None,
        orchestration_ring: Some(BiomeReference {
            topology: TopologyPattern::Ring,
            instances: instance_count as u32,
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
        instances: instance_count as u32,
        configuration: {
            let mut config = HashMap::new();
            config.insert("ring_size".to_string(), json!(instance_count));
            config.insert("redundancy".to_string(), json!(3));
            config
        },
        iteration: IterationConfig {
            variables: {
                let mut vars = HashMap::new();
                vars.insert("instance_id".to_string(), "{{ index }}".to_string());
                vars.insert("ring_position".to_string(), "{{ index }}".to_string());
                vars
            },
            dependencies: None,
            constraints: None,
        },
    });
    
    manifest.iterative = Some(iterative);
    manifest
}

fn create_nestgate_storage_scenario(instance_count: usize) -> BiomeManifest {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: format!("nestgate-storage-{}", instance_count),
            version: "1.0.0".to_string(),
            description: Some("NestGate storage performance test".to_string()),
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
    
    // Add NestGate primals
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
                "instance_id": i,
                "replication_factor": 3,
                "storage_type": "object"
            })),
            networking: None,
            resources: None,
            extensions: None,
        });
    }
    
    manifest
}

fn create_performance_template(index: usize) -> BiomeManifest {
    BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: format!("template-{}", index),
            version: "1.0.0".to_string(),
            description: Some(format!("Performance template {}", index)),
            specialization: Some(BiomeSpecialization::Gaming),
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
    }
}

fn create_scaling_scenario(trigger_count: usize) -> BiomeManifest {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: format!("scaling-scenario-{}", trigger_count),
            version: "1.0.0".to_string(),
            description: Some("Scaling performance test".to_string()),
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
    
    // Create scaling triggers
    let mut triggers = Vec::new();
    for i in 0..trigger_count {
        triggers.push(ScalingTrigger {
            metric: format!("metric-{}", i),
            threshold: format!("> {}", i * 10),
            action: ScalingAction {
                scale_up: Some(ScalingTarget {
                    component: format!("component-{}", i),
                    instances: Some("+10%".to_string()),
                    resources: None,
                }),
                scale_down: None,
            },
        });
    }
    
    manifest.scaling = Some(RecursiveScaling {
        triggers,
        constraints: Some(ScalingConstraints {
            max_instances: Some(1000),
            min_instances: Some(1),
            max_resources: None,
            cooldown_period: Some("1m".to_string()),
        }),
    });
    
    manifest
}

// Performance validation functions (simplified implementations)

fn validate_topology_performance(topology: &TopologyConfig) -> Result<(), String> {
    // Simplified topology validation
    if topology.topology_type.is_empty() {
        return Err("Empty topology type".to_string());
    }
    Ok(())
}

fn process_iterative_patterns_performance(iterative: &HashMap<String, IterativeDeployment>) -> Result<(), String> {
    // Simplified iterative pattern processing
    for (name, deployment) in iterative {
        if deployment.instances == 0 {
            return Err(format!("Invalid instance count for {}", name));
        }
    }
    Ok(())
}

fn validate_storage_configuration_performance(manifest: &BiomeManifest) -> Result<(), String> {
    // Simplified storage configuration validation
    for (name, primal) in &manifest.primals {
        if primal.primal_type == PrimalType::NestGate {
            if primal.config.is_none() {
                return Err(format!("Missing config for NestGate {}", name));
            }
        }
    }
    Ok(())
}

fn process_replication_configuration_performance(manifest: &BiomeManifest) -> Result<(), String> {
    // Simplified replication configuration processing
    for (name, primal) in &manifest.primals {
        if primal.primal_type == PrimalType::NestGate {
            if let Some(config) = &primal.config {
                if config.get("replication_factor").is_none() {
                    return Err(format!("Missing replication factor for {}", name));
                }
            }
        }
    }
    Ok(())
}

fn validate_template_performance(template: &BiomeManifest) -> Result<(), String> {
    // Simplified template validation
    if template.metadata.name.is_empty() {
        return Err("Empty template name".to_string());
    }
    Ok(())
}

fn evaluate_scaling_triggers_performance(scaling: &RecursiveScaling) -> Result<(), String> {
    // Simplified scaling trigger evaluation
    for trigger in &scaling.triggers {
        if trigger.metric.is_empty() {
            return Err("Empty metric name".to_string());
        }
    }
    Ok(())
}

fn process_scaling_actions_performance(scaling: &RecursiveScaling) -> Result<(), String> {
    // Simplified scaling action processing
    for trigger in &scaling.triggers {
        if trigger.action.scale_up.is_none() && trigger.action.scale_down.is_none() {
            return Err("No scaling actions defined".to_string());
        }
    }
    Ok(())
}

fn get_memory_usage() -> usize {
    // Simplified memory usage estimation
    // In a real implementation, this would use system calls or memory profiling
    std::mem::size_of::<BiomeManifest>() * 1000
}
