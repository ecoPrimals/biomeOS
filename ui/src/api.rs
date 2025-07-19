//! biomeOS API Integration Layer
//!
//! This module provides the API abstraction layer for the biomeOS UI to communicate
//! with the core biomeOS system and ecosystem primals. Follows API-driven architecture.

use anyhow::Result;
use biomeos_core::*;
use biomeos_core::byob::types::DeploymentInstance;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::*;
use crate::views::byob::types::{HealthStatus, PrimalCapability};

/// Main API client for biomeOS core integration
pub struct BiomeOSApi {
    /// Core biomeOS manager
    core: Arc<Mutex<Option<UniversalBiomeOSManager>>>,

    /// API endpoints for different services
    endpoints: HashMap<String, String>,

    /// HTTP client for external API calls
    client: reqwest::Client,

    /// Connection status
    connected: Arc<Mutex<bool>>,
}

impl BiomeOSApi {
    pub fn new() -> Self {
        let mut endpoints = HashMap::new();
        endpoints.insert("core".to_string(), "http://localhost:8080".to_string());
        endpoints.insert("metrics".to_string(), "http://localhost:8081".to_string());
        endpoints.insert("byob".to_string(), "http://localhost:8082".to_string());
        endpoints.insert("iso".to_string(), "http://localhost:8083".to_string());
        endpoints.insert("niches".to_string(), "http://localhost:8084".to_string());

        Self {
            core: Arc::new(Mutex::new(None)),
            endpoints,
            client: reqwest::Client::new(),
            connected: Arc::new(Mutex::new(false)),
        }
    }

    /// Initialize connection to biomeOS core
    pub async fn initialize(&self) -> Result<()> {
        let config = biomeos_core::BiomeOSConfig::default();
        let manager = UniversalBiomeOSManager::new(config);

        {
            let mut core = self.core.lock().await;
            *core = Some(manager);
        }

        {
            let mut connected = self.connected.lock().await;
            *connected = true;
        }

        Ok(())
    }

    /// Get connection status
    pub async fn is_connected(&self) -> bool {
        let connected = self.connected.lock().await;
        *connected
    }

    /// Get available primals
    pub async fn get_primals(&self) -> Result<Vec<APIPrimalInfo>> {
        // Return mock data for now - in real implementation this would call the universal adapter
        Ok(vec![
            APIPrimalInfo {
                id: "toadstool".to_string(),
                primal_type: "toadstool".to_string(),
                endpoint: "http://localhost:8084".to_string(),
                capabilities: vec!["container_orchestration".to_string(), "wasm_runtime".to_string()],
                health: "healthy".to_string(),
            },
            APIPrimalInfo {
                id: "songbird".to_string(),
                primal_type: "songbird".to_string(),
                endpoint: "http://localhost:8080".to_string(),
                capabilities: vec!["service_discovery".to_string(), "load_balancing".to_string()],
                health: "healthy".to_string(),
            },
        ])
    }

    /// Get system status
    pub async fn get_status(&self) -> Result<SystemStatus> {
        Ok(SystemStatus {
            overall_health: "healthy".to_string(),
            active_primals: vec![],
            resource_usage: ResourceUsage { cpu_percent: 0.0, memory_percent: 0.0, disk_percent: 0.0, network_bytes_per_sec: 0 },
            last_updated: chrono::Utc::now(),
        })
    }

    /// Install a Primal
    pub async fn install_primal(
        &self,
        primal_name: &str,
        mode: InstallationMode,
    ) -> Result<InstallationResponse> {
        let _mode = mode; // Silence unused variable warning
        
        // Mock implementation - in real system this would delegate to the universal adapter
        Ok(InstallationResponse {
            success: true,
            message: format!("Primal {} installation queued", primal_name),
            installation_id: uuid::Uuid::new_v4().to_string(),
        })
    }

    /// Get system metrics
    pub async fn get_metrics(&self) -> Result<SystemMetrics> {
        Ok(SystemMetrics {
            cpu_usage: 25.0,
            memory_usage: 45.0,
            disk_usage: 60.0,
            network_io: 1024,
            active_connections: 5,
            uptime_seconds: 3600,
        })
    }

    /// Shutdown the system
    pub async fn shutdown(&self) -> Result<()> {
        // Mock implementation
        Ok(())
    }

    /// Get logs
    pub async fn get_logs(&self) -> Result<Vec<LogEntry>> {
        Ok(vec![
            LogEntry {
                timestamp: chrono::Utc::now(),
                level: "INFO".to_string(),
                message: "System started".to_string(),
                component: "core".to_string(),
            },
        ])
    }

    /// Deploy a biome
    pub async fn deploy_biome(
        &self,
        _team_id: &str,
        _biome_name: &str,
        _description: &str,
        _manifest_path: &str,
    ) -> Result<DeploymentResponse> {
        // Mock implementation
        Ok(DeploymentResponse {
            success: true,
            message: "Biome deployment started".to_string(),
            deployment_id: uuid::Uuid::new_v4().to_string(),
        })
    }

    /// Get deployments for a team
    pub async fn get_team_deployments(&self, _team_id: &str) -> Result<Vec<DeploymentInstance>> {
        // Mock implementation
        Ok(vec![])
    }

    /// Get deployment status
    pub async fn get_deployment_status(&self, _deployment_id: &str) -> Result<DeploymentStatusInfo> {
        // Mock implementation
        Ok(DeploymentStatusInfo {
            status: "running".to_string(),
            message: "Deployment is running".to_string(),
            progress: 100.0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }

    /// Stop a deployment
    pub async fn stop_deployment(&self, _deployment_id: &str) -> Result<()> {
        // Mock implementation
        Ok(())
    }

    /// Remove a deployment
    pub async fn remove_deployment(&self, _deployment_id: &str) -> Result<()> {
        // Mock implementation
        Ok(())
    }

    /// Get ISO creation status
    pub async fn get_iso_status(&self) -> Result<ISOStatus> {
        Ok(ISOStatus {
            is_building: false,
            progress: 0.0,
            message: "Ready".to_string(),
            output_path: None,
            created_at: None,
        })
    }

    /// Create ISO
    pub async fn create_iso(&self, _config: ISOConfig) -> Result<ISOCreationResponse> {
        Ok(ISOCreationResponse {
            success: true,
            message: "ISO creation started".to_string(),
            job_id: uuid::Uuid::new_v4().to_string(),
        })
    }

    /// Get available niches
    pub async fn get_niches(&self) -> Result<Vec<NicheInfo>> {
        Ok(vec![
            NicheInfo {
                id: "ai-research".to_string(),
                name: "AI Research".to_string(),
                description: "Optimized for AI research and development".to_string(),
                category: "research".to_string(),
                template_path: "templates/ai-research.yaml".to_string(),
                requirements: vec!["gpu".to_string(), "memory_32gb".to_string()],
                features: vec!["tensorflow".to_string(), "pytorch".to_string()],
                status: "available".to_string(),
            },
        ])
    }

    /// Test a niche
    pub async fn test_niche(&self, _niche_id: &str) -> Result<NicheTestResponse> {
        Ok(NicheTestResponse {
            success: true,
            message: "Niche test completed".to_string(),
            test_results: vec![],
        })
    }

    /// Create a niche
    pub async fn create_niche(&self, _niche_yaml: &str) -> Result<NicheCreationResponse> {
        Ok(NicheCreationResponse {
            success: true,
            message: "Niche created successfully".to_string(),
            niche_id: uuid::Uuid::new_v4().to_string(),
        })
    }

    /// Get niche testing results
    pub async fn get_niche_test_results(&self, _niche_id: &str) -> Result<Vec<NicheTestResult>> {
        Ok(vec![])
    }

    /// Get niche template
    pub async fn get_niche_template(&self, _niche_id: &str) -> Result<String> {
        Ok("# Sample niche template\napiVersion: biomeOS/v1\nkind: Niche\n".to_string())
    }

    /// Update a niche
    pub async fn update_niche(&self, _niche_id: &str, _niche_yaml: &str) -> Result<()> {
        Ok(())
    }

    /// Delete a niche
    pub async fn delete_niche(&self, _niche_id: &str) -> Result<()> {
        Ok(())
    }

    /// Get marketplace info
    pub async fn get_marketplace_info(&self) -> Result<MarketplaceInfo> {
        Ok(MarketplaceInfo {
            total_niches: 12,
            featured_niches: vec!["ai-research".to_string(), "web-dev".to_string()],
            categories: vec!["research".to_string(), "development".to_string()],
            last_updated: chrono::Utc::now(),
        })
    }

    /// Get YAML editor suggestions
    pub async fn get_yaml_suggestions(&self, _content: &str) -> Result<Vec<YAMLSuggestion>> {
        Ok(vec![])
    }

    /// Validate YAML
    pub async fn validate_yaml(&self, _content: &str) -> Result<YAMLValidationResult> {
        Ok(YAMLValidationResult {
            valid: true,
            errors: vec![],
            warnings: vec![],
        })
    }

    /// Format YAML
    pub async fn format_yaml(&self, _content: &str) -> Result<String> {
        Ok("# Formatted YAML\n".to_string())
    }

    /// Discover primals using the new universal adapter architecture
    pub async fn discover_primals(&self) -> Result<PrimalDiscoveryResponse> {
        // Mock discovery - in real implementation this would use the universal adapter
        let config = biomeos_core::BiomeOSConfig::default();
        let _manager = UniversalBiomeOSManager::new(config);
        
        // Return mock discovered primals using API-compatible types
        Ok(PrimalDiscoveryResponse {
            discovered_primals: vec![
                APIPrimalInfo {
                    id: "toadstool".to_string(),
                    primal_type: "toadstool".to_string(),
                    endpoint: "http://localhost:8084".to_string(),
                    capabilities: vec!["container_orchestration".to_string(), "wasm_runtime".to_string()],
                    health: "healthy".to_string(),
                },
                APIPrimalInfo {
                    id: "songbird".to_string(),
                    primal_type: "songbird".to_string(),
                    endpoint: "http://localhost:8080".to_string(),
                    capabilities: vec!["service_discovery".to_string(), "load_balancing".to_string()],
                    health: "healthy".to_string(),
                },
            ],
        })
    }
}

// API Response Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationResponse {
    pub success: bool,
    pub message: String,
    pub installation_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_io: u64,
    pub active_connections: u32,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: String,
    pub message: String,
    pub component: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResponse {
    pub success: bool,
    pub message: String,
    pub deployment_id: String,
}

/// Deployment status information (different from enum)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentStatusInfo {
    pub status: String,
    pub message: String,
    pub progress: f64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ISOStatus {
    pub is_building: bool,
    pub progress: f64,
    pub message: String,
    pub output_path: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ISOConfig {
    pub name: String,
    pub description: String,
    pub primals: Vec<String>,
    pub custom_packages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ISOCreationResponse {
    pub success: bool,
    pub message: String,
    pub job_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub template_path: String,
    pub requirements: Vec<String>,
    pub features: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheTestResponse {
    pub success: bool,
    pub message: String,
    pub test_results: Vec<NicheTestResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheTestResult {
    pub test_name: String,
    pub passed: bool,
    pub message: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheCreationResponse {
    pub success: bool,
    pub message: String,
    pub niche_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceInfo {
    pub total_niches: u32,
    pub featured_niches: Vec<String>,
    pub categories: Vec<String>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YAMLSuggestion {
    pub text: String,
    pub description: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YAMLValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// API version of PrimalInfo that can be serialized
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIPrimalInfo {
    pub id: String,
    pub primal_type: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub health: String, // Using string instead of enum for API compatibility
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalDiscoveryResponse {
    pub discovered_primals: Vec<APIPrimalInfo>,
}
