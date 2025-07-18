//! Capability-Based Primal Client
//!
//! This client provides high-level operations based on capabilities rather than
//! specific primal implementations. It can adapt to work with any primal that
//! provides the needed capabilities.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{BiomeResult, BiomeError};
use super::{
    UniversalPrimalManager, CapabilityCategory, CapabilityRequirement,
    ServiceInfo, StorageSpec, Credentials, AuthToken
};

/// High-level capability-based operations
pub struct CapabilityClient {
    /// Universal primal manager
    manager: UniversalPrimalManager,
}

impl CapabilityClient {
    /// Create a new capability client
    pub fn new(manager: UniversalPrimalManager) -> Self {
        Self { manager }
    }
    
    /// Deploy a biome using any orchestration-capable primal
    pub async fn deploy_biome(&self, manifest: &BiomeManifest) -> BiomeResult<BiomeDeployment> {
        let manifest_str = serde_yaml::to_string(manifest)
            .map_err(|e| BiomeError::Serialization(e.to_string()))?;
        
        let deployment_id = self.manager.deploy_biome(&manifest_str).await?;
        
        Ok(BiomeDeployment {
            id: deployment_id,
            status: DeploymentStatus::Pending,
            created_at: chrono::Utc::now(),
            manifest: manifest.clone(),
        })
    }
    
    /// Scale a deployed service using any orchestration-capable primal
    pub async fn scale_service(&self, service_id: &str, replicas: u32) -> BiomeResult<()> {
        let params = serde_json::json!({
            "service_id": service_id,
            "replicas": replicas
        });
        
        self.manager.execute_capability(
            CapabilityCategory::Orchestration,
            "scale_service",
            params,
        ).await?;
        
        Ok(())
    }
    
    /// Get service logs using any orchestration-capable primal
    pub async fn get_service_logs(&self, service_id: &str, lines: Option<u32>) -> BiomeResult<Vec<LogEntry>> {
        let params = serde_json::json!({
            "service_id": service_id,
            "lines": lines
        });
        
        let result = self.manager.execute_capability(
            CapabilityCategory::Orchestration,
            "get_logs",
            params,
        ).await?;
        
        let logs: Vec<LogEntry> = serde_json::from_value(result)
            .map_err(|e| BiomeError::Serialization(e.to_string()))?;
        
        Ok(logs)
    }
    
    /// Discover services using any service-mesh-capable primal
    pub async fn discover_services(&self) -> BiomeResult<Vec<ServiceInfo>> {
        self.manager.discover_services().await
    }
    
    /// Route request to service using any service-mesh-capable primal
    pub async fn route_request(&self, request: ServiceRequest) -> BiomeResult<ServiceResponse> {
        let params = serde_json::to_value(&request)
            .map_err(|e| BiomeError::Serialization(e.to_string()))?;
        
        let result = self.manager.execute_capability(
            CapabilityCategory::ServiceMesh,
            "route_request",
            params,
        ).await?;
        
        let response: ServiceResponse = serde_json::from_value(result)
            .map_err(|e| BiomeError::Serialization(e.to_string()))?;
        
        Ok(response)
    }
    
    /// Create storage volume using any storage-capable primal
    pub async fn create_volume(&self, spec: &StorageSpec) -> BiomeResult<StorageVolume> {
        let volume_id = self.manager.provision_storage(spec).await?;
        
        Ok(StorageVolume {
            id: volume_id,
            size: spec.size.clone(),
            storage_class: spec.storage_class.clone(),
            access_modes: spec.access_modes.clone(),
            status: VolumeStatus::Provisioning,
            created_at: chrono::Utc::now(),
        })
    }
    
    /// Mount storage volume using any storage-capable primal
    pub async fn mount_volume(&self, volume_id: &str, mount_point: &str) -> BiomeResult<()> {
        let params = serde_json::json!({
            "volume_id": volume_id,
            "mount_point": mount_point
        });
        
        self.manager.execute_capability(
            CapabilityCategory::Storage,
            "mount_volume",
            params,
        ).await?;
        
        Ok(())
    }
    
    /// Backup data using any storage-capable primal
    pub async fn backup_data(&self, source: &str, destination: &str) -> BiomeResult<BackupJob> {
        let params = serde_json::json!({
            "source": source,
            "destination": destination
        });
        
        let result = self.manager.execute_capability(
            CapabilityCategory::Storage,
            "create_backup",
            params,
        ).await?;
        
        let job_id: String = serde_json::from_value(result)
            .map_err(|e| BiomeError::Serialization(e.to_string()))?;
        
        Ok(BackupJob {
            id: job_id,
            source: source.to_string(),
            destination: destination.to_string(),
            status: BackupStatus::Running,
            created_at: chrono::Utc::now(),
        })
    }
    
    /// Authenticate user using any security-capable primal
    pub async fn authenticate(&self, credentials: &Credentials) -> BiomeResult<AuthToken> {
        self.manager.authenticate(credentials).await
    }
    
    /// Create security policy using any security-capable primal
    pub async fn create_security_policy(&self, policy: &SecurityPolicy) -> BiomeResult<String> {
        let params = serde_json::to_value(policy)
            .map_err(|e| BiomeError::Serialization(e.to_string()))?;
        
        let result = self.manager.execute_capability(
            CapabilityCategory::Security,
            "create_policy",
            params,
        ).await?;
        
        let policy_id: String = serde_json::from_value(result)
            .map_err(|e| BiomeError::Serialization(e.to_string()))?;
        
        Ok(policy_id)
    }
    
    /// Encrypt data using any security-capable primal
    pub async fn encrypt_data(&self, data: &[u8], key_id: &str) -> BiomeResult<Vec<u8>> {
        let params = serde_json::json!({
            "data": base64::encode(data),
            "key_id": key_id
        });
        
        let result = self.manager.execute_capability(
            CapabilityCategory::Security,
            "encrypt_data",
            params,
        ).await?;
        
        let encrypted_data: String = serde_json::from_value(result)
            .map_err(|e| BiomeError::Serialization(e.to_string()))?;
        
        let data = base64::decode(&encrypted_data)
            .map_err(|e| BiomeError::Serialization(e.to_string()))?;
        
        Ok(data)
    }
    
    /// Execute AI operation using any intelligence-capable primal
    pub async fn execute_ai_operation(&self, operation: &AiOperation) -> BiomeResult<AiResult> {
        let params = serde_json::to_value(operation)
            .map_err(|e| BiomeError::Serialization(e.to_string()))?;
        
        let result = self.manager.execute_ai_operation(&operation.operation_type, params).await?;
        
        let ai_result: AiResult = serde_json::from_value(result)
            .map_err(|e| BiomeError::Serialization(e.to_string()))?;
        
        Ok(ai_result)
    }
    
    /// Deploy AI agent using any intelligence-capable primal
    pub async fn deploy_ai_agent(&self, agent_spec: &AiAgentSpec) -> BiomeResult<AiAgent> {
        let params = serde_json::to_value(agent_spec)
            .map_err(|e| BiomeError::Serialization(e.to_string()))?;
        
        let result = self.manager.execute_ai_operation("deploy_agent", params).await?;
        
        let agent_id: String = serde_json::from_value(result)
            .map_err(|e| BiomeError::Serialization(e.to_string()))?;
        
        Ok(AiAgent {
            id: agent_id,
            spec: agent_spec.clone(),
            status: AgentStatus::Deploying,
            created_at: chrono::Utc::now(),
        })
    }
    
    /// Execute custom capability operation
    pub async fn execute_custom_operation(
        &self,
        category: CapabilityCategory,
        operation: &str,
        params: serde_json::Value,
    ) -> BiomeResult<serde_json::Value> {
        self.manager.execute_capability(category, operation, params).await
    }
    
    /// Get ecosystem health status
    pub async fn get_ecosystem_health(&self) -> BiomeResult<EcosystemHealth> {
        let mut health = EcosystemHealth {
            overall_status: HealthStatus::Healthy,
            primal_health: HashMap::new(),
            capabilities: HashMap::new(),
            last_check: chrono::Utc::now(),
        };
        
        // Check health of all discovered primals
        for (primal_id, client) in &self.manager.discovered_primals {
            match client.health_check().await {
                Ok(primal_health) => {
                    health.primal_health.insert(primal_id.clone(), primal_health);
                },
                Err(_) => {
                    health.overall_status = HealthStatus::Degraded;
                    health.primal_health.insert(primal_id.clone(), PrimalHealth {
                        status: HealthStatus::Unhealthy,
                        details: HashMap::new(),
                        last_check: chrono::Utc::now(),
                    });
                }
            }
        }
        
        // Check capabilities availability
        for (category, primal_ids) in &self.manager.capability_map {
            let healthy_primals = primal_ids.iter()
                .filter(|id| {
                    health.primal_health.get(*id)
                        .map(|h| matches!(h.status, HealthStatus::Healthy))
                        .unwrap_or(false)
                })
                .count();
            
            health.capabilities.insert(category.clone(), healthy_primals > 0);
        }
        
        Ok(health)
    }
}

// Supporting types for capability operations

/// Biome manifest for deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeManifest {
    pub api_version: String,
    pub kind: String,
    pub metadata: ManifestMetadata,
    pub spec: BiomeSpec,
}

/// Biome manifest metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestMetadata {
    pub name: String,
    pub namespace: Option<String>,
    pub labels: HashMap<String, String>,
}

/// Biome specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeSpec {
    pub services: Vec<ServiceSpec>,
    pub storage: Option<Vec<StorageSpec>>,
    pub security: Option<SecuritySpec>,
    pub ai_agents: Option<Vec<AiAgentSpec>>,
}

/// Service specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSpec {
    pub name: String,
    pub image: String,
    pub ports: Vec<u16>,
    pub environment: HashMap<String, String>,
    pub resources: ResourceSpec,
}

/// Resource specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpec {
    pub cpu: String,
    pub memory: String,
    pub storage: Option<String>,
}

/// Security specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySpec {
    pub policies: Vec<String>,
    pub encryption: bool,
    pub authentication: bool,
}

/// AI agent specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAgentSpec {
    pub name: String,
    pub agent_type: String,
    pub capabilities: Vec<String>,
    pub resources: ResourceSpec,
}

/// Deployment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeDeployment {
    pub id: String,
    pub status: DeploymentStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub manifest: BiomeManifest,
}

/// Deployment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// Log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: String,
    pub message: String,
    pub service: String,
}

/// Service request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

/// Service response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

/// Storage volume
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageVolume {
    pub id: String,
    pub size: String,
    pub storage_class: Option<String>,
    pub access_modes: Vec<String>,
    pub status: VolumeStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Volume status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolumeStatus {
    Provisioning,
    Available,
    Bound,
    Released,
    Failed,
}

/// Backup job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupJob {
    pub id: String,
    pub source: String,
    pub destination: String,
    pub status: BackupStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Backup status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupStatus {
    Running,
    Completed,
    Failed,
}

/// Security policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub name: String,
    pub rules: Vec<SecurityRule>,
}

/// Security rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRule {
    pub action: String,
    pub resource: String,
    pub conditions: HashMap<String, String>,
}

/// AI operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiOperation {
    pub operation_type: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// AI result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResult {
    pub result_type: String,
    pub data: serde_json::Value,
}

/// AI agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAgent {
    pub id: String,
    pub spec: AiAgentSpec,
    pub status: AgentStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Agent status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Deploying,
    Running,
    Stopped,
    Failed,
}

/// Ecosystem health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemHealth {
    pub overall_status: HealthStatus,
    pub primal_health: HashMap<String, PrimalHealth>,
    pub capabilities: HashMap<CapabilityCategory, bool>,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Primal health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalHealth {
    pub status: HealthStatus,
    pub details: HashMap<String, String>,
    pub last_check: chrono::DateTime<chrono::Utc>,
} 