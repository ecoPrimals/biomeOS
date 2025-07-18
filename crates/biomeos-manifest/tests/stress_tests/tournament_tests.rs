//! Tournament stress tests for biomeOS

use super::common::*;
use biomeos_manifest::*;
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn tournament_stress_test() {
    println!("🎮 Running gaming tournament stress test...");

    let start = Instant::now();

    // Simulate a massive tournament with 10,000 concurrent players
    let tournament_config = create_massive_tournament_config(10000);

    // Test manifest creation under stress
    let creation_result = create_tournament_manifest_under_stress(tournament_config);
    assert!(
        creation_result.is_ok(),
        "Tournament manifest creation failed under stress"
    );

    let manifest = creation_result.unwrap();

    // Validate that all required components are present
    assert!(manifest.topology.is_some());
    assert!(manifest.nested_biomes.is_some());
    assert!(manifest.iterative.is_some());
    assert!(manifest.monitoring.is_some());
    assert!(manifest.scaling.is_some());

    // Test serialization under stress
    let serialization_result = serde_yaml::to_string(&manifest);
    assert!(
        serialization_result.is_ok(),
        "Serialization failed under stress"
    );

    // Test deserialization under stress
    let yaml_str = serialization_result.unwrap();
    let deserialization_result: Result<BiomeManifest, _> = serde_yaml::from_str(&yaml_str);
    assert!(
        deserialization_result.is_ok(),
        "Deserialization failed under stress"
    );

    let duration = start.elapsed();
    println!("✅ Tournament stress test completed in {:?}", duration);

    // Performance assertions
    assert!(
        duration.as_secs() < 30,
        "Tournament stress test took too long: {:?}",
        duration
    );
}

/// Create a tournament manifest under stress conditions
pub fn create_tournament_manifest_under_stress(
    config: TournamentConfig,
) -> Result<BiomeManifest, String> {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: "stress-tournament".to_string(),
            version: "1.0.0".to_string(),
            description: Some(format!(
                "Stress test tournament for {} players",
                config.max_players
            )),
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
        monitoring: None,
        scaling: None,
        custom: None,
    };

    // Add tournament-specific configuration
    let mut tournament_primal = PrimalSpec {
        primal_type: "tournament".to_string(),
        version: "1.0.0".to_string(),
        source: None,
        resources: None,
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

    // Configure tournament for stress testing
    tournament_primal.config = Some(json!({
        "max_players": config.max_players,
        "regions": config.regions,
        "physics_quality": config.physics_quality,
        "anti_cheat": config.anti_cheat,
        "match_duration": config.match_duration,
        "stress_testing": true
    }));

    // Add resource requirements for large tournaments
    tournament_primal.resources = Some(ResourceRequirements {
        cpu: Some(format!("{}", config.max_players / 100)), // 1 CPU per 100 players
        memory: Some(format!("{}Mi", config.max_players * 2)), // 2Mi per player
        storage: Some("100Gi".to_string()),
        network: Some("10Gbps".to_string()),
        gpu: Some("4".to_string()),
        custom: None,
    });

    // Add scaling configuration
    tournament_primal.scaling = Some(ScalingConfig {
        min_replicas: Some(1),
        max_replicas: Some((config.max_players / 1000).max(1) as u32), // 1 replica per 1000 players
        target_cpu_utilization: Some(70),
        target_memory_utilization: Some(80),
        scale_up_threshold: Some(85),
        scale_down_threshold: Some(30),
        scale_up_delay: Some("2m".to_string()),
        scale_down_delay: Some("5m".to_string()),
        custom: None,
    });

    manifest.primals.insert("tournament".to_string(), tournament_primal);

    // Add nested biomes for regional distribution
    let mut nested_biomes = HashMap::new();
    for region in &config.regions {
        let region_biome = BiomeManifest {
            api_version: "v1".to_string(),
            kind: "Biome".to_string(),
            metadata: ManifestMetadata {
                name: format!("tournament-{}", region),
                version: "1.0.0".to_string(),
                description: Some(format!("Regional tournament biome for {}", region)),
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
            monitoring: None,
            scaling: None,
            custom: None,
        };
        nested_biomes.insert(region.clone(), region_biome);
    }
    manifest.nested_biomes = Some(nested_biomes);

    // Add topology configuration
    manifest.topology = Some(TopologyConfig {
        deployment_strategy: Some("regional".to_string()),
        load_balancing: Some("round_robin".to_string()),
        failover: Some("automatic".to_string()),
        custom: None,
    });

    // Add iterative configuration for stress testing
    manifest.iterative = Some(IterativeConfig {
        enabled: true,
        max_iterations: Some(1000),
        iteration_delay: Some("100ms".to_string()),
        convergence_criteria: Some("stability".to_string()),
        custom: None,
    });

    // Add monitoring configuration
    manifest.monitoring = Some(MonitoringConfig {
        enabled: true,
        metrics_interval: Some("10s".to_string()),
        log_level: Some("info".to_string()),
        alerts: Some(vec!["high_cpu".to_string(), "high_memory".to_string()]),
        custom: None,
    });

    // Add scaling configuration
    manifest.scaling = Some(ScalingConfig {
        min_replicas: Some(1),
        max_replicas: Some((config.max_players / 500).max(1) as u32),
        target_cpu_utilization: Some(75),
        target_memory_utilization: Some(80),
        scale_up_threshold: Some(85),
        scale_down_threshold: Some(30),
        scale_up_delay: Some("2m".to_string()),
        scale_down_delay: Some("5m".to_string()),
        custom: None,
    });

    Ok(manifest)
} 