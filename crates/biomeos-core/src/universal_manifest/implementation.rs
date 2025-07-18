//! Universal Biome Manifest Implementation
//!
//! This module provides the implementation methods for the UniversalBiomeManifest.

use super::core::{UniversalBiomeManifest, ResourceSummary};
use super::validation::ValidationError;
use crate::{BiomeError, BiomeResult};
use std::collections::HashMap;

impl UniversalBiomeManifest {
    /// Create manifest from YAML
    pub fn from_yaml(yaml: &str) -> BiomeResult<Self> {
        serde_yaml::from_str(yaml).map_err(|e| BiomeError::Generic(format!("YAML parse error: {}", e)))
    }

    /// Create manifest from JSON
    pub fn from_json(json: &str) -> BiomeResult<Self> {
        serde_json::from_str(json).map_err(|e| BiomeError::Generic(format!("JSON parse error: {}", e)))
    }

    /// Convert manifest to YAML
    pub fn to_yaml(&self) -> BiomeResult<String> {
        serde_yaml::to_string(self).map_err(|e| BiomeError::Generic(format!("YAML serialize error: {}", e)))
    }

    /// Convert manifest to JSON
    pub fn to_json(&self) -> BiomeResult<String> {
        serde_json::to_string_pretty(self).map_err(|e| BiomeError::Generic(format!("JSON serialize error: {}", e)))
    }

    /// Validate the manifest
    pub fn validate(&self) -> BiomeResult<Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate API version
        if self.api_version.is_empty() {
            errors.push(ValidationError {
                code: "MISSING_API_VERSION".to_string(),
                message: "API version is required".to_string(),
                field: "api_version".to_string(),
                severity: super::validation::ValidationSeverity::Error,
            });
        }

        // Validate kind
        if self.kind != "Biome" {
            errors.push(ValidationError {
                code: "INVALID_KIND".to_string(),
                message: "Kind must be 'Biome'".to_string(),
                field: "kind".to_string(),
                severity: super::validation::ValidationSeverity::Error,
            });
        }

        // Validate metadata
        if self.metadata.name.is_empty() {
            errors.push(ValidationError {
                code: "MISSING_NAME".to_string(),
                message: "Biome name is required".to_string(),
                field: "metadata.name".to_string(),
                severity: super::validation::ValidationSeverity::Error,
            });
        }

        // Validate services
        if self.services.is_empty() {
            errors.push(ValidationError {
                code: "NO_SERVICES".to_string(),
                message: "At least one service is required".to_string(),
                field: "services".to_string(),
                severity: super::validation::ValidationSeverity::Warning,
            });
        }

        // Validate service names are unique
        let mut service_names = std::collections::HashSet::new();
        for service in &self.services {
            if !service_names.insert(&service.name) {
                errors.push(ValidationError {
                    code: "DUPLICATE_SERVICE_NAME".to_string(),
                    message: format!("Duplicate service name: {}", service.name),
                    field: "services".to_string(),
                    severity: super::validation::ValidationSeverity::Error,
                });
            }
        }

        Ok(errors)
    }

    /// Get all required capabilities
    pub fn get_all_required_capabilities(&self) -> Vec<String> {
        let mut capabilities = Vec::new();
        
        // Add capabilities from requirements
        for cap in &self.requirements.capabilities {
            capabilities.push(cap.capability.clone());
        }

        // Add capabilities from services
        for service in &self.services {
            // Service-specific capabilities would be added here
            // For now, we'll add basic runtime capabilities
            match service.runtime.runtime_type {
                super::services::RuntimeType::Container => {
                    capabilities.push("container_runtime".to_string());
                }
                super::services::RuntimeType::Wasm => {
                    capabilities.push("wasm_runtime".to_string());
                }
                _ => {}
            }
        }

        // Remove duplicates
        capabilities.sort();
        capabilities.dedup();
        capabilities
    }

    /// Get all optional capabilities
    pub fn get_all_optional_capabilities(&self) -> Vec<String> {
        let mut capabilities = Vec::new();
        
        // Add optional capabilities from requirements
        for cap in &self.requirements.capabilities {
            if cap.optional {
                capabilities.push(cap.capability.clone());
            }
        }

        // Remove duplicates
        capabilities.sort();
        capabilities.dedup();
        capabilities
    }

    /// Get resource summary
    pub fn get_resource_summary(&self) -> ResourceSummary {
        let mut total_cpu = 0.0;
        let mut total_memory_mb = 0;
        let mut total_storage_mb = 0;
        let mut network_bandwidth_mbps = 0;
        let mut gpu_required = false;
        let mut specialized_hardware = Vec::new();

        // Sum up service resources
        for service in &self.services {
            total_cpu += service.config.limits.cpu;
            total_memory_mb += service.config.limits.memory_mb;
            total_storage_mb += service.config.limits.disk_mb;
            network_bandwidth_mbps += service.config.limits.network_mbps;
        }

        // Check for GPU requirements
        for cap in &self.requirements.capabilities {
            if cap.capability.contains("gpu") {
                gpu_required = true;
            }
            if cap.capability.contains("hardware") {
                specialized_hardware.push(cap.capability.clone());
            }
        }

        // Add base resource requirements
        if let Some(cpu_str) = &self.requirements.resources.cpu {
            if let Ok(cpu_val) = cpu_str.parse::<f64>() {
                total_cpu += cpu_val;
            }
        }
        if let Some(memory_str) = &self.requirements.resources.memory {
            if let Ok(memory_val) = memory_str.parse::<u64>() {
                total_memory_mb += memory_val;
            }
        }
        if let Some(storage_str) = &self.requirements.resources.storage {
            if let Ok(storage_val) = storage_str.parse::<u64>() {
                total_storage_mb += storage_val;
            }
        }

        ResourceSummary {
            total_cpu,
            total_memory_mb,
            total_storage_mb,
            network_bandwidth_mbps,
            gpu_required,
            specialized_hardware,
        }
    }

    /// Get service by name
    pub fn get_service(&self, name: &str) -> Option<&super::services::ServiceDefinition> {
        self.services.iter().find(|s| s.name == name)
    }

    /// Get service dependencies
    pub fn get_service_dependencies(&self, service_name: &str) -> Vec<String> {
        if let Some(service) = self.get_service(service_name) {
            service.dependencies.iter().map(|d| d.service.clone()).collect()
        } else {
            Vec::new()
        }
    }

    /// Check if biome is valid
    pub fn is_valid(&self) -> bool {
        self.validate().map(|errors| errors.is_empty()).unwrap_or(false)
    }

    /// Get biome type
    pub fn get_biome_type(&self) -> &super::core::BiomeType {
        &self.metadata.biome_type
    }

    /// Get maturity level
    pub fn get_maturity_level(&self) -> &super::core::MaturityLevel {
        &self.metadata.maturity
    }

    /// Check if biome is production ready
    pub fn is_production_ready(&self) -> bool {
        matches!(self.metadata.maturity, super::core::MaturityLevel::Stable)
    }

    /// Get deployment strategy
    pub fn get_deployment_strategy(&self) -> &str {
        match &self.deployment.strategy {
            crate::universal_manifest::deployment::DeploymentStrategy::Automatic => "Automatic",
            crate::universal_manifest::deployment::DeploymentStrategy::Manual => "Manual",
            crate::universal_manifest::deployment::DeploymentStrategy::Hybrid => "Hybrid",
            crate::universal_manifest::deployment::DeploymentStrategy::Edge => "Edge",
            crate::universal_manifest::deployment::DeploymentStrategy::Cloud => "Cloud",
            crate::universal_manifest::deployment::DeploymentStrategy::Distributed => "Distributed",
        }
    }

    /// Get required primal types
    pub fn get_required_primal_types(&self) -> Vec<String> {
        self.deployment.primal_preferences.iter().map(|p| p.primal_type.clone()).collect()
    }
} 