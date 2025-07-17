use super::types::*;
use crate::BiomeResult;
use std::collections::HashMap;

/// Dependency registry for managing external dependencies
pub struct DependencyRegistry {
    pub dependencies: HashMap<DependencyId, ExternalDependency>,
    pub access_log: Vec<AccessLogEntry>,
}

/// Access log entry
#[derive(Debug, Clone)]
pub struct AccessLogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub dependency_id: DependencyId,
    pub access_context: AccessContext,
    pub decision: AccessDecision,
}

impl Default for DependencyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyRegistry {
    /// Create new dependency registry
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            access_log: Vec::new(),
        }
    }

    /// Register a new external dependency
    pub async fn register_dependency(
        &mut self,
        dependency: ExternalDependency,
    ) -> BiomeResult<DependencyId> {
        let dependency_id = dependency.id.clone();
        self.dependencies.insert(dependency_id.clone(), dependency);
        Ok(dependency_id)
    }

    /// Get dependency by ID
    pub fn get_dependency(&self, id: &DependencyId) -> Option<&ExternalDependency> {
        self.dependencies.get(id)
    }

    /// List all dependencies
    pub fn list_dependencies(&self) -> Vec<&ExternalDependency> {
        self.dependencies.values().collect()
    }

    /// Update dependency
    pub async fn update_dependency(&mut self, dependency: ExternalDependency) -> BiomeResult<()> {
        self.dependencies.insert(dependency.id.clone(), dependency);
        Ok(())
    }

    /// Remove dependency
    pub async fn remove_dependency(&mut self, id: &DependencyId) -> BiomeResult<()> {
        self.dependencies.remove(id);
        Ok(())
    }

    /// Log access attempt
    pub fn log_access(
        &mut self,
        dependency_id: DependencyId,
        context: AccessContext,
        decision: AccessDecision,
    ) {
        self.access_log.push(AccessLogEntry {
            timestamp: chrono::Utc::now(),
            dependency_id,
            access_context: context,
            decision,
        });
    }

    /// Get access history for dependency
    pub fn get_access_history(&self, dependency_id: &DependencyId) -> Vec<&AccessLogEntry> {
        self.access_log
            .iter()
            .filter(|entry| &entry.dependency_id == dependency_id)
            .collect()
    }

    /// Find dependencies by type
    pub fn find_by_type(&self, dep_type: &DependencyType) -> Vec<&ExternalDependency> {
        self.dependencies
            .values()
            .filter(|dep| {
                std::mem::discriminant(&dep.dependency_type) == std::mem::discriminant(dep_type)
            })
            .collect()
    }

    /// Find dependencies by vendor
    pub fn find_by_vendor(&self, vendor: &str) -> Vec<&ExternalDependency> {
        self.dependencies
            .values()
            .filter(|dep| dep.vendor == vendor)
            .collect()
    }

    /// Get dependencies requiring sovereign keys
    pub fn get_sovereign_key_dependencies(&self) -> Vec<&ExternalDependency> {
        self.dependencies
            .values()
            .filter(|dep| dep.access_requirements.sovereign_key_required)
            .collect()
    }

    /// Get dependencies with cat door access
    pub fn get_cat_door_dependencies(&self) -> Vec<&ExternalDependency> {
        self.dependencies
            .values()
            .filter(|dep| dep.access_requirements.cat_door_allowed)
            .collect()
    }
}
