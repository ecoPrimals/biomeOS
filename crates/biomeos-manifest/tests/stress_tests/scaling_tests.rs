//! Scaling and performance stress tests

use super::common::*;
use biomeos_manifest::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn songbird_ring_scaling_stress() {
    println!("🎵 Running Songbird ring scaling stress test...");

    let start = Instant::now();
    let ring_sizes = vec![10, 50, 100, 200];

    for ring_size in ring_sizes {
        println!("Testing ring size: {}", ring_size);
        
        let ring_result = create_songbird_ring(ring_size);
        assert!(
            ring_result.is_ok(),
            "Songbird ring creation failed for size {}: {:?}",
            ring_size,
            ring_result.err()
        );

        let manifest = ring_result.unwrap();
        
        // Validate ring structure
        assert!(manifest.nested_biomes.is_some());
        let nested_biomes = manifest.nested_biomes.as_ref().unwrap();
        assert_eq!(nested_biomes.len(), ring_size);
        
        // Validate Songbird-specific configuration
        assert!(manifest.primals.contains_key("songbird"));
        
        println!("✅ Ring size {} validated successfully", ring_size);
    }

    let duration = start.elapsed();
    println!("🏁 Songbird ring scaling completed in {:?}", duration);

    // Performance assertions
    assert!(
        duration.as_secs() < 120,
        "Songbird ring scaling took too long: {:?}",
        duration
    );
}

#[test]
fn rapid_scaling_stress() {
    println!("⚡ Running rapid scaling stress test...");

    let start = Instant::now();
    let scaling_configs = vec![
        (1, 10, 30),    // min, max, target_cpu
        (2, 20, 50),
        (5, 50, 70),
        (10, 100, 80),
    ];

    for (min, max, target_cpu) in scaling_configs {
        println!("Testing scaling config: min={}, max={}, target_cpu={}%", min, max, target_cpu);
        
        let manifest_result = create_rapid_scaling_manifest(min, max, target_cpu);
        assert!(
            manifest_result.is_ok(),
            "Rapid scaling manifest creation failed: {:?}",
            manifest_result.err()
        );

        let manifest = manifest_result.unwrap();
        
        // Validate scaling configuration
        assert!(manifest.scaling.is_some());
        let scaling = manifest.scaling.as_ref().unwrap();
        assert_eq!(scaling.min_replicas, Some(min));
        assert_eq!(scaling.max_replicas, Some(max));
        assert_eq!(scaling.target_cpu_utilization, Some(target_cpu));
        
        println!("✅ Scaling config validated successfully");
    }

    let duration = start.elapsed();
    println!("🏁 Rapid scaling stress test completed in {:?}", duration);

    // Performance assertions
    assert!(
        duration.as_secs() < 60,
        "Rapid scaling took too long: {:?}",
        duration
    );
}

#[test]
fn memory_pressure_stress() {
    println!("🧠 Running memory pressure stress test...");

    let start = Instant::now();
    let initial_memory = get_memory_usage();
    
    let component_counts = vec![100, 500, 1000, 2000];
    let mut peak_memory = initial_memory;

    for component_count in component_counts {
        println!("Testing with {} components", component_count);
        
        let manifest_result = super::biome_tests::create_memory_intensive_biome(component_count);
        assert!(
            manifest_result.is_ok(),
            "Memory intensive biome creation failed: {:?}",
            manifest_result.err()
        );

        let manifest = manifest_result.unwrap();
        
        // Validate component count
        assert_eq!(manifest.primals.len(), component_count + 1); // +1 for the web primal
        
        // Check memory usage
        let current_memory = get_memory_usage();
        peak_memory = peak_memory.max(current_memory);
        
        println!("✅ {} components validated, memory usage: {} bytes", component_count, current_memory);
    }

    let duration = start.elapsed();
    let memory_increase = peak_memory - initial_memory;
    
    println!("🏁 Memory pressure test completed in {:?}", duration);
    println!("💾 Peak memory increase: {} bytes", memory_increase);

    // Performance assertions
    assert!(
        duration.as_secs() < 90,
        "Memory pressure test took too long: {:?}",
        duration
    );
    
    // Memory assertions (allow reasonable memory growth)
    assert!(
        memory_increase < 100_000_000, // Less than 100MB increase
        "Memory usage increased too much: {} bytes",
        memory_increase
    );
}

/// Create a Songbird ring for testing
pub fn create_songbird_ring(ring_size: usize) -> Result<BiomeManifest, String> {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: format!("songbird-ring-{}", ring_size),
            version: "1.0.0".to_string(),
            description: Some(format!("Songbird ring with {} nodes", ring_size)),
            specialization: Some(BiomeSpecialization::AudioProcessing),
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

    // Add Songbird primal
    let songbird_primal = PrimalSpec {
        primal_type: "songbird".to_string(),
        version: "1.0.0".to_string(),
        source: None,
        resources: Some(ResourceRequirements {
            cpu: Some("500m".to_string()),
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
        scaling: Some(ScalingConfig {
            min_replicas: Some(1),
            max_replicas: Some(ring_size as u32),
            target_cpu_utilization: Some(75),
            target_memory_utilization: Some(80),
            scale_up_threshold: Some(85),
            scale_down_threshold: Some(30),
            scale_up_delay: Some("30s".to_string()),
            scale_down_delay: Some("2m".to_string()),
            custom: None,
        }),
        security: None,
        monitoring: None,
        custom: None,
    };

    manifest.primals.insert("songbird".to_string(), songbird_primal);

    // Create ring nodes as nested biomes
    let mut nested_biomes = HashMap::new();
    for i in 0..ring_size {
        let node_manifest = BiomeManifest {
            api_version: "v1".to_string(),
            kind: "Biome".to_string(),
            metadata: ManifestMetadata {
                name: format!("songbird-node-{}", i),
                version: "1.0.0".to_string(),
                description: Some(format!("Songbird ring node {}", i)),
                specialization: Some(BiomeSpecialization::AudioProcessing),
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
        nested_biomes.insert(format!("node-{}", i), node_manifest);
    }
    manifest.nested_biomes = Some(nested_biomes);

    // Add topology configuration for ring
    manifest.topology = Some(TopologyConfig {
        deployment_strategy: Some("ring".to_string()),
        load_balancing: Some("consistent_hash".to_string()),
        failover: Some("ring_recovery".to_string()),
        custom: None,
    });

    Ok(manifest)
}

/// Create a rapid scaling manifest for testing
pub fn create_rapid_scaling_manifest(
    min_replicas: u32,
    max_replicas: u32,
    target_cpu: u32,
) -> Result<BiomeManifest, String> {
    let mut manifest = super::biome_tests::create_stress_biome("rapid-scaling".to_string())?;
    
    // Add aggressive scaling configuration
    manifest.scaling = Some(ScalingConfig {
        min_replicas: Some(min_replicas),
        max_replicas: Some(max_replicas),
        target_cpu_utilization: Some(target_cpu),
        target_memory_utilization: Some(70),
        scale_up_threshold: Some(target_cpu + 10),
        scale_down_threshold: Some(target_cpu - 20),
        scale_up_delay: Some("10s".to_string()),   // Aggressive scaling
        scale_down_delay: Some("30s".to_string()), // Quick scale down
        custom: None,
    });

    // Add monitoring for rapid scaling
    manifest.monitoring = Some(MonitoringConfig {
        enabled: true,
        metrics_interval: Some("5s".to_string()),
        log_level: Some("debug".to_string()),
        alerts: Some(vec![
            "rapid_scale_up".to_string(),
            "rapid_scale_down".to_string(),
        ]),
        custom: None,
    });

    Ok(manifest)
} 