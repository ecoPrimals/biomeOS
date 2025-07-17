//! # Universal Biome Example
//!
//! This example demonstrates how to use the universal biome system with
//! capability-based discovery instead of hard-coded Primal names.
//!
//! The example shows:
//! - Creating a universal biome manifest
//! - Using capability-based requirements
//! - Discovering primals by capability
//! - Bootstrapping an ecosystem

use biomeos_core::{
    AlertingSpec, AuditSpec, AuditStorage, AuthenticationSpec, AuthorizationSpec,
    AvailabilityRequirements, BackupRequirements, BackupSpec, BackupStorage, BiomeRequirements,
    CapabilityRequirement, ComplianceSpec, Constraint,
    DeploymentPreferences, DeploymentStrategy, EncryptionSpec, FaultToleranceLevel,
    GlobalResourceSpec, KeyManagementSpec, LoadBalancingSpec, LogStorage, LoggingSpec, MetricsSpec,
    MetricsStorage, MonitoringSpec, NetworkSecuritySpec, NetworkTopology,
    NetworkingSpec, PerformanceRequirements, PrimalPreference, RuntimeSpec,
    ScalingRequirements, SecurityRequirements, ServiceConfig,
    ServiceDefinition, StorageSpec, TracingSpec, TracingStorage, UniversalBiomeCoordinator,
    UniversalBiomeManifest, UniversalBiomeMetadata,
    UniversalResourceRequirements, UniversalRuntimeType, ValidationRule, ValidationSpec,
};
use std::collections::HashMap;
use tokio;
use tracing::{info, Level};
// Note: uuid::Uuid import removed - IDs would be generated in full implementation

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Universal Biome Example");

    // Example 1: Create a universal biome manifest for a web application
    let web_app_manifest = create_web_app_manifest()?;
    info!("Created web application manifest");

    // Example 2: Create a universal biome manifest for AI/ML workload
    let ai_ml_manifest = create_ai_ml_manifest()?;
    info!("Created AI/ML workload manifest");

    // Example 3: Create a universal biome manifest for a distributed database
    let database_manifest = create_database_manifest()?;
    info!("Created database manifest");

    // Example 4: Bootstrap ecosystems using universal coordinator
    let _coordinator = UniversalBiomeCoordinator::new();

    // Note: In a real implementation, you would have actual primals running
    // that implement the UniversalPrimalProvider trait. This example shows
    // the structure without actual deployment.

    info!(
        "Web App Biome requires capabilities: {:?}",
        web_app_manifest.get_all_required_capabilities()
    );
    info!(
        "AI/ML Biome requires capabilities: {:?}",
        ai_ml_manifest.get_all_required_capabilities()
    );
    info!(
        "Database Biome requires capabilities: {:?}",
        database_manifest.get_all_required_capabilities()
    );

    // Example 5: Show how this is agnostic to primal implementations
    demonstrate_agnostic_approach(&web_app_manifest)?;

    info!("Universal Biome Example completed successfully");
    Ok(())
}

/// Create a universal biome manifest for a web application
fn create_web_app_manifest() -> Result<UniversalBiomeManifest, Box<dyn std::error::Error>> {
    let manifest = UniversalBiomeManifest {
        api_version: "biomeOS/v1".to_string(),
        kind: "Biome".to_string(),
        metadata: UniversalBiomeMetadata {
            name: "web-application".to_string(),
            description: "A scalable web application with load balancing and caching".to_string(),
            version: "1.0.0".to_string(),
            maintainer: Some("Web Team".to_string()),
            tags: vec![
                "web".to_string(),
                "application".to_string(),
                "scalable".to_string(),
            ],
            labels: HashMap::from([
                ("environment".to_string(), "production".to_string()),
                ("tier".to_string(), "frontend".to_string()),
            ]),
            annotations: HashMap::from([(
                "deployment.kubernetes.io/revision".to_string(),
                "1".to_string(),
            )]),
            created: Some(chrono::Utc::now()),
            modified: Some(chrono::Utc::now()),
        },
        requirements: BiomeRequirements {
            // Required capabilities - note: no hard-coded primal names!
            required: vec![
                CapabilityRequirement {
                    capability: "compute.container_orchestration".to_string(),
                    min_version: "1.0.0".to_string(),
                    max_version: None,
                    optional: false,
                    constraints: vec![Constraint {
                        constraint_type: "performance".to_string(),
                        value: serde_json::json!({"min_replicas": 2}),
                        error_message: "Must support at least 2 replicas".to_string(),
                    }],
                    fallback: None,
                },
                CapabilityRequirement {
                    capability: "networking.load_balancing".to_string(),
                    min_version: "1.0.0".to_string(),
                    max_version: None,
                    optional: false,
                    constraints: vec![],
                    fallback: Some("networking.basic_routing".to_string()),
                },
                CapabilityRequirement {
                    capability: "storage.persistent_volumes".to_string(),
                    min_version: "1.0.0".to_string(),
                    max_version: None,
                    optional: false,
                    constraints: vec![],
                    fallback: None,
                },
            ],
            // Optional capabilities that enhance the application
            optional: vec![
                CapabilityRequirement {
                    capability: "storage.caching".to_string(),
                    min_version: "1.0.0".to_string(),
                    max_version: None,
                    optional: true,
                    constraints: vec![],
                    fallback: None,
                },
                CapabilityRequirement {
                    capability: "monitoring.metrics_collection".to_string(),
                    min_version: "1.0.0".to_string(),
                    max_version: None,
                    optional: true,
                    constraints: vec![],
                    fallback: None,
                },
                CapabilityRequirement {
                    capability: "security.tls_termination".to_string(),
                    min_version: "1.0.0".to_string(),
                    max_version: None,
                    optional: true,
                    constraints: vec![],
                    fallback: None,
                },
            ],
            min_resources: UniversalResourceRequirements {
                cpu: Some("500m".to_string()),
                memory: Some("1Gi".to_string()),
                storage: Some("10Gi".to_string()),
                network: Some("100Mbps".to_string()),
                gpu: None,
            },
            preferred_resources: Some(UniversalResourceRequirements {
                cpu: Some("2000m".to_string()),
                memory: Some("4Gi".to_string()),
                storage: Some("50Gi".to_string()),
                network: Some("1Gbps".to_string()),
                gpu: None,
            }),
            max_resources: Some(UniversalResourceRequirements {
                cpu: Some("8000m".to_string()),
                memory: Some("16Gi".to_string()),
                storage: Some("200Gi".to_string()),
                network: Some("10Gbps".to_string()),
                gpu: None,
            }),
            performance: PerformanceRequirements {
                max_latency_ms: Some(200),
                min_throughput: Some("1000 rps".to_string()),
                max_error_rate: Some(0.01),
                min_uptime: Some(0.999),
            },
            availability: AvailabilityRequirements {
                high_availability: true,
                fault_tolerance: FaultToleranceLevel::High,
                disaster_recovery: true,
                backup: BackupRequirements {
                    required: true,
                    frequency: Some("daily".to_string()),
                    retention: Some("30d".to_string()),
                    encryption: true,
                },
            },
            scaling: ScalingRequirements {
                auto_scaling: true,
                min_instances: 2,
                max_instances: 10,
                triggers: vec![],
                policies: vec![],
            },
        },
        services: vec![
            ServiceDefinition {
                name: "web-frontend".to_string(),
                description: "React frontend application".to_string(),
                service_type: "web".to_string(),
                required_capabilities: vec![CapabilityRequirement {
                    capability: "compute.container_orchestration".to_string(),
                    min_version: "1.0.0".to_string(),
                    max_version: None,
                    optional: false,
                    constraints: vec![],
                    fallback: None,
                }],
                config: ServiceConfig {
                    source: "myapp/frontend:v1.0.0".to_string(),
                    runtime: RuntimeSpec {
                        runtime_type: UniversalRuntimeType::Container,
                        version: Some("latest".to_string()),
                        options: HashMap::new(),
                    },
                    environment: HashMap::from([
                        ("NODE_ENV".to_string(), "production".to_string()),
                        ("API_URL".to_string(), "http://api:3000".to_string()),
                    ]),
                    command: Some(vec!["npm".to_string(), "start".to_string()]),
                    working_dir: Some("/app".to_string()),
                    user: Some("node".to_string()),
                    security_context: None,
                },
                resources: UniversalResourceRequirements {
                    cpu: Some("200m".to_string()),
                    memory: Some("512Mi".to_string()),
                    storage: Some("1Gi".to_string()),
                    network: None,
                    gpu: None,
                },
                networking: biomeos_core::ServiceNetworking {
                    ports: vec![biomeos_core::PortSpec {
                        name: "http".to_string(),
                        port: 80,
                        protocol: "tcp".to_string(),
                        external: true,
                    }],
                    policies: vec![],
                    load_balancing: None,
                },
                storage: biomeos_core::ServiceStorage {
                    volumes: vec![],
                    persistent: vec![],
                    temporary: None,
                },
                health_checks: vec![biomeos_core::HealthCheckConfig {
                    name: "readiness".to_string(),
                    check_type: biomeos_core::HealthCheckType::Http {
                        path: "/health".to_string(),
                        port: 80,
                        headers: HashMap::new(),
                    },
                    interval: "30s".to_string(),
                    timeout: "5s".to_string(),
                    failure_threshold: 3,
                    success_threshold: 1,
                    initial_delay: Some("10s".to_string()),
                }],
                dependencies: vec![biomeos_core::ServiceDependency {
                    name: "api".to_string(),
                    dependency_type: biomeos_core::DependencyType::Service,
                    optional: false,
                    config: HashMap::new(),
                }],
            },
            ServiceDefinition {
                name: "api".to_string(),
                description: "REST API backend".to_string(),
                service_type: "api".to_string(),
                required_capabilities: vec![
                    CapabilityRequirement {
                        capability: "compute.container_orchestration".to_string(),
                        min_version: "1.0.0".to_string(),
                        max_version: None,
                        optional: false,
                        constraints: vec![],
                        fallback: None,
                    },
                    CapabilityRequirement {
                        capability: "storage.persistent_volumes".to_string(),
                        min_version: "1.0.0".to_string(),
                        max_version: None,
                        optional: false,
                        constraints: vec![],
                        fallback: None,
                    },
                ],
                config: ServiceConfig {
                    source: "myapp/api:v1.0.0".to_string(),
                    runtime: RuntimeSpec {
                        runtime_type: UniversalRuntimeType::Container,
                        version: Some("latest".to_string()),
                        options: HashMap::new(),
                    },
                    environment: HashMap::from([
                        ("NODE_ENV".to_string(), "production".to_string()),
                        (
                            "DB_URL".to_string(),
                            "postgresql://db:5432/myapp".to_string(),
                        ),
                    ]),
                    command: Some(vec!["node".to_string(), "server.js".to_string()]),
                    working_dir: Some("/app".to_string()),
                    user: Some("node".to_string()),
                    security_context: None,
                },
                resources: UniversalResourceRequirements {
                    cpu: Some("300m".to_string()),
                    memory: Some("512Mi".to_string()),
                    storage: Some("2Gi".to_string()),
                    network: None,
                    gpu: None,
                },
                networking: biomeos_core::ServiceNetworking {
                    ports: vec![biomeos_core::PortSpec {
                        name: "http".to_string(),
                        port: 3000,
                        protocol: "tcp".to_string(),
                        external: false,
                    }],
                    policies: vec![],
                    load_balancing: None,
                },
                storage: biomeos_core::ServiceStorage {
                    volumes: vec![],
                    persistent: vec![biomeos_core::PersistentStorage {
                        name: "api-data".to_string(),
                        size: "5Gi".to_string(),
                        storage_class: Some("fast-ssd".to_string()),
                        access_mode: "ReadWriteOnce".to_string(),
                        backup_policy: Some("daily".to_string()),
                    }],
                    temporary: None,
                },
                health_checks: vec![biomeos_core::HealthCheckConfig {
                    name: "health".to_string(),
                    check_type: biomeos_core::HealthCheckType::Http {
                        path: "/api/health".to_string(),
                        port: 3000,
                        headers: HashMap::new(),
                    },
                    interval: "30s".to_string(),
                    timeout: "5s".to_string(),
                    failure_threshold: 3,
                    success_threshold: 1,
                    initial_delay: Some("30s".to_string()),
                }],
                dependencies: vec![],
            },
        ],
        resources: GlobalResourceSpec {
            limits: UniversalResourceRequirements {
                cpu: Some("4000m".to_string()),
                memory: Some("8Gi".to_string()),
                storage: Some("100Gi".to_string()),
                network: Some("1Gbps".to_string()),
                gpu: None,
            },
            reservations: UniversalResourceRequirements {
                cpu: Some("1000m".to_string()),
                memory: Some("2Gi".to_string()),
                storage: Some("20Gi".to_string()),
                network: Some("100Mbps".to_string()),
                gpu: None,
            },
            quotas: vec![],
            pools: vec![],
        },
        security: SecurityRequirements {
            authentication: AuthenticationSpec {
                methods: vec!["oauth2".to_string(), "jwt".to_string()],
                mfa: false,
                token_expiration: Some("1h".to_string()),
                providers: vec![],
            },
            authorization: AuthorizationSpec {
                model: "rbac".to_string(),
                roles: vec![],
                permissions: vec![],
                policies: vec![],
            },
            encryption: EncryptionSpec {
                at_rest: true,
                in_transit: true,
                algorithms: vec!["AES-256".to_string(), "RSA-2048".to_string()],
                key_management: KeyManagementSpec {
                    provider: "vault".to_string(),
                    rotation: true,
                    rotation_interval: Some("90d".to_string()),
                    backup: true,
                },
            },
            network_security: NetworkSecuritySpec {
                isolation: true,
                firewall: vec![],
                vpn: false,
                ddos_protection: true,
            },
            audit: AuditSpec {
                enabled: true,
                events: vec!["authentication".to_string(), "authorization".to_string()],
                storage: AuditStorage {
                    storage_type: "persistent".to_string(),
                    config: HashMap::new(),
                },
                retention: "1y".to_string(),
            },
            compliance: vec![ComplianceSpec {
                standard: "SOC2".to_string(),
                version: "2019".to_string(),
                controls: vec!["CC1.1".to_string(), "CC2.1".to_string()],
            }],
        },
        networking: NetworkingSpec {
            topology: NetworkTopology::Mesh,
            policies: vec![],
            load_balancing: LoadBalancingSpec {
                enabled: true,
                algorithm: "round_robin".to_string(),
                health_check: None,
            },
            service_mesh: None,
        },
        storage: StorageSpec {
            storage_classes: vec![],
            default_class: "standard".to_string(),
            policies: vec![],
            backup: BackupSpec {
                enabled: true,
                schedule: "0 2 * * *".to_string(),
                retention: "30d".to_string(),
                encryption: true,
                storage: BackupStorage {
                    storage_type: "s3".to_string(),
                    config: HashMap::new(),
                },
            },
        },
        monitoring: MonitoringSpec {
            enabled: true,
            metrics: MetricsSpec {
                enabled: true,
                retention: "30d".to_string(),
                storage: MetricsStorage {
                    storage_type: "prometheus".to_string(),
                    config: HashMap::new(),
                },
            },
            logging: LoggingSpec {
                enabled: true,
                level: "info".to_string(),
                retention: "30d".to_string(),
                storage: LogStorage {
                    storage_type: "elasticsearch".to_string(),
                    config: HashMap::new(),
                },
            },
            alerting: AlertingSpec {
                enabled: true,
                rules: vec![],
                receivers: vec![],
            },
            tracing: TracingSpec {
                enabled: true,
                sampling_rate: 0.1,
                storage: TracingStorage {
                    storage_type: "jaeger".to_string(),
                    config: HashMap::new(),
                },
            },
        },
        deployment: DeploymentPreferences {
            strategy: DeploymentStrategy::Reliability,
            // Note: These are preferences, not requirements
            // Any primal providing the required capabilities can be used
            preferred_primals: vec![
                PrimalPreference {
                    primal_type: "kubernetes_operator".to_string(),
                    weight: 0.8,
                    constraints: vec![],
                },
                PrimalPreference {
                    primal_type: "container_orchestrator".to_string(),
                    weight: 0.6,
                    constraints: vec![],
                },
            ],
            constraints: vec![],
            policies: vec![],
        },
        validation: ValidationSpec {
            rules: vec![ValidationRule {
                name: "resource_limits".to_string(),
                rule_type: "resource".to_string(),
                expression: "cpu < 10000m AND memory < 32Gi".to_string(),
                message: "Resource limits exceeded".to_string(),
            }],
            policies: vec![],
        },
    };

    Ok(manifest)
}

/// Create a universal biome manifest for AI/ML workloads
fn create_ai_ml_manifest() -> Result<UniversalBiomeManifest, Box<dyn std::error::Error>> {
    let manifest = UniversalBiomeManifest {
        api_version: "biomeOS/v1".to_string(),
        kind: "Biome".to_string(),
        metadata: UniversalBiomeMetadata {
            name: "ai-ml-workload".to_string(),
            description: "AI/ML training and inference workload".to_string(),
            version: "1.0.0".to_string(),
            maintainer: Some("AI Team".to_string()),
            tags: vec![
                "ai".to_string(),
                "ml".to_string(),
                "training".to_string(),
                "inference".to_string(),
            ],
            labels: HashMap::from([
                ("workload_type".to_string(), "ai_ml".to_string()),
                ("gpu_required".to_string(), "true".to_string()),
            ]),
            annotations: HashMap::new(),
            created: Some(chrono::Utc::now()),
            modified: Some(chrono::Utc::now()),
        },
        requirements: BiomeRequirements {
            required: vec![
                CapabilityRequirement {
                    capability: "compute.gpu_orchestration".to_string(),
                    min_version: "1.0.0".to_string(),
                    max_version: None,
                    optional: false,
                    constraints: vec![Constraint {
                        constraint_type: "gpu".to_string(),
                        value: serde_json::json!({"min_gpu_memory": "8GB", "cuda_version": "11.0+"}),
                        error_message: "Requires GPU with at least 8GB memory and CUDA 11.0+"
                            .to_string(),
                    }],
                    fallback: None,
                },
                CapabilityRequirement {
                    capability: "storage.high_throughput".to_string(),
                    min_version: "1.0.0".to_string(),
                    max_version: None,
                    optional: false,
                    constraints: vec![],
                    fallback: None,
                },
                CapabilityRequirement {
                    capability: "ai.model_serving".to_string(),
                    min_version: "1.0.0".to_string(),
                    max_version: None,
                    optional: false,
                    constraints: vec![],
                    fallback: None,
                },
            ],
            optional: vec![
                CapabilityRequirement {
                    capability: "ai.distributed_training".to_string(),
                    min_version: "1.0.0".to_string(),
                    max_version: None,
                    optional: true,
                    constraints: vec![],
                    fallback: None,
                },
                CapabilityRequirement {
                    capability: "monitoring.gpu_metrics".to_string(),
                    min_version: "1.0.0".to_string(),
                    max_version: None,
                    optional: true,
                    constraints: vec![],
                    fallback: None,
                },
            ],
            min_resources: UniversalResourceRequirements {
                cpu: Some("2000m".to_string()),
                memory: Some("8Gi".to_string()),
                storage: Some("100Gi".to_string()),
                network: Some("1Gbps".to_string()),
                gpu: Some("1x Tesla V100".to_string()),
            },
            preferred_resources: Some(UniversalResourceRequirements {
                cpu: Some("8000m".to_string()),
                memory: Some("32Gi".to_string()),
                storage: Some("1Ti".to_string()),
                network: Some("10Gbps".to_string()),
                gpu: Some("4x Tesla V100".to_string()),
            }),
            max_resources: Some(UniversalResourceRequirements {
                cpu: Some("32000m".to_string()),
                memory: Some("128Gi".to_string()),
                storage: Some("10Ti".to_string()),
                network: Some("100Gbps".to_string()),
                gpu: Some("8x Tesla V100".to_string()),
            }),
            performance: PerformanceRequirements {
                max_latency_ms: Some(1000),
                min_throughput: Some("100 inferences/sec".to_string()),
                max_error_rate: Some(0.001),
                min_uptime: Some(0.99),
            },
            availability: AvailabilityRequirements {
                high_availability: false,
                fault_tolerance: FaultToleranceLevel::Basic,
                disaster_recovery: false,
                backup: BackupRequirements {
                    required: true,
                    frequency: Some("weekly".to_string()),
                    retention: Some("90d".to_string()),
                    encryption: true,
                },
            },
            scaling: ScalingRequirements {
                auto_scaling: true,
                min_instances: 1,
                max_instances: 5,
                triggers: vec![],
                policies: vec![],
            },
        },
        services: vec![ServiceDefinition {
            name: "model-training".to_string(),
            description: "TensorFlow model training service".to_string(),
            service_type: "ai_training".to_string(),
            required_capabilities: vec![CapabilityRequirement {
                capability: "compute.gpu_orchestration".to_string(),
                min_version: "1.0.0".to_string(),
                max_version: None,
                optional: false,
                constraints: vec![],
                fallback: None,
            }],
            config: ServiceConfig {
                source: "tensorflow/tensorflow:latest-gpu".to_string(),
                runtime: RuntimeSpec {
                    runtime_type: UniversalRuntimeType::Container,
                    version: Some("latest".to_string()),
                    options: HashMap::from([(
                        "gpu".to_string(),
                        serde_json::json!({"enabled": true}),
                    )]),
                },
                environment: HashMap::from([
                    ("NVIDIA_VISIBLE_DEVICES".to_string(), "all".to_string()),
                    ("CUDA_VISIBLE_DEVICES".to_string(), "0,1,2,3".to_string()),
                ]),
                command: Some(vec!["python".to_string(), "train.py".to_string()]),
                working_dir: Some("/workspace".to_string()),
                user: Some("root".to_string()),
                security_context: None,
            },
            resources: UniversalResourceRequirements {
                cpu: Some("4000m".to_string()),
                memory: Some("16Gi".to_string()),
                storage: Some("500Gi".to_string()),
                network: Some("1Gbps".to_string()),
                gpu: Some("4x Tesla V100".to_string()),
            },
            networking: biomeos_core::ServiceNetworking {
                ports: vec![biomeos_core::PortSpec {
                    name: "tensorboard".to_string(),
                    port: 6006,
                    protocol: "tcp".to_string(),
                    external: true,
                }],
                policies: vec![],
                load_balancing: None,
            },
            storage: biomeos_core::ServiceStorage {
                volumes: vec![],
                persistent: vec![biomeos_core::PersistentStorage {
                    name: "model-data".to_string(),
                    size: "1Ti".to_string(),
                    storage_class: Some("fast-ssd".to_string()),
                    access_mode: "ReadWriteOnce".to_string(),
                    backup_policy: Some("weekly".to_string()),
                }],
                temporary: None,
            },
            health_checks: vec![],
            dependencies: vec![],
        }],
        // ... rest of the manifest with appropriate defaults
        resources: GlobalResourceSpec {
            limits: UniversalResourceRequirements {
                cpu: Some("16000m".to_string()),
                memory: Some("64Gi".to_string()),
                storage: Some("2Ti".to_string()),
                network: Some("10Gbps".to_string()),
                gpu: Some("4x Tesla V100".to_string()),
            },
            reservations: UniversalResourceRequirements {
                cpu: Some("4000m".to_string()),
                memory: Some("16Gi".to_string()),
                storage: Some("500Gi".to_string()),
                network: Some("1Gbps".to_string()),
                gpu: Some("1x Tesla V100".to_string()),
            },
            quotas: vec![],
            pools: vec![],
        },
        security: SecurityRequirements {
            authentication: AuthenticationSpec {
                methods: vec!["api_key".to_string()],
                mfa: false,
                token_expiration: Some("24h".to_string()),
                providers: vec![],
            },
            authorization: AuthorizationSpec {
                model: "simple".to_string(),
                roles: vec![],
                permissions: vec![],
                policies: vec![],
            },
            encryption: EncryptionSpec {
                at_rest: true,
                in_transit: false,
                algorithms: vec!["AES-256".to_string()],
                key_management: KeyManagementSpec {
                    provider: "local".to_string(),
                    rotation: false,
                    rotation_interval: None,
                    backup: false,
                },
            },
            network_security: NetworkSecuritySpec {
                isolation: false,
                firewall: vec![],
                vpn: false,
                ddos_protection: false,
            },
            audit: AuditSpec {
                enabled: false,
                events: vec![],
                storage: AuditStorage {
                    storage_type: "file".to_string(),
                    config: HashMap::new(),
                },
                retention: "30d".to_string(),
            },
            compliance: vec![],
        },
        networking: NetworkingSpec {
            topology: NetworkTopology::Flat,
            policies: vec![],
            load_balancing: LoadBalancingSpec {
                enabled: false,
                algorithm: "round_robin".to_string(),
                health_check: None,
            },
            service_mesh: None,
        },
        storage: StorageSpec {
            storage_classes: vec![],
            default_class: "fast-ssd".to_string(),
            policies: vec![],
            backup: BackupSpec {
                enabled: true,
                schedule: "0 3 * * 0".to_string(),
                retention: "90d".to_string(),
                encryption: true,
                storage: BackupStorage {
                    storage_type: "s3".to_string(),
                    config: HashMap::new(),
                },
            },
        },
        monitoring: MonitoringSpec {
            enabled: true,
            metrics: MetricsSpec {
                enabled: true,
                retention: "30d".to_string(),
                storage: MetricsStorage {
                    storage_type: "prometheus".to_string(),
                    config: HashMap::new(),
                },
            },
            logging: LoggingSpec {
                enabled: true,
                level: "info".to_string(),
                retention: "30d".to_string(),
                storage: LogStorage {
                    storage_type: "file".to_string(),
                    config: HashMap::new(),
                },
            },
            alerting: AlertingSpec {
                enabled: false,
                rules: vec![],
                receivers: vec![],
            },
            tracing: TracingSpec {
                enabled: false,
                sampling_rate: 0.1,
                storage: TracingStorage {
                    storage_type: "memory".to_string(),
                    config: HashMap::new(),
                },
            },
        },
        deployment: DeploymentPreferences {
            strategy: DeploymentStrategy::Performance,
            preferred_primals: vec![
                PrimalPreference {
                    primal_type: "gpu_orchestrator".to_string(),
                    weight: 0.9,
                    constraints: vec![],
                },
                PrimalPreference {
                    primal_type: "cuda_runtime".to_string(),
                    weight: 0.8,
                    constraints: vec![],
                },
            ],
            constraints: vec![],
            policies: vec![],
        },
        validation: ValidationSpec {
            rules: vec![],
            policies: vec![],
        },
    };

    Ok(manifest)
}

/// Create a universal biome manifest for a distributed database
fn create_database_manifest() -> Result<UniversalBiomeManifest, Box<dyn std::error::Error>> {
    let manifest = UniversalBiomeManifest {
        api_version: "biomeOS/v1".to_string(),
        kind: "Biome".to_string(),
        metadata: UniversalBiomeMetadata {
            name: "distributed-database".to_string(),
            description: "High-availability distributed database cluster".to_string(),
            version: "1.0.0".to_string(),
            maintainer: Some("Database Team".to_string()),
            tags: vec![
                "database".to_string(),
                "distributed".to_string(),
                "high-availability".to_string(),
            ],
            labels: HashMap::from([
                ("database_type".to_string(), "postgresql".to_string()),
                ("cluster_size".to_string(), "3".to_string()),
            ]),
            annotations: HashMap::new(),
            created: Some(chrono::Utc::now()),
            modified: Some(chrono::Utc::now()),
        },
        requirements: BiomeRequirements {
            required: vec![
                CapabilityRequirement {
                    capability: "storage.distributed_file_system".to_string(),
                    min_version: "1.0.0".to_string(),
                    max_version: None,
                    optional: false,
                    constraints: vec![Constraint {
                        constraint_type: "replication".to_string(),
                        value: serde_json::json!({"min_replicas": 3}),
                        error_message: "Requires at least 3 replicas for high availability"
                            .to_string(),
                    }],
                    fallback: None,
                },
                CapabilityRequirement {
                    capability: "compute.stateful_workloads".to_string(),
                    min_version: "1.0.0".to_string(),
                    max_version: None,
                    optional: false,
                    constraints: vec![],
                    fallback: None,
                },
                CapabilityRequirement {
                    capability: "networking.service_discovery".to_string(),
                    min_version: "1.0.0".to_string(),
                    max_version: None,
                    optional: false,
                    constraints: vec![],
                    fallback: None,
                },
                CapabilityRequirement {
                    capability: "security.encryption_at_rest".to_string(),
                    min_version: "1.0.0".to_string(),
                    max_version: None,
                    optional: false,
                    constraints: vec![],
                    fallback: None,
                },
            ],
            optional: vec![
                CapabilityRequirement {
                    capability: "storage.automated_backup".to_string(),
                    min_version: "1.0.0".to_string(),
                    max_version: None,
                    optional: true,
                    constraints: vec![],
                    fallback: None,
                },
                CapabilityRequirement {
                    capability: "monitoring.database_metrics".to_string(),
                    min_version: "1.0.0".to_string(),
                    max_version: None,
                    optional: true,
                    constraints: vec![],
                    fallback: None,
                },
            ],
            min_resources: UniversalResourceRequirements {
                cpu: Some("1000m".to_string()),
                memory: Some("4Gi".to_string()),
                storage: Some("100Gi".to_string()),
                network: Some("1Gbps".to_string()),
                gpu: None,
            },
            preferred_resources: Some(UniversalResourceRequirements {
                cpu: Some("4000m".to_string()),
                memory: Some("16Gi".to_string()),
                storage: Some("500Gi".to_string()),
                network: Some("10Gbps".to_string()),
                gpu: None,
            }),
            max_resources: Some(UniversalResourceRequirements {
                cpu: Some("16000m".to_string()),
                memory: Some("64Gi".to_string()),
                storage: Some("2Ti".to_string()),
                network: Some("100Gbps".to_string()),
                gpu: None,
            }),
            performance: PerformanceRequirements {
                max_latency_ms: Some(10),
                min_throughput: Some("10000 queries/sec".to_string()),
                max_error_rate: Some(0.0001),
                min_uptime: Some(0.9999),
            },
            availability: AvailabilityRequirements {
                high_availability: true,
                fault_tolerance: FaultToleranceLevel::Critical,
                disaster_recovery: true,
                backup: BackupRequirements {
                    required: true,
                    frequency: Some("hourly".to_string()),
                    retention: Some("1y".to_string()),
                    encryption: true,
                },
            },
            scaling: ScalingRequirements {
                auto_scaling: true,
                min_instances: 3,
                max_instances: 9,
                triggers: vec![],
                policies: vec![],
            },
        },
        services: vec![ServiceDefinition {
            name: "postgresql-primary".to_string(),
            description: "PostgreSQL primary database instance".to_string(),
            service_type: "database".to_string(),
            required_capabilities: vec![CapabilityRequirement {
                capability: "storage.distributed_file_system".to_string(),
                min_version: "1.0.0".to_string(),
                max_version: None,
                optional: false,
                constraints: vec![],
                fallback: None,
            }],
            config: ServiceConfig {
                source: "postgres:13".to_string(),
                runtime: RuntimeSpec {
                    runtime_type: UniversalRuntimeType::Container,
                    version: Some("13".to_string()),
                    options: HashMap::new(),
                },
                environment: HashMap::from([
                    ("POSTGRES_DB".to_string(), "myapp".to_string()),
                    ("POSTGRES_USER".to_string(), "admin".to_string()),
                    ("POSTGRES_PASSWORD".to_string(), "changeme".to_string()),
                ]),
                command: None,
                working_dir: None,
                user: Some("postgres".to_string()),
                security_context: None,
            },
            resources: UniversalResourceRequirements {
                cpu: Some("2000m".to_string()),
                memory: Some("8Gi".to_string()),
                storage: Some("200Gi".to_string()),
                network: Some("1Gbps".to_string()),
                gpu: None,
            },
            networking: biomeos_core::ServiceNetworking {
                ports: vec![biomeos_core::PortSpec {
                    name: "postgresql".to_string(),
                    port: 5432,
                    protocol: "tcp".to_string(),
                    external: false,
                }],
                policies: vec![],
                load_balancing: None,
            },
            storage: biomeos_core::ServiceStorage {
                volumes: vec![],
                persistent: vec![biomeos_core::PersistentStorage {
                    name: "postgresql-data".to_string(),
                    size: "500Gi".to_string(),
                    storage_class: Some("fast-ssd".to_string()),
                    access_mode: "ReadWriteOnce".to_string(),
                    backup_policy: Some("hourly".to_string()),
                }],
                temporary: None,
            },
            health_checks: vec![biomeos_core::HealthCheckConfig {
                name: "postgresql-ready".to_string(),
                check_type: biomeos_core::HealthCheckType::Command {
                    command: vec![
                        "pg_isready".to_string(),
                        "-U".to_string(),
                        "admin".to_string(),
                    ],
                },
                interval: "30s".to_string(),
                timeout: "5s".to_string(),
                failure_threshold: 3,
                success_threshold: 1,
                initial_delay: Some("60s".to_string()),
            }],
            dependencies: vec![],
        }],
        // ... rest of the configuration with appropriate defaults
        resources: GlobalResourceSpec {
            limits: UniversalResourceRequirements {
                cpu: Some("8000m".to_string()),
                memory: Some("32Gi".to_string()),
                storage: Some("1Ti".to_string()),
                network: Some("10Gbps".to_string()),
                gpu: None,
            },
            reservations: UniversalResourceRequirements {
                cpu: Some("2000m".to_string()),
                memory: Some("8Gi".to_string()),
                storage: Some("200Gi".to_string()),
                network: Some("1Gbps".to_string()),
                gpu: None,
            },
            quotas: vec![],
            pools: vec![],
        },
        security: SecurityRequirements {
            authentication: AuthenticationSpec {
                methods: vec!["password".to_string(), "certificate".to_string()],
                mfa: false,
                token_expiration: None,
                providers: vec![],
            },
            authorization: AuthorizationSpec {
                model: "rbac".to_string(),
                roles: vec![],
                permissions: vec![],
                policies: vec![],
            },
            encryption: EncryptionSpec {
                at_rest: true,
                in_transit: true,
                algorithms: vec!["AES-256".to_string()],
                key_management: KeyManagementSpec {
                    provider: "vault".to_string(),
                    rotation: true,
                    rotation_interval: Some("30d".to_string()),
                    backup: true,
                },
            },
            network_security: NetworkSecuritySpec {
                isolation: true,
                firewall: vec![],
                vpn: false,
                ddos_protection: true,
            },
            audit: AuditSpec {
                enabled: true,
                events: vec!["authentication".to_string(), "data_access".to_string()],
                storage: AuditStorage {
                    storage_type: "persistent".to_string(),
                    config: HashMap::new(),
                },
                retention: "2y".to_string(),
            },
            compliance: vec![ComplianceSpec {
                standard: "PCI-DSS".to_string(),
                version: "3.2".to_string(),
                controls: vec!["3.4".to_string(), "8.2".to_string()],
            }],
        },
        networking: NetworkingSpec {
            topology: NetworkTopology::Segmented,
            policies: vec![],
            load_balancing: LoadBalancingSpec {
                enabled: true,
                algorithm: "least_connections".to_string(),
                health_check: None,
            },
            service_mesh: None,
        },
        storage: StorageSpec {
            storage_classes: vec![],
            default_class: "fast-ssd".to_string(),
            policies: vec![],
            backup: BackupSpec {
                enabled: true,
                schedule: "0 */6 * * *".to_string(),
                retention: "1y".to_string(),
                encryption: true,
                storage: BackupStorage {
                    storage_type: "s3".to_string(),
                    config: HashMap::new(),
                },
            },
        },
        monitoring: MonitoringSpec {
            enabled: true,
            metrics: MetricsSpec {
                enabled: true,
                retention: "90d".to_string(),
                storage: MetricsStorage {
                    storage_type: "prometheus".to_string(),
                    config: HashMap::new(),
                },
            },
            logging: LoggingSpec {
                enabled: true,
                level: "info".to_string(),
                retention: "90d".to_string(),
                storage: LogStorage {
                    storage_type: "elasticsearch".to_string(),
                    config: HashMap::new(),
                },
            },
            alerting: AlertingSpec {
                enabled: true,
                rules: vec![],
                receivers: vec![],
            },
            tracing: TracingSpec {
                enabled: false,
                sampling_rate: 0.1,
                storage: TracingStorage {
                    storage_type: "memory".to_string(),
                    config: HashMap::new(),
                },
            },
        },
        deployment: DeploymentPreferences {
            strategy: DeploymentStrategy::Reliability,
            preferred_primals: vec![
                PrimalPreference {
                    primal_type: "distributed_storage".to_string(),
                    weight: 0.9,
                    constraints: vec![],
                },
                PrimalPreference {
                    primal_type: "stateful_orchestrator".to_string(),
                    weight: 0.8,
                    constraints: vec![],
                },
            ],
            constraints: vec![],
            policies: vec![],
        },
        validation: ValidationSpec {
            rules: vec![],
            policies: vec![],
        },
    };

    Ok(manifest)
}

/// Demonstrate the agnostic approach - the same manifest can work with different primals
fn demonstrate_agnostic_approach(
    manifest: &UniversalBiomeManifest,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Demonstrating Agnostic Approach ===");

    // The same manifest can be satisfied by different primal implementations
    let scenarios = vec![
        (
            "Kubernetes Ecosystem",
            vec![
                "kubernetes_operator",
                "istio_service_mesh",
                "ceph_storage",
                "prometheus_monitoring",
            ],
        ),
        (
            "Docker Swarm Ecosystem",
            vec![
                "docker_swarm",
                "traefik_proxy",
                "glusterfs_storage",
                "grafana_monitoring",
            ],
        ),
        (
            "Custom Primal Ecosystem",
            vec![
                "my_custom_orchestrator",
                "my_load_balancer",
                "my_storage_system",
                "my_monitoring",
            ],
        ),
        (
            "Cloud Native Ecosystem",
            vec!["aws_ecs", "aws_elb", "aws_efs", "aws_cloudwatch"],
        ),
    ];

    for (scenario_name, primals) in scenarios {
        info!("Scenario: {}", scenario_name);
        info!("  Available primals: {:?}", primals);

        // The universal manifest doesn't need to change - it just requires capabilities
        let required_capabilities = manifest.get_all_required_capabilities();
        info!("  Required capabilities: {:?}", required_capabilities);

        // Any primal that provides these capabilities can satisfy the requirements
        info!("  ✅ This scenario can potentially satisfy the biome requirements");
        info!("  ✅ No code changes needed - just different primal implementations");
        info!("");
    }

    info!("=== Key Benefits ===");
    info!("✅ Same manifest works with any primal implementation");
    info!("✅ No hard-coded dependencies on specific primals");
    info!("✅ Capability-based matching enables flexibility");
    info!("✅ Easy to add new primal implementations");
    info!("✅ Biome authors focus on requirements, not implementation details");
    info!("");

    Ok(())
}

#[allow(dead_code)]
fn example_yaml_output() -> Result<(), Box<dyn std::error::Error>> {
    // Example of how the universal manifest would look in YAML
    let yaml_example = r#"
apiVersion: biomeOS/v1
kind: Biome
metadata:
  name: web-application
  description: A scalable web application
  version: 1.0.0
  tags:
    - web
    - application
    - scalable
  labels:
    environment: production
    tier: frontend

requirements:
  # No hard-coded primal names - just capabilities!
  required:
    - capability: compute.container_orchestration
      min_version: 1.0.0
      constraints:
        - type: performance
          value: {min_replicas: 2}
    - capability: networking.load_balancing
      min_version: 1.0.0
      fallback: networking.basic_routing
    - capability: storage.persistent_volumes
      min_version: 1.0.0
      
  optional:
    - capability: storage.caching
      min_version: 1.0.0
    - capability: monitoring.metrics_collection
      min_version: 1.0.0

deployment:
  strategy: reliability
  # These are preferences, not requirements
  preferred_primals:
    - primal_type: kubernetes_operator
      weight: 0.8
    - primal_type: container_orchestrator
      weight: 0.6
      
services:
  - name: web-frontend
    description: React frontend application
    required_capabilities:
      - capability: compute.container_orchestration
        min_version: 1.0.0
    config:
      source: myapp/frontend:v1.0.0
      runtime:
        type: container
        version: latest
      environment:
        NODE_ENV: production
        API_URL: http://api:3000
"#;

    info!("Example YAML structure:");
    info!("{}", yaml_example);

    Ok(())
}
