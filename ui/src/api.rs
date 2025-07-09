//! biomeOS API Integration Layer
//! 
//! This module provides the API abstraction layer for the biomeOS UI to communicate
//! with the core biomeOS system and ecosystem primals. Follows API-driven architecture.

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use biomeos_core::*;

use crate::state::*;

/// Main API client for biomeOS core integration
pub struct BiomeOSApi {
    /// Core biomeOS manager
    core: Arc<Mutex<Option<UniversalBiomeManager>>>,
    
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
        let manager = UniversalBiomeManager::new(config);
        
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

    /// Check if API is connected
    pub async fn is_connected(&self) -> bool {
        *self.connected.lock().await
    }

    /// Get system status
    pub async fn get_system_status(&self) -> Result<SystemStatusResponse> {
        // For now, return mock data
        // In real implementation, this would call biomeOS core APIs
        Ok(SystemStatusResponse {
            status: "online".to_string(),
            uptime: std::time::Duration::from_secs(3600),
            version: env!("CARGO_PKG_VERSION").to_string(),
            platform: self.get_platform_info().await?,
        })
    }

    /// Get platform information
    pub async fn get_platform_info(&self) -> Result<PlatformInfoResponse> {
        Ok(PlatformInfoResponse {
            os_type: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            cores: num_cpus::get() as u32,
            memory_gb: 8, // Mock data
            container_runtime: self.detect_container_runtime().await,
        })
    }

    /// Start installation process
    pub async fn start_installation(&self, mode: InstallationMode) -> Result<InstallationResponse> {
        // Initialize biomeOS if not already done
        if !self.is_connected().await {
            self.initialize().await?;
        }

        // Mock installation process
        Ok(InstallationResponse {
            installation_id: uuid::Uuid::new_v4().to_string(),
            status: "started".to_string(),
            estimated_duration: std::time::Duration::from_secs(300),
        })
    }

    /// Get installation progress
    pub async fn get_installation_progress(&self, installation_id: &str) -> Result<InstallationProgress> {
        // Mock progress data
        Ok(InstallationProgress {
            installation_id: installation_id.to_string(),
            current_step: "Platform Detection".to_string(),
            progress: 0.25,
            status: "in_progress".to_string(),
            ai_guidance: "Detecting your system capabilities...".to_string(),
            errors: Vec::new(),
        })
    }

    /// Discover available primals
    pub async fn discover_primals(&self) -> Result<PrimalDiscoveryResponse> {
        // Mock primal discovery
        Ok(PrimalDiscoveryResponse {
            discovered_primals: vec![
                PrimalInfo {
                    id: "toadstool".to_string(),
                    name: "Toadstool".to_string(),
                    description: "Universal compute runtime".to_string(),
                    version: "1.0.0".to_string(),
                    capabilities: vec!["containers".to_string(), "vms".to_string(), "native".to_string()],
                    dependencies: vec![],
                    api_endpoints: vec!["http://localhost:8080".to_string()],
                    installation_status: PrimalInstallationStatus::Installed,
                },
                PrimalInfo {
                    id: "songbird".to_string(),
                    name: "Songbird".to_string(),
                    description: "Service mesh and orchestration".to_string(),
                    version: "1.0.0".to_string(),
                    capabilities: vec!["mesh".to_string(), "discovery".to_string(), "routing".to_string()],
                    dependencies: vec![],
                    api_endpoints: vec!["http://localhost:8081".to_string()],
                    installation_status: PrimalInstallationStatus::Installed,
                },
            ],
            discovery_duration: std::time::Duration::from_secs(5),
        })
    }

    /// Validate YAML content
    pub async fn validate_yaml(&self, yaml_content: &str) -> Result<YamlValidationResponse> {
        // Mock validation
        let is_valid = !yaml_content.contains("invalid");
        
        Ok(YamlValidationResponse {
            is_valid,
            errors: if is_valid { 
                Vec::new() 
            } else { 
                vec!["Invalid YAML syntax".to_string()] 
            },
            warnings: Vec::new(),
        })
    }

    /// BYOB API Methods
    
    /// Create a new team workspace
    pub async fn create_team(&self, team_name: &str, description: &str) -> Result<TeamCreationResponse> {
        Ok(TeamCreationResponse {
            team_id: uuid::Uuid::new_v4().to_string(),
            team_name: team_name.to_string(),
            status: "created".to_string(),
            workspace_url: format!("http://localhost:8082/teams/{}", team_name),
        })
    }

    /// Deploy a biome for a team
    pub async fn deploy_biome(&self, team_id: &str, manifest_path: &str) -> Result<DeploymentResponse> {
        Ok(DeploymentResponse {
            deployment_id: uuid::Uuid::new_v4().to_string(),
            team_id: team_id.to_string(),
            status: "deploying".to_string(),
            estimated_completion: std::time::Duration::from_secs(300),
        })
    }

    /// Get team deployments
    pub async fn get_team_deployments(&self, team_id: &str) -> Result<Vec<DeploymentInfo>> {
        // Mock deployment data
        Ok(vec![
            DeploymentInfo {
                id: "dep-001".to_string(),
                name: "webapp-production".to_string(),
                team: team_id.to_string(),
                status: DeploymentStatus::Running,
                created_at: "2024-01-15 10:30:00".to_string(),
                updated_at: "2024-01-15 14:22:00".to_string(),
                services: vec![],
                resource_usage: ResourceUsage {
                    cpu_cores: 8.0,
                    memory_gb: 16.0,
                    storage_gb: 100.0,
                    network_mbps: 50.0,
                },
                health_score: 0.95,
            }
        ])
    }

    /// Get team resource usage
    pub async fn get_team_resources(&self, team_id: &str) -> Result<TeamResourceResponse> {
        Ok(TeamResourceResponse {
            team_id: team_id.to_string(),
            quota: ResourceQuota {
                max_cpu_cores: 20.0,
                max_memory_gb: 40.0,
                max_storage_gb: 200.0,
                max_deployments: 5,
                used_cpu_cores: 12.0,
                used_memory_gb: 24.0,
                used_storage_gb: 150.0,
                used_deployments: 3,
            },
            current_usage: ResourceUsage {
                cpu_cores: 12.0,
                memory_gb: 24.0,
                storage_gb: 150.0,
                network_mbps: 100.0,
            },
        })
    }

    /// ISO Creator API Methods
    
    /// Start ISO build process
    pub async fn start_iso_build(&self, config: &IsoConfig) -> Result<IsoBuildResponse> {
        Ok(IsoBuildResponse {
            build_id: uuid::Uuid::new_v4().to_string(),
            status: "started".to_string(),
            estimated_duration: std::time::Duration::from_secs(1800), // 30 minutes
            output_path: format!("/tmp/biomeos-isos/{}.iso", config.name),
        })
    }

    /// Get ISO build progress
    pub async fn get_iso_build_progress(&self, build_id: &str) -> Result<IsoBuildProgress> {
        Ok(IsoBuildProgress {
            build_id: build_id.to_string(),
            status: "building".to_string(),
            progress: 0.45,
            current_step: "Packaging components".to_string(),
            log_entries: vec![
                "Starting ISO build...".to_string(),
                "Collecting base components...".to_string(),
                "Adding niche packages...".to_string(),
                "Compressing filesystem...".to_string(),
            ],
        })
    }

    /// Get available niche packages for ISO
    pub async fn get_available_niches(&self) -> Result<Vec<NichePackageInfo>> {
        Ok(vec![
            NichePackageInfo {
                id: "gaming-tournament".to_string(),
                name: "Gaming Tournament Platform".to_string(),
                description: "Complete tournament management system".to_string(),
                version: "1.5.0".to_string(),
                size_mb: 450,
                category: "Gaming".to_string(),
                author: "Tournament Masters".to_string(),
            },
            NichePackageInfo {
                id: "ai-research".to_string(),
                name: "AI Research Laboratory".to_string(),
                description: "Machine learning research environment".to_string(),
                version: "2.1.0".to_string(),
                size_mb: 1200,
                category: "Research".to_string(),
                author: "Deep Learning Lab".to_string(),
            },
        ])
    }

    /// Niche Manager API Methods
    
    /// Create a new niche package
    pub async fn create_niche(&self, niche_yaml: &str) -> Result<NicheCreationResponse> {
        Ok(NicheCreationResponse {
            niche_id: uuid::Uuid::new_v4().to_string(),
            status: "created".to_string(),
            validation_results: vec![],
        })
    }

    /// Validate niche package
    pub async fn validate_niche(&self, niche_yaml: &str) -> Result<NicheValidationResponse> {
        let is_valid = !niche_yaml.contains("invalid");
        
        Ok(NicheValidationResponse {
            is_valid,
            errors: if is_valid { Vec::new() } else { vec!["Invalid niche syntax".to_string()] },
            warnings: vec![],
            suggestions: vec!["Consider adding resource limits".to_string()],
        })
    }

    /// Test niche package
    pub async fn test_niche(&self, niche_id: &str) -> Result<NicheTestResponse> {
        Ok(NicheTestResponse {
            test_id: uuid::Uuid::new_v4().to_string(),
            niche_id: niche_id.to_string(),
            status: "running".to_string(),
            test_results: vec![
                NicheTestResult {
                    test_name: "YAML Validation".to_string(),
                    status: "passed".to_string(),
                    message: "Niche YAML is valid".to_string(),
                    duration_ms: 45,
                },
                NicheTestResult {
                    test_name: "Resource Requirements".to_string(),
                    status: "passed".to_string(),
                    message: "Resource requirements are reasonable".to_string(),
                    duration_ms: 12,
                },
            ],
        })
    }

    /// Publish niche to marketplace
    pub async fn publish_niche(&self, niche_id: &str) -> Result<NichePublishResponse> {
        Ok(NichePublishResponse {
            publication_id: uuid::Uuid::new_v4().to_string(),
            niche_id: niche_id.to_string(),
            status: "published".to_string(),
            marketplace_url: format!("https://marketplace.biomeos.org/niches/{}", niche_id),
        })
    }

    /// Get marketplace niches
    pub async fn get_marketplace_niches(&self) -> Result<Vec<MarketplaceNicheInfo>> {
        Ok(vec![
            MarketplaceNicheInfo {
                package: NichePackageInfo {
                    id: "enterprise-crm".to_string(),
                    name: "Enterprise CRM Suite".to_string(),
                    description: "Complete customer relationship management system".to_string(),
                    version: "3.2.1".to_string(),
                    size_mb: 1500,
                    category: "Enterprise".to_string(),
                    author: "Enterprise Solutions Inc.".to_string(),
                },
                verified: true,
                featured: true,
                security_score: 9.2,
                community_rating: 4.8,
                downloads: 450,
            },
        ])
    }

    async fn detect_container_runtime(&self) -> Option<String> {
        // Try to detect Docker
        if let Ok(output) = tokio::process::Command::new("docker")
            .arg("--version")
            .output()
            .await 
        {
            if output.status.success() {
                return Some("docker".to_string());
            }
        }

        // Try to detect Podman
        if let Ok(output) = tokio::process::Command::new("podman")
            .arg("--version")
            .output()
            .await 
        {
            if output.status.success() {
                return Some("podman".to_string());
            }
        }

        None
    }

    /// Initialize biomeOS with configuration
    pub async fn initialize_biome(&self, mode: InstallationMode) -> Result<InitializationResponse> {
        // Create a basic BiomeOSConfig for initialization
        let config = biomeos_core::BiomeOSConfig::default();
        let manager = UniversalBiomeManager::new(config);
        
        {
            let mut core = self.core.lock().await;
            *core = Some(manager);
        }
        
        {
            let mut connected = self.connected.lock().await;
            *connected = true;
        }
        
        Ok(InitializationResponse {
            status: "initialized".to_string(),
            message: "BiomeOS initialized successfully".to_string(),
        })
    }
}

// API Response Types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatusResponse {
    pub status: String,
    pub uptime: std::time::Duration,
    pub version: String,
    pub platform: PlatformInfoResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfoResponse {
    pub os_type: String,
    pub architecture: String,
    pub cores: u32,
    pub memory_gb: u32,
    pub container_runtime: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationResponse {
    pub installation_id: String,
    pub status: String,
    pub estimated_duration: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationProgress {
    pub installation_id: String,
    pub current_step: String,
    pub progress: f32,
    pub status: String,
    pub ai_guidance: String,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalDiscoveryResponse {
    pub discovered_primals: Vec<PrimalInfo>,
    pub discovery_duration: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlValidationResponse {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializationResponse {
    pub status: String,
    pub message: String,
}

// BYOB API Response Types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamCreationResponse {
    pub team_id: String,
    pub team_name: String,
    pub status: String,
    pub workspace_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResponse {
    pub deployment_id: String,
    pub team_id: String,
    pub status: String,
    pub estimated_completion: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamResourceResponse {
    pub team_id: String,
    pub quota: ResourceQuota,
    pub current_usage: ResourceUsage,
}

// ISO Creator API Response Types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsoConfig {
    pub name: String,
    pub description: String,
    pub version: String,
    pub target_arch: String,
    pub included_primals: Vec<String>,
    pub included_niches: Vec<String>,
    pub compression_level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsoBuildResponse {
    pub build_id: String,
    pub status: String,
    pub estimated_duration: std::time::Duration,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsoBuildProgress {
    pub build_id: String,
    pub status: String,
    pub progress: f32,
    pub current_step: String,
    pub log_entries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NichePackageInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub size_mb: u64,
    pub category: String,
    pub author: String,
}

// Niche Manager API Response Types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheCreationResponse {
    pub niche_id: String,
    pub status: String,
    pub validation_results: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheValidationResponse {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheTestResponse {
    pub test_id: String,
    pub niche_id: String,
    pub status: String,
    pub test_results: Vec<NicheTestResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheTestResult {
    pub test_name: String,
    pub status: String,
    pub message: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NichePublishResponse {
    pub publication_id: String,
    pub niche_id: String,
    pub status: String,
    pub marketplace_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceNicheInfo {
    pub package: NichePackageInfo,
    pub verified: bool,
    pub featured: bool,
    pub security_score: f32,
    pub community_rating: f32,
    pub downloads: u64,
}

// Import types from views for consistency
use crate::views::byob::{DeploymentInfo, DeploymentStatus, ResourceUsage, ResourceQuota};

impl Default for BiomeOSApi {
    fn default() -> Self {
        Self::new()
    }
} 