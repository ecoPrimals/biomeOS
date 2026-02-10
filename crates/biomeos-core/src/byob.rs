//! Bring Your Own Beardog (BYOB) Manager

use anyhow::Result;
use biomeos_types::BiomeOSConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// BYOB Team configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByobTeamConfig {
    /// Unique team identifier
    pub team_id: String,
    /// How strongly the team is isolated
    pub isolation_level: IsolationLevel,
    /// Resource quotas for the team
    pub resource_limits: ResourceLimits,
    /// Capabilities this team is allowed to use
    pub allowed_capabilities: Vec<String>,
}

/// Isolation levels for BYOB teams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IsolationLevel {
    /// No isolation — shared everything
    None,
    /// Basic namespace isolation
    Basic,
    /// Strict resource and network isolation
    Strict,
    /// Full VM-level isolation
    Complete,
}

/// Resource limits for BYOB teams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum CPU usage percentage
    pub max_cpu_percent: f64,
    /// Maximum memory in MiB
    pub max_memory_mb: u64,
    /// Maximum disk space in MiB
    pub max_disk_mb: u64,
    /// Maximum network throughput in Mbps
    pub max_network_mbps: f64,
}

/// BYOB Manager for team isolation
pub struct ByobManager {
    config: BiomeOSConfig,
    teams: HashMap<String, ByobTeamConfig>,
}

impl ByobManager {
    /// Create new BYOB manager
    pub fn new(config: BiomeOSConfig) -> Self {
        Self {
            config,
            teams: HashMap::new(),
        }
    }

    /// Configure team isolation
    pub async fn configure_team(&mut self, team_id: String, config: ByobTeamConfig) -> Result<()> {
        self.validate_team_config(&team_id, &config)?;

        // Apply isolation settings
        let isolated_config = self.apply_isolation_settings(config.clone())?;

        self.teams.insert(team_id, isolated_config);
        Ok(())
    }

    /// Validate team configuration
    fn validate_team_config(&self, _team_id: &str, _config: &ByobTeamConfig) -> Result<()> {
        // Validation logic would go here
        Ok(())
    }

    /// Apply isolation settings to configuration
    fn apply_isolation_settings(&self, mut config: ByobTeamConfig) -> Result<ByobTeamConfig> {
        // Apply isolation logic based on self.config
        match config.isolation_level {
            IsolationLevel::Complete => {
                // Maximum isolation settings
                config.resource_limits.max_cpu_percent = 25.0;
                config.resource_limits.max_memory_mb = 512;
            }
            IsolationLevel::Strict => {
                // High isolation settings
                config.resource_limits.max_cpu_percent = 50.0;
                config.resource_limits.max_memory_mb = 1024;
            }
            _ => {
                // Default limits
            }
        }
        Ok(config)
    }

    /// Get team configuration
    pub fn get_team_config(&self, team_id: &str) -> Option<&ByobTeamConfig> {
        self.teams.get(team_id)
    }

    /// Remove team configuration
    pub fn remove_team(&mut self, team_id: &str) -> Option<ByobTeamConfig> {
        self.teams.remove(team_id)
    }

    /// Get system configuration for validation
    pub fn get_system_config(&self) -> &BiomeOSConfig {
        &self.config
    }

    /// Get the number of configured teams
    pub fn get_team_count(&self) -> usize {
        self.teams.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> BiomeOSConfig {
        BiomeOSConfig::default()
    }

    fn create_test_team_config(team_id: &str, level: IsolationLevel) -> ByobTeamConfig {
        ByobTeamConfig {
            team_id: team_id.to_string(),
            isolation_level: level,
            resource_limits: ResourceLimits {
                max_cpu_percent: 100.0,
                max_memory_mb: 2048,
                max_disk_mb: 10240,
                max_network_mbps: 100.0,
            },
            allowed_capabilities: vec!["compute".to_string(), "storage".to_string()],
        }
    }

    #[test]
    fn test_byob_manager_new() {
        let config = create_test_config();
        let manager = ByobManager::new(config);
        assert_eq!(manager.get_team_count(), 0);
    }

    #[tokio::test]
    async fn test_configure_team_basic() {
        let config = create_test_config();
        let mut manager = ByobManager::new(config);

        let team_config = create_test_team_config("team-1", IsolationLevel::Basic);
        let result = manager
            .configure_team("team-1".to_string(), team_config)
            .await;

        assert!(result.is_ok());
        assert_eq!(manager.get_team_count(), 1);
    }

    #[tokio::test]
    async fn test_configure_team_strict_isolation() {
        let config = create_test_config();
        let mut manager = ByobManager::new(config);

        let team_config = create_test_team_config("team-strict", IsolationLevel::Strict);
        manager
            .configure_team("team-strict".to_string(), team_config)
            .await
            .unwrap();

        let stored = manager.get_team_config("team-strict").unwrap();
        assert_eq!(stored.resource_limits.max_cpu_percent, 50.0);
        assert_eq!(stored.resource_limits.max_memory_mb, 1024);
    }

    #[tokio::test]
    async fn test_configure_team_complete_isolation() {
        let config = create_test_config();
        let mut manager = ByobManager::new(config);

        let team_config = create_test_team_config("team-complete", IsolationLevel::Complete);
        manager
            .configure_team("team-complete".to_string(), team_config)
            .await
            .unwrap();

        let stored = manager.get_team_config("team-complete").unwrap();
        assert_eq!(stored.resource_limits.max_cpu_percent, 25.0);
        assert_eq!(stored.resource_limits.max_memory_mb, 512);
    }

    #[test]
    fn test_get_team_config() {
        let config = create_test_config();
        let manager = ByobManager::new(config);

        assert!(manager.get_team_config("nonexistent").is_none());
    }

    #[tokio::test]
    async fn test_remove_team() {
        let config = create_test_config();
        let mut manager = ByobManager::new(config);

        let team_config = create_test_team_config("team-to-remove", IsolationLevel::Basic);
        manager
            .configure_team("team-to-remove".to_string(), team_config)
            .await
            .unwrap();
        assert_eq!(manager.get_team_count(), 1);

        let removed = manager.remove_team("team-to-remove");
        assert!(removed.is_some());
        assert_eq!(manager.get_team_count(), 0);
    }

    #[test]
    fn test_get_system_config() {
        let config = create_test_config();
        let manager = ByobManager::new(config);

        let sys_config = manager.get_system_config();
        // Just verify we can access the config
        // resources is a ResourceConfig struct, not an Option
        let _ = &sys_config.resources;
    }

    #[test]
    fn test_isolation_levels() {
        // Test that all isolation levels can be serialized/deserialized
        let levels = [
            IsolationLevel::None,
            IsolationLevel::Basic,
            IsolationLevel::Strict,
            IsolationLevel::Complete,
        ];

        for level in levels {
            let json = serde_json::to_string(&level).unwrap();
            let deserialized: IsolationLevel = serde_json::from_str(&json).unwrap();
            // Check round-trip works
            let json2 = serde_json::to_string(&deserialized).unwrap();
            assert_eq!(json, json2);
        }
    }

    #[test]
    fn test_resource_limits() {
        let limits = ResourceLimits {
            max_cpu_percent: 50.0,
            max_memory_mb: 1024,
            max_disk_mb: 5120,
            max_network_mbps: 50.0,
        };

        let json = serde_json::to_string(&limits).unwrap();
        let deserialized: ResourceLimits = serde_json::from_str(&json).unwrap();

        assert_eq!(limits.max_cpu_percent, deserialized.max_cpu_percent);
        assert_eq!(limits.max_memory_mb, deserialized.max_memory_mb);
    }

    #[test]
    fn test_byob_team_config_serialization() {
        let config = create_test_team_config("test-team", IsolationLevel::Strict);

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ByobTeamConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.team_id, deserialized.team_id);
        assert_eq!(
            config.allowed_capabilities.len(),
            deserialized.allowed_capabilities.len()
        );
    }
}
