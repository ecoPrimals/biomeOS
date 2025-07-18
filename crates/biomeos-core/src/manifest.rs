//! # toadStool Manifest System
//!
//! This module provides the manifest system for toadStool integration,
//! supporting WASM-first execution, capability-based security, and federation.

use crate::byob::{TeamWorkspace, ResourceQuota};
use crate::{BiomeError, BiomeResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// toadStool-compatible biome manifest
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ToadStoolManifest {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    pub metadata: BiomeMetadata,
    pub primals: HashMap<String, PrimalConfig>,
    pub services: Vec<ServiceConfig>,
    pub federation: Option<FederationConfig>,
    pub resources: Option<ResourceLimits>,
    pub health_checks: Option<Vec<HealthCheck>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BiomeMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub maintainer: Option<String>,
    pub labels: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PrimalConfig {
    pub enabled: bool,
    pub source: String,
    pub config: serde_yaml::Value,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServiceConfig {
    pub name: String,
    pub source: String,
    pub runtime: RuntimeType,
    pub resources: ResourceRequirements,
    pub network: Option<Vec<NetworkConfig>>,
    pub volumes: Option<Vec<VolumeMount>>,
    pub environment: Option<Vec<EnvVar>>,
    pub capabilities: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum RuntimeType {
    #[serde(rename = "wasm")]
    Wasm,
    #[serde(rename = "container")]
    Container,
    #[serde(rename = "process")]
    Process,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResourceRequirements {
    pub cpu: String,
    pub memory: String,
    pub storage: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NetworkConfig {
    pub port: u16,
    pub protocol: String,
    pub external: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VolumeMount {
    pub source: String,
    pub target: String,
    pub mode: String, // "ro", "rw"
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EnvVar {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FederationConfig {
    pub enabled: bool,
    pub trust_policy: String,
    pub allowed_peers: Vec<String>,
    pub shared_services: Vec<String>,
    pub discovery: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResourceLimits {
    pub cpu_limit: String,
    pub memory_limit: String,
    pub storage_limit: String,
    pub network_bandwidth: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HealthCheck {
    pub name: String,
    pub service: String,
    pub endpoint: String,
    pub interval: String,
    pub timeout: String,
    pub retries: u32,
}

/// Manifest generation and validation
pub struct ManifestGenerator {
    template_manager: TemplateManager,
    resource_calculator: ResourceCalculator,
    security_validator: SecurityValidator,
}

impl Default for ManifestGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ManifestGenerator {
    pub fn new() -> Self {
        Self {
            template_manager: TemplateManager::new(),
            resource_calculator: ResourceCalculator::new(),
            security_validator: SecurityValidator::new(),
        }
    }

    pub fn generate_from_byob(
        &self,
        team: &TeamWorkspace,
        niche: &NicheTemplate,
        resources: &ResourceAllocation,
    ) -> BiomeResult<ToadStoolManifest> {
        let manifest = ToadStoolManifest {
            api_version: "biomeOS/v1".to_string(),
            kind: "Biome".to_string(),
            metadata: self.generate_metadata(team, niche)?,
            primals: self.generate_primal_config(team, niche)?,
            services: self.generate_services(niche, resources)?,
            federation: self.generate_federation_config(team)?,
            resources: Some(self.calculate_resource_limits(resources)?),
            health_checks: Some(self.generate_health_checks(niche)?),
        };

        self.validate_manifest(&manifest)?;
        Ok(manifest)
    }

    fn generate_metadata(
        &self,
        team: &TeamWorkspace,
        niche: &NicheTemplate,
    ) -> BiomeResult<BiomeMetadata> {
        let mut labels = HashMap::new();
        labels.insert("team".to_string(), team.team_id.clone());
        labels.insert("niche".to_string(), niche.name.clone());
        labels.insert("runtime".to_string(), "toadstool".to_string());

        Ok(BiomeMetadata {
            name: format!(
                "{}-{}",
                team.team_id,
                niche.name.to_lowercase().replace(' ', "-")
            ),
            version: "1.0.0".to_string(),
            description: Some(niche.description.clone()),
            maintainer: Some(team.team_id.clone()),
            labels: Some(labels),
        })
    }

    fn generate_primal_config(
        &self,
        team: &TeamWorkspace,
        niche: &NicheTemplate,
    ) -> BiomeResult<HashMap<String, PrimalConfig>> {
        let mut primals = HashMap::new();

        // bearDog configuration
        primals.insert(
            "beardog".to_string(),
            PrimalConfig {
                enabled: true,
                source: "ecoprimals/beardog:v1.2.0".to_string(),
                config: serde_yaml::to_value(BearDogConfig {
                    policy_file: format!("./policies/{}.bd", team.team_id),
                    federation_mode: "team".to_string(),
                    human_forged_key: true,
                })
                .map_err(|e| BiomeError::ConfigError(e.to_string()))?,
            },
        );

        // nestGate configuration
        primals.insert(
            "nestgate".to_string(),
            PrimalConfig {
                enabled: true,
                source: "ecoprimals/nestgate:v1.1.0".to_string(),
                config: serde_yaml::to_value(NestGateConfig {
                    storage_path: format!("/data/{}", team.team_id),
                    default_pool: team.team_id.clone(),
                    encryption: "aes-256-gcm".to_string(),
                    auto_snapshot: true,
                })
                .map_err(|e| BiomeError::ConfigError(e.to_string()))?,
            },
        );

        // songBird configuration
        primals.insert(
            "songbird".to_string(),
            PrimalConfig {
                enabled: niche.requires_networking,
                source: "ecoprimals/songbird:v1.3.0".to_string(),
                config: serde_yaml::to_value(SongBirdConfig {
                    discovery_mode: "mdns".to_string(),
                    federation_port: 8080,
                    mesh_network: true,
                })
                .map_err(|e| BiomeError::ConfigError(e.to_string()))?,
            },
        );

        Ok(primals)
    }

    fn generate_services(
        &self,
        niche: &NicheTemplate,
        _resources: &ResourceAllocation,
    ) -> BiomeResult<Vec<ServiceConfig>> {
        let mut services = Vec::new();

        for service_template in &niche.services {
            let service = ServiceConfig {
                name: service_template.name.clone(),
                source: service_template.source.clone(),
                runtime: RuntimeType::Wasm, // WASM-first
                resources: ResourceRequirements {
                    cpu: service_template.cpu_request.clone(),
                    memory: service_template.memory_request.clone(),
                    storage: service_template.storage_request.clone(),
                },
                network: service_template.network_config.clone(),
                volumes: self.generate_volume_mounts(service_template)?,
                environment: service_template.environment.clone(),
                capabilities: Some(service_template.capabilities.clone()),
            };
            services.push(service);
        }

        Ok(services)
    }

    fn generate_federation_config(
        &self,
        team: &TeamWorkspace,
    ) -> BiomeResult<Option<FederationConfig>> {
        Ok(Some(FederationConfig {
            enabled: true,
            trust_policy: "beardog_verified".to_string(),
            allowed_peers: vec![format!("{}-peer", team.team_id)],
            shared_services: vec!["web-service".to_string()],
            discovery: vec!["mdns".to_string(), "dht".to_string()],
        }))
    }

    fn calculate_resource_limits(
        &self,
        resources: &ResourceAllocation,
    ) -> BiomeResult<ResourceLimits> {
        Ok(ResourceLimits {
            cpu_limit: format!("{:.1}", resources.cpu_cores),
            memory_limit: format!("{}MB", resources.memory_mb),
            storage_limit: format!("{}MB", resources.storage_mb),
            network_bandwidth: Some("100Mbps".to_string()),
        })
    }

    fn generate_health_checks(&self, niche: &NicheTemplate) -> BiomeResult<Vec<HealthCheck>> {
        let mut health_checks = Vec::new();

        for service_template in &niche.services {
            if let Some(network_config) = &service_template.network_config {
                if let Some(network) = network_config.first() {
                    health_checks.push(HealthCheck {
                        name: format!("{}-health", service_template.name),
                        service: service_template.name.clone(),
                        endpoint: format!("http://localhost:{}/health", network.port),
                        interval: "30s".to_string(),
                        timeout: "10s".to_string(),
                        retries: 3,
                    });
                }
            }
        }

        Ok(health_checks)
    }

    fn generate_volume_mounts(
        &self,
        service_template: &ServiceTemplate,
    ) -> BiomeResult<Option<Vec<VolumeMount>>> {
        let mut volumes = Vec::new();

        // Add nestGate storage mount
        volumes.push(VolumeMount {
            source: format!("nestgate://personal/{}", service_template.name),
            target: "/app/data".to_string(),
            mode: "rw".to_string(),
        });

        Ok(Some(volumes))
    }

    fn validate_manifest(&self, manifest: &ToadStoolManifest) -> BiomeResult<()> {
        // Validate API version
        if manifest.api_version != "biomeOS/v1" {
            return Err(BiomeError::ValidationError(format!(
                "Unsupported API version: {}",
                manifest.api_version
            )));
        }

        // Validate kind
        if manifest.kind != "Biome" {
            return Err(BiomeError::ValidationError(format!(
                "Invalid kind: {}",
                manifest.kind
            )));
        }

        // Validate service names are unique
        let mut service_names = std::collections::HashSet::new();
        for service in &manifest.services {
            if !service_names.insert(&service.name) {
                return Err(BiomeError::ValidationError(format!(
                    "Duplicate service name: {}",
                    service.name
                )));
            }
        }

        // Validate resource requirements
        for service in &manifest.services {
            self.validate_resource_requirements(&service.resources)?;
        }

        // Validate capabilities
        for service in &manifest.services {
            if let Some(capabilities) = &service.capabilities {
                for capability in capabilities {
                    self.security_validator.validate_capability(capability)?;
                }
            }
        }

        Ok(())
    }

    fn validate_resource_requirements(&self, resources: &ResourceRequirements) -> BiomeResult<()> {
        // Validate CPU format
        if resources.cpu.parse::<f64>().is_err() {
            return Err(BiomeError::ValidationError(
                "Invalid CPU format".to_string(),
            ));
        }

        // Validate memory format
        if !self.is_valid_memory_format(&resources.memory) {
            return Err(BiomeError::ValidationError(
                "Invalid memory format".to_string(),
            ));
        }

        Ok(())
    }

    fn is_valid_memory_format(&self, memory: &str) -> bool {
        // Check longer suffixes first to avoid partial matches
        let suffixes = ["TB", "GB", "MB", "KB", "B"];
        for suffix in &suffixes {
            if let Some(number_part) = memory.strip_suffix(suffix) {
                return number_part.parse::<u64>().is_ok();
            }
        }
        false
    }

    /// Get available templates from template manager
    pub fn get_available_templates(&self) -> Vec<String> {
        self.template_manager.list_templates()
    }

    /// Load template from template manager
    pub fn load_template(&self, template_name: &str) -> BiomeResult<ServiceTemplate> {
        self.template_manager.load_template(template_name)
    }

    /// Calculate resource requirements using resource calculator
    pub fn calculate_resources(&self, workspace: &TeamWorkspace) -> BiomeResult<ResourceCalculation> {
        self.resource_calculator.calculate_requirements(workspace)
    }

    /// Validate resource limits using resource calculator
    pub fn validate_resource_limits(&self, requirements: &ResourceCalculation, quota: &ResourceQuota) -> bool {
        self.resource_calculator.validate_against_quota(requirements, quota)
    }

    /// Get template recommendations based on resources
    pub fn get_template_recommendations(&self, available_resources: &ResourceCalculation) -> Vec<String> {
        self.template_manager.recommend_templates(available_resources)
    }
}

impl ToadStoolManifest {
    pub fn from_file<P: AsRef<Path>>(path: P) -> BiomeResult<Self> {
        let content = fs::read_to_string(path).map_err(BiomeError::IoError)?;
        let manifest: ToadStoolManifest =
            serde_yaml::from_str(&content).map_err(BiomeError::YamlError)?;
        Ok(manifest)
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> BiomeResult<()> {
        let content = serde_yaml::to_string(self).map_err(BiomeError::YamlError)?;
        fs::write(path, content).map_err(BiomeError::IoError)?;
        Ok(())
    }

    pub fn to_yaml_string(&self) -> BiomeResult<String> {
        serde_yaml::to_string(self).map_err(|e| BiomeError::ConfigError(e.to_string()))
    }
}

// Supporting configuration structures
#[derive(Debug, Serialize, Deserialize)]
struct BearDogConfig {
    policy_file: String,
    federation_mode: String,
    human_forged_key: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct NestGateConfig {
    storage_path: String,
    default_pool: String,
    encryption: String,
    auto_snapshot: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct SongBirdConfig {
    discovery_mode: String,
    federation_port: u16,
    mesh_network: bool,
}

// Template and resource structures (these would be imported from other modules)
#[derive(Debug, Clone)]
pub struct NicheTemplate {
    pub name: String,
    pub description: String,
    pub services: Vec<ServiceTemplate>,
    pub requires_networking: bool,
}

#[derive(Debug, Clone)]
pub struct ServiceTemplate {
    pub name: String,
    pub source: String,
    pub capabilities: Vec<String>,
    pub cpu_request: String,
    pub memory_request: String,
    pub storage_request: Option<String>,
    pub network_config: Option<Vec<NetworkConfig>>,
    pub environment: Option<Vec<EnvVar>>,
}

#[derive(Debug, Clone)]
pub struct ResourceAllocation {
    pub cpu_cores: f64,
    pub memory_mb: u64,
    pub storage_mb: u64,
}

#[derive(Debug, Clone)]
pub struct ResourceCalculation {
    pub cpu_cores: f64,
    pub memory_bytes: u64,
    pub storage_bytes: u64,
    pub estimated_cost: f64,
}

// Helper structures
struct TemplateManager {}
impl TemplateManager {
    fn new() -> Self {
        Self {}
    }

    fn list_templates(&self) -> Vec<String> {
        vec![
            "web-service".to_string(),
            "api-service".to_string(),
            "database".to_string(),
            "worker".to_string(),
            "frontend".to_string(),
        ]
    }

    fn load_template(&self, template_name: &str) -> BiomeResult<ServiceTemplate> {
        // Mock implementation - in real system would load from storage
        Ok(ServiceTemplate {
            name: template_name.to_string(),
            source: format!("biomeos/{}", template_name),
            cpu_request: "0.5".to_string(),
            memory_request: "512MB".to_string(),
            storage_request: Some("1GB".to_string()),
            capabilities: vec!["network.client".to_string()],
                         network_config: Some(vec![NetworkConfig {
                 port: 8080,
                 external: Some(true),
                 protocol: "http".to_string(),
             }]),
            environment: None,
        })
    }

    fn recommend_templates(&self, _available_resources: &ResourceCalculation) -> Vec<String> {
        // Mock implementation - in real system would analyze resource requirements
        vec![
            "web-service".to_string(),
            "api-service".to_string(),
        ]
    }
}

struct ResourceCalculator {}
impl ResourceCalculator {
    fn new() -> Self {
        Self {}
    }

    fn calculate_requirements(&self, workspace: &TeamWorkspace) -> BiomeResult<ResourceCalculation> {
        // Calculate based on workspace deployments
        let mut total_cpu = 0.0;
        let mut total_memory = 0;
        let mut total_storage = 0;

        for _deployment in &workspace.active_deployments {
            total_cpu += 0.5; // Default CPU per deployment
            total_memory += 512 * 1024 * 1024; // 512MB per deployment
            total_storage += 1024 * 1024 * 1024; // 1GB per deployment
        }

        Ok(ResourceCalculation {
            cpu_cores: total_cpu,
            memory_bytes: total_memory,
            storage_bytes: total_storage,
            estimated_cost: total_cpu * 0.10 + (total_memory as f64 / 1024.0 / 1024.0 / 1024.0) * 0.05,
        })
    }

    fn validate_against_quota(&self, requirements: &ResourceCalculation, quota: &ResourceQuota) -> bool {
        requirements.cpu_cores <= quota.max_cpu_cores
            && requirements.memory_bytes <= quota.max_memory_bytes
            && requirements.storage_bytes <= quota.max_storage_bytes
    }
}

struct SecurityValidator {}
impl SecurityValidator {
    fn new() -> Self {
        Self {}
    }

    fn validate_capability(&self, capability: &str) -> BiomeResult<()> {
        let valid_capabilities = [
            "network.client",
            "network.server",
            "fs.read",
            "fs.write",
            "crypto.sign",
            "crypto.verify",
            "crypto.hash",
            "all", // For primals only
        ];

        // Check if capability is in valid list or starts with valid prefix
        let is_valid = valid_capabilities.iter().any(|&valid| {
            capability == valid
                || capability.starts_with("fs.read:")
                || capability.starts_with("fs.write:")
        });

        if !is_valid {
            return Err(BiomeError::ValidationError(format!(
                "Invalid capability: {}",
                capability
            )));
        }

        Ok(())
    }


}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ResourceQuota;

    #[test]
    fn test_manifest_generation() {
        let generator = ManifestGenerator::new();

        let team = TeamWorkspace {
            created_at: chrono::Utc::now(),
            active_deployments: vec![],
            isolation_config: Default::default(),
            health_config: Default::default(),
            resource_usage: Default::default(),
            team_id: "test-team".to_string(),
            resource_quota: ResourceQuota {
                max_cpu_cores: 4.0,
                max_memory_bytes: 8 * 1024 * 1024 * 1024, // 8GB
                max_storage_bytes: 100 * 1024 * 1024 * 1024, // 100GB
                max_network_bandwidth_mbps: 1000,
                max_deployments: 10,
            },
        };

        let niche = NicheTemplate {
            name: "Web Service".to_string(),
            description: "A simple web service".to_string(),
            services: vec![ServiceTemplate {
                name: "web-service".to_string(),
                source: "ecoprimals/web-service:v1.0.0".to_string(),
                capabilities: vec!["network.client".to_string()],
                cpu_request: "0.5".to_string(),
                memory_request: "512MB".to_string(),
                storage_request: Some("1GB".to_string()),
                network_config: Some(vec![NetworkConfig {
                    port: 8080,
                    protocol: "http".to_string(),
                    external: Some(true),
                }]),
                environment: None,
            }],
            requires_networking: true,
        };

        let resources = ResourceAllocation {
            cpu_cores: 2.0,
            memory_mb: 2048,
            storage_mb: 10240,
        };

        let manifest = generator
            .generate_from_byob(&team, &niche, &resources)
            .unwrap();

        assert_eq!(manifest.api_version, "biomeOS/v1");
        assert_eq!(manifest.kind, "Biome");
        assert_eq!(manifest.metadata.name, "test-team-web-service");
        assert_eq!(manifest.services.len(), 1);
        assert_eq!(manifest.services[0].runtime, RuntimeType::Wasm);
        assert!(manifest.primals.contains_key("beardog"));
        assert!(manifest.primals.contains_key("nestgate"));
        assert!(manifest.primals.contains_key("songbird"));
    }

    #[test]
    fn test_manifest_validation() {
        let generator = ManifestGenerator::new();

        let manifest = ToadStoolManifest {
            api_version: "biomeOS/v1".to_string(),
            kind: "Biome".to_string(),
            metadata: BiomeMetadata {
                name: "test-biome".to_string(),
                version: "1.0.0".to_string(),
                description: None,
                maintainer: None,
                labels: None,
            },
            primals: HashMap::new(),
            services: vec![],
            federation: None,
            resources: None,
            health_checks: None,
        };

        assert!(generator.validate_manifest(&manifest).is_ok());
    }

    #[test]
    fn test_capability_validation() {
        let validator = SecurityValidator::new();

        assert!(validator.validate_capability("network.client").is_ok());
        assert!(validator.validate_capability("fs.read:/app/data").is_ok());
        assert!(validator.validate_capability("crypto.sign").is_ok());
        assert!(validator.validate_capability("invalid.capability").is_err());
    }

    #[test]
    fn test_resource_quota() {
        let resource_quota = ResourceQuota {
            max_cpu_cores: 4.0,
            max_memory_bytes: 8 * 1024 * 1024 * 1024, // 8GB
            max_storage_bytes: 100 * 1024 * 1024 * 1024, // 100GB
            max_network_bandwidth_mbps: 1000,
            max_deployments: 10,
        };

        assert!(resource_quota.max_cpu_cores >= 0.0);
        // Note: max_memory_bytes and max_storage_bytes are u64, so >= 0 is always true
        assert!(resource_quota.max_memory_bytes > 0);
        assert!(resource_quota.max_storage_bytes > 0);
    }

    #[test]
    fn test_memory_format_validation() {
        let generator = ManifestGenerator::new();

        // Test valid memory formats
        assert!(generator.is_valid_memory_format("512MB"));
        assert!(generator.is_valid_memory_format("1GB"));
        assert!(generator.is_valid_memory_format("256KB"));
        assert!(generator.is_valid_memory_format("1024B"));
        assert!(generator.is_valid_memory_format("2TB"));

        // Test invalid memory formats
        assert!(!generator.is_valid_memory_format("512"));
        assert!(!generator.is_valid_memory_format("512mb"));
        assert!(!generator.is_valid_memory_format("invalid"));
        assert!(!generator.is_valid_memory_format("512XB"));

        // Test edge cases
        assert!(generator.is_valid_memory_format("0B"));
        assert!(!generator.is_valid_memory_format(""));
        assert!(!generator.is_valid_memory_format("MB"));
    }
}
