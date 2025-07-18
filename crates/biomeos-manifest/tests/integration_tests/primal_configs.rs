//! Configurable primal configurations for integration tests
//!
//! This module provides pre-configured primal specifications that can be easily
//! customized for different test scenarios, removing hardcoded dependencies.

use biomeos_manifest::*;
use serde_json::{json, Value};
use crate::integration_tests::common::*;

/// Configuration presets for common primal types
#[derive(Debug, Clone)]
pub enum PrimalPreset {
    /// Service mesh orchestrator configuration
    ServiceMeshOrchestrator {
        topology: String,
        features: Vec<String>,
        discovery_method: String,
        datacenter: String,
    },
    /// Distributed storage configuration
    DistributedStorage {
        storage_type: String,
        replication_factor: u32,
        consistency: String,
        features: Vec<String>,
    },
    /// Gaming tournament configuration
    GamingTournament {
        topology: String,
        match_making: bool,
        player_balancing: bool,
        server_allocation: bool,
    },
    /// Real-time communication configuration
    RealTimeCommunication {
        protocol: String,
        encryption: bool,
        compression: bool,
        low_latency: bool,
    },
    /// Analytics and monitoring configuration
    AnalyticsMonitoring {
        metrics_collection: bool,
        log_aggregation: bool,
        alerting: bool,
        dashboard: bool,
    },
}

impl PrimalPreset {
    /// Convert preset to JSON configuration
    pub fn to_config(&self) -> Value {
        match self {
            PrimalPreset::ServiceMeshOrchestrator { topology, features, discovery_method, datacenter } => {
                json!({
                    "mode": "orchestrator",
                    "topology": topology,
                    "features": features,
                    "discovery": {
                        "method": discovery_method,
                        "refresh_interval": "30s"
                    },
                    "load_balancing": {
                        "algorithm": "round_robin",
                        "health_checks": true
                    },
                    "datacenter": datacenter
                })
            },
            PrimalPreset::DistributedStorage { storage_type, replication_factor, consistency, features } => {
                json!({
                    "mode": "distributed",
                    "storage_type": storage_type,
                    "replication_factor": replication_factor,
                    "consistency": consistency,
                    "features": features,
                    "encryption": {
                        "algorithm": "aes-256-gcm",
                        "key_rotation": true
                    },
                    "compression": {
                        "algorithm": "lz4",
                        "level": "fast"
                    }
                })
            },
            PrimalPreset::GamingTournament { topology, match_making, player_balancing, server_allocation } => {
                json!({
                    "mode": "orchestrator",
                    "topology": topology,
                    "features": [
                        "service_discovery",
                        "load_balancing",
                        "health_monitoring",
                        "cross_region_routing",
                        "tournament_coordination"
                    ],
                    "tournament": {
                        "match_making": match_making,
                        "player_balancing": player_balancing,
                        "server_allocation": server_allocation
                    },
                    "ring_config": {
                        "size": 3,
                        "redundancy": 2,
                        "consensus": "raft"
                    }
                })
            },
            PrimalPreset::RealTimeCommunication { protocol, encryption, compression, low_latency } => {
                json!({
                    "mode": "communication",
                    "protocol": protocol,
                    "encryption": encryption,
                    "compression": compression,
                    "low_latency": low_latency,
                    "features": [
                        "peer_to_peer",
                        "group_messaging",
                        "presence_detection"
                    ]
                })
            },
            PrimalPreset::AnalyticsMonitoring { metrics_collection, log_aggregation, alerting, dashboard } => {
                json!({
                    "mode": "monitoring",
                    "features": {
                        "metrics_collection": metrics_collection,
                        "log_aggregation": log_aggregation,
                        "alerting": alerting,
                        "dashboard": dashboard
                    },
                    "retention": {
                        "metrics": "30d",
                        "logs": "7d",
                        "alerts": "90d"
                    }
                })
            }
        }
    }
}

/// Resource presets for different deployment scenarios
#[derive(Debug, Clone)]
pub enum ResourcePreset {
    /// Lightweight deployment
    Lightweight,
    /// Standard deployment
    Standard,
    /// High-performance deployment
    HighPerformance,
    /// Gaming-optimized deployment
    GamingOptimized,
    /// Storage-optimized deployment
    StorageOptimized,
}

impl ResourcePreset {
    /// Convert preset to PrimalResources
    pub fn to_resources(&self) -> PrimalResources {
        match self {
            ResourcePreset::Lightweight => ResourcesBuilder::new()
                .with_cpu(1.0, 512)
                .with_memory(2048, 1024)
                .with_storage(5120, "ssd")
                .build(),
            ResourcePreset::Standard => ResourcesBuilder::new()
                .with_cpu(2.0, 1024)
                .with_memory(4096, 2048)
                .with_storage(10240, "ssd")
                .build(),
            ResourcePreset::HighPerformance => ResourcesBuilder::new()
                .with_cpu(4.0, 2048)
                .with_memory(8192, 4096)
                .with_storage(20480, "nvme")
                .build(),
            ResourcePreset::GamingOptimized => ResourcesBuilder::new()
                .with_cpu(6.0, 3072)
                .with_memory(12288, 6144)
                .with_storage(20480, "nvme")
                .build(),
            ResourcePreset::StorageOptimized => ResourcesBuilder::new()
                .with_cpu(4.0, 2048)
                .with_memory(8192, 4096)
                .with_storage(1048576, "ssd")
                .build(),
        }
    }
}

/// Networking presets for different scenarios
#[derive(Debug, Clone)]
pub enum NetworkingPreset {
    /// Basic networking with standard ports
    Basic { ports: Vec<u16> },
    /// Service mesh networking
    ServiceMesh { ports: Vec<u16>, discovery_method: String },
    /// Gaming networking with optimized ports
    Gaming { ports: Vec<u16>, low_latency: bool },
    /// Storage networking with multiple interfaces
    Storage { ports: Vec<u16>, service_name: String },
}

impl NetworkingPreset {
    /// Convert preset to PrimalNetworking
    pub fn to_networking(&self) -> PrimalNetworking {
        match self {
            NetworkingPreset::Basic { ports } => {
                NetworkingBuilder::new()
                    .with_ports(ports.clone())
                    .with_host("0.0.0.0")
                    .build()
            },
            NetworkingPreset::ServiceMesh { ports, discovery_method } => {
                NetworkingBuilder::new()
                    .with_ports(ports.clone())
                    .with_host("0.0.0.0")
                    .with_discovery(discovery_method, json!({
                        "datacenter": "dc1",
                        "encrypt": true
                    }))
                    .build()
            },
            NetworkingPreset::Gaming { ports, low_latency } => {
                NetworkingBuilder::new()
                    .with_ports(ports.clone())
                    .with_host("0.0.0.0")
                    .with_discovery("consul", json!({
                        "datacenter": "tournament",
                        "encrypt": true,
                        "cross_region": true,
                        "low_latency": low_latency
                    }))
                    .build()
            },
            NetworkingPreset::Storage { ports, service_name } => {
                NetworkingBuilder::new()
                    .with_ports(ports.clone())
                    .with_host("0.0.0.0")
                    .with_discovery("service_mesh", json!({
                        "service_name": service_name,
                        "tags": ["storage", "distributed", "object"]
                    }))
                    .build()
            },
        }
    }
}

/// Factory for creating common primal configurations
pub struct PrimalConfigFactory;

impl PrimalConfigFactory {
    /// Create a generic orchestrator primal
    pub fn create_orchestrator(
        name: &str,
        primal_type: PrimalType,
        priority: u32,
        preset: PrimalPreset,
        resource_preset: ResourcePreset,
        networking_preset: NetworkingPreset,
    ) -> PrimalBuilder {
        PrimalBuilder::new(name, primal_type)
            .with_priority_and_deps(priority, vec![])
            .with_version_and_timeout("1.0.0", "30s")
            .with_config(preset.to_config())
            .with_networking(networking_preset.to_networking())
            .with_resources(resource_preset.to_resources())
    }

    /// Create a generic storage primal
    pub fn create_storage(
        name: &str,
        primal_type: PrimalType,
        priority: u32,
        depends_on: Vec<String>,
        preset: PrimalPreset,
        resource_preset: ResourcePreset,
        networking_preset: NetworkingPreset,
    ) -> PrimalBuilder {
        PrimalBuilder::new(name, primal_type)
            .with_priority_and_deps(priority, depends_on)
            .with_version_and_timeout("1.0.0", "60s")
            .with_config(preset.to_config())
            .with_networking(networking_preset.to_networking())
            .with_resources(resource_preset.to_resources())
    }

    /// Create a generic gaming primal
    pub fn create_gaming(
        name: &str,
        primal_type: PrimalType,
        priority: u32,
        preset: PrimalPreset,
        resource_preset: ResourcePreset,
        networking_preset: NetworkingPreset,
    ) -> PrimalBuilder {
        PrimalBuilder::new(name, primal_type)
            .with_priority_and_deps(priority, vec![])
            .with_version_and_timeout("1.0.0", "45s")
            .with_config(preset.to_config())
            .with_networking(networking_preset.to_networking())
            .with_resources(resource_preset.to_resources())
    }

    /// Create a generic communication primal
    pub fn create_communication(
        name: &str,
        primal_type: PrimalType,
        priority: u32,
        depends_on: Vec<String>,
        preset: PrimalPreset,
        resource_preset: ResourcePreset,
        networking_preset: NetworkingPreset,
    ) -> PrimalBuilder {
        PrimalBuilder::new(name, primal_type)
            .with_priority_and_deps(priority, depends_on)
            .with_version_and_timeout("1.0.0", "30s")
            .with_config(preset.to_config())
            .with_networking(networking_preset.to_networking())
            .with_resources(resource_preset.to_resources())
    }

    /// Create a generic monitoring primal
    pub fn create_monitoring(
        name: &str,
        primal_type: PrimalType,
        priority: u32,
        depends_on: Vec<String>,
        preset: PrimalPreset,
        resource_preset: ResourcePreset,
        networking_preset: NetworkingPreset,
    ) -> PrimalBuilder {
        PrimalBuilder::new(name, primal_type)
            .with_priority_and_deps(priority, depends_on)
            .with_version_and_timeout("1.0.0", "30s")
            .with_config(preset.to_config())
            .with_networking(networking_preset.to_networking())
            .with_resources(resource_preset.to_resources())
    }
}

/// Dependency presets for common scenarios
pub struct DependencyPresets;

impl DependencyPresets {
    /// Create basic integration dependencies
    pub fn basic_integration(primal_a: &str, primal_b: &str) -> DependencyBuilder {
        DependencyBuilder::new()
            .require(
                &format!("{}-runtime", primal_a),
                ">=1.0.0",
                &format!("Required for {} functionality", primal_a)
            )
            .require(
                &format!("{}-runtime", primal_b),
                ">=1.0.0",
                &format!("Required for {} functionality", primal_b)
            )
            .suggest(
                "monitoring-stack",
                ">=2.0.0",
                "Enhanced monitoring for eco-primals"
            )
            .conflict(
                "legacy-orchestrator",
                "*",
                "Conflicts with modern orchestration"
            )
    }

    /// Create gaming-specific dependencies
    pub fn gaming_integration(orchestrator: &str, storage: &str) -> DependencyBuilder {
        DependencyBuilder::new()
            .require(
                &format!("{}-runtime", orchestrator),
                ">=1.0.0",
                &format!("Required for {} orchestration", orchestrator)
            )
            .require(
                &format!("{}-runtime", storage),
                ">=1.0.0",
                &format!("Required for {} storage", storage)
            )
            .require(
                "gaming-engine",
                ">=3.0.0",
                "Required for gaming tournament support"
            )
            .suggest(
                "performance-monitoring",
                ">=1.5.0",
                "Enhanced performance monitoring for gaming"
            )
            .feature(
                "cross-region-replication",
                "Enable cross-region replication for storage",
                vec![format!("{}-runtime", storage)],
                vec!["replication-service".to_string()],
                Some(json!({
                    "replication_factor": 2,
                    "encryption": true
                })),
                false
            )
            .feature(
                "advanced-load-balancing",
                "Enable advanced load balancing algorithms",
                vec![format!("{}-runtime", orchestrator)],
                vec!["load-balancer".to_string()],
                Some(json!({
                    "algorithms": ["weighted_round_robin", "least_connections", "ip_hash"]
                })),
                true
            )
    }

    /// Create data center dependencies
    pub fn data_center_integration(orchestrator: &str, storage: &str) -> DependencyBuilder {
        DependencyBuilder::new()
            .require(
                &format!("{}-runtime", orchestrator),
                ">=1.0.0",
                &format!("Required for {} service mesh", orchestrator)
            )
            .require(
                &format!("{}-runtime", storage),
                ">=1.0.0",
                &format!("Required for {} distributed storage", storage)
            )
            .suggest(
                "monitoring-stack",
                ">=2.0.0",
                "Enhanced monitoring for eco-primals"
            )
            .conflict(
                "legacy-orchestrator",
                "*",
                "Conflicts with modern orchestration"
            )
            .feature(
                "encryption-at-rest",
                "Enable encryption for stored data",
                vec![format!("{}-runtime", storage)],
                vec!["encryption-service".to_string()],
                Some(json!({
                    "algorithm": "aes-256-gcm",
                    "key_rotation": true
                })),
                true
            )
            .feature(
                "service-mesh-security",
                "Enable service mesh security features",
                vec![format!("{}-runtime", orchestrator)],
                vec!["security-service".to_string()],
                Some(json!({
                    "tls": true,
                    "mutual_tls": true,
                    "certificate_rotation": true
                })),
                true
            )
    }
}

/// Test scenario configurations
pub struct TestScenarios;

impl TestScenarios {
    /// Create a basic two-primal integration scenario
    pub fn two_primal_integration(
        orchestrator_name: &str,
        orchestrator_type: PrimalType,
        storage_name: &str,
        storage_type: PrimalType,
        specialization: BiomeSpecialization,
    ) -> TestManifestBuilder {
        let orchestrator = PrimalConfigFactory::create_orchestrator(
            orchestrator_name,
            orchestrator_type,
            1,
            PrimalPreset::ServiceMeshOrchestrator {
                topology: "mesh".to_string(),
                features: vec![
                    "service_discovery".to_string(),
                    "load_balancing".to_string(),
                    "health_monitoring".to_string(),
                    "traffic_routing".to_string(),
                ],
                discovery_method: "consul".to_string(),
                datacenter: "dc1".to_string(),
            },
            ResourcePreset::Standard,
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
                    "encryption".to_string(),
                    "compression".to_string(),
                    "deduplication".to_string(),
                    "versioning".to_string(),
                ],
            },
            ResourcePreset::StorageOptimized,
            NetworkingPreset::Storage {
                ports: vec![9000, 9001, 9002],
                service_name: format!("{}-storage", storage_name),
            },
        );

        TestManifestBuilder::new(&format!("{}-{}-integration", orchestrator_name, storage_name))
            .with_metadata(
                "1.0.0",
                &format!("Basic {}+{} integration test", orchestrator_name, storage_name),
                specialization
            )
            .with_tags(vec!["integration".to_string(), "eco-primals".to_string()])
            .with_attribution("biomeOS Test Suite", "MIT")
            .with_primal(orchestrator)
            .with_primal(storage)
            .with_dependencies(
                DependencyPresets::basic_integration(orchestrator_name, storage_name).build()
            )
    }

    /// Create a gaming tournament scenario
    pub fn gaming_tournament(
        orchestrator_name: &str,
        orchestrator_type: PrimalType,
        storage_name: &str,
        storage_type: PrimalType,
    ) -> TestManifestBuilder {
        let orchestrator = PrimalConfigFactory::create_gaming(
            orchestrator_name,
            orchestrator_type,
            1,
            PrimalPreset::GamingTournament {
                topology: "ring".to_string(),
                match_making: true,
                player_balancing: true,
                server_allocation: true,
            },
            ResourcePreset::GamingOptimized,
            NetworkingPreset::Gaming {
                ports: vec![8500, 8501, 8502, 8503],
                low_latency: true,
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
                consistency: "strong".to_string(),
                features: vec![
                    "encryption".to_string(),
                    "compression".to_string(),
                    "low_latency".to_string(),
                ],
            },
            ResourcePreset::HighPerformance,
            NetworkingPreset::Storage {
                ports: vec![9000, 9001, 9002],
                service_name: format!("{}-storage", storage_name),
            },
        );

        TestManifestBuilder::new("gaming-tournament-integration")
            .with_metadata(
                "1.0.0",
                &format!("{}+{} integration for gaming tournament", orchestrator_name, storage_name),
                BiomeSpecialization::GamingServer
            )
            .with_tags(vec![
                "gaming".to_string(),
                "tournament".to_string(),
                "integration".to_string(),
            ])
            .with_attribution("biomeOS Gaming Team", "MIT")
            .with_primal(orchestrator)
            .with_primal(storage)
            .with_dependencies(
                DependencyPresets::gaming_integration(orchestrator_name, storage_name).build()
            )
    }
} 