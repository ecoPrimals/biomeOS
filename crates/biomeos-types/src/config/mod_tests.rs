//! Configuration Module Tests
//!
//! Comprehensive tests for the configuration system covering:
//! - Defaults and metadata
//! - Validation (valid and invalid)
//! - Builder pattern
//! - Merge behavior
//! - Serialization round-trips
//! - Environment variable overrides
//! - File I/O
//! - Environment-specific configuration

#[cfg(test)]
mod config_tests {
    use crate::config::*;
    use std::collections::HashMap;

    /// Helper to create a test EnvironmentConfig
    fn test_env_config() -> EnvironmentConfig {
        EnvironmentConfig {
            name: "test".to_string(),
            description: None,
            variables: HashMap::new(),
            features: FeatureFlags::default(),
            limits: features::EnvironmentLimits {
                max_users: None,
                max_sessions: None,
                rate_limit: None,
                retention_days: None,
                storage_limit: None,
            },
            endpoints: HashMap::new(),
        }
    }

    // ========================================================================
    // Default and Metadata Tests
    // ========================================================================

    #[test]
    fn test_biomeos_config_default() {
        let config = BiomeOSConfig::default();
        assert!(!config.metadata.version.is_empty());
        assert!(!config.metadata.name.is_empty());
        assert_eq!(config.metadata.version, "1.0.0");
        assert_eq!(config.metadata.name, "default-biome-config");
    }

    #[test]
    fn test_config_metadata_default() {
        let metadata = ConfigMetadata::default();
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.name, "default-biome-config");
        assert!(metadata.description.is_none());
        assert!(metadata.author.is_none());
        assert!(metadata.tags.is_empty());
        assert!(metadata.custom.is_empty());
    }

    #[test]
    fn test_config_metadata_timestamps() {
        let before = chrono::Utc::now();
        let metadata = ConfigMetadata::default();
        let after = chrono::Utc::now();

        assert!(metadata.created_at >= before);
        assert!(metadata.created_at <= after);
        assert!(metadata.modified_at >= before);
        assert!(metadata.modified_at <= after);
    }

    #[test]
    fn test_config_clone() {
        let config = BiomeOSConfig::default();
        let cloned = config.clone();
        assert_eq!(config.metadata.version, cloned.metadata.version);
        assert_eq!(config.metadata.name, cloned.metadata.name);
        assert_eq!(config.network.port, cloned.network.port);
    }

    #[test]
    fn test_config_debug() {
        let config = BiomeOSConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("BiomeOSConfig"));
        assert!(debug_str.contains("metadata"));
    }

    // ========================================================================
    // Validation Tests
    // ========================================================================

    #[test]
    fn test_validate_default_config_passes() {
        let config = BiomeOSConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_port_zero_fails() {
        let mut config = BiomeOSConfig::default();
        config.network.port = 0;
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("Port cannot be 0"));
    }

    #[test]
    fn test_validate_request_timeout_zero_fails() {
        let mut config = BiomeOSConfig::default();
        config.system.timeouts.default_request_timeout = std::time::Duration::from_secs(0);
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("Request timeout cannot be 0"));
    }

    #[test]
    fn test_validate_max_connections_zero_fails() {
        let mut config = BiomeOSConfig::default();
        config.system.limits.max_connections = 0;
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("Max connections cannot be 0"));
    }

    #[test]
    fn test_validate_metrics_interval_zero_fails() {
        let mut config = BiomeOSConfig::default();
        config.observability.metrics.enabled = true;
        config.observability.metrics.interval = std::time::Duration::from_secs(0);
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("Metrics interval cannot be 0"));
    }

    #[test]
    fn test_validate_metrics_disabled_zero_interval_ok() {
        let mut config = BiomeOSConfig::default();
        config.observability.metrics.enabled = false;
        config.observability.metrics.interval = std::time::Duration::from_secs(0);
        // Should pass because metrics are disabled
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_session_timeout_zero_fails() {
        let mut config = BiomeOSConfig::default();
        config.security.session.timeout = std::time::Duration::from_secs(0);
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("Session timeout cannot be 0"));
    }

    #[test]
    fn test_validate_custom_port_passes() {
        let mut config = BiomeOSConfig::default();
        config.network.port = 9090;
        assert!(config.validate().is_ok());
    }

    // ========================================================================
    // Builder Tests
    // ========================================================================

    #[test]
    fn test_builder_basic() -> Result<(), Box<dyn std::error::Error>> {
        let config = BiomeOSConfig::builder()
            .name("test-config")
            .version("2.0.0")
            .port(8080)
            .build()?;

        assert_eq!(config.metadata.name, "test-config");
        assert_eq!(config.metadata.version, "2.0.0");
        assert_eq!(config.network.port, 8080);
        Ok(())
    }

    #[test]
    fn test_builder_debug_mode() -> Result<(), Box<dyn std::error::Error>> {
        let config = BiomeOSConfig::builder().debug(true).build()?;
        assert!(config.features.debug);
        Ok(())
    }

    #[test]
    fn test_builder_experimental() -> Result<(), Box<dyn std::error::Error>> {
        let config = BiomeOSConfig::builder().experimental(true).build()?;
        assert!(config.features.experimental);
        Ok(())
    }

    #[test]
    fn test_builder_environment() -> Result<(), Box<dyn std::error::Error>> {
        let config = BiomeOSConfig::builder()
            .environment(Environment::Production)
            .build()?;
        assert_eq!(config.system.environment, Environment::Production);
        Ok(())
    }

    #[test]
    fn test_builder_bind_address() -> Result<(), Box<dyn std::error::Error>> {
        let config = BiomeOSConfig::builder().bind_address("0.0.0.0").build()?;
        assert_eq!(config.network.bind_address, "0.0.0.0");
        Ok(())
    }

    #[test]
    fn test_builder_log_level() -> Result<(), Box<dyn std::error::Error>> {
        let config = BiomeOSConfig::builder()
            .log_level(observability::LogLevel::Debug)
            .build()?;
        assert_eq!(format!("{:?}", config.observability.logging.level), "Debug");
        Ok(())
    }

    #[test]
    fn test_builder_add_extension() -> Result<(), Box<dyn std::error::Error>> {
        let config = BiomeOSConfig::builder()
            .add_extension("custom_key", serde_json::json!("custom_value"))
            .build()?;
        assert_eq!(
            config.metadata.custom.get("custom_key"),
            Some(&serde_json::json!("custom_value"))
        );
        Ok(())
    }

    #[test]
    fn test_builder_add_environment() -> Result<(), Box<dyn std::error::Error>> {
        let config = BiomeOSConfig::builder()
            .add_environment("staging", test_env_config())
            .build()?;
        assert!(config.environments.contains_key("staging"));
        Ok(())
    }

    #[test]
    fn test_builder_invalid_port_fails() {
        let result = BiomeOSConfig::builder().port(0).build();
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_default_impl() {
        let builder = BiomeOSConfigBuilder::default();
        let config = builder.build();
        assert!(config.is_ok());
    }

    // ========================================================================
    // Merge Tests
    // ========================================================================

    #[test]
    fn test_merge_basic() -> Result<(), Box<dyn std::error::Error>> {
        let mut base = BiomeOSConfig::default();
        let other = BiomeOSConfig::builder()
            .version("3.0.0")
            .port(9090)
            .build()?;

        base.merge(other)?;
        assert_eq!(base.metadata.version, "3.0.0");
        assert_eq!(base.network.port, 9090);
        Ok(())
    }

    #[test]
    fn test_merge_preserves_metadata_custom() -> Result<(), Box<dyn std::error::Error>> {
        let mut base = BiomeOSConfig::default();
        base.metadata
            .custom
            .insert("base_key".to_string(), serde_json::json!("base_value"));

        let mut other = BiomeOSConfig::default();
        other
            .metadata
            .custom
            .insert("other_key".to_string(), serde_json::json!("other_value"));

        base.merge(other)?;
        assert!(base.metadata.custom.contains_key("base_key"));
        assert!(base.metadata.custom.contains_key("other_key"));
        Ok(())
    }

    #[test]
    fn test_merge_environments() -> Result<(), Box<dyn std::error::Error>> {
        let mut base = BiomeOSConfig::default();
        base.environments
            .insert("dev".to_string(), test_env_config());

        let mut other = BiomeOSConfig::default();
        other
            .environments
            .insert("prod".to_string(), test_env_config());

        base.merge(other)?;
        assert!(base.environments.contains_key("dev"));
        assert!(base.environments.contains_key("prod"));
        Ok(())
    }

    #[test]
    fn test_merge_invalid_result_fails() {
        let mut base = BiomeOSConfig::default();
        let mut other = BiomeOSConfig::default();
        other.network.port = 0; // Invalid

        let result = base.merge(other);
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_updates_modified_at() -> Result<(), Box<dyn std::error::Error>> {
        let mut base = BiomeOSConfig::default();
        let original_modified = base.metadata.modified_at;

        // Small sleep to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(2));

        let other = BiomeOSConfig::default();
        base.merge(other)?;

        assert!(base.metadata.modified_at >= original_modified);
        Ok(())
    }

    // ========================================================================
    // Serialization Tests
    // ========================================================================

    #[test]
    fn test_config_serde_json_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let config = BiomeOSConfig::builder()
            .name("json-test")
            .port(4321)
            .debug(true)
            .build()?;

        let json = serde_json::to_string(&config)?;
        let parsed: BiomeOSConfig = serde_json::from_str(&json)?;

        assert_eq!(parsed.metadata.name, "json-test");
        assert_eq!(parsed.network.port, 4321);
        assert!(parsed.features.debug);
        Ok(())
    }

    #[test]
    fn test_config_serde_yaml_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let config = BiomeOSConfig::builder()
            .name("yaml-test")
            .version("2.0.0")
            .build()?;

        let yaml = serde_yaml::to_string(&config)?;
        let parsed: BiomeOSConfig = serde_yaml::from_str(&yaml)?;

        assert_eq!(parsed.metadata.name, "yaml-test");
        assert_eq!(parsed.metadata.version, "2.0.0");
        Ok(())
    }

    #[test]
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
    fn test_empty_yaml_uses_defaults() -> Result<(), Box<dyn std::error::Error>> {
        let yaml = "{}";
        let config: BiomeOSConfig = serde_yaml::from_str(yaml)?;
        assert_eq!(config.metadata.version, "1.0.0");
        assert_eq!(config.metadata.name, "default-biome-config");
        Ok(())
    }

    #[test]
    fn test_partial_json_uses_defaults() -> Result<(), Box<dyn std::error::Error>> {
        // Only metadata section — other sections use #[serde(default)]
        let json = r#"{"metadata": {"version": "2.0.0"}}"#;
        let config: BiomeOSConfig = serde_json::from_str(json)?;
        assert_eq!(config.metadata.version, "2.0.0");
        // Network should have defaults
        assert!(config.network.port > 0);
        Ok(())
    }

    #[test]
    fn test_metadata_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let mut metadata = ConfigMetadata {
            description: Some("Test description".to_string()),
            author: Some("test-author".to_string()),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            ..Default::default()
        };
        metadata
            .custom
            .insert("key".to_string(), serde_json::json!({"nested": true}));

        let json = serde_json::to_string(&metadata)?;
        let parsed: ConfigMetadata = serde_json::from_str(&json)?;

        assert_eq!(parsed.description, Some("Test description".to_string()));
        assert_eq!(parsed.author, Some("test-author".to_string()));
        assert_eq!(parsed.tags.len(), 2);
        assert!(parsed.custom.contains_key("key"));
        Ok(())
    }

    // ========================================================================
    // Environment Variable Tests
    // ========================================================================

    // NOTE: Environment variable tests are grouped into a single test function to avoid
    // race conditions when tests run in parallel. Env vars are process-global state.
    #[test]
    #[ignore = "env-var tests are thread-unsafe; run with --test-threads=1"]
    fn test_config_from_env_all_overrides() {
        // Port
        std::env::set_var("BIOMEOS_PORT", "8888");
        let config = BiomeOSConfig::from_env();
        assert_eq!(config.network.port, 8888);
        std::env::remove_var("BIOMEOS_PORT");

        // Invalid port ignored
        std::env::set_var("BIOMEOS_PORT", "not_a_number");
        let config = BiomeOSConfig::from_env();
        assert!(config.network.port > 0);
        std::env::remove_var("BIOMEOS_PORT");

        // Bind address
        std::env::set_var("BIOMEOS_BIND_ADDRESS", "192.168.1.1");
        let config = BiomeOSConfig::from_env();
        assert_eq!(config.network.bind_address, "192.168.1.1");
        std::env::remove_var("BIOMEOS_BIND_ADDRESS");

        // Debug true
        std::env::set_var("BIOMEOS_DEBUG", "true");
        let config = BiomeOSConfig::from_env();
        assert!(config.features.debug);
        std::env::remove_var("BIOMEOS_DEBUG");

        // Debug false
        std::env::set_var("BIOMEOS_DEBUG", "false");
        let config = BiomeOSConfig::from_env();
        assert!(!config.features.debug);
        std::env::remove_var("BIOMEOS_DEBUG");

        // Experimental
        std::env::set_var("BIOMEOS_EXPERIMENTAL", "true");
        let config = BiomeOSConfig::from_env();
        assert!(config.features.experimental);
        std::env::remove_var("BIOMEOS_EXPERIMENTAL");

        // Log levels
        for (level_str, expected_debug) in [
            ("trace", "Trace"),
            ("debug", "Debug"),
            ("info", "Info"),
            ("warn", "Warn"),
            ("error", "Error"),
            ("off", "Off"),
            ("TRACE", "Trace"),
            ("INFO", "Info"),
            ("unknown", "Info"),
        ] {
            std::env::set_var("BIOMEOS_LOG_LEVEL", level_str);
            let config = BiomeOSConfig::from_env();
            assert_eq!(
                format!("{:?}", config.observability.logging.level),
                expected_debug,
                "Failed for log level: {}",
                level_str
            );
            std::env::remove_var("BIOMEOS_LOG_LEVEL");
        }
    }

    // ========================================================================
    // File I/O Tests
    // ========================================================================

    #[test]
    fn test_config_file_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("test-config.yaml");

        let config = BiomeOSConfig::builder()
            .name("file-test")
            .version("1.5.0")
            .port(7777)
            .build()?;

        config.to_file(&path)?;
        let loaded = BiomeOSConfig::from_file(&path)?;

        assert_eq!(loaded.metadata.name, "file-test");
        assert_eq!(loaded.metadata.version, "1.5.0");
        assert_eq!(loaded.network.port, 7777);
        Ok(())
    }

    #[test]
    fn test_config_from_file_not_found() {
        let result = BiomeOSConfig::from_file("/nonexistent/path/config.yaml");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to read"));
    }

    #[test]
    fn test_config_from_file_invalid_yaml() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("bad.yaml");
        std::fs::write(&path, "not: valid: yaml: [[")?;

        let result = BiomeOSConfig::from_file(&path);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_config_from_file_invalid_values() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("invalid.yaml");
        // Port 0 should fail validation
        std::fs::write(&path, "network:\n  port: 0\n")?;

        let result = BiomeOSConfig::from_file(&path);
        assert!(result.is_err());
        Ok(())
    }

    // ========================================================================
    // Environment-Specific Configuration Tests
    // ========================================================================

    #[test]
    fn test_for_environment_basic() -> Result<(), Box<dyn std::error::Error>> {
        let mut config = BiomeOSConfig::default();
        let mut env_config = test_env_config();
        env_config.features.debug = true;
        config
            .environments
            .insert("staging".to_string(), env_config);

        let staging_config = config.for_environment("staging")?;
        assert!(staging_config.features.debug);
        Ok(())
    }

    #[test]
    fn test_for_environment_missing_returns_base() -> Result<(), Box<dyn std::error::Error>> {
        let config = BiomeOSConfig::default();
        let result = config.for_environment("nonexistent")?;
        // Should return a clone of the base config
        assert_eq!(result.metadata.version, config.metadata.version);
        Ok(())
    }

    #[test]
    fn test_for_environment_with_endpoints() -> Result<(), Box<dyn std::error::Error>> {
        let mut config = BiomeOSConfig::default();
        let mut env_config = test_env_config();
        env_config.endpoints.insert(
            "api".to_string(),
            "https://api.staging.example.com".to_string(),
        );
        config
            .environments
            .insert("staging".to_string(), env_config);

        let staging = config.for_environment("staging")?;
        assert!(staging.metadata.custom.contains_key("api_endpoint"));
        assert_eq!(
            staging.metadata.custom.get("api_endpoint"),
            Some(&serde_json::json!("https://api.staging.example.com"))
        );
        Ok(())
    }

    // ========================================================================
    // Sub-Config Default Tests
    // ========================================================================

    #[test]
    fn test_subconfigs_have_defaults() {
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
    fn test_system_config_defaults() {
        let sys = SystemConfig::default();
        assert_eq!(sys.name, "biomeos");
        assert_eq!(sys.environment, Environment::Development);
        assert_eq!(sys.organization_scale, OrganizationScale::Individual);
        assert_eq!(
            sys.timeouts.default_request_timeout,
            std::time::Duration::from_secs(30)
        );
        assert_eq!(sys.limits.max_connections, 1000);
    }

    #[test]
    fn test_timeout_config_defaults() {
        let timeouts = TimeoutConfig::default();
        assert_eq!(
            timeouts.connection_timeout,
            std::time::Duration::from_secs(10)
        );
        assert_eq!(
            timeouts.health_check_timeout,
            std::time::Duration::from_secs(5)
        );
        assert_eq!(
            timeouts.shutdown_timeout,
            std::time::Duration::from_secs(30)
        );
    }

    #[test]
    fn test_system_limits_defaults() {
        let limits = SystemLimits::default();
        assert_eq!(limits.max_connections, 1000);
        assert_eq!(limits.max_request_size, 10 * 1024 * 1024);
        assert_eq!(limits.max_upload_size, 100 * 1024 * 1024);
        assert!(limits.max_memory_usage.is_none());
        assert!(limits.max_cpu_usage.is_none());
    }

    #[test]
    fn test_environment_variants() {
        let envs = vec![
            Environment::Development,
            Environment::Testing,
            Environment::Staging,
            Environment::Production,
            Environment::Custom("custom-env".to_string()),
        ];
        for env in envs {
            let json = serde_json::to_string(&env).expect("serialize env");
            let parsed: Environment = serde_json::from_str(&json).expect("parse env");
            assert_eq!(format!("{:?}", env), format!("{:?}", parsed));
        }
    }

    #[test]
    fn test_organization_scale_variants() {
        let scales = vec![
            OrganizationScale::Individual,
            OrganizationScale::Team,
            OrganizationScale::Department,
            OrganizationScale::Enterprise,
            OrganizationScale::Global,
        ];
        for scale in scales {
            let json = serde_json::to_string(&scale).expect("serialize scale");
            let parsed: OrganizationScale = serde_json::from_str(&json).expect("parse scale");
            assert_eq!(format!("{:?}", scale), format!("{:?}", parsed));
        }
    }

    #[test]
    fn test_feature_flags_default() {
        let features = FeatureFlags::default();
        let _ = features.debug;
    }

    #[test]
    fn test_security_config_default() {
        let security = SecurityConfig::default();
        let _ = security.authentication;
        let _ = security.authorization;
        let _ = security.encryption;
    }
}
