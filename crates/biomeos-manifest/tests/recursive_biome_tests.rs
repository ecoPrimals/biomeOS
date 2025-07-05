//! Comprehensive tests for recursive BYOB functionality
//! 
//! This test suite validates the recursive biome architecture with focus on
//! Songbird and NestGate as our first complete eco-primals.

use biomeos_manifest::*;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_biome_manifest_with_recursive_fields() {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: "test-recursive".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test recursive biome".to_string()),
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

    // Test recursive biome references
    let mut biomes = HashMap::new();
    biomes.insert("orchestration-ring".to_string(), BiomeReference {
        topology: TopologyPattern::Ring,
        instances: 3,
        regions: Some(vec!["us-east".to_string(), "eu-west".to_string()]),
        template: "songbird-orchestrator".to_string(),
        depends_on: None,
        placement_strategy: Some("region_distributed".to_string()),
        hosts: None,
    });
    
    manifest.biomes = Some(biomes);

    // Test topology configuration
    manifest.topology = Some(TopologyConfig {
        topology_type: "recursive".to_string(),
        layers: None,
        orchestration_ring: Some(BiomeReference {
            topology: TopologyPattern::Ring,
            instances: 3,
            regions: Some(vec!["us-east".to_string(), "eu-west".to_string(), "ap-southeast".to_string()]),
            template: "songbird-orchestrator".to_string(),
            depends_on: None,
            placement_strategy: Some("region_distributed".to_string()),
            hosts: None,
        }),
        physics_layer: None,
        compute_layers: None,
    });

    // Validate the structure
    assert_eq!(manifest.metadata.name, "test-recursive");
    assert!(manifest.biomes.is_some());
    assert!(manifest.topology.is_some());
    
    let biomes = manifest.biomes.as_ref().unwrap();
    assert!(biomes.contains_key("orchestration-ring"));
    
    let topology = manifest.topology.as_ref().unwrap();
    assert_eq!(topology.topology_type, "recursive");
    assert!(topology.orchestration_ring.is_some());
}

#[test]
fn test_songbird_nestgate_orchestration() {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: "songbird-nestgate-test".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test Songbird+NestGate orchestration".to_string()),
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

    // Add Songbird primal (service mesh orchestration)
    manifest.primals.insert("songbird".to_string(), PrimalSpec {
        enabled: true,
        primal_type: PrimalType::Songbird,
        priority: 1,
        version: Some("1.0.0".to_string()),
        source: None,
        depends_on: vec![],
        startup_timeout: Some("30s".to_string()),
        config: Some(json!({
            "mode": "orchestrator",
            "topology": "mesh",
            "features": ["service_discovery", "load_balancing", "health_monitoring"]
        })),
        networking: None,
        resources: None,
        extensions: None,
    });

    // Add NestGate primal (storage orchestration)
    manifest.primals.insert("nestgate".to_string(), PrimalSpec {
        enabled: true,
        primal_type: PrimalType::NestGate,
        priority: 2,
        version: Some("1.0.0".to_string()),
        source: None,
        depends_on: vec!["songbird".to_string()],
        startup_timeout: Some("45s".to_string()),
        config: Some(json!({
            "mode": "distributed",
            "storage_type": "object",
            "replication_factor": 3,
            "features": ["encryption", "compression", "deduplication"]
        })),
        networking: None,
        resources: None,
        extensions: None,
    });

    // Validate Songbird configuration
    let songbird = manifest.primals.get("songbird").unwrap();
    assert_eq!(songbird.primal_type, PrimalType::Songbird);
    assert_eq!(songbird.priority, 1);
    assert!(songbird.enabled);
    
    // Validate NestGate configuration
    let nestgate = manifest.primals.get("nestgate").unwrap();
    assert_eq!(nestgate.primal_type, PrimalType::NestGate);
    assert_eq!(nestgate.priority, 2);
    assert!(nestgate.depends_on.contains(&"songbird".to_string()));
    
    // Validate dependency chain
    assert!(songbird.depends_on.is_empty());
    assert!(!nestgate.depends_on.is_empty());
}

#[test]
fn test_topology_patterns() {
    // Test all topology patterns
    let ring = TopologyPattern::Ring;
    let mesh = TopologyPattern::Mesh;
    let cluster = TopologyPattern::Cluster;
    let hierarchy = TopologyPattern::Hierarchy;
    let singleton = TopologyPattern::Singleton;
    let custom = TopologyPattern::Custom("custom-pattern".to_string());

    assert_eq!(ring, TopologyPattern::Ring);
    assert_eq!(mesh, TopologyPattern::Mesh);
    assert_eq!(cluster, TopologyPattern::Cluster);
    assert_eq!(hierarchy, TopologyPattern::Hierarchy);
    assert_eq!(singleton, TopologyPattern::Singleton);
    
    match custom {
        TopologyPattern::Custom(ref name) => assert_eq!(name, "custom-pattern"),
        _ => panic!("Custom pattern not matched correctly"),
    }
}

#[test]
fn test_primal_type_enum() {
    // Test all primal types
    let beardog = PrimalType::BearDog;
    let songbird = PrimalType::Songbird;
    let nestgate = PrimalType::NestGate;
    let toadstool = PrimalType::Toadstool;
    let squirrel = PrimalType::Squirrel;
    let custom = PrimalType::Custom("custom-primal".to_string());

    assert_eq!(beardog, PrimalType::BearDog);
    assert_eq!(songbird, PrimalType::Songbird);
    assert_eq!(nestgate, PrimalType::NestGate);
    assert_eq!(toadstool, PrimalType::Toadstool);
    assert_eq!(squirrel, PrimalType::Squirrel);
    
    match custom {
        PrimalType::Custom(ref name) => assert_eq!(name, "custom-primal"),
        _ => panic!("Custom primal type not matched correctly"),
    }
}

#[test]
fn test_biome_specialization_gaming() {
    // Test gaming-related specializations
    let gaming_server = BiomeSpecialization::GamingServer;
    let gaming_client = BiomeSpecialization::GamingClient;
    let gaming_dev = BiomeSpecialization::GamingDevelopment;
    let general_gaming = BiomeSpecialization::Gaming;

    assert_eq!(gaming_server, BiomeSpecialization::GamingServer);
    assert_eq!(gaming_client, BiomeSpecialization::GamingClient);
    assert_eq!(gaming_dev, BiomeSpecialization::GamingDevelopment);
    assert_eq!(general_gaming, BiomeSpecialization::Gaming);
}

#[test]
fn test_byob_sharing_config() {
    let sharing = SharingConfig {
        public: true,
        license: Some("MIT".to_string()),
        repository: Some("https://github.com/example/biome".to_string()),
        registry: Some("biome-registry.example.com".to_string()),
        fork_permissions: ForkPermissions::Public,
        attribution_required: true,
    };

    assert!(sharing.public);
    assert_eq!(sharing.license, Some("MIT".to_string()));
    assert_eq!(sharing.fork_permissions, ForkPermissions::Public);
    assert!(sharing.attribution_required);
}

#[test]
fn test_byob_niche_classification() {
    let niche = NicheClassification {
        primary: "gaming-tournament".to_string(),
        secondary: vec!["esports".to_string(), "competitive-gaming".to_string()],
        custom: vec!["physics-simulation".to_string(), "distributed-gaming".to_string()],
    };

    assert_eq!(niche.primary, "gaming-tournament");
    assert_eq!(niche.secondary.len(), 2);
    assert_eq!(niche.custom.len(), 2);
    assert!(niche.secondary.contains(&"esports".to_string()));
    assert!(niche.custom.contains(&"physics-simulation".to_string()));
}

#[test]
fn test_manifest_serialization() {
    let manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: "test-manifest".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test manifest for serialization".to_string()),
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

    // Test YAML serialization
    let yaml_result = serde_yaml::to_string(&manifest);
    assert!(yaml_result.is_ok());
    
    let yaml_str = yaml_result.unwrap();
    assert!(yaml_str.contains("api_version: v1"));
    assert!(yaml_str.contains("name: test-manifest"));
    
    // Test YAML deserialization
    let deserialized_result: Result<BiomeManifest, _> = serde_yaml::from_str(&yaml_str);
    assert!(deserialized_result.is_ok());
    
    let deserialized = deserialized_result.unwrap();
    assert_eq!(deserialized.metadata.name, "test-manifest");
    assert_eq!(deserialized.api_version, "v1");
}
