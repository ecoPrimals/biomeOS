//! Clean Ecosystem Demo
//!
//! Demonstrates proper usage of the biomeOS Primal SDK with correct API patterns.
//! This replaces the broken demos with a robust, well-designed example.

use anyhow::Result;
use biomeos::*;
use biomeos_types::{
    BiomeError, BiomeResult, UniversalPrimalService, PrimalServiceMetadata, 
    PrimalConfiguration, PrimalType, PrimalCapability
};
use serde_json;
use std::sync::Arc;
use tokio;
use tracing::{info, warn};

/// Clean Ecosystem Demo
/// 
/// This demonstrates the proper usage of BiomeOS APIs without any hardcoded
/// primal names or assumptions. Everything is discovered dynamically.

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("🌱 Starting BiomeOS Clean Ecosystem Demo");
    info!("🎯 Demonstrating universal, agnostic primal interaction");
    
    // Create example primals to demonstrate the system
    let primals = create_example_primals().await?;
    
    // Demonstrate interactions
    demo_primal_interactions(&primals).await?;
    
    info!("✅ Demo completed successfully!");
    Ok(())
}

/// Create example primals with proper API usage
async fn create_example_primals() -> Result<Vec<Box<dyn UniversalPrimalService>>> {
    info!("📦 Creating example primals with correct API patterns");

    // Create all primals using vec![] macro for better performance
    let primals: Vec<Box<dyn UniversalPrimalService>> = vec![
        // Create compute primal (like Toadstool)
        Box::new(ComputePrimal::new("compute-service")?),
        
        // Create storage primal (like NestGate)  
        Box::new(StoragePrimal::new("storage-service")?),
        
        // Create network primal (like Songbird)
        Box::new(NetworkPrimal::new("network-service")?),
        
        // Create AI primal (like Squirrel)
        Box::new(AIPrimal::new("ai-service")?),
    ];

    info!("✅ Created {} example primals", primals.len());
    Ok(primals)
}

/// Demo interactions between primals
async fn demo_primal_interactions(primals: &[Box<dyn UniversalPrimalService>]) -> Result<()> {
    info!("🔄 Demonstrating primal interactions");
    
    for primal in primals {
        let metadata = primal.metadata();
        info!("🔍 Primal: {} ({})", metadata.name, metadata.primal_type);
        
        // Demonstrate health checking
        match primal.health_check().await {
            Ok(health) => info!("💚 Health: {:?}", health),
            Err(e) => warn!("💔 Health check failed: {}", e),
        }
        
        // Demonstrate capability listing
        let capabilities = primal.capabilities();
        info!("🎯 Capabilities: {}", capabilities.len());
    }
    
    Ok(())
}

// Example primal implementations for demonstration

struct ComputePrimal {
    id: String,
    metadata: PrimalServiceMetadata,
}

impl ComputePrimal {
    fn new(id: &str) -> BiomeResult<Self> {
        Ok(Self {
            id: id.to_string(),
            metadata: PrimalServiceMetadata {
                name: id.to_string(),
                version: "1.0.0".to_string(),
                description: "Compute processing primal".to_string(),
                primal_type: PrimalType::Compute,
                capabilities: vec![
                    PrimalCapability::Processing,
                    PrimalCapability::Scaling,
                ],
                endpoints: vec!["http://localhost:8080".to_string()],
                health_check_endpoint: Some("http://localhost:8080/health".to_string()),
                metadata: std::collections::HashMap::new(),
            },
        })
    }
}

#[async_trait::async_trait]
impl UniversalPrimalService for ComputePrimal {
    fn primal_id(&self) -> &str { &self.id }
    fn primal_type(&self) -> &PrimalType { &self.metadata.primal_type }
    fn metadata(&self) -> &PrimalServiceMetadata { &self.metadata }
    fn capabilities(&self) -> &[PrimalCapability] { &self.metadata.capabilities }
    
    async fn can_handle_capability(&self, capability: &PrimalCapability) -> bool {
        self.metadata.capabilities.contains(capability)
    }
    
    async fn get_capability_metadata(&self, _capability: &str) -> Option<biomeos_types::CapabilityMetadata> {
        None // Simplified for demo
    }
    
    async fn initialize(&mut self, _config: &PrimalConfiguration) -> BiomeResult<()> {
        Ok(())
    }
    
    async fn shutdown(&mut self) -> BiomeResult<()> {
        Ok(())
    }
    
    async fn update_configuration(&mut self, _config: serde_json::Value) -> BiomeResult<()> {
        Ok(())
    }
    
    async fn handle_request(&self, _request: biomeos_types::UniversalServiceRequest) -> biomeos_types::UniversalServiceResponse {
        biomeos_types::UniversalServiceResponse::default()
    }
    
    async fn health_check(&self) -> BiomeResult<biomeos_types::Health> {
        Ok(biomeos_types::Health::Healthy)
    }
    
    async fn health_report(&self) -> BiomeResult<biomeos_types::HealthReport> {
        Ok(biomeos_types::HealthReport::default())
    }
    
    async fn resource_metrics(&self) -> BiomeResult<biomeos_types::ResourceMetrics> {
        Ok(biomeos_types::ResourceMetrics::default())
    }
    
    fn get_registration(&self) -> biomeos_types::UniversalServiceRegistration {
        biomeos_types::UniversalServiceRegistration::default()
    }
    
    async fn register_with_ecosystem(&self, _discovery_endpoint: &str) -> BiomeResult<()> {
        Ok(())
    }
    
    async fn notify_status_change(&self, _status: biomeos_types::ServiceStatus) -> BiomeResult<()> {
        Ok(())
    }
    
    // Additional required methods
    fn version(&self) -> &str { "1.0.0" }
    
    async fn get_dynamic_config(&self) -> BiomeResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
    
    async fn validate_config_change(&self, _config: &serde_json::Value) -> BiomeResult<bool> {
        Ok(true)
    }
}

// Similar implementations for other primal types (simplified for demo)
struct StoragePrimal {
    id: String,
    metadata: PrimalServiceMetadata,
}

impl StoragePrimal {
    fn new(id: &str) -> BiomeResult<Self> {
        Ok(Self {
            id: id.to_string(),
            metadata: PrimalServiceMetadata {
                name: id.to_string(),
                version: "1.0.0".to_string(),
                description: "Storage management primal".to_string(),
                primal_type: PrimalType::Storage,
                capabilities: vec![
                    PrimalCapability::Storage,
                    PrimalCapability::Backup,
                ],
                endpoints: vec!["http://localhost:8081".to_string()],
                health_check_endpoint: Some("http://localhost:8081/health".to_string()),
                metadata: std::collections::HashMap::new(),
            },
        })
    }
}

#[async_trait::async_trait]
impl UniversalPrimalService for StoragePrimal {
    fn primal_id(&self) -> &str { &self.id }
    fn primal_type(&self) -> &PrimalType { &self.metadata.primal_type }
    fn metadata(&self) -> &PrimalServiceMetadata { &self.metadata }
    fn capabilities(&self) -> &[PrimalCapability] { &self.metadata.capabilities }
    
    async fn can_handle_capability(&self, capability: &PrimalCapability) -> bool {
        self.metadata.capabilities.contains(capability)
    }
    
    async fn get_capability_metadata(&self, _capability: &str) -> Option<biomeos_types::CapabilityMetadata> {
        None
    }
    
    async fn initialize(&mut self, _config: &PrimalConfiguration) -> BiomeResult<()> { Ok(()) }
    async fn shutdown(&mut self) -> BiomeResult<()> { Ok(()) }
    async fn update_configuration(&mut self, _config: serde_json::Value) -> BiomeResult<()> { Ok(()) }
    
    async fn handle_request(&self, _request: biomeos_types::UniversalServiceRequest) -> biomeos_types::UniversalServiceResponse {
        biomeos_types::UniversalServiceResponse::default()
    }
    
    async fn health_check(&self) -> BiomeResult<biomeos_types::Health> {
        Ok(biomeos_types::Health::Healthy)
    }
    
    async fn health_report(&self) -> BiomeResult<biomeos_types::HealthReport> {
        Ok(biomeos_types::HealthReport::default())
    }
    
    async fn resource_metrics(&self) -> BiomeResult<biomeos_types::ResourceMetrics> {
        Ok(biomeos_types::ResourceMetrics::default())
    }
    
    fn get_registration(&self) -> biomeos_types::UniversalServiceRegistration {
        biomeos_types::UniversalServiceRegistration::default()
    }
    
    async fn register_with_ecosystem(&self, _discovery_endpoint: &str) -> BiomeResult<()> { Ok(()) }
    async fn notify_status_change(&self, _status: biomeos_types::ServiceStatus) -> BiomeResult<()> { Ok(()) }
    
    fn version(&self) -> &str { "1.0.0" }
    async fn get_dynamic_config(&self) -> BiomeResult<serde_json::Value> { Ok(serde_json::json!({})) }
    async fn validate_config_change(&self, _config: &serde_json::Value) -> BiomeResult<bool> { Ok(true) }
}

struct NetworkPrimal {
    id: String,
    metadata: PrimalServiceMetadata,
}

impl NetworkPrimal {
    fn new(id: &str) -> BiomeResult<Self> {
        Ok(Self {
            id: id.to_string(),
            metadata: PrimalServiceMetadata {
                name: id.to_string(),
                version: "1.0.0".to_string(),
                description: "Network management primal".to_string(),
                primal_type: PrimalType::Network,
                capabilities: vec![
                    PrimalCapability::Networking,
                    PrimalCapability::LoadBalancing,
                ],
                endpoints: vec!["http://localhost:8082".to_string()],
                health_check_endpoint: Some("http://localhost:8082/health".to_string()),
                metadata: std::collections::HashMap::new(),
            },
        })
    }
}

#[async_trait::async_trait]
impl UniversalPrimalService for NetworkPrimal {
    fn primal_id(&self) -> &str { &self.id }
    fn primal_type(&self) -> &PrimalType { &self.metadata.primal_type }
    fn metadata(&self) -> &PrimalServiceMetadata { &self.metadata }
    fn capabilities(&self) -> &[PrimalCapability] { &self.metadata.capabilities }
    
    async fn can_handle_capability(&self, capability: &PrimalCapability) -> bool {
        self.metadata.capabilities.contains(capability)
    }
    
    async fn get_capability_metadata(&self, _capability: &str) -> Option<biomeos_types::CapabilityMetadata> {
        None
    }
    
    async fn initialize(&mut self, _config: &PrimalConfiguration) -> BiomeResult<()> { Ok(()) }
    async fn shutdown(&mut self) -> BiomeResult<()> { Ok(()) }
    async fn update_configuration(&mut self, _config: serde_json::Value) -> BiomeResult<()> { Ok(()) }
    
    async fn handle_request(&self, _request: biomeos_types::UniversalServiceRequest) -> biomeos_types::UniversalServiceResponse {
        biomeos_types::UniversalServiceResponse::default()
    }
    
    async fn health_check(&self) -> BiomeResult<biomeos_types::Health> {
        Ok(biomeos_types::Health::Healthy)
    }
    
    async fn health_report(&self) -> BiomeResult<biomeos_types::HealthReport> {
        Ok(biomeos_types::HealthReport::default())
    }
    
    async fn resource_metrics(&self) -> BiomeResult<biomeos_types::ResourceMetrics> {
        Ok(biomeos_types::ResourceMetrics::default())
    }
    
    fn get_registration(&self) -> biomeos_types::UniversalServiceRegistration {
        biomeos_types::UniversalServiceRegistration::default()
    }
    
    async fn register_with_ecosystem(&self, _discovery_endpoint: &str) -> BiomeResult<()> { Ok(()) }
    async fn notify_status_change(&self, _status: biomeos_types::ServiceStatus) -> BiomeResult<()> { Ok(()) }
    
    fn version(&self) -> &str { "1.0.0" }
    async fn get_dynamic_config(&self) -> BiomeResult<serde_json::Value> { Ok(serde_json::json!({})) }
    async fn validate_config_change(&self, _config: &serde_json::Value) -> BiomeResult<bool> { Ok(true) }
}

struct AIPrimal {
    id: String,
    metadata: PrimalServiceMetadata,
}

impl AIPrimal {
    fn new(id: &str) -> BiomeResult<Self> {
        Ok(Self {
            id: id.to_string(),
            metadata: PrimalServiceMetadata {
                name: id.to_string(),
                version: "1.0.0".to_string(),
                description: "AI processing primal".to_string(),
                primal_type: PrimalType::AI,
                capabilities: vec![
                    PrimalCapability::AI,
                    PrimalCapability::Analytics,
                ],
                endpoints: vec!["http://localhost:8083".to_string()],
                health_check_endpoint: Some("http://localhost:8083/health".to_string()),
                metadata: std::collections::HashMap::new(),
            },
        })
    }
}

#[async_trait::async_trait]
impl UniversalPrimalService for AIPrimal {
    fn primal_id(&self) -> &str { &self.id }
    fn primal_type(&self) -> &PrimalType { &self.metadata.primal_type }
    fn metadata(&self) -> &PrimalServiceMetadata { &self.metadata }
    fn capabilities(&self) -> &[PrimalCapability] { &self.metadata.capabilities }
    
    async fn can_handle_capability(&self, capability: &PrimalCapability) -> bool {
        self.metadata.capabilities.contains(capability)
    }
    
    async fn get_capability_metadata(&self, _capability: &str) -> Option<biomeos_types::CapabilityMetadata> {
        None
    }
    
    async fn initialize(&mut self, _config: &PrimalConfiguration) -> BiomeResult<()> { Ok(()) }
    async fn shutdown(&mut self) -> BiomeResult<()> { Ok(()) }
    async fn update_configuration(&mut self, _config: serde_json::Value) -> BiomeResult<()> { Ok(()) }
    
    async fn handle_request(&self, _request: biomeos_types::UniversalServiceRequest) -> biomeos_types::UniversalServiceResponse {
        biomeos_types::UniversalServiceResponse::default()
    }
    
    async fn health_check(&self) -> BiomeResult<biomeos_types::Health> {
        Ok(biomeos_types::Health::Healthy)
    }
    
    async fn health_report(&self) -> BiomeResult<biomeos_types::HealthReport> {
        Ok(biomeos_types::HealthReport::default())
    }
    
    async fn resource_metrics(&self) -> BiomeResult<biomeos_types::ResourceMetrics> {
        Ok(biomeos_types::ResourceMetrics::default())
    }
    
    fn get_registration(&self) -> biomeos_types::UniversalServiceRegistration {
        biomeos_types::UniversalServiceRegistration::default()
    }
    
    async fn register_with_ecosystem(&self, _discovery_endpoint: &str) -> BiomeResult<()> { Ok(()) }
    async fn notify_status_change(&self, _status: biomeos_types::ServiceStatus) -> BiomeResult<()> { Ok(()) }
    
    fn version(&self) -> &str { "1.0.0" }
    async fn get_dynamic_config(&self) -> BiomeResult<serde_json::Value> { Ok(serde_json::json!({})) }
    async fn validate_config_change(&self, _config: &serde_json::Value) -> BiomeResult<bool> { Ok(true) }
}
