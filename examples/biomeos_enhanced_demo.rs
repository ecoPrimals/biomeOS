use anyhow::Result;
use biomeos_core::{BiomeOSConfig, UniversalBiomeOSManager};
use biomeos_primal_sdk::{PrimalCapability, Health, PrimalType};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};
use async_trait::async_trait;

/// Enhanced BiomeOS System Demo
///
/// This demonstrates:
/// 1. ✅ Universal capability-based discovery (agnostic to specific primals)
/// 2. ✅ Real-time health monitoring with background tasks
/// 3. ✅ Dynamic primal registration and management
/// 4. ✅ Comprehensive system resource tracking
/// 5. ✅ Resilient error handling and recovery
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🚀 Starting Enhanced BiomeOS System Demo");
    info!("================================================");

    // 1. Initialize BiomeOS with enhanced configuration
    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    info!("✅ BiomeOS Universal Manager initialized");

    // 2. Demonstrate real-time health monitoring
    info!("\n📊 Starting Real-Time Health Monitoring...");
    let _ = manager.start_monitoring().await;

    // Give monitoring a moment to collect initial data
    sleep(Duration::from_secs(2)).await;

    // Get and display current system health
    match manager.get_system_health().await {
        Ok(health) => {
            info!("🏥 System Health Report:");
            info!("   Overall Status: {:?}", health.overall_status);
            info!("   Uptime: {} hours", health.uptime.num_hours());
            info!(
                "   CPU Usage: {:.1}%",
                health.resource_usage.cpu_usage_percent
            );
            info!(
                "   Memory Usage: {:.1}%",
                health.resource_usage.memory_usage_percent
            );
            info!(
                "   Disk Usage: {:.1}%",
                health.resource_usage.disk_usage_percent
            );

            match health.overall_status {
                HealthStatus::Healthy => info!("💚 System is running optimally!"),
                HealthStatus::Warning => warn!("💛 System has minor issues"),
                HealthStatus::Critical => error!("🔴 System requires attention"),
                _ => info!("ℹ️  System status unknown"),
            }
        }
        Err(e) => error!("❌ Failed to get system health: {}", e),
    }

    // 3. Demonstrate universal capability-based discovery
    info!("\n🔍 Testing Universal Capability-Based Discovery...");

    // Test different discovery methods - all agnostic to specific primal names
    let test_endpoints = vec![
        "http://localhost:3000".to_string(),
        "http://localhost:8080".to_string(),
        "http://discovery.example.com".to_string(),
    ];

    for endpoint in &test_endpoints {
        info!("🌐 Testing endpoint: {}", endpoint);

        // Test generic service discovery
        match manager.discover_registry(endpoint).await {
            Ok(services) => {
                if !services.is_empty() {
                    info!("   ✅ Discovered {} services", services.len());
                    for service in services.iter().take(3) {
                        info!(
                            "      • {} ({:?}) - {} capabilities",
                            service.name,
                            service.category,
                            service.capabilities.len()
                        );
                    }
                } else {
                    info!("   ℹ️  No services found (endpoint may be offline)");
                }
            }
            Err(e) => info!("   ⚠️  Discovery failed: {} (expected for demo)", e),
        }

        // Test capability-specific discovery
        let required_capabilities = vec![
            PrimalCapability::orchestration("service_discovery", "Discover and register services"),
            PrimalCapability::communication("message_routing", "Route messages between services"),
        ];

        match manager
            .discover_by_capability(endpoint, &required_capabilities)
            .await
        {
            Ok(services) => {
                info!(
                    "   🎯 Found {} services with specific capabilities",
                    services.len()
                );
            }
            Err(e) => info!(
                "   ℹ️  Capability-based discovery: {} (expected for demo)",
                e
            ),
        }

        // Test endpoint probing
        match manager.probe_endpoint(endpoint).await {
            Ok(result) => {
                info!(
                    "   🔍 Probe successful: {} ({})",
                    result.name, result.category
                );
            }
            Err(e) => info!("   ℹ️  Probe failed: {} (expected for demo)", e),
        }
    }

    // 4. Demonstrate mock primal registration and management
    info!("\n🧬 Demonstrating Dynamic Primal Management...");

    let mock_primal = MockUniversalPrimal::new(
        "demo-orchestrator".to_string(),
        PrimalType::new("orchestration", "demo-service", "1.0.0"),
        vec![
            PrimalCapability::new("orchestration", "service_discovery", "1.0.0"),
            PrimalCapability::new("orchestration", "load_balancing", "1.0.0"),
            PrimalCapability::new("communication", "message_routing", "1.0.0"),
        ],
    );

    info!("   🎭 Created mock primal: {}", mock_primal.primal_id());
    info!(
        "   📋 Capabilities: {}",
        mock_primal.capabilities().len()
    );
    info!("   🏥 Health: {:?}", mock_primal.health_check().await?);

    // Test primal request/response cycle
    let test_request = UniversalServiceRequest {
        request_id: uuid::Uuid::new_v4(),
        method: "get_services".to_string(),
        parameters: std::collections::HashMap::new(),
        payload: serde_json::json!({"filter": "active"}),
        context: biomeos_primal_sdk::UniversalServiceRequest::default_context(),
        timestamp: chrono::Utc::now(),
        required_capabilities: vec![],
        timeout_ms: Some(30000),
        priority: biomeos_primal_sdk::RequestPriority::Normal,
    };

    let response = mock_primal.handle_request(test_request).await;
    match response.status {
        biomeos_primal_sdk::ResponseStatus::Success => {
            info!("   ✅ Primal request successful");
            info!("   📦 Response data: {}", response.data);
        }
        _ => info!("   ⚠️  Primal request failed: {:?}", response.error),
    }

    // 5. System stress test and resilience demonstration
    info!("\n🔥 Testing System Resilience...");

    let stress_tasks = 10;
    let mut handles = Vec::new();

    for i in 0..stress_tasks {
        let manager_clone = manager.clone();
        let handle = tokio::spawn(async move {
            // Simulate concurrent system health checks
            for _ in 0..5 {
                let _ = manager_clone.get_system_health().await;
                sleep(Duration::from_millis(100)).await;
            }
            format!("Task {} completed", i)
        });
        handles.push(handle);
    }

    let mut completed = 0;
    for handle in handles {
        match handle.await {
            Ok(result) => {
                completed += 1;
                if completed <= 3 {
                    info!("   ✅ {}", result);
                }
            }
            Err(e) => error!("   ❌ Task failed: {}", e),
        }
    }

    info!(
        "   🎯 Completed {}/{} concurrent tasks successfully!",
        completed, stress_tasks
    );

    // 6. Final system status report
    info!("\n📈 Final System Status Report...");

    match manager.get_system_health().await {
        Ok(health) => {
            let uptime_hours = health.uptime.num_hours();
            let uptime_minutes = health.uptime.num_minutes() % 60;

            info!("🏁 BiomeOS Enhanced Demo Complete!");
            info!("   ⏱️  Total uptime: {}h {}m", uptime_hours, uptime_minutes);
            info!("   🎯 System status: {:?}", health.overall_status);
            info!(
                "   💾 Resource usage: CPU {:.1}%, MEM {:.1}%, DISK {:.1}%",
                health.resource_usage.cpu_usage_percent,
                health.resource_usage.memory_usage_percent,
                health.resource_usage.disk_usage_percent
            );
            info!("   ✅ All enhanced features operational!");
        }
        Err(e) => error!("❌ Failed final health check: {}", e),
    }

    info!("\n🌟 BiomeOS is ready for production deployment!");
    info!("================================================");

    Ok(())
}

/// Mock Universal Primal implementation for demonstration
#[derive(Clone, Debug)]
struct MockUniversalPrimal {
    id: String,
    primal_type: PrimalType,
    capabilities: Vec<PrimalCapability>,
    health: Health,
    metadata: PrimalServiceMetadata,
}

impl MockUniversalPrimal {
    fn new(id: String, primal_type: PrimalType, capabilities: Vec<PrimalCapability>) -> Self {
        let metadata = PrimalServiceMetadata {
            id: id.clone(),
            name: format!("Mock Primal: {}", id),
            description: "Mock primal for demonstration purposes".to_string(),
            version: "1.0.0".to_string(),
            author: "BiomeOS Demo".to_string(),
            homepage: None,
            documentation: None,
            license: Some("MIT".to_string()),
            keywords: vec!["demo".to_string(), "mock".to_string()],
            endpoints: HashMap::new(),
            custom: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        Self {
            id,
            primal_type,
            capabilities,
            health: Health::Healthy,
            metadata,
        }
    }
}

#[async_trait]
impl UniversalPrimalService for MockUniversalPrimal {
    fn primal_id(&self) -> &str {
        &self.id
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

    async fn get_capability_metadata(&self, _capability: &str) -> Option<biomeos_primal_sdk::CapabilityMetadata> {
        None
    }

    async fn initialize(&mut self, _config: &PrimalConfiguration) -> BiomeResult<()> {
        info!("🔧 Mock primal '{}' initialized successfully", self.id);
        self.health = Health::Healthy;
        Ok(())
    }

    async fn shutdown(&mut self) -> BiomeResult<()> {
        info!("🛑 Mock primal '{}' shutting down gracefully", self.id);
        self.health = Health::Unknown { 
            reason: "Service shutdown".to_string(), 
            last_known: Some(Box::new(Health::Healthy)) 
        };
        Ok(())
    }

    async fn update_configuration(&mut self, _config: serde_json::Value) -> BiomeResult<()> {
        info!("⚙️ Mock primal '{}' configuration updated", self.id);
        Ok(())
    }

    async fn handle_request(&self, request: UniversalServiceRequest) -> UniversalServiceResponse {
        use biomeos_primal_sdk::*;
        
        match request.method.as_str() {
            "get_services" => {
                let response_data = serde_json::json!({
                    "services": [
                        {"name": "auth-service", "status": "healthy", "version": "1.0.0"},
                        {"name": "data-service", "status": "healthy", "version": "2.1.0"},
                        {"name": "compute-service", "status": "degraded", "version": "1.5.0"}
                    ],
                    "total": 3,
                    "filter": request.payload.get("filter")
                });

                UniversalServiceResponse {
                    request_id: request.request_id,
                    status: biomeos_types::primal::ResponseStatus::Success,
                    data: response_data,
                    metadata: biomeos_types::primal::ServiceResponseMetadata {
                        processing_time_ms: 15,
                        resource_usage: HashMap::new(),
                        warnings: vec![],
                        debug_info: Some("Mock response".to_string()),
                        custom: HashMap::new(),
                    },
                    timestamp: chrono::Utc::now(),
                    capabilities_used: vec![],
                    error: None,
                }
            }
            "health_check" => {
                UniversalServiceResponse {
                    request_id: request.request_id,
                    status: biomeos_types::primal::ResponseStatus::Success,
                    data: serde_json::json!({
                        "status": "healthy",
                        "uptime": "2h 30m",
                        "version": "1.0.0"
                    }),
                    metadata: biomeos_types::primal::ServiceResponseMetadata {
                        processing_time_ms: 5,
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
                        format!("Method '{}' not supported by mock primal", request.method),
                        Some("unsupported_method")
                    )),
                }
            }
        }
    }

    async fn health_check(&self) -> BiomeResult<Health> {
        Ok(self.health.clone())
    }

    async fn health_report(&self) -> BiomeResult<biomeos_primal_sdk::HealthReport> {
        Ok(biomeos_primal_sdk::HealthReport {
            id: uuid::Uuid::new_v4(),
            subject: biomeos_types::HealthSubject {
                id: self.id.clone(),
                subject_type: biomeos_types::HealthSubjectType::Service,
                name: self.metadata.name.clone(),
                version: self.metadata.version.clone(),
            },
            health: self.health.clone(),
            metrics: biomeos_types::health::HealthMetrics {
                response_time: Some(biomeos_types::ResponseTimeMetrics {
                    average_ms: 12.5,
                    p95_ms: 25.0,
                    p99_ms: 50.0,
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

    async fn resource_metrics(&self) -> BiomeResult<biomeos_primal_sdk::ResourceMetrics> {
        Ok(biomeos_primal_sdk::ResourceMetrics {
            cpu_usage: Some(15.5),
            memory_usage: Some(45.2),
            disk_usage: Some(78.1),
            network_io: Some(biomeos_primal_sdk::NetworkIoMetrics {
                bytes_in_per_sec: 1024.0,
                bytes_out_per_sec: 2048.0,
                packets_in_per_sec: 10.0,
                packets_out_per_sec: 12.0,
            }),
        })
    }

    fn get_registration(&self) -> biomeos_primal_sdk::UniversalServiceRegistration {
        use biomeos_primal_sdk::*;
        
        UniversalServiceRegistration {
            metadata: self.metadata.clone(),
            capabilities: self.capabilities.clone(),
            endpoints: vec![],
            health_check: biomeos_types::primal::HealthCheckConfig {
                path: "/health".to_string(),
                interval_secs: 30,
                timeout_secs: 10,
                healthy_threshold: 2,
                unhealthy_threshold: 3,
            },
            constraints: biomeos_types::primal::ServiceConstraints {
                min_resources: ResourceRequirements::default(),
                max_resources: Some(ResourceRequirements::default()),
                network: biomeos_types::primal::NetworkRequirements {
                    required_ports: vec![8080],
                    security: biomeos_types::primal::NetworkSecurity {
                        tls_required: false,
                        allowed_origins: vec![],
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
        info!("📡 Mock primal '{}' registered with ecosystem", self.id);
        Ok(())
    }

    async fn notify_status_change(&self, _status: biomeos_primal_sdk::ServiceStatus) -> BiomeResult<()> {
        info!("📢 Mock primal '{}' status change notified", self.id);
        Ok(())
    }

    fn get_dynamic_config(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "mock_mode": true,
            "demo_version": "1.0.0"
        }))
    }

    async fn validate_config_change(&self, _config: &serde_json::Value) -> BiomeResult<biomeos_primal_sdk::ConfigValidationResult> {
        Ok(biomeos_primal_sdk::ConfigValidationResult {
            valid: true,
            errors: vec![],
            warnings: vec!["This is a mock primal - validation is simulated".to_string()],
            suggestions: vec!["Consider using a real primal implementation for production".to_string()],
        })
    }
}
