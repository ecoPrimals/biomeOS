//! Hello World BiomeOS Primal Example
//!
//! This is a minimal example of a primal that implements the UniversalPrimalService trait.
//! It demonstrates the basic structure and required methods for creating primals.

use async_trait::async_trait;
use biomeos_primal_sdk::{
    UniversalPrimalService, PrimalServiceMetadata, UniversalServiceRequest, UniversalServiceResponse,
    PrimalCapability, PrimalConfiguration, PrimalType, BiomeResult, BiomeError, Health,
    ResourceRequirements, ResourceMetrics, NetworkIoMetrics, HealthReport, ConfigValidationResult,
    UniversalServiceRegistration, ServiceStatus, CapabilityMetadata
};
use std::collections::HashMap;

/// Hello World Primal
/// 
/// A simple demonstration primal that responds to requests with greeting messages.
/// This shows the minimal implementation required for the UniversalPrimalService trait.
pub struct HelloWorld {
    metadata: PrimalServiceMetadata,
    primal_type: PrimalType,
    capabilities: Vec<PrimalCapability>,
    health: Health,
}

impl HelloWorld {
    /// Create a new HelloWorld primal instance
    pub fn new() -> Self {
        let metadata = PrimalServiceMetadata {
            id: "hello-world".to_string(),
            name: "Hello World Primal".to_string(),
            description: "A simple greeting primal for demonstration".to_string(),
            version: "1.0.0".to_string(),
            author: "BiomeOS Community".to_string(),
            homepage: Some("https://github.com/biomeOS/community-examples".to_string()),
            documentation: None,
            license: Some("MIT".to_string()),
            keywords: vec!["example".to_string(), "greeting".to_string(), "demo".to_string()],
            endpoints: HashMap::new(),
            custom: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let primal_type = PrimalType::new("community", "hello-world", "1.0.0");
        
        let capabilities = vec![
            PrimalCapability::new("communication", "greeting", "1.0.0"),
            PrimalCapability::new("demo", "hello_world", "1.0.0"),
        ];

        Self {
            metadata,
            primal_type,
            capabilities,
            health: Health::Healthy,
        }
    }
}

impl Default for HelloWorld {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UniversalPrimalService for HelloWorld {
    fn primal_id(&self) -> &str {
        &self.metadata.id
    }

    fn primal_type(&self) -> &PrimalType {
        &self.primal_type
    }

    fn metadata(&self) -> &PrimalServiceMetadata {
        &self.metadata
    }

    fn version(&self) -> &str {
        &self.metadata.version
    }

    fn capabilities(&self) -> &[PrimalCapability] {
        &self.capabilities
    }

    async fn can_handle_capability(&self, capability: &PrimalCapability) -> bool {
        self.capabilities.contains(capability)
    }

    async fn get_capability_metadata(&self, capability: &str) -> Option<CapabilityMetadata> {
        match capability {
            "greeting" => Some(CapabilityMetadata {
                name: "greeting".to_string(),
                description: "Provides greeting functionality".to_string(),
                version: "1.0.0".to_string(),
                parameters: vec![],
                examples: vec![],
                dependencies: vec![],
            }),
            _ => None,
        }
    }

    async fn initialize(&mut self, _config: &PrimalConfiguration) -> BiomeResult<()> {
        println!("👋 Hello World primal initialized!");
        self.health = Health::Healthy;
        Ok(())
    }

    async fn shutdown(&mut self) -> BiomeResult<()> {
        println!("👋 Hello World primal shutting down. Goodbye!");
        self.health = Health::Unknown {
            reason: "Service shutdown".to_string(),
            last_known: Some(Box::new(Health::Healthy)),
        };
        Ok(())
    }

    async fn update_configuration(&mut self, _config: serde_json::Value) -> BiomeResult<()> {
        println!("⚙️ Hello World primal configuration updated");
        Ok(())
    }

    async fn handle_request(&self, request: UniversalServiceRequest) -> UniversalServiceResponse {
        match request.method.as_str() {
            "say_hello" => {
                let name = request.payload
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("World");

                let greeting = format!("Hello, {}! 👋", name);
                
                UniversalServiceResponse {
                    request_id: request.request_id,
                    status: biomeos_types::primal::ResponseStatus::Success,
                    data: serde_json::json!({
                        "greeting": greeting,
                        "primal": "hello-world",
                        "timestamp": chrono::Utc::now()
                    }),
                    metadata: biomeos_types::primal::ServiceResponseMetadata {
                        processing_time_ms: 1,
                        resource_usage: HashMap::new(),
                        warnings: vec![],
                        debug_info: Some("Hello World response".to_string()),
                        custom: HashMap::new(),
                    },
                    timestamp: chrono::Utc::now(),
                    capabilities_used: vec![
                        PrimalCapability::new("communication", "greeting", "1.0.0")
                    ],
                    error: None,
                }
            }
            "get_info" => {
                UniversalServiceResponse {
                    request_id: request.request_id,
                    status: biomeos_types::primal::ResponseStatus::Success,
                    data: serde_json::json!({
                        "name": self.metadata.name,
                        "version": self.metadata.version,
                        "description": self.metadata.description,
                        "capabilities": self.capabilities.len(),
                        "health": format!("{:?}", self.health)
                    }),
                    metadata: biomeos_types::primal::ServiceResponseMetadata {
                        processing_time_ms: 2,
                        resource_usage: HashMap::new(),
                        warnings: vec![],
                        debug_info: None,
                        custom: HashMap::new(),
                    },
                    timestamp: chrono::Utc::now(),
                    capabilities_used: vec![],
                    error: None,
                }
            }
            _ => {
                UniversalServiceResponse {
                    request_id: request.request_id,
                    status: biomeos_types::primal::ResponseStatus::Error,
                    data: serde_json::Value::Null,
                    metadata: biomeos_types::primal::ServiceResponseMetadata {
                        processing_time_ms: 1,
                        resource_usage: HashMap::new(),
                        warnings: vec![],
                        debug_info: Some(format!("Unsupported method: {}", request.method)),
                        custom: HashMap::new(),
                    },
                    timestamp: chrono::Utc::now(),
                    capabilities_used: vec![],
                    error: Some(BiomeError::config_error(
                        format!("Method '{}' not supported", request.method),
                        Some("unsupported_method")
                    )),
                }
            }
        }
    }

    async fn health_check(&self) -> BiomeResult<Health> {
        Ok(self.health.clone())
    }

    async fn health_report(&self) -> BiomeResult<HealthReport> {
        Ok(HealthReport {
            id: uuid::Uuid::new_v4(),
            subject: biomeos_types::HealthSubject {
                id: self.metadata.id.clone(),
                subject_type: biomeos_types::HealthSubjectType::Service,
                name: self.metadata.name.clone(),
                version: self.metadata.version.clone(),
            },
            health: self.health.clone(),
            metrics: biomeos_types::health::HealthMetrics {
                response_time: Some(biomeos_types::ResponseTimeMetrics {
                    average_ms: 1.5,
                    p95_ms: 3.0,
                    p99_ms: 5.0,
                }),
                resources: None,
                errors: None,
                availability: None,
                custom: HashMap::new(),
            },
            history: vec![],
            components: HashMap::new(),
            generated_at: chrono::Utc::now(),
            next_check_at: Some(chrono::Utc::now() + chrono::Duration::minutes(5)),
        })
    }

    async fn resource_metrics(&self) -> BiomeResult<ResourceMetrics> {
        Ok(ResourceMetrics {
            cpu_usage: Some(0.1), // Very lightweight
            memory_usage: Some(2.5),
            disk_usage: Some(0.0),
            network_io: Some(NetworkIoMetrics {
                bytes_in_per_sec: 50.0,
                bytes_out_per_sec: 100.0,
                packets_in_per_sec: 1.0,
                packets_out_per_sec: 1.0,
            }),
        })
    }

    fn get_registration(&self) -> UniversalServiceRegistration {
        UniversalServiceRegistration {
            metadata: self.metadata.clone(),
            capabilities: self.capabilities.clone(),
            endpoints: vec![],
            health_check: biomeos_types::primal::HealthCheckConfig {
                path: "/health".to_string(),
                interval_secs: 30,
                timeout_secs: 5,
                healthy_threshold: 1,
                unhealthy_threshold: 3,
            },
            constraints: biomeos_types::primal::ServiceConstraints {
                min_resources: ResourceRequirements::default(),
                max_resources: Some(ResourceRequirements {
                    cpu_cores: Some(0.1),
                    memory_mb: Some(50),
                    disk_gb: Some(0.1),
                    network_mbps: Some(1.0),
                }),
                network: biomeos_types::primal::NetworkRequirements {
                    required_ports: vec![],
                    security: biomeos_types::primal::NetworkSecurity {
                        tls_required: false,
                        allowed_origins: vec!["*".to_string()],
                        rate_limiting: HashMap::new(),
                    },
                    bandwidth_requirements: HashMap::new(),
                },
                security: biomeos_types::primal::SecurityRequirements {
                    authentication_required: false,
                    authorization_scopes: vec![],
                    encryption: biomeos_types::primal::EncryptionRequirements {
                        at_rest: false,
                        in_transit: false,
                        key_management: None,
                    },
                    compliance_requirements: vec![],
                },
                deployment_constraints: HashMap::new(),
            },
            registered_at: chrono::Utc::now(),
        }
    }

    async fn register_with_ecosystem(&self, _discovery_endpoint: &str) -> BiomeResult<()> {
        println!("📡 Hello World primal registered with ecosystem");
        Ok(())
    }

    async fn notify_status_change(&self, status: ServiceStatus) -> BiomeResult<()> {
        println!("📢 Hello World status changed to: {:?}", status);
        Ok(())
    }

    fn get_dynamic_config(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "greeting_format": "Hello, {name}! 👋",
            "supported_languages": ["en"],
            "max_name_length": 100
        }))
    }

    async fn validate_config_change(&self, config: &serde_json::Value) -> BiomeResult<ConfigValidationResult> {
        let mut result = ConfigValidationResult {
            valid: true,
            errors: vec![],
            warnings: vec![],
            suggestions: vec![],
        };

        if let Some(greeting_format) = config.get("greeting_format") {
            if !greeting_format.is_string() {
                result.valid = false;
                result.errors.push("greeting_format must be a string".to_string());
            }
        }

        if let Some(max_name_length) = config.get("max_name_length") {
            if let Some(length) = max_name_length.as_u64() {
                if length > 1000 {
                    result.warnings.push("max_name_length > 1000 may cause performance issues".to_string());
                }
            } else {
                result.errors.push("max_name_length must be a number".to_string());
                result.valid = false;
            }
        }

        Ok(result)
    }
}
