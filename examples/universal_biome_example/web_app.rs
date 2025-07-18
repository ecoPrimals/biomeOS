//! Web Application Biome Example
//!
//! This module demonstrates creating a universal biome manifest for a web application
//! with frontend, API, and database components.

use biomeos_core::{
    AuthenticationSpec, AuthorizationSpec, AuditSpec, AuditStorage, AvailabilityRequirements,
    BackupRequirements, BackupSpec, BackupStorage, BiomeRequirements, CapabilityRequirement,
    ComplianceSpec, Constraint, DeploymentPreferences, DeploymentStrategy, EncryptionSpec,
    FaultToleranceLevel, GlobalResourceSpec, KeyManagementSpec, LoadBalancingSpec,
    LogStorage, LoggingSpec, MetricsSpec, MetricsStorage, MonitoringSpec, NetworkSecuritySpec,
    NetworkTopology, NetworkingSpec, PerformanceRequirements, PrimalPreference, RuntimeSpec,
    ScalingRequirements, SecurityRequirements, ServiceConfig, ServiceDefinition, StorageSpec,
    TracingSpec, TracingStorage, UniversalBiomeManifest, UniversalBiomeMetadata,
    UniversalResourceRequirements, UniversalRuntimeType, ValidationRule, ValidationSpec,
    ServiceNetworking, ServiceStorage, PortSpec, HealthCheckConfig, HealthCheckType,
    ServiceDependency, DependencyType, PersistentStorage, AlertingSpec, MetricsStorage,
};
use std::collections::HashMap;

/// Create a universal biome manifest for a web application
pub fn create_web_app_manifest() -> Result<UniversalBiomeManifest, Box<dyn std::error::Error>> {
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
            create_web_frontend_service(),
            create_api_service(),
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
        security: create_web_app_security_requirements(),
        networking: create_web_app_networking_spec(),
        storage: create_web_app_storage_spec(),
        monitoring: create_web_app_monitoring_spec(),
        deployment: create_web_app_deployment_preferences(),
        validation: create_web_app_validation_spec(),
    };

    Ok(manifest)
}

/// Create the web frontend service definition
fn create_web_frontend_service() -> ServiceDefinition {
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
        networking: ServiceNetworking {
            ports: vec![PortSpec {
                name: "http".to_string(),
                port: 80,
                protocol: "tcp".to_string(),
                external: true,
            }],
            policies: vec![],
            load_balancing: None,
        },
        storage: ServiceStorage {
            volumes: vec![],
            persistent: vec![],
            temporary: None,
        },
        health_checks: vec![HealthCheckConfig {
            name: "readiness".to_string(),
            check_type: HealthCheckType::Http {
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
        dependencies: vec![ServiceDependency {
            name: "api".to_string(),
            dependency_type: DependencyType::Service,
            optional: false,
            config: HashMap::new(),
        }],
    }
}

/// Create the API service definition
fn create_api_service() -> ServiceDefinition {
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
                ("DB_URL".to_string(), "postgresql://db:5432/myapp".to_string()),
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
        networking: ServiceNetworking {
            ports: vec![PortSpec {
                name: "http".to_string(),
                port: 3000,
                protocol: "tcp".to_string(),
                external: false,
            }],
            policies: vec![],
            load_balancing: None,
        },
        storage: ServiceStorage {
            volumes: vec![],
            persistent: vec![PersistentStorage {
                name: "api-data".to_string(),
                size: "5Gi".to_string(),
                storage_class: Some("fast-ssd".to_string()),
                access_mode: "ReadWriteOnce".to_string(),
                backup_policy: Some("daily".to_string()),
            }],
            temporary: None,
        },
        health_checks: vec![HealthCheckConfig {
            name: "health".to_string(),
            check_type: HealthCheckType::Http {
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
    }
}

/// Create security requirements for the web application
fn create_web_app_security_requirements() -> SecurityRequirements {
    SecurityRequirements {
        authentication: AuthenticationSpec {
            method: "oauth2".to_string(),
            config: HashMap::new(),
            mfa_enabled: false,
        },
        authorization: AuthorizationSpec {
            method: "rbac".to_string(),
            config: HashMap::new(),
            rbac_enabled: true,
        },
        encryption: EncryptionSpec {
            algorithm: "AES-256".to_string(),
            key_size: 256,
            at_rest: true,
            in_transit: true,
        },
        compliance: ComplianceSpec {
            frameworks: vec!["SOC2".to_string()],
            requirements: HashMap::new(),
            monitoring_enabled: true,
        },
        audit: AuditSpec {
            enabled: true,
            level: "info".to_string(),
            storage: AuditStorage {
                storage_type: "persistent".to_string(),
                config: HashMap::new(),
            },
        },
        backup: BackupSpec {
            enabled: true,
            schedule: "0 2 * * *".to_string(),
            storage: BackupStorage {
                storage_type: "s3".to_string(),
                config: HashMap::new(),
            },
        },
        key_management: KeyManagementSpec {
            service: "vault".to_string(),
            config: HashMap::new(),
            rotation_enabled: true,
        },
    }
}

/// Create networking specification for the web application
fn create_web_app_networking_spec() -> NetworkingSpec {
    NetworkingSpec {
        policies: vec![],
        dns: biomeos_core::DnsConfig {
            servers: vec!["8.8.8.8".to_string()],
            search_domains: vec!["default.svc.cluster.local".to_string()],
        },
        load_balancing: biomeos_core::LoadBalancingConfig {
            algorithm: "round_robin".to_string(),
            health_check: "http".to_string(),
        },
        topology: NetworkTopology::Mesh,
    }
}

/// Create storage specification for the web application
fn create_web_app_storage_spec() -> StorageSpec {
    StorageSpec {
        storage_classes: vec![],
        default_class: "standard".to_string(),
        policies: vec![],
        backup: BackupSpec {
            enabled: true,
            schedule: "0 2 * * *".to_string(),
            storage: BackupStorage {
                storage_type: "s3".to_string(),
                config: HashMap::new(),
            },
        },
    }
}

/// Create monitoring specification for the web application
fn create_web_app_monitoring_spec() -> MonitoringSpec {
    MonitoringSpec {
        enabled: true,
        config: biomeos_core::MonitoringConfig {
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
                    storage_type: "elasticsearch".to_string(),
                    config: HashMap::new(),
                },
            },
            tracing: TracingSpec {
                enabled: true,
                endpoint: "jaeger:14268".to_string(),
                storage: TracingStorage {
                    storage_type: "jaeger".to_string(),
                    config: HashMap::new(),
                },
            },
            alerting: AlertingSpec {
                enabled: true,
                rules: vec![],
                channels: vec![],
            },
        },
    }
}

/// Create deployment preferences for the web application
fn create_web_app_deployment_preferences() -> DeploymentPreferences {
    DeploymentPreferences {
        strategy: DeploymentStrategy::Automatic,
        primal_preferences: vec![
            PrimalPreference {
                primal_type: "kubernetes_operator".to_string(),
                priority: 80,
                constraints: HashMap::new(),
            },
            PrimalPreference {
                primal_type: "container_orchestrator".to_string(),
                priority: 60,
                constraints: HashMap::new(),
            },
        ],
        region_preferences: vec!["us-east-1".to_string()],
    }
}

/// Create validation specification for the web application
fn create_web_app_validation_spec() -> ValidationSpec {
    ValidationSpec {
        rules: vec![ValidationRule {
            name: "resource_limits".to_string(),
            rule_type: "resource".to_string(),
            expression: "cpu < 10000m AND memory < 32Gi".to_string(),
            message: "Resource limits exceeded".to_string(),
        }],
        policies: vec![],
    }
} 