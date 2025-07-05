//! Gaming Tournament Integration Tests
//! 
//! This test suite validates the complete gaming tournament recursive architecture
//! with ring of Songbirds and physics Toadstool working together.

use biomeos_manifest::*;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_gaming_tournament_manifest_parsing() {
    let yaml_content = r#"
apiVersion: v1
kind: Biome
metadata:
  name: "gaming-tournament-test"
  version: "1.0.0"
  description: "Gaming tournament test manifest"
  specialization: "gaming-server"

topology:
  topology_type: "recursive"
  orchestration_ring:
    topology: "ring"
    instances: 3
    regions: ["us-east", "eu-west", "ap-southeast"]
    template: "songbird-orchestrator"
    placement_strategy: "region_distributed"
  physics_layer:
    topology: "singleton"
    instances: 1
    template: "physics-toadstool"
    placement_strategy: "central"
    depends_on: ["orchestration_ring"]

nested_biomes:
  songbird-orchestrator:
    apiVersion: v1
    kind: Biome
    metadata:
      name: "songbird-orchestrator"
      specialization: "networking-lab"
    primals:
      toadstool:
        primal_type: "toadstool"
        priority: 1
        config:
          mode: "host"
          resources:
            cpu: "4"
            memory: "16GB"
      songbird:
        primal_type: "songbird"
        priority: 2
        depends_on: ["toadstool"]
        config:
          mode: "orchestrator"
          topology: "ring"
  
  physics-toadstool:
    apiVersion: v1
    kind: Biome
    metadata:
      name: "physics-toadstool"
      specialization: "gaming-development"
    primals:
      toadstool:
        primal_type: "toadstool"
        priority: 1
        config:
          mode: "compute"
          specialization: "physics"
          resources:
            cpu: "32"
            memory: "128GB"
            gpu: "rtx-4090"
"#;

    let result: Result<BiomeManifest, _> = serde_yaml::from_str(yaml_content);
    assert!(result.is_ok());
    
    let manifest = result.unwrap();
    assert_eq!(manifest.metadata.name, "gaming-tournament-test");
    assert_eq!(manifest.metadata.specialization, Some(BiomeSpecialization::GamingServer));
    
    // Validate topology
    let topology = manifest.topology.as_ref().unwrap();
    assert_eq!(topology.topology_type, "recursive");
    assert!(topology.orchestration_ring.is_some());
    assert!(topology.physics_layer.is_some());
    
    let orchestration_ring = topology.orchestration_ring.as_ref().unwrap();
    assert_eq!(orchestration_ring.topology, TopologyPattern::Ring);
    assert_eq!(orchestration_ring.instances, 3);
    assert_eq!(orchestration_ring.regions.as_ref().unwrap().len(), 3);
    
    // Validate nested biomes
    let nested_biomes = manifest.nested_biomes.as_ref().unwrap();
    assert!(nested_biomes.contains_key("songbird-orchestrator"));
    assert!(nested_biomes.contains_key("physics-toadstool"));
    
    let songbird_biome = nested_biomes.get("songbird-orchestrator").unwrap();
    assert!(songbird_biome.primals.contains_key("songbird"));
    assert!(songbird_biome.primals.contains_key("toadstool"));
    
    let physics_biome = nested_biomes.get("physics-toadstool").unwrap();
    assert!(physics_biome.primals.contains_key("toadstool"));
    
    let physics_toadstool = physics_biome.primals.get("toadstool").unwrap();
    let config = physics_toadstool.config.as_ref().unwrap();
    assert_eq!(config["specialization"], "physics");
}

#[test]
fn test_gaming_tournament_recursive_validation() {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: "tournament-validation-test".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test recursive validation".to_string()),
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

    // Setup gaming tournament topology
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
            hosts: Some(vec![BiomeReference {
                topology: TopologyPattern::Cluster,
                instances: 4,
                regions: None,
                template: "game-server-cluster".to_string(),
                depends_on: None,
                placement_strategy: Some("local_region".to_string()),
                hosts: None,
            }]),
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

    // Setup recursive monitoring
    manifest.monitoring = Some(RecursiveMonitoring {
        recursive: true,
        aggregation: "hierarchical".to_string(),
        metrics: vec![
            LayerMetrics {
                layer: "orchestration_ring".to_string(),
                collect: vec!["latency".to_string(), "throughput".to_string(), "ring_integrity".to_string()],
                thresholds: Some({
                    let mut thresholds = HashMap::new();
                    thresholds.insert("latency".to_string(), "< 50ms".to_string());
                    thresholds.insert("throughput".to_string(), "> 1000 rps".to_string());
                    thresholds
                }),
            },
            LayerMetrics {
                layer: "physics_layer".to_string(),
                collect: vec!["cpu_usage".to_string(), "physics_fps".to_string(), "sync_latency".to_string()],
                thresholds: Some({
                    let mut thresholds = HashMap::new();
                    thresholds.insert("cpu_usage".to_string(), "< 80%".to_string());
                    thresholds.insert("physics_fps".to_string(), "> 120".to_string());
                    thresholds
                }),
            },
        ],
    });

    // Setup recursive scaling
    manifest.scaling = Some(RecursiveScaling {
        triggers: vec![
            ScalingTrigger {
                metric: "player_count".to_string(),
                threshold: "> 800".to_string(),
                action: ScalingAction {
                    scale_up: Some(ScalingTarget {
                        component: "game-server-cluster".to_string(),
                        instances: Some("+25%".to_string()),
                        resources: None,
                    }),
                    scale_down: None,
                },
            },
            ScalingTrigger {
                metric: "physics_load".to_string(),
                threshold: "> 85%".to_string(),
                action: ScalingAction {
                    scale_up: Some(ScalingTarget {
                        component: "physics-toadstool".to_string(),
                        instances: None,
                        resources: Some({
                            let mut resources = HashMap::new();
                            resources.insert("cpu".to_string(), "+50%".to_string());
                            resources.insert("memory".to_string(), "+50%".to_string());
                            resources
                        }),
                    }),
                    scale_down: None,
                },
            },
        ],
        constraints: Some(ScalingConstraints {
            max_instances: Some(50),
            min_instances: Some(3),
            max_resources: Some({
                let mut resources = HashMap::new();
                resources.insert("cpu".to_string(), "128".to_string());
                resources.insert("memory".to_string(), "512GB".to_string());
                resources
            }),
            cooldown_period: Some("5m".to_string()),
        }),
    });

    // Validate the complete tournament setup
    let topology = manifest.topology.as_ref().unwrap();
    assert_eq!(topology.topology_type, "recursive");
    
    let orchestration_ring = topology.orchestration_ring.as_ref().unwrap();
    assert_eq!(orchestration_ring.topology, TopologyPattern::Ring);
    assert_eq!(orchestration_ring.instances, 3);
    assert!(orchestration_ring.hosts.is_some());
    
    let hosts = orchestration_ring.hosts.as_ref().unwrap();
    assert_eq!(hosts.len(), 1);
    assert_eq!(hosts[0].topology, TopologyPattern::Cluster);
    assert_eq!(hosts[0].instances, 4);
    
    let physics_layer = topology.physics_layer.as_ref().unwrap();
    assert_eq!(physics_layer.topology, TopologyPattern::Singleton);
    assert_eq!(physics_layer.instances, 1);
    assert!(physics_layer.depends_on.is_some());
    
    // Validate monitoring configuration
    let monitoring = manifest.monitoring.as_ref().unwrap();
    assert!(monitoring.recursive);
    assert_eq!(monitoring.aggregation, "hierarchical");
    assert_eq!(monitoring.metrics.len(), 2);
    
    // Validate scaling configuration
    let scaling = manifest.scaling.as_ref().unwrap();
    assert_eq!(scaling.triggers.len(), 2);
    assert!(scaling.constraints.is_some());
    
    let constraints = scaling.constraints.as_ref().unwrap();
    assert_eq!(constraints.max_instances, Some(50));
    assert_eq!(constraints.min_instances, Some(3));
}

#[test]
fn test_iterative_ring_formation() {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: "ring-formation-test".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test iterative ring formation".to_string()),
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

    // Setup iterative ring formation
    let mut iterative = HashMap::new();
    iterative.insert("songbird-ring".to_string(), IterativeDeployment {
        pattern: TopologyPattern::Ring,
        instances: 3,
        configuration: {
            let mut config = HashMap::new();
            config.insert("ring_size".to_string(), json!(3));
            config.insert("redundancy".to_string(), json!(2));
            config.insert("leader_election".to_string(), json!(true));
            config.insert("consensus".to_string(), json!("raft"));
            config
        },
        iteration: IterationConfig {
            variables: {
                let mut vars = HashMap::new();
                vars.insert("ring_position".to_string(), "{{ index }}".to_string());
                vars.insert("next_peer".to_string(), "{{ (index + 1) % ring_size }}".to_string());
                vars.insert("prev_peer".to_string(), "{{ (index - 1 + ring_size) % ring_size }}".to_string());
                vars.insert("region".to_string(), "{{ regions[index] }}".to_string());
                vars.insert("datacenter".to_string(), "tournament-{{ region }}".to_string());
                vars
            },
            dependencies: Some(vec!["beardog".to_string()]), // BearDog must start first
            constraints: Some(vec![
                "no_single_point_of_failure".to_string(),
                "cross_region_redundancy".to_string(),
            ]),
        },
    });

    manifest.iterative = Some(iterative);

    // Validate iterative configuration
    let iterative_deployments = manifest.iterative.as_ref().unwrap();
    assert!(iterative_deployments.contains_key("songbird-ring"));
    
    let ring_deployment = iterative_deployments.get("songbird-ring").unwrap();
    assert_eq!(ring_deployment.pattern, TopologyPattern::Ring);
    assert_eq!(ring_deployment.instances, 3);
    
    // Validate ring configuration
    assert_eq!(ring_deployment.configuration.get("ring_size").unwrap(), &json!(3));
    assert_eq!(ring_deployment.configuration.get("redundancy").unwrap(), &json!(2));
    assert_eq!(ring_deployment.configuration.get("leader_election").unwrap(), &json!(true));
    
    // Validate iteration variables
    let iteration = &ring_deployment.iteration;
    assert!(iteration.variables.contains_key("ring_position"));
    assert!(iteration.variables.contains_key("next_peer"));
    assert!(iteration.variables.contains_key("prev_peer"));
    assert!(iteration.variables.contains_key("region"));
    
    // Validate dependencies and constraints
    assert!(iteration.dependencies.is_some());
    let deps = iteration.dependencies.as_ref().unwrap();
    assert!(deps.contains(&"beardog".to_string()));
    
    assert!(iteration.constraints.is_some());
    let constraints = iteration.constraints.as_ref().unwrap();
    assert!(constraints.contains(&"no_single_point_of_failure".to_string()));
    assert!(constraints.contains(&"cross_region_redundancy".to_string()));
}

#[test]
fn test_tournament_scheduling() {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: "tournament-scheduling-test".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test tournament scheduling".to_string()),
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

    // Setup tournament schedules
    manifest.schedules.insert("tournament-active".to_string(), ScheduleConfig {
        active: "18:00-23:00".to_string(),
        timezone: Some("UTC".to_string()),
        config_overrides: Some(json!({
            "max_players": 2000,
            "match_duration": "20m",
            "server_allocation": "aggressive"
        })),
        primal_overrides: {
            let mut overrides = HashMap::new();
            overrides.insert("physics-toadstool".to_string(), json!({
                "resources": {
                    "cpu": "64",
                    "memory": "256GB",
                    "gpu": "dual-rtx-4090"
                },
                "physics_quality": "ultra",
                "tick_rate": 144
            }));
            overrides.insert("songbird".to_string(), json!({
                "load_balancing": {
                    "algorithm": "least_latency",
                    "max_connections_per_server": 128
                }
            }));
            overrides
        },
        service_overrides: {
            let mut overrides = HashMap::new();
            overrides.insert("match-maker".to_string(), json!({
                "algorithm": "elo_strict",
                "max_wait_time": "15s",
                "skill_variance": 0.05
            }));
            overrides
        },
        enabled: true,
    });

    manifest.schedules.insert("practice-mode".to_string(), ScheduleConfig {
        active: "09:00-17:00".to_string(),
        timezone: Some("UTC".to_string()),
        config_overrides: Some(json!({
            "max_players": 500,
            "match_duration": "10m",
            "server_allocation": "conservative"
        })),
        primal_overrides: {
            let mut overrides = HashMap::new();
            overrides.insert("physics-toadstool".to_string(), json!({
                "resources": {
                    "cpu": "16",
                    "memory": "64GB"
                },
                "physics_quality": "medium",
                "tick_rate": 60
            }));
            overrides
        },
        service_overrides: {
            let mut overrides = HashMap::new();
            overrides.insert("match-maker".to_string(), json!({
                "algorithm": "balanced",
                "max_wait_time": "60s",
                "skill_variance": 0.2
            }));
            overrides
        },
        enabled: true,
    });

    manifest.schedules.insert("maintenance-window".to_string(), ScheduleConfig {
        active: "03:00-05:00".to_string(),
        timezone: Some("UTC".to_string()),
        config_overrides: Some(json!({
            "maintenance_mode": true,
            "accept_new_players": false
        })),
        primal_overrides: {
            let mut overrides = HashMap::new();
            overrides.insert("nestgate".to_string(), json!({
                "backup_mode": true,
                "compression": "max",
                "deduplication": true
            }));
            overrides
        },
        service_overrides: HashMap::new(),
        enabled: true,
    });

    // Validate tournament schedules
    assert_eq!(manifest.schedules.len(), 3);
    
    let tournament_active = manifest.schedules.get("tournament-active").unwrap();
    assert_eq!(tournament_active.active, "18:00-23:00");
    assert!(tournament_active.enabled);
    assert!(tournament_active.config_overrides.is_some());
    assert_eq!(tournament_active.primal_overrides.len(), 2);
    assert_eq!(tournament_active.service_overrides.len(), 1);
    
    let practice_mode = manifest.schedules.get("practice-mode").unwrap();
    assert_eq!(practice_mode.active, "09:00-17:00");
    assert!(practice_mode.enabled);
    
    let maintenance = manifest.schedules.get("maintenance-window").unwrap();
    assert_eq!(maintenance.active, "03:00-05:00");
    assert!(maintenance.enabled);
    
    // Validate specific overrides
    let tournament_config = tournament_active.config_overrides.as_ref().unwrap();
    assert_eq!(tournament_config["max_players"], 2000);
    
    let physics_overrides = &tournament_active.primal_overrides["physics-toadstool"];
    assert_eq!(physics_overrides["resources"]["cpu"], "64");
    assert_eq!(physics_overrides["physics_quality"], "ultra");
    
    let matchmaker_overrides = &tournament_active.service_overrides["match-maker"];
    assert_eq!(matchmaker_overrides["algorithm"], "elo_strict");
    assert_eq!(matchmaker_overrides["max_wait_time"], "15s");
}

#[test]
fn test_tournament_environments() {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: "tournament-environments-test".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test tournament environments".to_string()),
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

    // Setup tournament environments
    manifest.environments.insert("production".to_string(), EnvironmentConfig {
        description: Some("Production tournament environment".to_string()),
        extends: None,
        primals: {
            let mut primals = HashMap::new();
            primals.insert("beardog".to_string(), PrimalSpec {
                enabled: true,
                primal_type: PrimalType::BearDog,
                priority: 1,
                version: Some("1.0.0".to_string()),
                source: None,
                depends_on: vec![],
                startup_timeout: Some("30s".to_string()),
                config: Some(json!({
                    "security_level": "high",
                    "audit_logging": true,
                    "threat_detection": true
                })),
                networking: None,
                resources: None,
                extensions: None,
            });
            primals
        },
        services: HashMap::new(),
        environment: {
            let mut env = HashMap::new();
            env.insert("TOURNAMENT_ENV".to_string(), "production".to_string());
            env.insert("LOG_LEVEL".to_string(), "INFO".to_string());
            env.insert("METRICS_ENABLED".to_string(), "true".to_string());
            env.insert("ANTI_CHEAT_ENABLED".to_string(), "true".to_string());
            env
        },
        resources: Some(ResourceSpec {
            cpu: Some(CpuSpec {
                max_cores: Some(64.0),
                shares: Some(8192),
            }),
            memory: Some(MemorySpec {
                max_mb: Some(262144),
                swap_mb: Some(131072),
            }),
            storage: Some(StorageSpec {
                max_mb: Some(2097152),
                storage_type: Some("nvme".to_string()),
            }),
        }),
        security: Some(SecuritySpec {
            auth: None,
            tls: Some(TlsSpec {
                enabled: true,
                certificates: Some(json!({
                    "cert_path": "/etc/certs/tournament.crt",
                    "key_path": "/etc/certs/tournament.key"
                })),
            }),
        }),
        mycorrhiza: Some(MycorrhizaConfig {
            energy_flow: EnergyFlowState::PrivateOpen,
            personal_ai: PersonalAiConfig {
                enabled: false,
                local_models: vec![],
                api_keys: vec![],
            },
            trusted_externals: TrustedExternalsConfig {
                enabled: true,
                grants: vec!["tournament-api".to_string(), "anti-cheat-service".to_string()],
            },
            commercial_access: CommercialAccessConfig {
                enabled: true,
                licensed_providers: vec!["tournament-hosting-llc".to_string()],
            },
            enforcement: EnforcementConfig {
                deep_packet_inspection: true,
                api_signature_detection: true,
                behavioral_analysis: true,
                threat_response: ThreatResponse::BlockAndPreserve,
            },
        }),
    });

    manifest.environments.insert("staging".to_string(), EnvironmentConfig {
        description: Some("Staging environment for testing".to_string()),
        extends: Some("production".to_string()),
        primals: HashMap::new(),
        services: HashMap::new(),
        environment: {
            let mut env = HashMap::new();
            env.insert("TOURNAMENT_ENV".to_string(), "staging".to_string());
            env.insert("LOG_LEVEL".to_string(), "DEBUG".to_string());
            env.insert("ANTI_CHEAT_ENABLED".to_string(), "false".to_string());
            env
        },
        resources: Some(ResourceSpec {
            cpu: Some(CpuSpec {
                max_cores: Some(16.0),
                shares: Some(2048),
            }),
            memory: Some(MemorySpec {
                max_mb: Some(65536),
                swap_mb: Some(32768),
            }),
            storage: Some(StorageSpec {
                max_mb: Some(524288),
                storage_type: Some("ssd".to_string()),
            }),
        }),
        security: None,
        mycorrhiza: None,
    });

    manifest.environments.insert("development".to_string(), EnvironmentConfig {
        description: Some("Development environment".to_string()),
        extends: None,
        primals: {
            let mut primals = HashMap::new();
            primals.insert("toadstool".to_string(), PrimalSpec {
                enabled: true,
                primal_type: PrimalType::Toadstool,
                priority: 1,
                version: Some("dev".to_string()),
                source: None,
                depends_on: vec![],
                startup_timeout: Some("60s".to_string()),
                config: Some(json!({
                    "mode": "development",
                    "hot_reload": true,
                    "debug_mode": true
                })),
                networking: None,
                resources: None,
                extensions: None,
            });
            primals
        },
        services: HashMap::new(),
        environment: {
            let mut env = HashMap::new();
            env.insert("TOURNAMENT_ENV".to_string(), "development".to_string());
            env.insert("LOG_LEVEL".to_string(), "TRACE".to_string());
            env.insert("HOT_RELOAD".to_string(), "true".to_string());
            env.insert("ANTI_CHEAT_ENABLED".to_string(), "false".to_string());
            env
        },
        resources: Some(ResourceSpec {
            cpu: Some(CpuSpec {
                max_cores: Some(8.0),
                shares: Some(1024),
            }),
            memory: Some(MemorySpec {
                max_mb: Some(16384),
                swap_mb: Some(8192),
            }),
            storage: Some(StorageSpec {
                max_mb: Some(102400),
                storage_type: Some("ssd".to_string()),
            }),
        }),
        security: None,
        mycorrhiza: None,
    });

    // Validate environments
    assert_eq!(manifest.environments.len(), 3);
    
    let production = manifest.environments.get("production").unwrap();
    assert!(production.extends.is_none());
    assert_eq!(production.primals.len(), 1);
    assert!(production.primals.contains_key("beardog"));
    assert!(production.resources.is_some());
    assert!(production.security.is_some());
    assert!(production.mycorrhiza.is_some());
    
    let staging = manifest.environments.get("staging").unwrap();
    assert_eq!(staging.extends, Some("production".to_string()));
    assert_eq!(staging.primals.len(), 0); // Inherits from production
    assert!(staging.resources.is_some());
    
    let development = manifest.environments.get("development").unwrap();
    assert!(development.extends.is_none());
    assert_eq!(development.primals.len(), 1);
    assert!(development.primals.contains_key("toadstool"));
    
    // Validate environment variables
    assert_eq!(production.environment.get("TOURNAMENT_ENV"), Some(&"production".to_string()));
    assert_eq!(staging.environment.get("TOURNAMENT_ENV"), Some(&"staging".to_string()));
    assert_eq!(development.environment.get("TOURNAMENT_ENV"), Some(&"development".to_string()));
    
    // Validate security configurations
    let prod_security = production.security.as_ref().unwrap();
    assert!(prod_security.tls.is_some());
    let tls = prod_security.tls.as_ref().unwrap();
    assert!(tls.enabled);
    
    // Validate MYCORRHIZA configurations
    let prod_mycorrhiza = production.mycorrhiza.as_ref().unwrap();
    assert_eq!(prod_mycorrhiza.energy_flow, EnergyFlowState::PrivateOpen);
    assert!(prod_mycorrhiza.trusted_externals.enabled);
    assert_eq!(prod_mycorrhiza.enforcement.threat_response, ThreatResponse::BlockAndPreserve);
}

#[test]
fn test_complete_tournament_serialization() {
    // Test that a complete gaming tournament manifest can be serialized and deserialized
    let yaml_content = std::fs::read_to_string("templates/gaming-tournament-recursive.biome.yaml")
        .expect("Failed to read gaming tournament template");
    
    let result: Result<BiomeManifest, _> = serde_yaml::from_str(&yaml_content);
    assert!(result.is_ok(), "Failed to parse gaming tournament template: {:?}", result.err());
    
    let manifest = result.unwrap();
    
    // Validate core structure
    assert_eq!(manifest.metadata.name, "gaming-tournament-recursive");
    assert_eq!(manifest.metadata.specialization, Some(BiomeSpecialization::GamingServer));
    
    // Validate topology
    assert!(manifest.topology.is_some());
    let topology = manifest.topology.as_ref().unwrap();
    assert_eq!(topology.topology_type, "recursive");
    
    // Validate nested biomes
    assert!(manifest.nested_biomes.is_some());
    let nested_biomes = manifest.nested_biomes.as_ref().unwrap();
    assert!(nested_biomes.contains_key("songbird-orchestrator"));
    assert!(nested_biomes.contains_key("physics-toadstool"));
    
    // Validate iterative patterns
    assert!(manifest.iterative.is_some());
    let iterative = manifest.iterative.as_ref().unwrap();
    assert!(iterative.contains_key("ring-formation"));
    
    // Validate monitoring
    assert!(manifest.monitoring.is_some());
    let monitoring = manifest.monitoring.as_ref().unwrap();
    assert!(monitoring.recursive);
    assert_eq!(monitoring.aggregation, "hierarchical");
    
    // Validate scaling
    assert!(manifest.scaling.is_some());
    let scaling = manifest.scaling.as_ref().unwrap();
    assert!(!scaling.triggers.is_empty());
    
    // Test round-trip serialization
    let serialized = serde_yaml::to_string(&manifest).expect("Failed to serialize manifest");
    let deserialized: BiomeManifest = serde_yaml::from_str(&serialized)
        .expect("Failed to deserialize manifest");
    
    assert_eq!(manifest.metadata.name, deserialized.metadata.name);
    assert_eq!(manifest.metadata.specialization, deserialized.metadata.specialization);
}
