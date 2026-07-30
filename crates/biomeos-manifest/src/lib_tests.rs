
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
    let result =
        BiomeManifestProcessor::save_to_file(&manifest, "/nonexistent/readonly/path/manifest.yaml");
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
