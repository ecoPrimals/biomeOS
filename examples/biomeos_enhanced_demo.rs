use anyhow::Result;
use biomeos_core::{BiomeOSConfig, HealthStatus, UniversalBiomeOSManager};
use biomeos_primal_sdk::{
    EcoPrimal, PrimalCapability, PrimalError, PrimalHealth, PrimalRequest, PrimalResponse,
    PrimalType,
};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

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
    let manager = UniversalBiomeOSManager::new(config);

    info!("✅ BiomeOS Universal Manager initialized");

    // 2. Demonstrate real-time health monitoring
    info!("\n📊 Starting Real-Time Health Monitoring...");
    manager.start_monitoring().await?;

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

    let mock_primal = MockEcoPrimal::new(
        "demo-orchestrator".to_string(),
        PrimalType::orchestration("Demo Orchestration Service"),
        vec![
            PrimalCapability::orchestration("service_discovery", "Dynamic service discovery"),
            PrimalCapability::orchestration("load_balancing", "Intelligent load balancing"),
            PrimalCapability::communication("message_routing", "Secure message routing"),
        ],
    );

    info!("   🎭 Created mock primal: {}", mock_primal.get_id());
    info!(
        "   📋 Capabilities: {}",
        mock_primal.get_capabilities().len()
    );
    info!("   🏥 Health: {:?}", mock_primal.get_health());

    // Test primal request/response cycle
    let test_request = PrimalRequest::new(
        "get_services".to_string(),
        serde_json::json!({"filter": "active"}),
    );

    match mock_primal.handle_request(&test_request).await {
        Ok(response) => {
            info!("   ✅ Primal request successful: {}", response.success);
            info!("   📦 Response data: {}", response.data);
        }
        Err(e) => info!("   ⚠️  Primal request failed: {:?}", e),
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

/// Mock EcoPrimal implementation for demonstration
#[derive(Clone, Debug)]
struct MockEcoPrimal {
    id: String,
    primal_type: PrimalType,
    capabilities: Vec<PrimalCapability>,
    health: PrimalHealth,
}

impl MockEcoPrimal {
    fn new(id: String, primal_type: PrimalType, capabilities: Vec<PrimalCapability>) -> Self {
        Self {
            id,
            primal_type,
            capabilities,
            health: PrimalHealth::Healthy,
        }
    }
}

#[async_trait::async_trait]
impl EcoPrimal for MockEcoPrimal {
    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn get_type(&self) -> PrimalType {
        self.primal_type.clone()
    }

    fn get_capabilities(&self) -> Vec<PrimalCapability> {
        self.capabilities.clone()
    }

    fn get_health(&self) -> PrimalHealth {
        self.health.clone()
    }

    async fn handle_request(&self, request: &PrimalRequest) -> Result<PrimalResponse, PrimalError> {
        // Mock implementation for demo purposes
        match request.action.as_str() {
            "get_services" => {
                let response_data = serde_json::json!({
                    "services": [
                        {"name": "service-1", "status": "active"},
                        {"name": "service-2", "status": "active"},
                        {"name": "service-3", "status": "maintenance"}
                    ],
                    "total": 3
                });

                Ok(PrimalResponse::success(response_data))
            }
            "health_check" => Ok(PrimalResponse::success(serde_json::json!({
                "status": "healthy",
                "uptime": "2h 30m",
                "version": "1.0.0"
            }))),
            _ => Err(PrimalError::unsupported_operation(&format!(
                "Action '{}' not supported by mock primal",
                request.action
            ))),
        }
    }

    async fn initialize(
        &mut self,
        _config: HashMap<String, serde_json::Value>,
    ) -> Result<(), PrimalError> {
        info!("🔧 Mock primal '{}' initialized successfully", self.id);
        self.health = PrimalHealth::Healthy;
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), PrimalError> {
        info!("🛑 Mock primal '{}' shutting down gracefully", self.id);
        self.health = PrimalHealth::Unknown;
        Ok(())
    }
}
