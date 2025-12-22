//! # Unified BiomeOS Architecture Showcase
//!
//! This comprehensive demo showcases the unified BiomeOS architecture,
//! demonstrating how all the major components work together seamlessly:
//! - Unified type system (biomeos-types)  
//! - AI-first error handling
//! - Service registration and discovery
//! - Health monitoring system
//! - Configuration management
//! - Cross-primal communication

use anyhow::Result;
use biomeos_core::UniversalBiomeOSManager;
use biomeos_types::{
    BiomeOSConfig, BiomeError, BiomeResult, Health, PrimalCapability, PrimalType,
    SystemConfig, Environment, OrganizationScale, UniversalPrimalService,
    UniversalServiceRequest, UniversalServiceResponse, ServiceStatus,
};
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn, error};

/// Demo primal that implements the unified interface
#[derive(Debug, Clone)]
struct DemoUnifiedPrimal {
    id: String,
    primal_type: PrimalType,
    capabilities: Vec<PrimalCapability>,
    health_status: Health,
}

impl DemoUnifiedPrimal {
    fn new(name: &str, category: &str) -> Self {
        let primal_type = PrimalType::community(category.to_string(), name.to_string());
        
        let capabilities = match category {
            "compute" => vec![
                PrimalCapability::Computing,
                PrimalCapability::SystemManagement,
            ],
            "storage" => vec![
                PrimalCapability::Storage,
                PrimalCapability::FileSystem,
            ],
            "security" => vec![
                PrimalCapability::Authentication,
                PrimalCapability::Authorization,
            ],
            _ => vec![PrimalCapability::Custom { 
                name: "generic".to_string(), 
                description: "Generic capability".to_string() 
            }],
        };

        Self {
            id: format!("{}-{}", category, name),
            primal_type,
            capabilities,
            health_status: Health::Healthy,
        }
    }
}

#[async_trait]
impl UniversalPrimalService for DemoUnifiedPrimal {
    fn primal_id(&self) -> &str {
        &self.id
    }

    fn primal_type(&self) -> &PrimalType {
        &self.primal_type
    }

    fn metadata(&self) -> &biomeos_types::PrimalServiceMetadata {
        // For demo purposes, create minimal metadata
        // In a real implementation, this would be stored as a field
        static METADATA: once_cell::sync::Lazy<biomeos_types::PrimalServiceMetadata> = 
            once_cell::sync::Lazy::new(|| {
                biomeos_types::PrimalServiceMetadata {
                    name: "Demo Primal".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Demonstration primal for unified architecture".to_string(),
                    tags: vec!["demo".to_string(), "unified".to_string()],
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }
            });
        &METADATA
    }

    fn capabilities(&self) -> &[PrimalCapability] {
        &self.capabilities
    }

    async fn can_handle_capability(&self, capability: &PrimalCapability) -> bool {
        self.capabilities.contains(capability)
    }

    async fn get_capability_metadata(&self, capability: &str) -> Option<biomeos_types::CapabilityMetadata> {
        if self.capabilities.iter().any(|c| format!("{:?}", c).contains(capability)) {
            Some(biomeos_types::CapabilityMetadata {
                name: capability.to_string(),
                version: "1.0.0".to_string(),
                description: format!("Demo capability: {}", capability),
                parameters: vec![],
                security_requirements: vec![],
                resource_requirements: biomeos_types::ResourceRequirements::default(),
            })
        } else {
            None
        }
    }

    async fn initialize(&mut self, _config: &biomeos_types::PrimalConfiguration) -> BiomeResult<()> {
        info!("🚀 Initializing {} primal: {}", self.primal_type.category, self.primal_type.name);
        self.health_status = Health::Starting { 
            phase: biomeos_types::StartupPhase::Initializing, 
            progress: 50 
        };
        
        // Simulate initialization time
        sleep(Duration::from_millis(500)).await;
        
        self.health_status = Health::Healthy;
        info!("✅ {} primal initialized successfully", self.id);
        Ok(())
    }

    async fn shutdown(&mut self) -> BiomeResult<()> {
        info!("🔄 Shutting down {} primal", self.id);
        self.health_status = Health::Stopping { 
            phase: biomeos_types::ShutdownPhase::Stopping, 
            progress: 50 
        };
        
        sleep(Duration::from_millis(200)).await;
        info!("✅ {} primal shut down successfully", self.id);
        Ok(())
    }

    async fn update_configuration(&mut self, config: serde_json::Value) -> BiomeResult<()> {
        info!("🔧 Updating configuration for {}: {:?}", self.id, config);
        // Simulate configuration update
        Ok(())
    }

    async fn handle_request(&self, request: UniversalServiceRequest) -> UniversalServiceResponse {
        info!("📨 {} handling request: {:?}", self.id, request.operation);
        
        let response_data = match request.operation.as_str() {
            "health_check" => json!({
                "status": "healthy",
                "primal_id": self.id,
                "timestamp": chrono::Utc::now()
            }),
            "get_capabilities" => json!({
                "capabilities": self.capabilities,
                "primal_type": self.primal_type
            }),
            "process_data" => json!({
                "result": "Data processed successfully",
                "processed_by": self.id,
                "input_size": request.payload.to_string().len()
            }),
            _ => json!({
                "error": "Unknown operation",
                "supported_operations": ["health_check", "get_capabilities", "process_data"]
            }),
        };

        UniversalServiceResponse {
            request_id: request.request_id,
            status: ServiceStatus::Success,
            data: response_data,
            metadata: biomeos_types::ServiceResponseMetadata {
                processing_time_ms: 100,
                resource_usage: HashMap::new(),
                source_service: self.id.clone(),
                timestamp: chrono::Utc::now(),
            },
            error: None,
        }
    }

    async fn health_check(&self) -> BiomeResult<Health> {
        Ok(self.health_status.clone())
    }

    async fn health_report(&self) -> BiomeResult<biomeos_types::HealthReport> {
        Ok(biomeos_types::HealthReport {
            subject: biomeos_types::HealthSubject {
                id: self.id.clone(),
                name: format!("{} Primal", self.primal_type.name),
                health_type: biomeos_types::HealthSubjectType::Service,
            },
            health: self.health_status.clone(),
            components: vec![],
            metrics: Some(biomeos_types::ResourceMetrics {
                cpu_usage: 15.5,
                memory_usage: 128.0,
                disk_usage: 1024.0,
                network_io: biomeos_types::NetworkIoMetrics {
                    bytes_in: 1000,
                    bytes_out: 2000,
                    packets_in: 10,
                    packets_out: 15,
                },
            }),
            issues: vec![],
            remediation_actions: vec![],
            last_updated: chrono::Utc::now(),
        })
    }

    async fn resource_metrics(&self) -> BiomeResult<biomeos_types::ResourceMetrics> {
        Ok(biomeos_types::ResourceMetrics {
            cpu_usage: 15.5,
            memory_usage: 128.0,
            disk_usage: 1024.0,
            network_io: biomeos_types::NetworkIoMetrics {
                bytes_in: 1000,
                bytes_out: 2000,
                packets_in: 10,
                packets_out: 15,
            },
        })
    }

    fn get_registration(&self) -> biomeos_types::UniversalServiceRegistration {
        biomeos_types::UniversalServiceRegistration {
            service_id: uuid::Uuid::new_v4(),
            metadata: self.metadata().clone(),
            capabilities: self.capabilities.clone(),
            endpoints: vec![
                biomeos_types::ServiceEndpoint {
                    name: "primary".to_string(),
                    url: format!("http://{}:8080", self.id),
                    protocol: "http".to_string(),
                    health_check_path: Some("/health".to_string()),
                }
            ],
            dependencies: vec![],
            resource_requirements: biomeos_types::ResourceRequirements::default(),
            security_requirements: biomeos_types::SecurityRequirements::default(),
            registration_time: chrono::Utc::now(),
            last_heartbeat: chrono::Utc::now(),
        }
    }

    async fn register_with_ecosystem(&self, discovery_endpoint: &str) -> BiomeResult<()> {
        info!("🌐 Registering {} with ecosystem at {}", self.id, discovery_endpoint);
        // Simulate registration with ecosystem
        sleep(Duration::from_millis(200)).await;
        info!("✅ {} registered with ecosystem", self.id);
        Ok(())
    }

    async fn validate_configuration(&self, config: &serde_json::Value) -> biomeos_types::ConfigValidationResult {
        info!("🔍 Validating configuration for {}", self.id);
        
        // Simple validation logic
        if config.is_object() {
            biomeos_types::ConfigValidationResult {
                valid: true,
                errors: vec![],
                warnings: vec![],
                suggestions: vec![],
            }
        } else {
            biomeos_types::ConfigValidationResult {
                valid: false,
                errors: vec!["Configuration must be a JSON object".to_string()],
                warnings: vec![],
                suggestions: vec!["Provide a valid JSON object configuration".to_string()],
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize comprehensive logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    println!("\n🎯 BiomeOS Unified Architecture Showcase");
    println!("═════════════════════════════════════════");
    println!("Demonstrating world-class unified architecture:\n");

    // 1. Configuration System Demo
    demo_unified_configuration().await?;

    // 2. Type System Demo  
    demo_unified_types().await?;

    // 3. Error System Demo
    demo_ai_first_error_handling().await?;

    // 4. Service Interface Demo
    demo_unified_service_interface().await?;

    // 5. Health System Demo
    demo_comprehensive_health_system().await?;

    // 6. Ecosystem Integration Demo
    demo_ecosystem_integration().await?;

    println!("\n🎉 Unified Architecture Showcase Complete!");
    println!("═══════════════════════════════════════════");
    println!("\n✨ Key Achievements Demonstrated:");
    println!("• 🏗️  Unified type system - Single source of truth");
    println!("• 🤖 AI-first error handling - Smart context & retry");
    println!("• ⚡ Modern async patterns - Production-ready performance");
    println!("• 🔄 Universal service interface - One trait, all functionality");
    println!("• 💊 Comprehensive health monitoring - 8-state system");
    println!("• 🌐 Ecosystem integration - Cross-primal communication");
    println!("• 🎛️  Hierarchical configuration - Environment-aware");
    println!("• 📊 Rich metrics and monitoring - Enterprise-grade observability");

    Ok(())
}

async fn demo_unified_configuration() -> Result<()> {
    println!("1️⃣ Unified Configuration System");
    println!("────────────────────────────────");

    // Create production-grade configuration
    let config = BiomeOSConfig {
        metadata: biomeos_types::ConfigMetadata {
            version: "1.0.0".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by: "demo-system".to_string(),
            description: "Demo configuration showcasing unified system".to_string(),
            tags: vec!["demo".to_string(), "unified".to_string()],
        },
        system: SystemConfig {
            name: "BiomeOS Unified Demo".to_string(),
            environment: Environment::Development,
            organization_scale: OrganizationScale::Team,
            timeouts: biomeos_types::TimeoutConfig::default(),
            workers: biomeos_types::WorkerConfig::default(),
            temp_dir: None,
            data_dir: std::path::PathBuf::from("/tmp/biomeos-demo"),
            config_dir: std::path::PathBuf::from("/etc/biomeos-demo"),
            log_dir: std::path::PathBuf::from("/var/log/biomeos-demo"),
            limits: biomeos_types::SystemLimits::default(),
        },
        network: biomeos_types::NetworkConfig::default(),
        security: biomeos_types::SecurityConfig::default(),
        resources: biomeos_types::ResourceConfig::default(),
        discovery: biomeos_types::DiscoveryConfig::default(),
        health: biomeos_types::HealthMonitoringConfig {
            enabled: true,
            check_interval: Duration::from_secs(30),
            timeout: Duration::from_secs(10),
            retry_attempts: 3,
            detailed_reporting: true,
            metrics_collection: true,
            auto_remediation: true,
        },
        observability: biomeos_types::ObservabilityConfig::default(),
        ui: biomeos_types::UIConfig::default(),
        environments: HashMap::new(),
        features: biomeos_types::FeatureFlags::default(),
    };

    info!("✅ Created unified configuration for {} environment", config.system.environment);
    info!("   Organization scale: {:?}", config.system.organization_scale);
    info!("   Health monitoring: {}", if config.health.enabled { "enabled" } else { "disabled" });
    
    println!("   ✅ Hierarchical configuration system working");
    Ok(())
}

async fn demo_unified_types() -> Result<()> {
    println!("\n2️⃣ Unified Type System");  
    println!("──────────────────────");

    // Demonstrate unified type creation
    let compute_primal = PrimalType::community("compute".to_string(), "demo-compute".to_string());
    let storage_primal = PrimalType::community("storage".to_string(), "demo-storage".to_string());
    
    info!("✅ Created compute primal type: {} / {}", compute_primal.category, compute_primal.name);
    info!("✅ Created storage primal type: {} / {}", storage_primal.category, storage_primal.name);

    // Demonstrate capability system
    let capabilities = vec![
        PrimalCapability::Computing,
        PrimalCapability::Storage,
        PrimalCapability::Authentication,
        PrimalCapability::Custom { 
            name: "demo-capability".to_string(), 
            description: "Demonstration of extensible capability system".to_string() 
        },
    ];

    info!("✅ Created {} unified capabilities", capabilities.len());
    
    println!("   ✅ Single source of truth for all types");
    Ok(())
}

async fn demo_ai_first_error_handling() -> Result<()> {
    println!("\n3️⃣ AI-First Error Handling");
    println!("─────────────────────────────");

    // Demonstrate comprehensive error creation
    let config_error = BiomeError::configuration_error(
        "Invalid port configuration".to_string(),
        Some("http_port".to_string()),
        Some("/etc/biomeos/config.yaml".to_string()),
    );

    let network_error = BiomeError::network_error(
        "Connection timeout".to_string(), 
        Some("http://demo-service:8080".to_string()),
        Some(404),
        Some(5000),
    );

    info!("✅ Created configuration error with rich context");
    info!("✅ Created network error with retry strategies");

    // Demonstrate error handling
    match simulate_operation_with_error().await {
        Ok(result) => info!("Operation succeeded: {}", result),
        Err(err) => {
            error!("Operation failed with AI-first error handling: {}", err);
            info!("   Error contains rich context for AI assistance and automation");
        }
    }

    println!("   ✅ AI-first error system with comprehensive context");
    Ok(())
}

async fn simulate_operation_with_error() -> BiomeResult<String> {
    // Simulate an operation that might fail
    Err(BiomeError::discovery_error(
        "Service discovery timeout - Songbird unavailable".to_string(),
        Some("http://songbird:8080/discover".to_string()),
        Some(503),
        Some("network_scan".to_string()),
    ))
}

async fn demo_unified_service_interface() -> Result<()> {
    println!("\n4️⃣ Unified Service Interface");
    println!("───────────────────────────────");

    // Create demo primals using unified interface
    let mut compute_primal = DemoUnifiedPrimal::new("demo-compute", "compute");
    let mut storage_primal = DemoUnifiedPrimal::new("demo-storage", "storage");
    let security_primal = DemoUnifiedPrimal::new("demo-security", "security");

    // Initialize primals
    let config = biomeos_types::PrimalConfiguration::default();
    compute_primal.initialize(&config).await?;
    storage_primal.initialize(&config).await?;

    info!("✅ Initialized primals using unified interface");

    // Demonstrate capability querying
    let compute_can_compute = compute_primal.can_handle_capability(&PrimalCapability::Computing).await;
    let storage_can_store = storage_primal.can_handle_capability(&PrimalCapability::Storage).await;
    
    info!("✅ Compute primal can handle computing: {}", compute_can_compute);
    info!("✅ Storage primal can handle storage: {}", storage_can_store);

    // Demonstrate request handling
    let request = UniversalServiceRequest {
        request_id: uuid::Uuid::new_v4(),
        operation: "process_data".to_string(),
        payload: json!({"data": "demo payload", "size": 1024}),
        metadata: HashMap::new(),
        context: biomeos_types::ServiceRequestContext::default(),
        timestamp: chrono::Utc::now(),
    };

    let response = compute_primal.handle_request(request).await;
    info!("✅ Request processed: {:?}", response.status);

    println!("   ✅ One comprehensive interface for all primals");
    Ok(())
}

async fn demo_comprehensive_health_system() -> Result<()> {
    println!("\n5️⃣ Comprehensive Health System");
    println!("─────────────────────────────────");

    let primal = DemoUnifiedPrimal::new("demo-health", "monitoring");

    // Demonstrate health states
    let health = primal.health_check().await?;
    info!("✅ Current health status: {:?}", health);

    let health_report = primal.health_report().await?;
    info!("✅ Detailed health report generated");
    info!("   Subject: {}", health_report.subject.name);
    info!("   Health: {:?}", health_report.health);
    
    if let Some(metrics) = &health_report.metrics {
        info!("   CPU Usage: {:.1}%", metrics.cpu_usage);
        info!("   Memory Usage: {:.1} MB", metrics.memory_usage);
        info!("   Network I/O: {} bytes in, {} bytes out", 
              metrics.network_io.bytes_in, metrics.network_io.bytes_out);
    }

    let resource_metrics = primal.resource_metrics().await?;
    info!("✅ Resource metrics collected successfully");

    println!("   ✅ 8-state health system with rich metrics");
    Ok(())
}

async fn demo_ecosystem_integration() -> Result<()> {
    println!("\n6️⃣ Ecosystem Integration");
    println!("───────────────────────────");

    let primal = DemoUnifiedPrimal::new("demo-integration", "ecosystem");

    // Demonstrate service registration
    let registration = primal.get_registration();
    info!("✅ Service registration created: {}", registration.metadata.name);
    info!("   Capabilities: {} registered", registration.capabilities.len());
    info!("   Endpoints: {} configured", registration.endpoints.len());

    // Demonstrate ecosystem registration
    primal.register_with_ecosystem("http://songbird:8080/register").await?;

    // Demonstrate configuration validation
    let test_config = json!({
        "timeout": 5000,
        "retries": 3,
        "enable_monitoring": true
    });

    let validation = primal.validate_configuration(&test_config).await;
    info!("✅ Configuration validation: {}", if validation.valid { "passed" } else { "failed" });

    println!("   ✅ Cross-primal communication and registration");
    Ok(())
} 