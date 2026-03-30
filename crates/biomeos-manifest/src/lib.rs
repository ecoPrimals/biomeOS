// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! `BiomeOS` Manifest Management
//!
//! This crate provides comprehensive manifest parsing, validation, and management
//! capabilities for biomeOS.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

/// TOML-based niche manifest parsing with graph support
pub mod niche;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

// Import from biomeos-types using the actual available exports
use biomeos_types::{
    BiomeError,
    BiomeResult,
    manifest::manifest_core::NetworkSpec,
    manifest::manifest_extensions::BiomeDependency,
    // Import from the actual modules that exist
    manifest::{BiomeManifest, ServiceSpec},
};

/// Core manifest processing and validation functionality
pub struct BiomeManifestProcessor;

impl BiomeManifestProcessor {
    /// Validate a biome manifest for correctness
    pub fn validate(manifest: &BiomeManifest) -> BiomeResult<()> {
        // Validate metadata
        if manifest.metadata.name.is_empty() {
            return Err(BiomeError::validation_error(
                "Manifest name cannot be empty".to_string(),
                vec![],
            ));
        }

        // Validate services - BiomeManifest.services is a HashMap<String, ServiceSpec>
        for (service_name, service) in &manifest.services {
            Self::validate_service(service_name, service).map_err(|e| {
                BiomeError::validation_error(
                    format!("Invalid service '{service_name}': {e}"),
                    vec![],
                )
            })?;
        }

        // Validate dependencies
        for dependency in &manifest.dependencies {
            Self::validate_dependency(dependency)?;
        }

        // Validate networks
        for (network_name, network) in &manifest.networks {
            Self::validate_network(network_name, network).map_err(|e| {
                BiomeError::validation_error(
                    format!("Invalid network '{network_name}': {e}"),
                    vec![],
                )
            })?;
        }

        Ok(())
    }

    /// Validate service specification
    fn validate_service(service_name: &str, service: &ServiceSpec) -> BiomeResult<()> {
        // Validate that the service has essential fields
        if service.metadata.name.is_empty() {
            return Err(BiomeError::validation_error(
                format!("Service '{service_name}' must have a name"),
                vec![],
            ));
        }

        // Validate ports - using ServiceSpec structure that actually exists
        for port in &service.ports {
            // Port validation - u16 cannot be > 65535, so this comparison is removed
            if port.port == 0 {
                return Err(BiomeError::validation_error(
                    format!("Service '{service_name}' has invalid port: 0"),
                    vec![],
                ));
            }
        }

        Ok(())
    }

    /// Validate dependency specification  
    fn validate_dependency(dependency: &BiomeDependency) -> BiomeResult<()> {
        if dependency.name.is_empty() {
            return Err(BiomeError::validation_error(
                "Dependency must have a name".to_string(),
                vec![],
            ));
        }
        Ok(())
    }

    /// Validate network specification
    fn validate_network(network_name: &str, _network: &NetworkSpec) -> BiomeResult<()> {
        // Basic network validation
        if network_name.is_empty() {
            return Err(BiomeError::validation_error(
                "Network must have a name".to_string(),
                vec![],
            ));
        }
        Ok(())
    }

    /// Load manifest from YAML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> BiomeResult<BiomeManifest> {
        let content = fs::read_to_string(path).map_err(|e| {
            BiomeError::validation_error(format!("Failed to read manifest file: {e}"), vec![])
        })?;

        let manifest: BiomeManifest = serde_yaml::from_str(&content).map_err(|e| {
            BiomeError::validation_error(format!("Failed to parse manifest YAML: {e}"), vec![])
        })?;

        Self::validate(&manifest)?;
        Ok(manifest)
    }

    /// Save manifest to YAML file
    pub fn save_to_file<P: AsRef<Path>>(manifest: &BiomeManifest, path: P) -> BiomeResult<()> {
        Self::validate(manifest)?;

        let yaml_content = serde_yaml::to_string(manifest).map_err(|e| {
            BiomeError::validation_error(
                format!("Failed to serialize manifest to YAML: {e}"),
                vec![],
            )
        })?;

        fs::write(path, yaml_content).map_err(|e| {
            BiomeError::validation_error(format!("Failed to write manifest file: {e}"), vec![])
        })?;

        Ok(())
    }

    /// Load manifest from YAML content
    pub fn load_from_yaml(content: &str) -> BiomeResult<BiomeManifest> {
        let manifest: BiomeManifest = serde_yaml::from_str(content).map_err(|e| {
            BiomeError::validation_error(format!("Failed to parse YAML manifest: {e}"), vec![])
        })?;

        Self::validate(&manifest)?;
        Ok(manifest)
    }

    /// Save manifest to YAML string
    pub fn save_to_yaml(manifest: &BiomeManifest) -> BiomeResult<String> {
        Self::validate(manifest)?;

        serde_yaml::to_string(manifest).map_err(|e| {
            BiomeError::validation_error(
                format!("Failed to serialize manifest to YAML: {e}"),
                vec![],
            )
        })
    }
}

/// Manifest template generator for common use cases
pub struct BiomeManifestTemplates;

impl BiomeManifestTemplates {
    /// Generate a basic web application manifest
    #[must_use]
    pub fn web_application(name: &str, image: &str) -> BiomeManifest {
        use biomeos_types::{
            Environment, ManifestMetadata,
            manifest::{BiomeSpec, BiomeType},
        };
        use chrono::Utc;
        use std::collections::HashMap;

        BiomeManifest {
            metadata: ManifestMetadata {
                name: format!("{name}-biome"),
                version: "1.0.0".to_string(),
                api_version: "biomeOS/v1".to_string(),
                kind: "BiomeManifest".to_string(),
                description: Some(format!("Web application biome for {name}")),
                author: Some("BiomeOS Templates".to_string()),
                license: Some("MIT".to_string()),
                repository: None,
                documentation: None,
                tags: vec!["web".to_string(), "application".to_string()],
                labels: HashMap::new(),
                annotations: HashMap::new(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                namespace: None,
            },
            spec: BiomeSpec {
                biome_type: BiomeType::Application {
                    app_type: "web".to_string(),
                    framework: Some(image.to_string()),
                },
                environment: Environment::Production,
                resources: None,
                health: None,
                scaling: None,
                config: {
                    let mut config = HashMap::new();
                    config.insert(
                        "image".to_string(),
                        serde_json::Value::String(image.to_string()),
                    );
                    config.insert(
                        "port".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(
                            biomeos_types::constants::network::DEFAULT_HTTP_PORT,
                        )),
                    );
                    config
                },
                lifecycle: None,
                networking: None,
                security: None,
            },
            services: HashMap::new(),
            networks: HashMap::new(),
            volumes: HashMap::new(),
            secrets: HashMap::new(),
            configs: HashMap::new(),
            dependencies: vec![],
        }
    }

    /// Generate a database service manifest
    #[must_use]
    pub fn database(name: &str, db_type: &str, volume_size_gb: f64) -> BiomeManifest {
        use biomeos_types::{
            Environment, ManifestMetadata,
            manifest::{BiomeSpec, BiomeType},
        };
        use chrono::Utc;
        use std::collections::HashMap;

        BiomeManifest {
            metadata: ManifestMetadata {
                name: format!("{name}-db-biome"),
                version: "1.0.0".to_string(),
                api_version: "biomeOS/v1".to_string(),
                kind: "BiomeManifest".to_string(),
                description: Some(format!("{db_type} database biome for {name}")),
                author: Some("BiomeOS Templates".to_string()),
                license: Some("MIT".to_string()),
                repository: None,
                documentation: None,
                tags: vec!["database".to_string(), db_type.to_string()],
                labels: HashMap::new(),
                annotations: HashMap::new(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                namespace: None,
            },
            spec: BiomeSpec {
                biome_type: BiomeType::Service {
                    service_type: "database".to_string(),
                    protocol: Some(db_type.to_string()),
                },
                environment: Environment::Production,
                resources: None,
                health: None,
                scaling: None,
                config: {
                    let mut config = HashMap::new();
                    config.insert(
                        "database_type".to_string(),
                        serde_json::Value::String(db_type.to_string()),
                    );
                    config.insert(
                        "volume_size_gb".to_string(),
                        serde_json::Value::Number(
                            serde_json::Number::from_f64(volume_size_gb)
                                .unwrap_or_else(|| serde_json::Number::from(10)),
                        ),
                    );
                    config
                },
                lifecycle: None,
                networking: None,
                security: None,
            },
            services: HashMap::new(),
            networks: HashMap::new(),
            volumes: HashMap::new(),
            secrets: HashMap::new(),
            configs: HashMap::new(),
            dependencies: vec![],
        }
    }
}

/// Analysis tools for biome manifests
pub struct ManifestAnalyzer;

impl ManifestAnalyzer {
    /// Get all capabilities required by services in the manifest
    #[must_use]
    pub fn get_manifest_capabilities(manifest: &BiomeManifest) -> Vec<String> {
        let mut capabilities = Vec::new();

        // BiomeManifest.services is a HashMap<String, ServiceSpec> at the top level
        for service in manifest.services.values() {
            // Convert PrimalCapability to String for the return type
            let service_caps: Vec<String> = service
                .metadata
                .capabilities
                .iter()
                .map(|cap| format!("{cap:?}")) // Convert PrimalCapability to String representation
                .collect();
            capabilities.extend(service_caps);
        }

        capabilities.sort();
        capabilities.dedup();

        capabilities
    }

    /// Check if manifest has security requirements
    #[must_use]
    pub fn has_security_requirements(manifest: &BiomeManifest) -> bool {
        // Check biome-level security
        manifest.spec.security.is_some() ||
        // Check service-level security - BiomeManifest.services is at top level
        manifest.services.iter().any(|(_name, s)| s.security.is_some())
    }

    /// Get all services that have specific capabilities
    #[must_use]
    pub fn get_services_with_capabilities(
        manifest: &BiomeManifest,
        required_caps: &[String],
    ) -> Vec<String> {
        let mut matching_services = Vec::new();

        // BiomeManifest.services is at top level
        for (service_name, service) in &manifest.services {
            // Convert PrimalCapability to String for comparison
            let service_caps: Vec<String> = service
                .metadata
                .capabilities
                .iter()
                .map(|cap| format!("{cap:?}"))
                .collect();
            if required_caps.iter().any(|cap| service_caps.contains(cap)) {
                matching_services.push(service_name.clone());
            }
        }

        matching_services
    }

    /// Get all exposed ports from the manifest
    #[must_use]
    pub fn get_exposed_ports(manifest: &BiomeManifest) -> Vec<u16> {
        let mut ports = Vec::new();

        // BiomeManifest.services is at top level
        for service in manifest.services.values() {
            for port in &service.ports {
                ports.push(port.port);
            }
        }

        ports.sort_unstable();
        ports.dedup();
        ports
    }

    /// Check if the manifest has any security policies defined
    #[must_use]
    pub fn has_security_policies(manifest: &BiomeManifest) -> bool {
        manifest.spec.security.is_some()
            || manifest
                .services
                .iter()
                .any(|(_name, s)| s.security.is_some())
    }

    /// Get service dependency graph
    #[must_use]
    pub fn get_dependency_graph(manifest: &BiomeManifest) -> HashMap<String, Vec<String>> {
        let mut graph = HashMap::new();

        for (service_name, service) in &manifest.services {
            let dependencies: Vec<String> = service
                .depends_on
                .iter()
                .map(|dep| dep.service.clone())
                .collect();
            graph.insert(service_name.clone(), dependencies);
        }

        graph
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use biomeos_types::manifest::service::{
        DependencyCondition, ImagePullPolicy, ImageSpec, PortProtocol, PortSpec, RestartPolicy,
        ServiceDependency, ServiceMetadata, ServiceSpec,
    };
    use biomeos_types::primal::PrimalCapability;
    use tempfile::TempDir;

    fn sample_service_spec(name: &str, empty_name: bool, port: u16) -> ServiceSpec {
        ServiceSpec {
            metadata: ServiceMetadata {
                name: if empty_name {
                    String::new()
                } else {
                    name.to_string()
                },
                description: None,
                version: "1".to_string(),
                labels: std::collections::HashMap::new(),
                annotations: std::collections::HashMap::new(),
                primal_type: None,
                capabilities: vec![PrimalCapability::new("compute", "gpu", "1.0")],
            },
            image: ImageSpec::Container {
                name: "img".to_string(),
                tag: "t".to_string(),
                registry: None,
                pull_policy: ImagePullPolicy::IfNotPresent,
                pull_secrets: vec![],
            },
            ports: vec![PortSpec {
                name: "http".to_string(),
                port,
                target_port: None,
                protocol: PortProtocol::Tcp,
                expose: true,
                load_balancer: None,
                health_check: None,
            }],
            environment: std::collections::HashMap::new(),
            volumes: vec![],
            resources: None,
            health_checks: vec![],
            depends_on: vec![ServiceDependency {
                service: "other".to_string(),
                condition: DependencyCondition::ServiceStarted,
                restart: false,
            }],
            config: std::collections::HashMap::new(),
            scaling: None,
            security: Some(serde_json::json!({ "level": "strict" })),
            restart_policy: RestartPolicy::Always,
            deployment: None,
        }
    }

    #[test]
    fn test_manifest_validation_success() {
        let manifest = BiomeManifest::default();
        assert!(BiomeManifestProcessor::validate(&manifest).is_ok());
    }

    #[test]
    fn test_manifest_validation_empty_name() {
        let mut manifest = BiomeManifest::default();
        manifest.metadata.name = String::new();
        assert!(BiomeManifestProcessor::validate(&manifest).is_err());
    }

    #[test]
    fn test_web_application_template() {
        let manifest = BiomeManifestTemplates::web_application("my-app", "nginx");
        assert_eq!(manifest.metadata.name, "my-app-biome");
        // Services are empty in this template - config is in spec.config
        assert!(manifest.services.is_empty());
    }

    #[test]
    fn test_database_template() {
        let manifest = BiomeManifestTemplates::database("my-db", "postgres", 10.0);
        assert_eq!(manifest.metadata.name, "my-db-db-biome");
        // Services are empty in this template - config is in spec.config
        assert!(manifest.services.is_empty());
        assert!(BiomeManifestProcessor::validate(&manifest).is_ok());
    }

    #[test]
    fn test_manifest_round_trip() {
        let original = BiomeManifestTemplates::web_application("test-app", "nginx");
        let yaml = BiomeManifestProcessor::save_to_yaml(&original).unwrap();
        let loaded = BiomeManifestProcessor::load_from_yaml(&yaml).unwrap();

        assert_eq!(original.metadata.name, loaded.metadata.name);
        assert_eq!(original.services.len(), loaded.services.len());
    }

    #[test]
    fn test_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test-manifest.yaml");

        let manifest = BiomeManifestTemplates::web_application("file-test", "nginx");

        // Save to file
        BiomeManifestProcessor::save_to_file(&manifest, &file_path).unwrap();

        // Load from file
        let loaded = BiomeManifestProcessor::load_from_file(&file_path).unwrap();

        assert_eq!(manifest.metadata.name, loaded.metadata.name);
    }

    #[test]
    fn test_capability_analysis() {
        let manifest = BiomeManifestTemplates::web_application("test", "nginx");
        let capabilities = ManifestAnalyzer::get_manifest_capabilities(&manifest);

        // Default template doesn't have capabilities, but this tests the function
        assert!(capabilities.is_empty());
    }

    #[test]
    fn test_exposed_ports_analysis() {
        let manifest = BiomeManifestTemplates::web_application("test", "nginx");
        let ports = ManifestAnalyzer::get_exposed_ports(&manifest);

        // Template creates empty services, so no exposed ports
        assert!(ports.is_empty());
    }

    #[test]
    fn test_manifest_validation_empty_dependency_name() {
        let mut manifest = BiomeManifest::default();
        manifest.metadata.name = "test".to_string();
        manifest.dependencies = vec![
            biomeos_types::manifest::manifest_extensions::BiomeDependency {
                name: String::new(),
                version: None,
                optional: false,
                source: biomeos_types::manifest::manifest_extensions::DependencySource::Local {
                    path: "/tmp".to_string(),
                },
            },
        ];
        let result = BiomeManifestProcessor::validate(&manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_manifest_validation_empty_network_name() {
        let mut manifest = BiomeManifest::default();
        manifest.metadata.name = "test".to_string();
        manifest.networks.insert(
            String::new(),
            biomeos_types::manifest::networking_core::NetworkSpec::default(),
        );
        let result = BiomeManifestProcessor::validate(&manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_yaml_invalid() {
        let result = BiomeManifestProcessor::load_from_yaml("not: valid: yaml: [");
        assert!(result.is_err());
    }

    #[test]
    fn test_has_security_requirements_false() {
        let manifest = BiomeManifestTemplates::web_application("test", "nginx");
        assert!(!ManifestAnalyzer::has_security_requirements(&manifest));
    }

    #[test]
    fn test_has_security_policies_false() {
        let manifest = BiomeManifestTemplates::web_application("test", "nginx");
        assert!(!ManifestAnalyzer::has_security_policies(&manifest));
    }

    #[test]
    fn test_get_dependency_graph_empty() {
        let manifest = BiomeManifestTemplates::web_application("test", "nginx");
        let graph = ManifestAnalyzer::get_dependency_graph(&manifest);
        assert!(graph.is_empty());
    }

    #[test]
    fn test_get_services_with_capabilities_empty() {
        let manifest = BiomeManifestTemplates::web_application("test", "nginx");
        let services =
            ManifestAnalyzer::get_services_with_capabilities(&manifest, &["compute".to_string()]);
        assert!(services.is_empty());
    }

    #[test]
    fn test_database_template_volume_size() {
        let manifest = BiomeManifestTemplates::database("app", "postgres", 25.5);
        assert!(manifest.spec.config.contains_key("volume_size_gb"));
        assert!(manifest.spec.config.contains_key("database_type"));
    }

    #[test]
    fn test_load_from_file_nonexistent() {
        let result = BiomeManifestProcessor::load_from_file("/nonexistent/manifest.yaml");
        assert!(result.is_err());
    }

    #[test]
    fn test_save_to_file_invalid_path() {
        let manifest = BiomeManifestTemplates::web_application("test", "nginx");
        let result = BiomeManifestProcessor::save_to_file(
            &manifest,
            "/nonexistent/readonly/path/manifest.yaml",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_service_metadata_empty_name_fails() {
        let mut m = BiomeManifest::default();
        m.services.insert(
            "svc".to_string(),
            sample_service_spec("ignored", true, 8080),
        );
        let err = BiomeManifestProcessor::validate(&m).unwrap_err();
        assert!(err.to_string().contains("svc"));
    }

    #[test]
    fn test_validation_service_port_zero_fails() {
        let mut m = BiomeManifest::default();
        m.services
            .insert("web".to_string(), sample_service_spec("web", false, 0));
        assert!(BiomeManifestProcessor::validate(&m).is_err());
    }

    #[test]
    fn test_manifest_analyzer_ports_caps_security_deps() {
        let cap = PrimalCapability::new("compute", "gpu", "1.0");
        let cap_dbg = format!("{cap:?}");
        let mut m = BiomeManifest::default();
        m.services
            .insert("api".to_string(), sample_service_spec("api", false, 8443));

        let ports = ManifestAnalyzer::get_exposed_ports(&m);
        assert_eq!(ports, vec![8443]);
        let caps = ManifestAnalyzer::get_manifest_capabilities(&m);
        assert!(caps.iter().any(|c| c.contains("compute")));
        assert!(ManifestAnalyzer::has_security_requirements(&m));
        assert!(ManifestAnalyzer::has_security_policies(&m));

        let names = ManifestAnalyzer::get_services_with_capabilities(&m, &[cap_dbg]);
        assert_eq!(names, vec!["api".to_string()]);

        let g = ManifestAnalyzer::get_dependency_graph(&m);
        assert_eq!(g.get("api").cloned().unwrap(), vec!["other".to_string()]);
    }
}
