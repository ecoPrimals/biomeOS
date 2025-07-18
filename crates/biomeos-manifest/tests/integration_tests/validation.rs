//! Validation utilities for integration tests
//!
//! This module provides comprehensive validation functions for testing
//! primal integrations, configurations, and manifests.

use biomeos_manifest::*;
use serde_json::Value;
use std::collections::HashMap;

/// Comprehensive validation suite for primal integration tests
pub struct IntegrationValidator;

impl IntegrationValidator {
    /// Validate complete primal integration
    pub fn validate_primal_integration(
        manifest: &BiomeManifest,
        orchestrator_name: &str,
        storage_name: &str,
        expected_orchestrator_type: PrimalType,
        expected_storage_type: PrimalType,
    ) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Basic primal existence validation
        result.combine(Self::validate_primal_existence(manifest, orchestrator_name));
        result.combine(Self::validate_primal_existence(manifest, storage_name));

        if result.has_errors() {
            return result;
        }

        let orchestrator = manifest.primals.get(orchestrator_name).unwrap();
        let storage = manifest.primals.get(storage_name).unwrap();

        // Type validation
        result.combine(Self::validate_primal_type(orchestrator, expected_orchestrator_type, orchestrator_name));
        result.combine(Self::validate_primal_type(storage, expected_storage_type, storage_name));

        // Dependency validation
        result.combine(Self::validate_primal_dependencies(storage, &[orchestrator_name.to_string()], storage_name));

        // Priority validation
        result.combine(Self::validate_priority_ordering(orchestrator, storage, orchestrator_name, storage_name));

        // Configuration validation
        result.combine(Self::validate_primal_configuration(orchestrator, orchestrator_name));
        result.combine(Self::validate_primal_configuration(storage, storage_name));

        // Networking validation
        result.combine(Self::validate_primal_networking(orchestrator, orchestrator_name));
        result.combine(Self::validate_primal_networking(storage, storage_name));

        // Resources validation
        result.combine(Self::validate_primal_resources(orchestrator, orchestrator_name));
        result.combine(Self::validate_primal_resources(storage, storage_name));

        result
    }

    /// Validate primal existence in manifest
    pub fn validate_primal_existence(manifest: &BiomeManifest, primal_name: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        if !manifest.primals.contains_key(primal_name) {
            result.add_error(&format!("Primal '{}' not found in manifest", primal_name));
        }
        
        result
    }

    /// Validate primal type matches expected
    pub fn validate_primal_type(primal: &PrimalSpec, expected_type: PrimalType, primal_name: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        if primal.primal_type != expected_type {
            result.add_error(&format!(
                "Primal '{}' has type {:?}, expected {:?}",
                primal_name, primal.primal_type, expected_type
            ));
        }
        
        result
    }

    /// Validate primal dependencies
    pub fn validate_primal_dependencies(primal: &PrimalSpec, expected_deps: &[String], primal_name: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        for dep in expected_deps {
            if !primal.depends_on.contains(dep) {
                result.add_error(&format!("Primal '{}' missing dependency '{}'", primal_name, dep));
            }
        }
        
        result
    }

    /// Validate priority ordering
    pub fn validate_priority_ordering(primal_a: &PrimalSpec, primal_b: &PrimalSpec, name_a: &str, name_b: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        if primal_a.priority >= primal_b.priority {
            result.add_error(&format!(
                "Priority ordering incorrect: '{}' priority {} should be less than '{}' priority {}",
                name_a, primal_a.priority, name_b, primal_b.priority
            ));
        }
        
        result
    }

    /// Validate primal configuration
    pub fn validate_primal_configuration(primal: &PrimalSpec, primal_name: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        if primal.config.is_none() {
            result.add_warning(&format!("Primal '{}' has no configuration", primal_name));
            return result;
        }
        
        let config = primal.config.as_ref().unwrap();
        
        // Validate required configuration fields
        if !config.get("mode").is_some() {
            result.add_error(&format!("Primal '{}' missing 'mode' configuration", primal_name));
        }
        
        if !config.get("features").is_some() {
            result.add_warning(&format!("Primal '{}' missing 'features' configuration", primal_name));
        }
        
        // Validate configuration values
        if let Some(mode) = config.get("mode") {
            if !mode.is_string() {
                result.add_error(&format!("Primal '{}' 'mode' should be a string", primal_name));
            }
        }
        
        if let Some(features) = config.get("features") {
            if !features.is_array() {
                result.add_error(&format!("Primal '{}' 'features' should be an array", primal_name));
            }
        }
        
        result
    }

    /// Validate primal networking configuration
    pub fn validate_primal_networking(primal: &PrimalSpec, primal_name: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        if primal.networking.is_none() {
            result.add_warning(&format!("Primal '{}' has no networking configuration", primal_name));
            return result;
        }
        
        let networking = primal.networking.as_ref().unwrap();
        
        // Validate ports
        if let Some(ports) = &networking.ports {
            if ports.is_empty() {
                result.add_warning(&format!("Primal '{}' has empty ports list", primal_name));
            }
            
            for port in ports {
                if *port < 1024 {
                    result.add_warning(&format!("Primal '{}' uses privileged port {}", primal_name, port));
                }
                if *port > 65535 {
                    result.add_error(&format!("Primal '{}' uses invalid port {}", primal_name, port));
                }
            }
        }
        
        // Validate host
        if let Some(host) = &networking.host {
            if host.is_empty() {
                result.add_error(&format!("Primal '{}' has empty host", primal_name));
            }
        }
        
        // Validate discovery
        if let Some(discovery) = &networking.discovery {
            if discovery.method.is_empty() {
                result.add_error(&format!("Primal '{}' has empty discovery method", primal_name));
            }
        }
        
        result
    }

    /// Validate primal resources configuration
    pub fn validate_primal_resources(primal: &PrimalSpec, primal_name: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        if primal.resources.is_none() {
            result.add_warning(&format!("Primal '{}' has no resource limits", primal_name));
            return result;
        }
        
        let resources = primal.resources.as_ref().unwrap();
        
        // Validate CPU
        if let Some(cpu) = &resources.cpu {
            if let Some(max_cores) = cpu.max_cores {
                if max_cores <= 0.0 {
                    result.add_error(&format!("Primal '{}' has invalid CPU max_cores: {}", primal_name, max_cores));
                }
                if max_cores > 128.0 {
                    result.add_warning(&format!("Primal '{}' has very high CPU max_cores: {}", primal_name, max_cores));
                }
            }
        }
        
        // Validate memory
        if let Some(memory) = &resources.memory {
            if let Some(max_mb) = memory.max_mb {
                if max_mb == 0 {
                    result.add_error(&format!("Primal '{}' has invalid memory max_mb: {}", primal_name, max_mb));
                }
                if max_mb > 1024 * 1024 {
                    result.add_warning(&format!("Primal '{}' has very high memory max_mb: {}MB", primal_name, max_mb));
                }
            }
        }
        
        // Validate storage
        if let Some(storage) = &resources.storage {
            if let Some(max_mb) = storage.max_mb {
                if max_mb == 0 {
                    result.add_error(&format!("Primal '{}' has invalid storage max_mb: {}", primal_name, max_mb));
                }
            }
        }
        
        result
    }

    /// Validate manifest metadata
    pub fn validate_manifest_metadata(manifest: &BiomeManifest) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        // Validate required fields
        if manifest.metadata.name.is_empty() {
            result.add_error("Manifest name is empty");
        }
        
        if manifest.metadata.version.is_empty() {
            result.add_error("Manifest version is empty");
        }
        
        // Validate version format (basic semantic version check)
        if !Self::is_valid_version(&manifest.metadata.version) {
            result.add_warning(&format!("Manifest version '{}' is not a valid semantic version", manifest.metadata.version));
        }
        
        // Validate specialization
        if manifest.metadata.specialization.is_none() {
            result.add_info("Manifest has no specialization specified");
        }
        
        result
    }

    /// Validate dependency configuration
    pub fn validate_dependency_config(dependencies: &DependencyConfig) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        // Validate required dependencies
        for dep in &dependencies.requires {
            if dep.name.is_empty() {
                result.add_error("Required dependency has empty name");
            }
            
            if dep.optional {
                result.add_warning(&format!("Required dependency '{}' is marked as optional", dep.name));
            }
        }
        
        // Validate suggested dependencies
        for dep in &dependencies.suggests {
            if dep.name.is_empty() {
                result.add_error("Suggested dependency has empty name");
            }
            
            if !dep.optional {
                result.add_warning(&format!("Suggested dependency '{}' is marked as required", dep.name));
            }
        }
        
        // Validate conflicting dependencies
        for dep in &dependencies.conflicts {
            if dep.name.is_empty() {
                result.add_error("Conflicting dependency has empty name");
            }
        }
        
        // Validate features
        for (name, feature) in &dependencies.features {
            if name.is_empty() {
                result.add_error("Feature has empty name");
            }
            
            if feature.description.is_empty() {
                result.add_warning(&format!("Feature '{}' has empty description", name));
            }
            
            if feature.dependencies.is_empty() {
                result.add_warning(&format!("Feature '{}' has no dependencies", name));
            }
            
            if feature.services.is_empty() {
                result.add_warning(&format!("Feature '{}' has no services", name));
            }
        }
        
        result
    }

    /// Validate gaming-specific configurations
    pub fn validate_gaming_configuration(manifest: &BiomeManifest, orchestrator_name: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        // Check specialization
        if manifest.metadata.specialization != Some(BiomeSpecialization::GamingServer) {
            result.add_error("Gaming integration should have GamingServer specialization");
        }
        
        // Check orchestrator configuration
        if let Some(orchestrator) = manifest.primals.get(orchestrator_name) {
            if let Some(config) = &orchestrator.config {
                // Check tournament configuration
                if let Some(tournament) = config.get("tournament") {
                    if !tournament.is_object() {
                        result.add_error("Tournament configuration should be an object");
                    } else {
                        let tournament_obj = tournament.as_object().unwrap();
                        
                        if !tournament_obj.contains_key("match_making") {
                            result.add_warning("Tournament missing match_making configuration");
                        }
                        
                        if !tournament_obj.contains_key("player_balancing") {
                            result.add_warning("Tournament missing player_balancing configuration");
                        }
                        
                        if !tournament_obj.contains_key("server_allocation") {
                            result.add_warning("Tournament missing server_allocation configuration");
                        }
                    }
                } else {
                    result.add_error("Gaming orchestrator missing tournament configuration");
                }
            }
        }
        
        result
    }

    /// Check if version string is valid semantic version
    fn is_valid_version(version: &str) -> bool {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return false;
        }
        
        for part in parts {
            if part.parse::<u32>().is_err() {
                return false;
            }
        }
        
        true
    }
}

/// Validation result container
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub info: Vec<String>,
}

impl ValidationResult {
    /// Create new empty validation result
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
        }
    }

    /// Add error message
    pub fn add_error(&mut self, message: &str) {
        self.errors.push(message.to_string());
    }

    /// Add warning message
    pub fn add_warning(&mut self, message: &str) {
        self.warnings.push(message.to_string());
    }

    /// Add info message
    pub fn add_info(&mut self, message: &str) {
        self.info.push(message.to_string());
    }

    /// Check if result has errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if result has warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Check if result is completely valid (no errors or warnings)
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }

    /// Combine with another validation result
    pub fn combine(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self.info.extend(other.info);
    }

    /// Get summary of validation result
    pub fn summary(&self) -> String {
        format!(
            "Validation: {} errors, {} warnings, {} info",
            self.errors.len(),
            self.warnings.len(),
            self.info.len()
        )
    }
}

/// Specialized validators for different integration scenarios
pub struct ScenarioValidators;

impl ScenarioValidators {
    /// Validate data center integration scenario
    pub fn validate_data_center_scenario(manifest: &BiomeManifest, orchestrator_name: &str, storage_name: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        // Check specialization
        if manifest.metadata.specialization != Some(BiomeSpecialization::DataCenter) {
            result.add_error("Data center integration should have DataCenter specialization");
        }
        
        // Check encryption features
        if let Some(deps) = &manifest.dependencies {
            if !deps.features.contains_key("encryption-at-rest") {
                result.add_warning("Data center integration should have encryption-at-rest feature");
            }
            
            if !deps.features.contains_key("service-mesh-security") {
                result.add_warning("Data center integration should have service-mesh-security feature");
            }
        }
        
        result
    }

    /// Validate cross-region integration scenario
    pub fn validate_cross_region_scenario(manifest: &BiomeManifest, orchestrator_name: &str, storage_name: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        // Check high replication factor
        if let Some(storage) = manifest.primals.get(storage_name) {
            if let Some(config) = &storage.config {
                if let Some(replication) = config.get("replication_factor") {
                    if let Some(factor) = replication.as_u64() {
                        if factor < 3 {
                            result.add_warning("Cross-region storage should have replication factor >= 3");
                        }
                    }
                }
            }
        }
        
        // Check cross-region features
        if let Some(orchestrator) = manifest.primals.get(orchestrator_name) {
            if let Some(config) = &orchestrator.config {
                if let Some(features) = config.get("features") {
                    if let Some(features_array) = features.as_array() {
                        if !features_array.contains(&Value::String("cross_region_routing".to_string())) {
                            result.add_warning("Cross-region orchestrator should have cross_region_routing feature");
                        }
                    }
                }
            }
        }
        
        result
    }

    /// Validate performance optimization scenario
    pub fn validate_performance_scenario(manifest: &BiomeManifest, orchestrator_name: &str, storage_name: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        // Check high-performance resources
        if let Some(orchestrator) = manifest.primals.get(orchestrator_name) {
            if let Some(resources) = &orchestrator.resources {
                if let Some(storage_spec) = &resources.storage {
                    if let Some(storage_type) = &storage_spec.storage_type {
                        if storage_type != "nvme" {
                            result.add_warning("Performance orchestrator should use NVMe storage");
                        }
                    }
                }
            }
        }
        
        if let Some(storage) = manifest.primals.get(storage_name) {
            if let Some(resources) = &storage.resources {
                if let Some(storage_spec) = &resources.storage {
                    if let Some(storage_type) = &storage_spec.storage_type {
                        if storage_type != "nvme" {
                            result.add_warning("Performance storage should use NVMe storage");
                        }
                    }
                }
            }
        }
        
        result
    }
} 