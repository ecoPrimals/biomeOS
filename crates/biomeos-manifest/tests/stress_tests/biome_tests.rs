//! Biome creation and nesting stress tests

use super::common::*;
use biomeos_manifest::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn concurrent_biome_creation_stress() {
    println!("🔄 Running concurrent biome creation stress test...");

    let start = Instant::now();
    let biome_count = 100;
    let results = Arc::new(Mutex::new(Vec::new()));

    // Spawn concurrent threads to create biomes
    let mut handles = vec![];
    for i in 0..biome_count {
        let results = Arc::clone(&results);
        let handle = thread::spawn(move || {
            let biome_name = format!("stress-biome-{}", i);
            let result = create_stress_biome(biome_name);
            results.lock().unwrap().push(result);
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    let all_results = results.lock().unwrap();
    let success_count = all_results.iter().filter(|r| r.is_ok()).count();

    println!("✅ Created {} biomes successfully out of {}", success_count, biome_count);

    // Assertions
    assert_eq!(success_count, biome_count, "Not all biomes were created successfully");

    let duration = start.elapsed();
    println!("🏁 Concurrent biome creation completed in {:?}", duration);

    // Performance assertions
    assert!(
        duration.as_secs() < 60,
        "Concurrent biome creation took too long: {:?}",
        duration
    );
}

#[test]
fn recursive_nesting_stress() {
    println!("🔄 Running recursive nesting stress test...");

    let start = Instant::now();

    // Test deep nesting
    let deep_biome_result = create_deeply_nested_biome(20);
    assert!(
        deep_biome_result.is_ok(),
        "Deep nesting failed: {:?}",
        deep_biome_result.err()
    );

    // Test wide nesting
    let wide_biome_result = create_widely_nested_biome(50);
    assert!(
        wide_biome_result.is_ok(),
        "Wide nesting failed: {:?}",
        wide_biome_result.err()
    );

    let duration = start.elapsed();
    println!("✅ Recursive nesting stress test completed in {:?}", duration);

    // Performance assertions
    assert!(
        duration.as_secs() < 45,
        "Recursive nesting took too long: {:?}",
        duration
    );
}

/// Create a stress biome for testing
pub fn create_stress_biome(name: String) -> Result<BiomeManifest, String> {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: name.clone(),
            version: "1.0.0".to_string(),
            description: Some(format!("Stress test biome: {}", name)),
            specialization: Some(BiomeSpecialization::WebServer),
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

    // Add a simple primal for testing
    let primal = PrimalSpec {
        primal_type: "web".to_string(),
        version: "1.0.0".to_string(),
        source: None,
        resources: Some(ResourceRequirements {
            cpu: Some("100m".to_string()),
            memory: Some("128Mi".to_string()),
            storage: Some("1Gi".to_string()),
            network: Some("100Mbps".to_string()),
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
    };

    manifest.primals.insert("web".to_string(), primal);

    // Simulate some processing time
    thread::sleep(Duration::from_millis(10));

    Ok(manifest)
}

/// Create a deeply nested biome for testing
pub fn create_deeply_nested_biome(depth: usize) -> Result<BiomeManifest, String> {
    if depth == 0 {
        return create_stress_biome("leaf-biome".to_string());
    }

    let mut parent_manifest = create_stress_biome(format!("nested-biome-{}", depth))?;
    
    // Create nested biome
    let child_manifest = create_deeply_nested_biome(depth - 1)?;
    
    let mut nested_biomes = HashMap::new();
    nested_biomes.insert(format!("child-{}", depth), child_manifest);
    parent_manifest.nested_biomes = Some(nested_biomes);

    Ok(parent_manifest)
}

/// Create a widely nested biome for testing
pub fn create_widely_nested_biome(width: usize) -> Result<BiomeManifest, String> {
    let mut parent_manifest = create_stress_biome("wide-parent".to_string())?;
    
    let mut nested_biomes = HashMap::new();
    for i in 0..width {
        let child_manifest = create_stress_biome(format!("wide-child-{}", i))?;
        nested_biomes.insert(format!("child-{}", i), child_manifest);
    }
    
    parent_manifest.nested_biomes = Some(nested_biomes);

    Ok(parent_manifest)
}

/// Create a memory-intensive biome for testing
pub fn create_memory_intensive_biome(component_count: usize) -> Result<BiomeManifest, String> {
    let mut manifest = create_stress_biome("memory-intensive".to_string())?;
    
    // Add many primals to increase memory usage
    for i in 0..component_count {
        let primal = PrimalSpec {
            primal_type: format!("component-{}", i),
            version: "1.0.0".to_string(),
            source: None,
            resources: Some(ResourceRequirements {
                cpu: Some("10m".to_string()),
                memory: Some("64Mi".to_string()),
                storage: Some("100Mi".to_string()),
                network: Some("10Mbps".to_string()),
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
        };
        manifest.primals.insert(format!("component-{}", i), primal);
    }

    Ok(manifest)
} 