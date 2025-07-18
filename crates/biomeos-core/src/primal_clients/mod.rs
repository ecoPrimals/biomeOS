//! Universal Primal Client System
//!
//! This module provides capability-based primal discovery and interaction,
//! allowing biomeOS to work with any primal (current or future) that provides
//! the needed capabilities.

pub mod capability_client;
pub mod primal_discovery;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{BiomeResult, BiomeError};

/// Universal capability categories that any primal can provide
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CapabilityCategory {
    /// Compute orchestration (containers, WASM, native processes, etc.)
    Orchestration,
    /// Service discovery and networking
    ServiceMesh,
    /// Storage management and provisioning
    Storage,
    /// Security and authentication
    Security,
    /// AI and machine learning
    Intelligence,
    /// Custom capability defined by community primals
    Custom(String),
}

/// Specific capability requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRequirement {
    /// Category of capability needed
    pub category: CapabilityCategory,
    /// Specific operations required
    pub operations: Vec<String>,
    /// Minimum version required
    pub min_version: Option<String>,
    /// Whether this capability is optional
    pub optional: bool,
}

/// Capability response from a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityResponse {
    /// Category this primal provides
    pub category: CapabilityCategory,
    /// Operations this primal can perform
    pub operations: Vec<String>,
    /// Version of the capability
    pub version: String,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Universal primal client trait - works with any primal
#[async_trait]
pub trait UniversalPrimalClient: Send + Sync {
    /// Discover what capabilities this primal provides
    async fn discover_capabilities(&self) -> BiomeResult<Vec<CapabilityResponse>>;
    
    /// Execute a capability-based request
    async fn execute_capability(
        &self,
        category: CapabilityCategory,
        operation: &str,
        params: serde_json::Value,
    ) -> BiomeResult<serde_json::Value>;
    
    /// Check if this primal can fulfill a requirement
    async fn can_fulfill(&self, requirement: &CapabilityRequirement) -> bool;
    
    /// Get primal health status
    async fn health_check(&self) -> BiomeResult<PrimalHealth>;
    
    /// Get primal endpoint information
    fn endpoint(&self) -> &str;
}

/// Primal health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalHealth {
    /// Overall health status
    pub status: HealthStatus,
    /// Detailed health information
    pub details: HashMap<String, String>,
    /// Last health check timestamp
    pub last_check: chrono::DateTime<chrono::Utc>,
}

/// Health status enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Universal primal manager - finds and uses primals based on capabilities
// Clone removed - contains trait objects
pub struct UniversalPrimalManager {
    /// Discovered primals
    discovered_primals: HashMap<String, Box<dyn UniversalPrimalClient>>,
    /// Capability to primal mapping
    capability_map: HashMap<CapabilityCategory, Vec<String>>,
    /// Discovery service
    discovery: Box<dyn PrimalDiscovery>,
}

/// Primal discovery trait
#[async_trait]
pub trait PrimalDiscovery: Send + Sync {
    /// Discover primals on the network
    async fn discover_primals(&self) -> BiomeResult<Vec<DiscoveredPrimal>>;
    
    /// Create a client for a discovered primal
    async fn create_client(&self, primal: &DiscoveredPrimal) -> BiomeResult<Box<dyn UniversalPrimalClient>>;
}

/// Discovered primal information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPrimal {
    /// Primal identifier (could be anything)
    pub id: String,
    /// Primal type (could be any string)
    pub primal_type: String,
    /// Endpoint URL
    pub endpoint: String,
    /// Primal metadata
    pub metadata: HashMap<String, String>,
}

impl UniversalPrimalManager {
    /// Create a new universal primal manager
    pub fn new(discovery: Box<dyn PrimalDiscovery>) -> Self {
        Self {
            discovered_primals: HashMap::new(),
            capability_map: HashMap::new(),
            discovery,
        }
    }
    
    /// Discover and register all available primals
    pub async fn discover_ecosystem(&mut self) -> BiomeResult<()> {
        let discovered = self.discovery.discover_primals().await?;
        
        for primal in discovered {
            // Create client for this primal
            let client = self.discovery.create_client(&primal).await?;
            
            // Discover what capabilities it provides
            let capabilities = client.discover_capabilities().await?;
            
            // Register this primal for each capability it provides
            for capability in capabilities {
                self.capability_map
                    .entry(capability.category)
                    .or_insert_with(Vec::new)
                    .push(primal.id.clone());
            }
            
            // Store the client
            self.discovered_primals.insert(primal.id.clone(), client);
        }
        
        Ok(())
    }
    
    /// Get reference to discovered primals
    pub fn get_discovered_primals(&self) -> &HashMap<String, Box<dyn UniversalPrimalClient>> {
        &self.discovered_primals
    }
    
    /// Find a primal that can fulfill a capability requirement
    pub async fn find_capable_primal(&self, requirement: &CapabilityRequirement) -> BiomeResult<&dyn UniversalPrimalClient> {
        // Get primals that claim to provide this capability
        let potential_primals = self.capability_map
            .get(&requirement.category)
            .ok_or_else(|| BiomeError::NotFound(format!("No primals found for capability: {:?}", requirement.category)))?;
        
        // Find the first primal that can actually fulfill the requirement
        for primal_id in potential_primals {
            if let Some(client) = self.discovered_primals.get(primal_id) {
                if client.can_fulfill(requirement).await {
                    return Ok(client.as_ref());
                }
            }
        }
        
        Err(BiomeError::NotFound(format!("No capable primal found for requirement: {:?}", requirement)))
    }
    
    /// Execute a capability-based operation
    pub async fn execute_capability(
        &self,
        category: CapabilityCategory,
        operation: &str,
        params: serde_json::Value,
    ) -> BiomeResult<serde_json::Value> {
        let requirement = CapabilityRequirement {
            category: category.clone(),
            operations: vec![operation.to_string()],
            min_version: None,
            optional: false,
        };
        
        let client = self.find_capable_primal(&requirement).await?;
        client.execute_capability(category, operation, params).await
    }
    
    /// Deploy a biome using any capable orchestration primal
    pub async fn deploy_biome(&self, manifest: &str) -> BiomeResult<String> {
        let deployment_id = Uuid::new_v4().to_string();
        
        let params = serde_json::json!({
            "manifest": manifest,
            "deployment_id": deployment_id
        });
        
        self.execute_capability(
            CapabilityCategory::Orchestration,
            "deploy_biome",
            params,
        ).await?;
        
        Ok(deployment_id)
    }
    
    /// Discover services using any capable service mesh primal
    pub async fn discover_services(&self) -> BiomeResult<Vec<ServiceInfo>> {
        let result = self.execute_capability(
            CapabilityCategory::ServiceMesh,
            "discover_services",
            serde_json::json!({}),
        ).await?;
        
        let services: Vec<ServiceInfo> = serde_json::from_value(result)
            .map_err(|e| BiomeError::Serialization(e.to_string()))?;
        
        Ok(services)
    }
    
    /// Provision storage using any capable storage primal
    pub async fn provision_storage(&self, spec: &StorageSpec) -> BiomeResult<String> {
        let params = serde_json::to_value(spec)
            .map_err(|e| BiomeError::Serialization(e.to_string()))?;
        
        let result = self.execute_capability(
            CapabilityCategory::Storage,
            "provision_volume",
            params,
        ).await?;
        
        let volume_id: String = serde_json::from_value(result)
            .map_err(|e| BiomeError::Serialization(e.to_string()))?;
        
        Ok(volume_id)
    }
    
    /// Authenticate using any capable security primal
    pub async fn authenticate(&self, credentials: &Credentials) -> BiomeResult<AuthToken> {
        let params = serde_json::to_value(credentials)
            .map_err(|e| BiomeError::Serialization(e.to_string()))?;
        
        let result = self.execute_capability(
            CapabilityCategory::Security,
            "authenticate",
            params,
        ).await?;
        
        let token: AuthToken = serde_json::from_value(result)
            .map_err(|e| BiomeError::Serialization(e.to_string()))?;
        
        Ok(token)
    }
    
    /// Execute AI operation using any capable intelligence primal
    pub async fn execute_ai_operation(&self, operation: &str, params: serde_json::Value) -> BiomeResult<serde_json::Value> {
        self.execute_capability(
            CapabilityCategory::Intelligence,
            operation,
            params,
        ).await
    }
}

/// Service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub health: String,
}

/// Storage specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSpec {
    pub size: String,
    pub storage_class: Option<String>,
    pub access_modes: Vec<String>,
}

/// Authentication credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

/// Authentication token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

// Re-export from discovery module
pub use primal_discovery::{NetworkPrimalDiscovery, HttpPrimalClient}; 