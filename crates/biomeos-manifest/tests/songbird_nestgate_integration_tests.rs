//! Integration tests for Songbird and NestGate eco-primals
//! 
//! This test suite validates that Songbird (service mesh) and NestGate (storage)
//! work together harmoniously in various biome configurations.

use biomeos_manifest::*;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_songbird_nestgate_basic_integration() {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: "songbird-nestgate-integration".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Basic Songbird+NestGate integration test".to_string()),
            specialization: Some(BiomeSpecialization::DataCenter),
            tags: Some(vec!["integration".to_string(), "eco-primals".to_string()]),
            author: Some("biomeOS Test Suite".to_string()),
            created: None,
            repository: None,
            license: Some("MIT".to_string()),
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

    // Add Songbird for service mesh orchestration
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
            "features": [
                "service_discovery",
                "load_balancing",
                "health_monitoring",
                "traffic_routing"
            ],
            "discovery": {
                "method": "consul",
                "refresh_interval": "30s"
            },
            "load_balancing": {
                "algorithm": "round_robin",
                "health_checks": true
            }
        })),
        networking: Some(PrimalNetworking {
            ports: Some(vec![8500, 8501, 8502]),
            host: Some("0.0.0.0".to_string()),
            discovery: Some(DiscoverySpec {
                method: "consul".to_string(),
                config: Some(json!({
                    "datacenter": "dc1",
                    "encrypt": true
                })),
            }),
        }),
        resources: Some(PrimalResources {
            cpu: Some(CpuSpec {
                max_cores: Some(2.0),
                shares: Some(1024),
            }),
            memory: Some(MemorySpec {
                max_mb: Some(4096),
                swap_mb: Some(2048),
            }),
            storage: Some(StorageSpec {
                max_mb: Some(10240),
                storage_type: Some("ssd".to_string()),
            }),
        }),
        extensions: None,
    });

    // Add NestGate for distributed storage
    manifest.primals.insert("nestgate".to_string(), PrimalSpec {
        enabled: true,
        primal_type: PrimalType::NestGate,
        priority: 2,
        version: Some("1.0.0".to_string()),
        source: None,
        depends_on: vec!["songbird".to_string()],
        startup_timeout: Some("60s".to_string()),
        config: Some(json!({
            "mode": "distributed",
            "storage_type": "object",
            "replication_factor": 3,
            "consistency": "eventual",
            "features": [
                "encryption",
                "compression",
                "deduplication",
                "versioning"
            ],
            "encryption": {
                "algorithm": "aes-256-gcm",
                "key_rotation": true
            },
            "compression": {
                "algorithm": "lz4",
                "level": "fast"
            },
            "discovery": {
                "use_songbird": true,
                "service_name": "nestgate-storage"
            }
        })),
        networking: Some(PrimalNetworking {
            ports: Some(vec![9000, 9001, 9002]),
            host: Some("0.0.0.0".to_string()),
            discovery: Some(DiscoverySpec {
                method: "songbird".to_string(),
                config: Some(json!({
                    "service_name": "nestgate-storage",
                    "tags": ["storage", "distributed", "object"]
                })),
            }),
        }),
        resources: Some(PrimalResources {
            cpu: Some(CpuSpec {
                max_cores: Some(4.0),
                shares: Some(2048),
            }),
            memory: Some(MemorySpec {
                max_mb: Some(8192),
                swap_mb: Some(4096),
            }),
            storage: Some(StorageSpec {
                max_mb: Some(1048576), // 1TB
                storage_type: Some("ssd".to_string()),
            }),
        }),
        extensions: None,
    });

    // Validate the integration
    let songbird = manifest.primals.get("songbird").unwrap();
    let nestgate = manifest.primals.get("nestgate").unwrap();

    // Check that NestGate depends on Songbird
    assert!(nestgate.depends_on.contains(&"songbird".to_string()));
    
    // Check that both primals are enabled
    assert!(songbird.enabled);
    assert!(nestgate.enabled);
    
    // Check priority ordering (Songbird starts first)
    assert!(songbird.priority < nestgate.priority);
    
    // Check that NestGate is configured to use Songbird for discovery
    let nestgate_config = nestgate.config.as_ref().unwrap();
    assert_eq!(nestgate_config["discovery"]["use_songbird"], true);
    
    // Check that both have proper networking configuration
    assert!(songbird.networking.is_some());
    assert!(nestgate.networking.is_some());
    
    let nestgate_net = nestgate.networking.as_ref().unwrap();
    assert_eq!(nestgate_net.discovery.as_ref().unwrap().method, "songbird");
}

#[test]
fn test_songbird_nestgate_gaming_tournament_integration() {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: "gaming-tournament-integration".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Songbird+NestGate integration for gaming tournament".to_string()),
            specialization: Some(BiomeSpecialization::GamingServer),
            tags: Some(vec!["gaming".to_string(), "tournament".to_string(), "integration".to_string()]),
            author: Some("biomeOS Gaming Team".to_string()),
            created: None,
            repository: None,
            license: Some("MIT".to_string()),
            created_by: None,
            forked_from: None,
            sharing: None,
            niches: Some(NicheClassification {
                primary: "gaming-tournament".to_string(),
                secondary: vec!["esports".to_string(), "competitive-gaming".to_string()],
                custom: vec!["songbird-orchestration".to_string(), "nestgate-storage".to_string()],
            }),
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

    // Add Songbird for tournament orchestration
    manifest.primals.insert("songbird".to_string(), PrimalSpec {
        enabled: true,
        primal_type: PrimalType::Songbird,
        priority: 1,
        version: Some("1.0.0".to_string()),
        source: None,
        depends_on: vec![],
        startup_timeout: Some("45s".to_string()),
        config: Some(json!({
            "mode": "orchestrator",
            "topology": "ring",
            "features": [
                "service_discovery",
                "load_balancing",
                "health_monitoring",
                "cross_region_routing",
                "tournament_coordination"
            ],
            "tournament": {
                "match_making": true,
                "player_balancing": true,
                "server_allocation": true
            },
            "ring_config": {
                "size": 3,
                "redundancy": 2,
                "consensus": "raft"
            }
        })),
        networking: Some(PrimalNetworking {
            ports: Some(vec![8500, 8501, 8502, 8503]),
            host: Some("0.0.0.0".to_string()),
            discovery: Some(DiscoverySpec {
                method: "consul".to_string(),
                config: Some(json!({
                    "datacenter": "tournament",
                    "encrypt": true,
                    "cross_region": true
                })),
            }),
        }),
        resources: Some(PrimalResources {
            cpu: Some(CpuSpec {
                max_cores: Some(4.0),
                shares: Some(2048),
            }),
            memory: Some(MemorySpec {
                max_mb: Some(8192),
                swap_mb: Some(4096),
            }),
            storage: Some(StorageSpec {
                max_mb: Some(20480),
                storage_type: Some("ssd".to_string()),
            }),
        }),
        extensions: None,
    });

    // Add NestGate for tournament data storage
    manifest.primals.insert("nestgate".to_string(), PrimalSpec {
        enabled: true,
        primal_type: PrimalType::NestGate,
        priority: 2,
        version: Some("1.0.0".to_string()),
        source: None,
        depends_on: vec!["songbird".to_string()],
        startup_timeout: Some("90s".to_string()),
        config: Some(json!({
            "mode": "distributed",
            "storage_type": "hybrid",
            "replication_factor": 3,
            "consistency": "strong",
            "features": [
                "encryption",
                "compression",
                "deduplication",
                "versioning",
                "high_iops"
            ],
            "tournament": {
                "player_data": true,
                "match_replays": true,
                "statistics": true,
                "real_time_streaming": true
            },
            "performance": {
                "high_iops": true,
                "low_latency": true,
                "burst_capacity": true
            },
            "discovery": {
                "use_songbird": true,
                "service_name": "tournament-storage"
            }
        })),
        networking: Some(PrimalNetworking {
            ports: Some(vec![9000, 9001, 9002, 9003]),
            host: Some("0.0.0.0".to_string()),
            discovery: Some(DiscoverySpec {
                method: "songbird".to_string(),
                config: Some(json!({
                    "service_name": "tournament-storage",
                    "tags": ["storage", "tournament", "high-performance"]
                })),
            }),
        }),
        resources: Some(PrimalResources {
            cpu: Some(CpuSpec {
                max_cores: Some(8.0),
                shares: Some(4096),
            }),
            memory: Some(MemorySpec {
                max_mb: Some(16384),
                swap_mb: Some(8192),
            }),
            storage: Some(StorageSpec {
                max_mb: Some(2097152), // 2TB
                storage_type: Some("nvme".to_string()),
            }),
        }),
        extensions: None,
    });

    // Add tournament-specific services
    manifest.services.insert("tournament-api".to_string(), ServiceSpec {
        runtime: RuntimeType::Container,
        source: None,
        primal: "songbird".to_string(),
        image: Some("tournament-api:latest".to_string()),
        depends_on: vec!["nestgate".to_string()],
        ports: vec!["8080:8080".to_string()],
        volumes: vec!["tournament-data:/data".to_string()],
        environment: {
            let mut env = HashMap::new();
            env.insert("STORAGE_BACKEND".to_string(), "nestgate".to_string());
            env.insert("DISCOVERY_SERVICE".to_string(), "songbird".to_string());
            env.insert("TOURNAMENT_MODE".to_string(), "competitive".to_string());
            env
        },
        config: Some(json!({
            "tournament": {
                "max_players": 1000,
                "match_duration": "15m",
                "replay_retention": "30d"
            }
        })),
    });

    manifest.services.insert("match-maker".to_string(), ServiceSpec {
        runtime: RuntimeType::Container,
        source: None,
        primal: "songbird".to_string(),
        image: Some("match-maker:latest".to_string()),
        depends_on: vec!["tournament-api".to_string()],
        ports: vec!["8081:8081".to_string()],
        volumes: vec!["match-data:/data".to_string()],
        environment: {
            let mut env = HashMap::new();
            env.insert("STORAGE_BACKEND".to_string(), "nestgate".to_string());
            env.insert("DISCOVERY_SERVICE".to_string(), "songbird".to_string());
            env.insert("BALANCING_STRATEGY".to_string(), "skill_based".to_string());
            env
        },
        config: Some(json!({
            "matching": {
                "algorithm": "elo_based",
                "max_wait_time": "30s",
                "skill_variance": 0.1
            }
        })),
    });

    // Validate the gaming tournament integration
    let songbird = manifest.primals.get("songbird").unwrap();
    let nestgate = manifest.primals.get("nestgate").unwrap();

    // Check tournament-specific configurations
    let songbird_config = songbird.config.as_ref().unwrap();
    assert_eq!(songbird_config["tournament"]["match_making"], true);
    assert_eq!(songbird_config["ring_config"]["size"], 3);

    let nestgate_config = nestgate.config.as_ref().unwrap();
    assert_eq!(nestgate_config["tournament"]["player_data"], true);
    assert_eq!(nestgate_config["performance"]["high_iops"], true);

    // Check service dependencies
    let tournament_api = manifest.services.get("tournament-api").unwrap();
    assert!(tournament_api.depends_on.contains(&"nestgate".to_string()));
    assert_eq!(tournament_api.primal, "songbird");

    let match_maker = manifest.services.get("match-maker").unwrap();
    assert!(match_maker.depends_on.contains(&"tournament-api".to_string()));
    assert_eq!(match_maker.primal, "songbird");

    // Check that services are configured to use both primals
    assert_eq!(tournament_api.environment.get("STORAGE_BACKEND"), Some(&"nestgate".to_string()));
    assert_eq!(tournament_api.environment.get("DISCOVERY_SERVICE"), Some(&"songbird".to_string()));
}

#[test]
fn test_songbird_nestgate_multi_region_deployment() {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: "multi-region-songbird-nestgate".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Multi-region Songbird+NestGate deployment".to_string()),
            specialization: Some(BiomeSpecialization::EdgeComputing),
            tags: Some(vec!["multi-region".to_string(), "edge".to_string(), "distributed".to_string()]),
            author: Some("biomeOS Edge Team".to_string()),
            created: None,
            repository: None,
            license: Some("MIT".to_string()),
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

    // Setup recursive topology for multi-region deployment
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
        compute_layers: Some(vec![BiomeReference {
            topology: TopologyPattern::Mesh,
            instances: 9, // 3 per region
            regions: Some(vec!["us-east".to_string(), "eu-west".to_string(), "ap-southeast".to_string()]),
            template: "nestgate-storage".to_string(),
            depends_on: Some(vec!["orchestration_ring".to_string()]),
            placement_strategy: Some("region_balanced".to_string()),
            hosts: None,
        }]),
    });

    // Setup nested biomes
    let mut nested_biomes = HashMap::new();

    // Songbird orchestrator for each region
    let mut songbird_orchestrator = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: "songbird-orchestrator".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Regional Songbird orchestrator".to_string()),
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

    songbird_orchestrator.primals.insert("songbird".to_string(), PrimalSpec {
        enabled: true,
        primal_type: PrimalType::Songbird,
        priority: 1,
        version: Some("1.0.0".to_string()),
        source: None,
        depends_on: vec![],
        startup_timeout: Some("30s".to_string()),
        config: Some(json!({
            "mode": "orchestrator",
            "topology": "ring",
            "features": [
                "service_discovery",
                "load_balancing",
                "health_monitoring",
                "cross_region_routing",
                "wan_federation"
            ],
            "wan_federation": {
                "enabled": true,
                "encryption": true,
                "gossip_wan": true
            },
            "region_awareness": true
        })),
        networking: Some(PrimalNetworking {
            ports: Some(vec![8500, 8501, 8502, 8503, 8504]),
            host: Some("0.0.0.0".to_string()),
            discovery: Some(DiscoverySpec {
                method: "consul".to_string(),
                config: Some(json!({
                    "datacenter": "{{ region }}",
                    "encrypt": true,
                    "wan_federation": true
                })),
            }),
        }),
        resources: None,
        extensions: None,
    });

    // NestGate storage cluster for each region
    let mut nestgate_storage = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: "nestgate-storage".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Regional NestGate storage cluster".to_string()),
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

    nestgate_storage.primals.insert("nestgate".to_string(), PrimalSpec {
        enabled: true,
        primal_type: PrimalType::NestGate,
        priority: 1,
        version: Some("1.0.0".to_string()),
        source: None,
        depends_on: vec![],
        startup_timeout: Some("60s".to_string()),
        config: Some(json!({
            "mode": "distributed",
            "storage_type": "object",
            "replication_factor": 3,
            "consistency": "strong",
            "features": [
                "encryption",
                "compression",
                "deduplication",
                "cross_region_replication"
            ],
            "cross_region": {
                "enabled": true,
                "replication_factor": 2,
                "encryption": true
            },
            "discovery": {
                "use_songbird": true,
                "service_name": "nestgate-{{ region }}"
            }
        })),
        networking: Some(PrimalNetworking {
            ports: Some(vec![9000, 9001, 9002, 9003, 9004]),
            host: Some("0.0.0.0".to_string()),
            discovery: Some(DiscoverySpec {
                method: "songbird".to_string(),
                config: Some(json!({
                    "service_name": "nestgate-{{ region }}",
                    "tags": ["storage", "{{ region }}", "distributed"]
                })),
            }),
        }),
        resources: None,
        extensions: None,
    });

    nested_biomes.insert("songbird-orchestrator".to_string(), songbird_orchestrator);
    nested_biomes.insert("nestgate-storage".to_string(), nestgate_storage);
    manifest.nested_biomes = Some(nested_biomes);

    // Validate multi-region topology
    let topology = manifest.topology.as_ref().unwrap();
    assert_eq!(topology.topology_type, "recursive");
    assert!(topology.orchestration_ring.is_some());
    assert!(topology.compute_layers.is_some());

    let orchestration_ring = topology.orchestration_ring.as_ref().unwrap();
    assert_eq!(orchestration_ring.instances, 3);
    assert_eq!(orchestration_ring.regions.as_ref().unwrap().len(), 3);

    let compute_layers = topology.compute_layers.as_ref().unwrap();
    assert_eq!(compute_layers.len(), 1);
    assert_eq!(compute_layers[0].instances, 9);

    // Validate nested biomes
    let nested_biomes = manifest.nested_biomes.as_ref().unwrap();
    assert!(nested_biomes.contains_key("songbird-orchestrator"));
    assert!(nested_biomes.contains_key("nestgate-storage"));

    let songbird_biome = nested_biomes.get("songbird-orchestrator").unwrap();
    let songbird_primal = songbird_biome.primals.get("songbird").unwrap();
    let songbird_config = songbird_primal.config.as_ref().unwrap();
    assert_eq!(songbird_config["wan_federation"]["enabled"], true);

    let nestgate_biome = nested_biomes.get("nestgate-storage").unwrap();
    let nestgate_primal = nestgate_biome.primals.get("nestgate").unwrap();
    let nestgate_config = nestgate_primal.config.as_ref().unwrap();
    assert_eq!(nestgate_config["cross_region"]["enabled"], true);
}

#[test]
fn test_songbird_nestgate_dependency_validation() {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: "dependency-test".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test dependency validation".to_string()),
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

    // Add dependencies configuration
    manifest.dependencies = Some(DependencyConfig {
        requires: vec![
            DependencySpec {
                name: "songbird-runtime".to_string(),
                version: Some(">=1.0.0".to_string()),
                source: None,
                optional: false,
                reason: Some("Required for service mesh orchestration".to_string()),
            },
            DependencySpec {
                name: "nestgate-runtime".to_string(),
                version: Some(">=1.0.0".to_string()),
                source: None,
                optional: false,
                reason: Some("Required for distributed storage".to_string()),
            },
        ],
        suggests: vec![
            DependencySpec {
                name: "monitoring-stack".to_string(),
                version: Some(">=2.0.0".to_string()),
                source: None,
                optional: true,
                reason: Some("Enhanced monitoring for eco-primals".to_string()),
            },
        ],
        conflicts: vec![
            DependencySpec {
                name: "legacy-orchestrator".to_string(),
                version: Some("*".to_string()),
                source: None,
                optional: false,
                reason: Some("Conflicts with Songbird orchestration".to_string()),
            },
        ],
        features: {
            let mut features = HashMap::new();
            features.insert("cross-region-replication".to_string(), FeatureSpec {
                description: "Enable cross-region replication between NestGate instances".to_string(),
                dependencies: vec!["nestgate-runtime".to_string()],
                services: vec!["replication-service".to_string()],
                config: Some(json!({
                    "replication_factor": 2,
                    "encryption": true
                })),
                default_enabled: false,
            });
            features.insert("advanced-load-balancing".to_string(), FeatureSpec {
                description: "Enable advanced load balancing algorithms in Songbird".to_string(),
                dependencies: vec!["songbird-runtime".to_string()],
                services: vec!["load-balancer".to_string()],
                config: Some(json!({
                    "algorithms": ["weighted_round_robin", "least_connections", "ip_hash"]
                })),
                default_enabled: true,
            });
            features
        },
    });

    // Validate dependencies
    let dependencies = manifest.dependencies.as_ref().unwrap();
    assert_eq!(dependencies.requires.len(), 2);
    assert_eq!(dependencies.suggests.len(), 1);
    assert_eq!(dependencies.conflicts.len(), 1);
    assert_eq!(dependencies.features.len(), 2);

    // Check required dependencies
    let songbird_dep = dependencies.requires.iter().find(|d| d.name == "songbird-runtime").unwrap();
    assert!(!songbird_dep.optional);
    assert_eq!(songbird_dep.version, Some(">=1.0.0".to_string()));

    let nestgate_dep = dependencies.requires.iter().find(|d| d.name == "nestgate-runtime").unwrap();
    assert!(!nestgate_dep.optional);
    assert_eq!(nestgate_dep.version, Some(">=1.0.0".to_string()));

    // Check feature configurations
    let cross_region_feature = dependencies.features.get("cross-region-replication").unwrap();
    assert!(!cross_region_feature.default_enabled);
    assert!(cross_region_feature.dependencies.contains(&"nestgate-runtime".to_string()));

    let load_balancing_feature = dependencies.features.get("advanced-load-balancing").unwrap();
    assert!(load_balancing_feature.default_enabled);
    assert!(load_balancing_feature.dependencies.contains(&"songbird-runtime".to_string()));
}

#[test]
fn test_songbird_nestgate_serialization_compatibility() {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: "serialization-test".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test serialization compatibility".to_string()),
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

    // Add both primals
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
            "topology": "mesh"
        })),
        networking: None,
        resources: None,
        extensions: None,
    });

    manifest.primals.insert("nestgate".to_string(), PrimalSpec {
        enabled: true,
        primal_type: PrimalType::NestGate,
        priority: 2,
        version: Some("1.0.0".to_string()),
        source: None,
        depends_on: vec!["songbird".to_string()],
        startup_timeout: Some("60s".to_string()),
        config: Some(json!({
            "mode": "distributed",
            "storage_type": "object"
        })),
        networking: None,
        resources: None,
        extensions: None,
    });

    // Test YAML serialization
    let yaml_result = serde_yaml::to_string(&manifest);
    assert!(yaml_result.is_ok());

    let yaml_str = yaml_result.unwrap();
    
    // Verify key elements are present
    assert!(yaml_str.contains("primal_type: Songbird"));
    assert!(yaml_str.contains("primal_type: NestGate"));
    assert!(yaml_str.contains("depends_on:"));
    assert!(yaml_str.contains("- songbird"));

    // Test deserialization
    let deserialized_result: Result<BiomeManifest, _> = serde_yaml::from_str(&yaml_str);
    assert!(deserialized_result.is_ok());

    let deserialized = deserialized_result.unwrap();
    assert_eq!(deserialized.primals.len(), 2);
    assert!(deserialized.primals.contains_key("songbird"));
    assert!(deserialized.primals.contains_key("nestgate"));

    let deserialized_nestgate = deserialized.primals.get("nestgate").unwrap();
    assert!(deserialized_nestgate.depends_on.contains(&"songbird".to_string()));
}
