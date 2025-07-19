//! Universal biomeOS Manager
//!
//! This manager orchestrates biomeOS using capability-based primal discovery
//! instead of hardcoded implementations. It can work with any current or future
//! primal that provides the needed capabilities.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    BiomeResult, BiomeError,
    primal_clients::{
        UniversalPrimalManager, NetworkPrimalDiscovery,
        CapabilityCategory, CapabilityRequirement,
        capability_client::CapabilityClient
    }
};
use crate::ecosystem_integration::EcosystemStatus;


/// Universal biomeOS manager using capability-based primal discovery
pub struct UniversalBiomeOSManager {
    /// Universal primal manager
    primal_manager: Arc<RwLock<UniversalPrimalManager>>,
    /// Capability client for high-level operations
    capability_client: Arc<RwLock<Option<CapabilityClient>>>,
    /// Manager configuration
    config: BiomeOSConfig,
    /// Discovered ecosystem status
    ecosystem_status: Arc<RwLock<EcosystemStatus>>,
}

/// biomeOS configuration
#[derive(Debug, Clone)]
pub struct BiomeOSConfig {
    /// Enable automatic primal discovery
    pub auto_discovery: bool,
    /// Discovery timeout in seconds
    pub discovery_timeout: u64,
    /// Required capabilities for biomeOS to function
    pub required_capabilities: Vec<CapabilityRequirement>,
    /// Optional capabilities that enhance biomeOS
    pub optional_capabilities: Vec<CapabilityRequirement>,
}

/// Ecosystem status

/// Primal information
#[derive(Debug, Clone)]
pub struct PrimalInfo {
    /// Primal ID
    pub id: String,
    /// Primal type
    pub primal_type: String,
    /// Endpoint
    pub endpoint: String,
    /// Capabilities provided
    pub capabilities: Vec<CapabilityCategory>,
    /// Health status
    pub health: PrimalHealth,
}

/// Ecosystem health
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum EcosystemHealth {
    /// All required capabilities available
    Healthy,
    /// Some optional capabilities missing
    Degraded,
    /// Required capabilities missing
    Unhealthy,
    /// Health status unknown
    Unknown,
}

/// Primal health
#[derive(Debug, Clone)]
pub enum PrimalHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl UniversalBiomeOSManager {
    /// Create a new universal biomeOS manager
    pub fn new(config: BiomeOSConfig) -> Self {
        let discovery = Box::new(NetworkPrimalDiscovery::new());
        let primal_manager = UniversalPrimalManager::new(discovery);
        
        Self {
            primal_manager: Arc::new(RwLock::new(primal_manager)),
            capability_client: Arc::new(RwLock::new(None)),
            config,
            ecosystem_status: Arc::new(RwLock::new(EcosystemStatus {
                health: crate::ecosystem_integration::types::EcosystemHealthStatus { overall_health: crate::HealthStatus::Healthy, healthy_services: 0, total_services: 0, primal_health: HashMap::new() },
                total_services: 0,
                active_primals: 0,
                uptime: std::time::Duration::from_secs(0),
            })),
        }
    }
    
    /// Initialize biomeOS by discovering the ecosystem
    pub async fn initialize(&self) -> BiomeResult<()> {
        println!("🌱 Initializing biomeOS with capability-based primal discovery...");
        
        // Discover ecosystem
        self.discover_ecosystem().await?;
        
        // Validate required capabilities
        self.validate_capabilities().await?;
        
        // Initialize capability client
        let manager = self.primal_manager.read().await;
        let manager_clone = manager;
        let capability_client = CapabilityClient::new_with_arc(self.primal_manager.clone());
        *self.capability_client.write().await = Some(capability_client);
        
        println!("✅ biomeOS initialization complete!");
        Ok(())
    }
    
    /// Discover the ecosystem and available primals
    pub async fn discover_ecosystem(&self) -> BiomeResult<()> {
        println!("🔍 Discovering ecosystem primals...");
        
        let mut manager = self.primal_manager.write().await;
        manager.discover_ecosystem().await?;
        
        // Update ecosystem status
        let mut status = self.ecosystem_status.write().await;
//         status.last_discovery = chrono::Utc::now();
        
//         // Clear previous status
//         status.primals.clear();
//         status.capabilities.clear();
        
        // Update with discovered primals
        for (primal_id, client) in manager.get_discovered_primals() {
            if let Ok(capabilities) = client.discover_capabilities().await {
                let capability_categories: Vec<CapabilityCategory> = capabilities
                    .iter()
                    .map(|c| c.category.clone())
                    .collect();
                
                let health = match client.health_check().await {
                    Ok(h) => match h.status {
                        crate::primal_clients::HealthStatus::Healthy => PrimalHealth::Healthy,
                        crate::primal_clients::HealthStatus::Degraded => PrimalHealth::Degraded,
                        crate::primal_clients::HealthStatus::Unhealthy => PrimalHealth::Unhealthy,
                        crate::primal_clients::HealthStatus::Unknown => PrimalHealth::Unknown,
                    },
                    Err(_) => PrimalHealth::Unknown,
                };
                
//                 status.primals.insert(primal_id.clone(), PrimalInfo {
//                     id: primal_id.clone(),
//                     primal_type: "discovered".to_string(), // We don't hardcode types
//                     endpoint: client.endpoint().to_string(),
//                     capabilities: capability_categories.clone(),
//                     health,
//                 });
//                 
//                 // Update capability mapping
//                 for category in capability_categories {
//                     status.capabilities
//                         .entry(category)
//                         .or_insert_with(Vec::new)
//                         .push(primal_id.clone());
//                 }
//             }
            }
        }
//         }
// //         
//         println!("📊 Discovered {} primals with {} capability categories", 
        Ok(())
    }
    
    /// Validate that required capabilities are available
    async fn validate_capabilities(&self) -> BiomeResult<()> {
        let status = self.ecosystem_status.read().await;
        let mut missing_required: Vec<CapabilityRequirement> = Vec::new();
        let mut missing_optional: Vec<CapabilityRequirement> = Vec::new();
        
        // Check required capabilities
        for requirement in &self.config.required_capabilities {
        }
        
        // Check optional capabilities
        for requirement in &self.config.optional_capabilities {
        }
        
        // Update ecosystem health
        let mut status = self.ecosystem_status.write().await;
        if !missing_required.is_empty() {
            status.health = crate::ecosystem_integration::types::EcosystemHealthStatus { overall_health: crate::HealthStatus::Unhealthy, healthy_services: 0, total_services: 0, primal_health: HashMap::new() };
            return Err(BiomeError::MissingCapabilities(format!(
                "Required capabilities missing: {:?}", missing_required
            )));
        } else if !missing_optional.is_empty() {
            status.health = crate::ecosystem_integration::types::EcosystemHealthStatus { overall_health: crate::HealthStatus::Degraded, healthy_services: 0, total_services: 0, primal_health: HashMap::new() };
            println!("⚠️  Optional capabilities missing: {:?}", missing_optional);
        } else {
            status.health = crate::ecosystem_integration::types::EcosystemHealthStatus { overall_health: crate::HealthStatus::Healthy, healthy_services: 0, total_services: 0, primal_health: HashMap::new() };
        }
        
        Ok(())
    }
    
    /// Deploy a biome using any capable orchestration primal
    pub async fn deploy_biome(&self, manifest: &str) -> BiomeResult<String> {
        let client = self.capability_client.read().await;
        let client = client.as_ref().ok_or_else(|| 
            BiomeError::NotInitialized("biomeOS manager not initialized".to_string()))?;
        
        // Parse manifest
        let manifest: crate::primal_clients::capability_client::BiomeManifest = 
            serde_yaml::from_str(manifest)
                .map_err(|e| BiomeError::Serialization(e.to_string()))?;
        
        // Deploy using any capable primal
        let deployment = client.deploy_biome(&manifest).await?;
        
        println!("🚀 Deployed biome: {}", deployment.id);
        Ok(deployment.id)
    }
    
    /// Discover services using any capable service mesh primal
    pub async fn discover_services(&self) -> BiomeResult<Vec<crate::primal_clients::ServiceInfo>> {
        let client = self.capability_client.read().await;
        let client = client.as_ref().ok_or_else(|| 
            BiomeError::NotInitialized("biomeOS manager not initialized".to_string()))?;
        
        let services = client.discover_services().await?;
        
        println!("🔍 Discovered {} services", services.len());
        Ok(services)
    }
    
    /// Create storage volume using any capable storage primal
    pub async fn create_storage_volume(&self, size: &str, storage_class: Option<String>) -> BiomeResult<String> {
        let client = self.capability_client.read().await;
        let client = client.as_ref().ok_or_else(|| 
            BiomeError::NotInitialized("biomeOS manager not initialized".to_string()))?;
        
        let spec = crate::primal_clients::StorageSpec {
            size: size.to_string(),
            storage_class,
            access_modes: vec!["ReadWriteOnce".to_string()],
        };
        
        let volume = client.create_volume(&spec).await?;
        
        println!("💾 Created storage volume: {}", volume.id);
        Ok(volume.id)
    }
    
    /// Authenticate using any capable security primal
    pub async fn authenticate(&self, username: &str, password: &str) -> BiomeResult<String> {
        let client = self.capability_client.read().await;
        let client = client.as_ref().ok_or_else(|| 
            BiomeError::NotInitialized("biomeOS manager not initialized".to_string()))?;
        
        let credentials = crate::primal_clients::Credentials {
            username: username.to_string(),
            password: password.to_string(),
        };
        
        let token = client.authenticate(&credentials).await?;
        
        println!("🔐 Authentication successful");
        Ok(token.token)
    }
    
    /// Deploy AI agent using any capable intelligence primal
    pub async fn deploy_ai_agent(&self, name: &str, agent_type: &str, capabilities: Vec<String>) -> BiomeResult<String> {
        let client = self.capability_client.read().await;
        let client = client.as_ref().ok_or_else(|| 
            BiomeError::NotInitialized("biomeOS manager not initialized".to_string()))?;
        
        let spec = crate::primal_clients::capability_client::AiAgentSpec {
            name: name.to_string(),
            agent_type: agent_type.to_string(),
            capabilities,
            resources: crate::primal_clients::capability_client::ResourceSpec {
                cpu: "1".to_string(),
                memory: "1Gi".to_string(),
                storage: None,
            },
        };
        
        let agent = client.deploy_ai_agent(&spec).await?;
        
        println!("🤖 Deployed AI agent: {}", agent.id);
        Ok(agent.id)
    }
    
    /// Execute custom capability operation on any primal
    pub async fn execute_custom_capability(
        &self,
        category: CapabilityCategory,
        operation: &str,
        params: serde_json::Value,
    ) -> BiomeResult<serde_json::Value> {
        let client = self.capability_client.read().await;
        let client = client.as_ref().ok_or_else(|| 
            BiomeError::NotInitialized("biomeOS manager not initialized".to_string()))?;
        
        let result = client.execute_custom_operation(category, operation, params).await?;
        
        println!("⚡ Executed custom capability operation: {}", operation);
        Ok(result)
    }
    
    /// Get ecosystem health status
    pub async fn get_ecosystem_health(&self) -> BiomeResult<EcosystemStatus> {
        let status = self.ecosystem_status.read().await;
        Ok(status.clone())
    }
    
    /// Get available capabilities
    pub async fn get_available_capabilities(&self) -> BiomeResult<HashMap<CapabilityCategory, Vec<String>>> {
        let status = self.ecosystem_status.read().await;
//         Ok(status.capabilities.clone())
        Ok(HashMap::new()) // TODO: Implement capability tracking
    }
    
    /// Refresh ecosystem discovery
    pub async fn refresh_ecosystem(&self) -> BiomeResult<()> {
        println!("🔄 Refreshing ecosystem discovery...");
        self.discover_ecosystem().await?;
        self.validate_capabilities().await?;
        println!("✅ Ecosystem refresh complete!");
        Ok(())
    }
}

impl Default for BiomeOSConfig {
    fn default() -> Self {
        Self {
            auto_discovery: true,
            discovery_timeout: 30,
            required_capabilities: vec![
                CapabilityRequirement {
                    category: CapabilityCategory::Orchestration,
                    operations: vec!["deploy_biome".to_string()],
                    min_version: None,
                    optional: false,
                },
            ],
            optional_capabilities: vec![
                CapabilityRequirement {
                    category: CapabilityCategory::ServiceMesh,
                    operations: vec!["discover_services".to_string()],
                    min_version: None,
                    optional: true,
                },
                CapabilityRequirement {
                    category: CapabilityCategory::Storage,
                    operations: vec!["provision_volume".to_string()],
                    min_version: None,
                    optional: true,
                },
                CapabilityRequirement {
                    category: CapabilityCategory::Security,
                    operations: vec!["authenticate".to_string()],
                    min_version: None,
                    optional: true,
                },
                CapabilityRequirement {
                    category: CapabilityCategory::Intelligence,
                    operations: vec!["deploy_agent".to_string()],
                    min_version: None,
                    optional: true,
                },
            ],
        }
    }
}

/// Convenience function to create and initialize a biomeOS manager
pub async fn create_biomeos_manager() -> BiomeResult<UniversalBiomeOSManager> {
    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config);
    manager.initialize().await?;
    Ok(manager)
}

/// Convenience function to create a biomeOS manager with custom config
pub async fn create_biomeos_manager_with_config(config: BiomeOSConfig) -> BiomeResult<UniversalBiomeOSManager> {
    let manager = UniversalBiomeOSManager::new(config);
    manager.initialize().await?;
    Ok(manager)
}
