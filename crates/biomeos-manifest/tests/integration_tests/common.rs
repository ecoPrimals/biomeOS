//! Common utilities and builders for integration tests
//!
//! This module provides generic test builders and utilities that can work with
//! any primal types, eliminating hardcoded dependencies on specific primals.

use biomeos_manifest::*;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Builder for creating test BiomeManifest instances with configurable primals
#[derive(Debug, Clone)]
pub struct TestManifestBuilder {
    name: String,
    version: String,
    description: Option<String>,
    specialization: Option<BiomeSpecialization>,
    tags: Option<Vec<String>>,
    author: Option<String>,
    license: Option<String>,
    primals: HashMap<String, PrimalSpec>,
    services: HashMap<String, ServiceSpec>,
    dependencies: Option<DependencyConfig>,
}

impl TestManifestBuilder {
    /// Create a new builder with default values
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: None,
            specialization: None,
            tags: None,
            author: None,
            license: None,
            primals: HashMap::new(),
            services: HashMap::new(),
            dependencies: None,
        }
    }

    /// Set basic metadata
    pub fn with_metadata(mut self, 
        version: &str, 
        description: &str, 
        specialization: BiomeSpecialization
    ) -> Self {
        self.version = version.to_string();
        self.description = Some(description.to_string());
        self.specialization = Some(specialization);
        self
    }

    /// Set tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Set author and license
    pub fn with_attribution(mut self, author: &str, license: &str) -> Self {
        self.author = Some(author.to_string());
        self.license = Some(license.to_string());
        self
    }

    /// Add a primal with configurable settings
    pub fn with_primal(mut self, primal_builder: PrimalBuilder) -> Self {
        let primal = primal_builder.build();
        self.primals.insert(primal.name.clone(), primal.spec);
        self
    }

    /// Add dependencies
    pub fn with_dependencies(mut self, dependencies: DependencyConfig) -> Self {
        self.dependencies = Some(dependencies);
        self
    }

    /// Build the final BiomeManifest
    pub fn build(self) -> BiomeManifest {
        BiomeManifest {
            api_version: "v1".to_string(),
            kind: "Biome".to_string(),
            metadata: ManifestMetadata {
                name: self.name,
                version: self.version,
                description: self.description,
                specialization: self.specialization,
                tags: self.tags,
                author: self.author,
                created: None,
                repository: None,
                license: self.license,
                created_by: None,
                forked_from: None,
                sharing: None,
                niches: None,
                template: None,
                custom: None,
            },
            sources: SourceConfig::default(),
            primals: self.primals,
            services: self.services,
            mycorrhiza: MycorrhizaConfig::default(),
            volumes: HashMap::new(),
            networks: HashMap::new(),
            networking: None,
            security: None,
            resources: None,
            schedules: HashMap::new(),
            environments: HashMap::new(),
            dependencies: self.dependencies,
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
}

/// Builder for creating configurable primal specifications
#[derive(Debug, Clone)]
pub struct PrimalBuilder {
    name: String,
    spec: PrimalSpec,
}

impl PrimalBuilder {
    /// Create a new primal builder
    pub fn new(name: &str, primal_type: PrimalType) -> Self {
        Self {
            name: name.to_string(),
            spec: PrimalSpec {
                enabled: true,
                primal_type,
                priority: 1,
                version: Some("1.0.0".to_string()),
                source: None,
                depends_on: vec![],
                startup_timeout: Some("30s".to_string()),
                config: None,
                networking: None,
                resources: None,
                extensions: None,
            },
        }
    }

    /// Set priority and dependencies
    pub fn with_priority_and_deps(mut self, priority: u32, depends_on: Vec<String>) -> Self {
        self.spec.priority = priority;
        self.spec.depends_on = depends_on;
        self
    }

    /// Set version and timeout
    pub fn with_version_and_timeout(mut self, version: &str, timeout: &str) -> Self {
        self.spec.version = Some(version.to_string());
        self.spec.startup_timeout = Some(timeout.to_string());
        self
    }

    /// Set configuration
    pub fn with_config(mut self, config: Value) -> Self {
        self.spec.config = Some(config);
        self
    }

    /// Set networking configuration
    pub fn with_networking(mut self, networking: PrimalNetworking) -> Self {
        self.spec.networking = Some(networking);
        self
    }

    /// Set resource limits
    pub fn with_resources(mut self, resources: PrimalResources) -> Self {
        self.spec.resources = Some(resources);
        self
    }

    /// Build the primal with its name
    pub fn build(self) -> NamedPrimal {
        NamedPrimal {
            name: self.name,
            spec: self.spec,
        }
    }
}

/// Helper struct for named primal specifications
#[derive(Debug, Clone)]
pub struct NamedPrimal {
    pub name: String,
    pub spec: PrimalSpec,
}

/// Builder for creating networking configurations
#[derive(Debug, Clone)]
pub struct NetworkingBuilder {
    ports: Option<Vec<u16>>,
    host: Option<String>,
    discovery: Option<DiscoverySpec>,
}

impl NetworkingBuilder {
    /// Create a new networking builder
    pub fn new() -> Self {
        Self {
            ports: None,
            host: None,
            discovery: None,
        }
    }

    /// Set ports
    pub fn with_ports(mut self, ports: Vec<u16>) -> Self {
        self.ports = Some(ports);
        self
    }

    /// Set host
    pub fn with_host(mut self, host: &str) -> Self {
        self.host = Some(host.to_string());
        self
    }

    /// Set discovery configuration
    pub fn with_discovery(mut self, method: &str, config: Value) -> Self {
        self.discovery = Some(DiscoverySpec {
            method: method.to_string(),
            config: Some(config),
        });
        self
    }

    /// Build the networking configuration
    pub fn build(self) -> PrimalNetworking {
        PrimalNetworking {
            ports: self.ports,
            host: self.host,
            discovery: self.discovery,
        }
    }
}

/// Builder for creating resource specifications
#[derive(Debug, Clone)]
pub struct ResourcesBuilder {
    cpu: Option<CpuSpec>,
    memory: Option<MemorySpec>,
    storage: Option<StorageSpec>,
}

impl ResourcesBuilder {
    /// Create a new resources builder
    pub fn new() -> Self {
        Self {
            cpu: None,
            memory: None,
            storage: None,
        }
    }

    /// Set CPU limits
    pub fn with_cpu(mut self, max_cores: f64, shares: u64) -> Self {
        self.cpu = Some(CpuSpec {
            max_cores: Some(max_cores),
            shares: Some(shares),
        });
        self
    }

    /// Set memory limits
    pub fn with_memory(mut self, max_mb: u64, swap_mb: u64) -> Self {
        self.memory = Some(MemorySpec {
            max_mb: Some(max_mb),
            swap_mb: Some(swap_mb),
        });
        self
    }

    /// Set storage limits
    pub fn with_storage(mut self, max_mb: u64, storage_type: &str) -> Self {
        self.storage = Some(StorageSpec {
            max_mb: Some(max_mb),
            storage_type: Some(storage_type.to_string()),
        });
        self
    }

    /// Build the resources configuration
    pub fn build(self) -> PrimalResources {
        PrimalResources {
            cpu: self.cpu,
            memory: self.memory,
            storage: self.storage,
        }
    }
}

/// Builder for creating dependency configurations
#[derive(Debug, Clone)]
pub struct DependencyBuilder {
    requires: Vec<DependencySpec>,
    suggests: Vec<DependencySpec>,
    conflicts: Vec<DependencySpec>,
    features: HashMap<String, FeatureSpec>,
}

impl DependencyBuilder {
    /// Create a new dependency builder
    pub fn new() -> Self {
        Self {
            requires: Vec::new(),
            suggests: Vec::new(),
            conflicts: Vec::new(),
            features: HashMap::new(),
        }
    }

    /// Add a required dependency
    pub fn require(mut self, name: &str, version: &str, reason: &str) -> Self {
        self.requires.push(DependencySpec {
            name: name.to_string(),
            version: Some(version.to_string()),
            source: None,
            optional: false,
            reason: Some(reason.to_string()),
        });
        self
    }

    /// Add a suggested dependency
    pub fn suggest(mut self, name: &str, version: &str, reason: &str) -> Self {
        self.suggests.push(DependencySpec {
            name: name.to_string(),
            version: Some(version.to_string()),
            source: None,
            optional: true,
            reason: Some(reason.to_string()),
        });
        self
    }

    /// Add a conflicting dependency
    pub fn conflict(mut self, name: &str, version: &str, reason: &str) -> Self {
        self.conflicts.push(DependencySpec {
            name: name.to_string(),
            version: Some(version.to_string()),
            source: None,
            optional: false,
            reason: Some(reason.to_string()),
        });
        self
    }

    /// Add a feature specification
    pub fn feature(mut self, name: &str, description: &str, dependencies: Vec<String>, services: Vec<String>, config: Option<Value>, default_enabled: bool) -> Self {
        self.features.insert(name.to_string(), FeatureSpec {
            description: description.to_string(),
            dependencies,
            services,
            config,
            default_enabled,
        });
        self
    }

    /// Build the dependency configuration
    pub fn build(self) -> DependencyConfig {
        DependencyConfig {
            requires: self.requires,
            suggests: self.suggests,
            conflicts: self.conflicts,
            features: self.features,
        }
    }
}

/// Common test assertions for integration tests
pub mod assertions {
    use super::*;

    /// Assert that a primal has the expected basic configuration
    pub fn assert_primal_basic_config(primal: &PrimalSpec, expected_type: PrimalType, expected_priority: u32) {
        assert_eq!(primal.primal_type, expected_type);
        assert_eq!(primal.priority, expected_priority);
        assert!(primal.enabled);
    }

    /// Assert that a primal has the expected dependencies
    pub fn assert_primal_dependencies(primal: &PrimalSpec, expected_deps: &[String]) {
        for dep in expected_deps {
            assert!(primal.depends_on.contains(dep), "Primal missing dependency: {}", dep);
        }
    }

    /// Assert that a primal has proper networking configuration
    pub fn assert_primal_networking(primal: &PrimalSpec, expected_ports: Option<&[u16]>) {
        assert!(primal.networking.is_some(), "Primal missing networking configuration");
        
        if let Some(ports) = expected_ports {
            let networking = primal.networking.as_ref().unwrap();
            if let Some(primal_ports) = &networking.ports {
                for port in ports {
                    assert!(primal_ports.contains(port), "Primal missing port: {}", port);
                }
            }
        }
    }

    /// Assert that dependency configuration is valid
    pub fn assert_dependency_config(deps: &DependencyConfig, expected_requires: usize, expected_suggests: usize, expected_conflicts: usize) {
        assert_eq!(deps.requires.len(), expected_requires);
        assert_eq!(deps.suggests.len(), expected_suggests);
        assert_eq!(deps.conflicts.len(), expected_conflicts);
    }

    /// Assert that a feature has the expected configuration
    pub fn assert_feature_config(feature: &FeatureSpec, expected_deps: &[String], expected_services: &[String], expected_default: bool) {
        assert_eq!(feature.default_enabled, expected_default);
        
        for dep in expected_deps {
            assert!(feature.dependencies.contains(dep), "Feature missing dependency: {}", dep);
        }
        
        for service in expected_services {
            assert!(feature.services.contains(service), "Feature missing service: {}", service);
        }
    }
}

/// Test utilities for common scenarios
pub mod utils {
    use super::*;

    /// Create a basic orchestrator primal configuration
    pub fn create_orchestrator_primal(name: &str, primal_type: PrimalType, priority: u32) -> PrimalBuilder {
        PrimalBuilder::new(name, primal_type)
            .with_priority_and_deps(priority, vec![])
            .with_version_and_timeout("1.0.0", "30s")
            .with_config(json!({
                "mode": "orchestrator",
                "topology": "mesh",
                "features": [
                    "service_discovery",
                    "load_balancing",
                    "health_monitoring"
                ]
            }))
            .with_networking(
                NetworkingBuilder::new()
                    .with_ports(vec![8500, 8501, 8502])
                    .with_host("0.0.0.0")
                    .with_discovery("consul", json!({
                        "datacenter": "dc1",
                        "encrypt": true
                    }))
                    .build()
            )
            .with_resources(
                ResourcesBuilder::new()
                    .with_cpu(2.0, 1024)
                    .with_memory(4096, 2048)
                    .with_storage(10240, "ssd")
                    .build()
            )
    }

    /// Create a basic storage primal configuration
    pub fn create_storage_primal(name: &str, primal_type: PrimalType, priority: u32, depends_on: Vec<String>) -> PrimalBuilder {
        PrimalBuilder::new(name, primal_type)
            .with_priority_and_deps(priority, depends_on)
            .with_version_and_timeout("1.0.0", "60s")
            .with_config(json!({
                "mode": "distributed",
                "storage_type": "object",
                "replication_factor": 3,
                "consistency": "eventual",
                "features": [
                    "encryption",
                    "compression",
                    "deduplication"
                ]
            }))
            .with_networking(
                NetworkingBuilder::new()
                    .with_ports(vec![9000, 9001, 9002])
                    .with_host("0.0.0.0")
                    .with_discovery("service_mesh", json!({
                        "service_name": "storage-service",
                        "tags": ["storage", "distributed", "object"]
                    }))
                    .build()
            )
            .with_resources(
                ResourcesBuilder::new()
                    .with_cpu(4.0, 2048)
                    .with_memory(8192, 4096)
                    .with_storage(1048576, "ssd")
                    .build()
            )
    }

    /// Create a gaming-specific primal configuration
    pub fn create_gaming_primal(name: &str, primal_type: PrimalType, priority: u32) -> PrimalBuilder {
        PrimalBuilder::new(name, primal_type)
            .with_priority_and_deps(priority, vec![])
            .with_version_and_timeout("1.0.0", "45s")
            .with_config(json!({
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
            }))
            .with_networking(
                NetworkingBuilder::new()
                    .with_ports(vec![8500, 8501, 8502, 8503])
                    .with_host("0.0.0.0")
                    .with_discovery("consul", json!({
                        "datacenter": "tournament",
                        "encrypt": true,
                        "cross_region": true
                    }))
                    .build()
            )
            .with_resources(
                ResourcesBuilder::new()
                    .with_cpu(6.0, 3072)
                    .with_memory(12288, 6144)
                    .with_storage(20480, "nvme")
                    .build()
            )
    }
} 