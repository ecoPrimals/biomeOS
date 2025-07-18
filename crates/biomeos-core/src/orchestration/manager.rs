//! Universal orchestration manager
//!
//! This module contains the main orchestration manager that coordinates
//! multiple orchestrators with sovereignty-first preferences.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::interface::UniversalOrchestrationInterface;
use super::types::*;

/// Universal Orchestration Manager
pub struct UniversalOrchestrationManager {
    pub orchestrators: HashMap<String, Box<dyn UniversalOrchestrationInterface>>,
    pub default_orchestrator: Option<String>,
    pub orchestrator_preference: Vec<String>,
    pub sovereignty_requirements: OrchestrationSovereigntyRequirements,
}

/// Orchestration sovereignty requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationSovereigntyRequirements {
    pub require_sovereign_orchestrator: bool,
    pub allow_cloud_managed_orchestrator: bool,
    pub crypto_lock_external_apis: bool,
    pub prefer_self_hosted: bool,
    pub require_air_gapped_deployment: bool,
}

impl Default for UniversalOrchestrationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl UniversalOrchestrationManager {
    /// Create new orchestration manager with sovereignty-first defaults
    pub fn new() -> Self {
        Self {
            orchestrators: HashMap::new(),
            default_orchestrator: None,
            orchestrator_preference: vec![
                "none".to_string(),       // Direct deployment (most sovereign)
                "podman".to_string(),     // Container-native, no orchestrator
                "k3s".to_string(),        // Lightweight K8s
                "nomad".to_string(),      // Simple, sovereign-friendly
                "kubernetes".to_string(), // Full-featured
            ],
            sovereignty_requirements: OrchestrationSovereigntyRequirements {
                require_sovereign_orchestrator: false,
                allow_cloud_managed_orchestrator: true,
                crypto_lock_external_apis: true,
                prefer_self_hosted: true,
                require_air_gapped_deployment: false,
            },
        }
    }

    /// Create manager with strict sovereignty requirements
    pub fn new_sovereign() -> Self {
        Self {
            orchestrators: HashMap::new(),
            default_orchestrator: None,
            orchestrator_preference: vec![
                "none".to_string(),       // Direct deployment only
                "podman".to_string(),     // Container-native
                "k3s".to_string(),        // Lightweight K8s
                "nomad".to_string(),      // Simple orchestrator
            ],
            sovereignty_requirements: OrchestrationSovereigntyRequirements {
                require_sovereign_orchestrator: true,
                allow_cloud_managed_orchestrator: false,
                crypto_lock_external_apis: true,
                prefer_self_hosted: true,
                require_air_gapped_deployment: true,
            },
        }
    }

    /// Create manager with cloud-friendly settings
    pub fn new_cloud_friendly() -> Self {
        Self {
            orchestrators: HashMap::new(),
            default_orchestrator: None,
            orchestrator_preference: vec![
                "kubernetes".to_string(),     // Full-featured
                "eks".to_string(),            // Amazon EKS
                "gke".to_string(),            // Google GKE
                "aks".to_string(),            // Microsoft AKS
                "nomad".to_string(),          // HashiCorp Nomad
                "docker-swarm".to_string(),   // Docker Swarm
            ],
            sovereignty_requirements: OrchestrationSovereigntyRequirements {
                require_sovereign_orchestrator: false,
                allow_cloud_managed_orchestrator: true,
                crypto_lock_external_apis: false,
                prefer_self_hosted: false,
                require_air_gapped_deployment: false,
            },
        }
    }

    /// Add orchestrator
    pub fn add_orchestrator(
        &mut self,
        name: String,
        orchestrator: Box<dyn UniversalOrchestrationInterface>,
    ) {
        self.orchestrators.insert(name, orchestrator);
    }

    /// Remove orchestrator
    pub fn remove_orchestrator(&mut self, name: &str) -> Option<Box<dyn UniversalOrchestrationInterface>> {
        self.orchestrators.remove(name)
    }

    /// List available orchestrators
    pub fn list_orchestrators(&self) -> Vec<String> {
        self.orchestrators.keys().cloned().collect()
    }

    /// Set default orchestrator
    pub fn set_default_orchestrator(&mut self, name: String) {
        self.default_orchestrator = Some(name);
    }

    /// Get default orchestrator
    pub fn get_default_orchestrator(&self) -> Option<&Box<dyn UniversalOrchestrationInterface>> {
        if let Some(default_name) = &self.default_orchestrator {
            self.orchestrators.get(default_name)
        } else {
            None
        }
    }

    /// Get orchestrator by name
    pub fn get_orchestrator(&self, name: &str) -> Option<&Box<dyn UniversalOrchestrationInterface>> {
        self.orchestrators.get(name)
    }

    /// Get best available orchestrator based on sovereignty requirements
    pub async fn get_best_orchestrator(&self) -> Option<&Box<dyn UniversalOrchestrationInterface>> {
        // Try default first if set
        if let Some(default_orchestrator) = self.get_default_orchestrator() {
            if let Ok(info) = default_orchestrator.orchestrator_info().await {
                if self.meets_sovereignty_requirements(&info.sovereignty_compliance) {
                    return Some(default_orchestrator);
                }
            }
        }

        // Try orchestrators in preference order
        for orchestrator_name in &self.orchestrator_preference {
            if let Some(orchestrator) = self.orchestrators.get(orchestrator_name) {
                // Check sovereignty compliance
                if let Ok(info) = orchestrator.orchestrator_info().await {
                    if self.meets_sovereignty_requirements(&info.sovereignty_compliance) {
                        return Some(orchestrator);
                    }
                }
            }
        }

        None
    }

    /// Check if orchestrator meets sovereignty requirements
    fn meets_sovereignty_requirements(&self, compliance: &OrchestrationSovereignty) -> bool {
        match compliance {
            OrchestrationSovereignty::FullSovereignty => true,
            OrchestrationSovereignty::PartialSovereignty { .. } => {
                !self.sovereignty_requirements.require_sovereign_orchestrator
            }
            OrchestrationSovereignty::CloudManaged { .. } => {
                self.sovereignty_requirements
                    .allow_cloud_managed_orchestrator
            }
        }
    }

    /// Update sovereignty requirements
    pub fn update_sovereignty_requirements(&mut self, requirements: OrchestrationSovereigntyRequirements) {
        self.sovereignty_requirements = requirements;
    }

    /// Get current sovereignty requirements
    pub fn get_sovereignty_requirements(&self) -> &OrchestrationSovereigntyRequirements {
        &self.sovereignty_requirements
    }

    /// Set orchestrator preference order
    pub fn set_orchestrator_preference(&mut self, preference: Vec<String>) {
        self.orchestrator_preference = preference;
    }

    /// Get orchestrator preference order
    pub fn get_orchestrator_preference(&self) -> &[String] {
        &self.orchestrator_preference
    }

    /// Check if manager has any orchestrators
    pub fn has_orchestrators(&self) -> bool {
        !self.orchestrators.is_empty()
    }

    /// Get orchestrator count
    pub fn orchestrator_count(&self) -> usize {
        self.orchestrators.len()
    }

    /// Validate orchestrator configuration
    pub async fn validate_configuration(&self) -> ValidationResult {
        let mut issues = vec![];
        let mut warnings = vec![];

        // Check if any orchestrators are available
        if self.orchestrators.is_empty() {
            issues.push("No orchestrators configured".to_string());
        }

        // Check sovereignty compliance
        let mut sovereign_count = 0;
        for (name, orchestrator) in &self.orchestrators {
            if let Ok(info) = orchestrator.orchestrator_info().await {
                if let OrchestrationSovereignty::FullSovereignty = info.sovereignty_compliance {
                    sovereign_count += 1;
                }
            } else {
                warnings.push(format!("Cannot get info for orchestrator: {}", name));
            }
        }

        // Check if sovereignty requirements can be met
        if self.sovereignty_requirements.require_sovereign_orchestrator && sovereign_count == 0 {
            issues.push("Sovereign orchestrator required but none available".to_string());
        }

        // Check if default orchestrator exists
        if let Some(default_name) = &self.default_orchestrator {
            if !self.orchestrators.contains_key(default_name) {
                issues.push(format!("Default orchestrator '{}' not found", default_name));
            }
        }

        ValidationResult {
            valid: issues.is_empty(),
            issues,
            warnings,
        }
    }

    /// Get manager statistics
    pub async fn get_statistics(&self) -> ManagerStatistics {
        let mut stats = ManagerStatistics::default();
        stats.total_orchestrators = self.orchestrators.len();

        let mut sovereign_count = 0;
        let mut cloud_managed_count = 0;
        let mut available_count = 0;

        for orchestrator in self.orchestrators.values() {
            if let Ok(info) = orchestrator.orchestrator_info().await {
                available_count += 1;
                match info.sovereignty_compliance {
                    OrchestrationSovereignty::FullSovereignty => sovereign_count += 1,
                    OrchestrationSovereignty::CloudManaged { .. } => cloud_managed_count += 1,
                    _ => {}
                }
            }
        }

        stats.available_orchestrators = available_count;
        stats.sovereign_orchestrators = sovereign_count;
        stats.cloud_managed_orchestrators = cloud_managed_count;
        stats.default_orchestrator = self.default_orchestrator.clone();

        stats
    }
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
}

/// Manager statistics
#[derive(Debug, Clone, Default)]
pub struct ManagerStatistics {
    pub total_orchestrators: usize,
    pub available_orchestrators: usize,
    pub sovereign_orchestrators: usize,
    pub cloud_managed_orchestrators: usize,
    pub default_orchestrator: Option<String>,
}

impl Default for OrchestrationSovereigntyRequirements {
    fn default() -> Self {
        Self {
            require_sovereign_orchestrator: false,
            allow_cloud_managed_orchestrator: true,
            crypto_lock_external_apis: true,
            prefer_self_hosted: true,
            require_air_gapped_deployment: false,
        }
    }
}

impl OrchestrationSovereigntyRequirements {
    /// Create strict sovereignty requirements
    pub fn strict() -> Self {
        Self {
            require_sovereign_orchestrator: true,
            allow_cloud_managed_orchestrator: false,
            crypto_lock_external_apis: true,
            prefer_self_hosted: true,
            require_air_gapped_deployment: true,
        }
    }

    /// Create lenient sovereignty requirements
    pub fn lenient() -> Self {
        Self {
            require_sovereign_orchestrator: false,
            allow_cloud_managed_orchestrator: true,
            crypto_lock_external_apis: false,
            prefer_self_hosted: false,
            require_air_gapped_deployment: false,
        }
    }

    /// Create balanced sovereignty requirements
    pub fn balanced() -> Self {
        Self {
            require_sovereign_orchestrator: false,
            allow_cloud_managed_orchestrator: true,
            crypto_lock_external_apis: true,
            prefer_self_hosted: true,
            require_air_gapped_deployment: false,
        }
    }
} 