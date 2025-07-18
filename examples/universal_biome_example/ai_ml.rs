//! AI/ML Biome Example
//!
//! This module demonstrates creating a universal biome manifest for AI/ML workloads
//! with GPU orchestration, high-performance storage, and model serving capabilities.

use biomeos_core::{
    AuthenticationSpec, AuthorizationSpec, AuditSpec, AuditStorage, AvailabilityRequirements,
    BackupRequirements, BackupSpec, BackupStorage, BiomeRequirements, CapabilityRequirement,
    ComplianceSpec, Constraint, DeploymentPreferences, DeploymentStrategy, EncryptionSpec,
    FaultToleranceLevel, GlobalResourceSpec, KeyManagementSpec, LoadBalancingSpec,
    LogStorage, LoggingSpec, MetricsSpec, MetricsStorage, MonitoringSpec, NetworkSecuritySpec,
    NetworkTopology, NetworkingSpec, PerformanceRequirements, PrimalPreference, RuntimeSpec,
    ScalingRequirements, SecurityRequirements, ServiceConfig, ServiceDefinition, StorageSpec,
    TracingSpec, TracingStorage, UniversalBiomeManifest, UniversalBiomeMetadata,
    UniversalResourceRequirements, UniversalRuntimeType, ValidationSpec,
    ServiceNetworking, ServiceStorage, PortSpec, PersistentStorage, AlertingSpec,
    DnsConfig, LoadBalancingConfig, MonitoringConfig,
};
use std::collections::HashMap;

/// Create a universal biome manifest for AI/ML workloads
pub fn create_ai_ml_manifest() -> Result<UniversalBiomeManifest, Box<dyn std::error::Error>> {
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
        services: vec![create_model_training_service()],
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
        security: create_ai_ml_security_requirements(),
        networking: create_ai_ml_networking_spec(),
        storage: create_ai_ml_storage_spec(),
        monitoring: create_ai_ml_monitoring_spec(),
        deployment: create_ai_ml_deployment_preferences(),
        validation: ValidationSpec {
            rules: vec![],
            policies: vec![],
        },
    };

    Ok(manifest)
}

/// Create the model training service definition
fn create_model_training_service() -> ServiceDefinition {
    ServiceDefinition {
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
        networking: ServiceNetworking {
            ports: vec![PortSpec {
                name: "tensorboard".to_string(),
                port: 6006,
                protocol: "tcp".to_string(),
                external: true,
            }],
            policies: vec![],
            load_balancing: None,
        },
        storage: ServiceStorage {
            volumes: vec![],
            persistent: vec![PersistentStorage {
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
    }
}

/// Create security requirements for AI/ML workloads
fn create_ai_ml_security_requirements() -> SecurityRequirements {
    SecurityRequirements {
        authentication: AuthenticationSpec {
            method: "api_key".to_string(),
            config: HashMap::new(),
            mfa_enabled: false,
        },
        authorization: AuthorizationSpec {
            method: "simple".to_string(),
            config: HashMap::new(),
            rbac_enabled: false,
        },
        encryption: EncryptionSpec {
            algorithm: "AES-256".to_string(),
            key_size: 256,
            at_rest: true,
            in_transit: false,
        },
        compliance: ComplianceSpec {
            frameworks: vec![],
            requirements: HashMap::new(),
            monitoring_enabled: false,
        },
        audit: AuditSpec {
            enabled: false,
            level: "info".to_string(),
            storage: AuditStorage {
                storage_type: "file".to_string(),
                config: HashMap::new(),
            },
        },
        backup: BackupSpec {
            enabled: true,
            schedule: "0 3 * * 0".to_string(),
            storage: BackupStorage {
                storage_type: "s3".to_string(),
                config: HashMap::new(),
            },
        },
        key_management: KeyManagementSpec {
            service: "local".to_string(),
            config: HashMap::new(),
            rotation_enabled: false,
        },
    }
}

/// Create networking specification for AI/ML workloads
fn create_ai_ml_networking_spec() -> NetworkingSpec {
    NetworkingSpec {
        policies: vec![],
        dns: DnsConfig {
            servers: vec!["8.8.8.8".to_string()],
            search_domains: vec!["cluster.local".to_string()],
        },
        load_balancing: LoadBalancingConfig {
            algorithm: "round_robin".to_string(),
            health_check: "tcp".to_string(),
        },
        topology: NetworkTopology::Flat,
    }
}

/// Create storage specification for AI/ML workloads
fn create_ai_ml_storage_spec() -> StorageSpec {
    StorageSpec {
        storage_classes: vec![],
        default_class: "fast-ssd".to_string(),
        policies: vec![],
        backup: BackupSpec {
            enabled: true,
            schedule: "0 3 * * 0".to_string(),
            storage: BackupStorage {
                storage_type: "s3".to_string(),
                config: HashMap::new(),
            },
        },
    }
}

/// Create monitoring specification for AI/ML workloads
fn create_ai_ml_monitoring_spec() -> MonitoringSpec {
    MonitoringSpec {
        enabled: true,
        config: MonitoringConfig {
            metrics: MetricsSpec {
                enabled: true,
                port: 9090,
                path: "/metrics".to_string(),
                storage: MetricsStorage {
                    storage_type: "prometheus".to_string(),
                    config: HashMap::new(),
                },
            },
            logging: LoggingSpec {
                enabled: true,
                level: "info".to_string(),
                format: "json".to_string(),
                storage: LogStorage {
                    storage_type: "file".to_string(),
                    config: HashMap::new(),
                },
            },
            tracing: TracingSpec {
                enabled: false,
                endpoint: "jaeger:14268".to_string(),
                storage: TracingStorage {
                    storage_type: "memory".to_string(),
                    config: HashMap::new(),
                },
            },
            alerting: AlertingSpec {
                enabled: false,
                rules: vec![],
                channels: vec![],
            },
        },
    }
}

/// Create deployment preferences for AI/ML workloads
fn create_ai_ml_deployment_preferences() -> DeploymentPreferences {
    DeploymentPreferences {
        strategy: DeploymentStrategy::Edge,
        primal_preferences: vec![
            PrimalPreference {
                primal_type: "gpu_orchestrator".to_string(),
                priority: 90,
                constraints: HashMap::new(),
            },
            PrimalPreference {
                primal_type: "cuda_runtime".to_string(),
                priority: 80,
                constraints: HashMap::new(),
            },
        ],
        region_preferences: vec!["us-west-2".to_string()],
    }
} 