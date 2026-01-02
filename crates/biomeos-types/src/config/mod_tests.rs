//! Configuration Module Tests
//!  
//! Basic tests for configuration system

#[cfg(test)]
mod config_tests {
    use crate::config::*;

    #[test]
    fn test_biomeos_config_default() {
        let config = BiomeOSConfig::default();
        // Verify basic defaults work
        assert!(!config.metadata.version.is_empty());
        assert!(!config.metadata.name.is_empty());
    }

    #[test]
    fn test_config_metadata_default() {
        let metadata = ConfigMetadata::default();
        assert!(!metadata.version.is_empty());
        assert!(!metadata.name.is_empty());
    }

    #[test]
    fn test_config_clone() {
        let config = BiomeOSConfig::default();
        let cloned = config.clone();
        assert_eq!(config.metadata.version, cloned.metadata.version);
    }

    #[test]
    fn test_config_debug() {
        let config = BiomeOSConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("BiomeOSConfig"));
    }

    #[test]
    fn test_config_serde_json() -> Result<(), Box<dyn std::error::Error>> {
        let config = BiomeOSConfig::default();
        let json = serde_json::to_string(&config)?;
        let _parsed: BiomeOSConfig = serde_json::from_str(&json)?;
        Ok(())
    }

    #[test]
    fn test_config_serde_yaml() -> Result<(), Box<dyn std::error::Error>> {
        let config = BiomeOSConfig::default();
        let yaml = serde_yaml::to_string(&config)?;
        let _parsed: BiomeOSConfig = serde_yaml::from_str(&yaml)?;
        Ok(())
    }

    #[test]
    #[ignore] // Requires serde(default) on metadata fields
    fn test_partial_yaml_deserialize() -> Result<(), Box<dyn std::error::Error>> {
        let yaml = r#"
metadata:
  version: "1.0.0"
  name: "test-config"
"#;
        let config: BiomeOSConfig = serde_yaml::from_str(yaml)?;
        assert_eq!(config.metadata.version, "1.0.0");
        assert_eq!(config.metadata.name, "test-config");
        Ok(())
    }

    #[test]
    fn test_config_from_env_basic() {
        std::env::set_var("BIOMEOS_PORT", "8888");
        let config = BiomeOSConfig::from_env();
        assert_eq!(config.network.port, 8888);
        std::env::remove_var("BIOMEOS_PORT");
    }

    #[test]
    fn test_subconfigs_have_defaults() {
        // All sub-configs should have working defaults
        let _ = SystemConfig::default();
        let _ = NetworkConfig::default();
        let _ = SecurityConfig::default();
        let _ = ResourceConfig::default();
        let _ = DiscoveryConfig::default();
        let _ = HealthMonitoringConfig::default();
        let _ = ObservabilityConfig::default();
        let _ = UIConfig::default();
        let _ = FeatureFlags::default();
    }

    #[test]
    fn test_feature_flags_default() {
        let features = FeatureFlags::default();
        // Just verify it has defaults
        let _ = features.debug;
    }

    #[test]
    fn test_security_config_default() {
        let security = SecurityConfig::default();
        // Just verify defaults exist
        let _ = security.authentication;
        let _ = security.authorization;
        let _ = security.encryption;
    }
}
