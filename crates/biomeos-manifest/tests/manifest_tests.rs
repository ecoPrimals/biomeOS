//! Unit tests for the biomeos-manifest module

use biomeos_manifest::*;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_biome_manifest_creation() {
    let manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: "test-biome".to_string(),
            version: "1.0.0".to_string(),
            description: Some("A test biome".to_string()),
            tags: Some(vec!["test".to_string(), "development".to_string()]),
            author: Some("Test Author".to_string()),
            created: Some(chrono::Utc::now()),
            specialization: Some(BiomeSpecialization::AiResearch),
            repository: Some("https://github.com/test/biome".to_string()),
            license: Some("MIT".to_string()),
            created_by: Some("testuser".to_string()),
            forked_from: None,
            sharing: None,
            niches: None,
            template: None,
            custom: None,
        },
        sources: SourceConfig::default(),
        primals: HashMap::new(),
        services: HashMap::new(),
        mycorrhiza: MycorrhizaConfig::default(),
        volumes: HashMap::new(),
        networks: HashMap::new(),
        networking: None,
        security: None,
        resources: None,
        schedules: HashMap::new(),
        environments: HashMap::new(),
        dependencies: None,
        extensions: None,
        biomes: None,
        nested_biomes: None,
        topology: None,
        iterative: None,
        templates: None,
        monitoring: None,
        scaling: None,
    };

    assert_eq!(manifest.api_version, "v1");
    assert_eq!(manifest.kind, "Biome");
    assert_eq!(manifest.metadata.name, "test-biome");
    assert_eq!(manifest.metadata.version, "1.0.0");
    assert!(manifest.metadata.description.is_some());
    assert!(manifest.metadata.tags.is_some());
    assert!(manifest.metadata.author.is_some());
}

#[test]
fn test_primal_spec_creation() {
    let primal = PrimalSpec {
        enabled: true,
        primal_type: PrimalType::Songbird,
        priority: 10,
        version: Some("1.0.0".to_string()),
        source: Some(SourceSpec {
            source_type: SourceType::Git,
            location: "https://github.com/biomeos/songbird".to_string(),
            version: Some("v1.0.0".to_string()),
            auth: None,
            build_command: None,
            watch: false,
            checksum: None,
        }),
        depends_on: vec!["toadstool".to_string()],
        startup_timeout: Some("30s".to_string()),
        config: Some(json!({"port": 8080})),
        networking: None,
        resources: None,
        extensions: None,
    };

    assert!(primal.enabled);
    assert!(matches!(primal.primal_type, PrimalType::Songbird));
    assert_eq!(primal.priority, 10);
    assert_eq!(primal.version, Some("1.0.0".to_string()));
    assert!(primal.source.is_some());
    assert_eq!(primal.depends_on.len(), 1);
    assert_eq!(primal.depends_on[0], "toadstool");
}

#[test]
fn test_service_spec_creation() {
    let service = ServiceSpec {
        runtime: RuntimeType::Container,
        source: Some(SourceSpec {
            source_type: SourceType::Container,
            location: "nginx:latest".to_string(),
            version: None,
            auth: None,
            build_command: None,
            watch: false,
            checksum: None,
        }),
        primal: "toadstool".to_string(),
        image: Some("nginx:latest".to_string()),
        depends_on: vec!["database".to_string()],
        ports: vec!["80:80".to_string(), "443:443".to_string()],
        volumes: vec!["/data:/var/www".to_string()],
        environment: {
            let mut env = HashMap::new();
            env.insert("NODE_ENV".to_string(), "production".to_string());
            env
        },
        config: Some(json!({"worker_processes": 4})),
    };

    assert!(matches!(service.runtime, RuntimeType::Container));
    assert!(service.source.is_some());
    assert_eq!(service.primal, "toadstool");
    assert_eq!(service.depends_on.len(), 1);
    assert_eq!(service.ports.len(), 2);
    assert_eq!(service.volumes.len(), 1);
    assert_eq!(service.environment.len(), 1);
}

#[test]
fn test_primal_types() {
    let beardog = PrimalType::BearDog;
    let songbird = PrimalType::Songbird;
    let nestgate = PrimalType::NestGate;
    let toadstool = PrimalType::Toadstool;
    let squirrel = PrimalType::Squirrel;
    let custom = PrimalType::Custom("custom-primal".to_string());

    assert!(matches!(beardog, PrimalType::BearDog));
    assert!(matches!(songbird, PrimalType::Songbird));
    assert!(matches!(nestgate, PrimalType::NestGate));
    assert!(matches!(toadstool, PrimalType::Toadstool));
    assert!(matches!(squirrel, PrimalType::Squirrel));
    assert!(matches!(custom, PrimalType::Custom(_)));
}

#[test]
fn test_runtime_types() {
    let container = RuntimeType::Container;
    let wasm = RuntimeType::Wasm;
    let native = RuntimeType::Native;
    let gpu = RuntimeType::GPU;
    let agent = RuntimeType::Agent;

    assert!(matches!(container, RuntimeType::Container));
    assert!(matches!(wasm, RuntimeType::Wasm));
    assert!(matches!(native, RuntimeType::Native));
    assert!(matches!(gpu, RuntimeType::GPU));
    assert!(matches!(agent, RuntimeType::Agent));
}

#[test]
fn test_source_types() {
    let git = SourceType::Git;
    let local = SourceType::Local;
    let http = SourceType::Http;
    let container = SourceType::Container;
    let custom = SourceType::Custom("custom-source".to_string());

    assert!(matches!(git, SourceType::Git));
    assert!(matches!(local, SourceType::Local));
    assert!(matches!(http, SourceType::Http));
    assert!(matches!(container, SourceType::Container));
    assert!(matches!(custom, SourceType::Custom(_)));
}

#[test]
fn test_biome_specializations() {
    let ai_research = BiomeSpecialization::AiResearch;
    let data_science = BiomeSpecialization::DataScience;
    let web_dev = BiomeSpecialization::WebDevelopment;
    let gaming = BiomeSpecialization::Gaming;
    let scientific = BiomeSpecialization::Scientific;
    let enterprise = BiomeSpecialization::Enterprise;
    let edge = BiomeSpecialization::Edge;
    let custom = BiomeSpecialization::Custom("custom-spec".to_string());

    assert!(matches!(ai_research, BiomeSpecialization::AiResearch));
    assert!(matches!(data_science, BiomeSpecialization::DataScience));
    assert!(matches!(web_dev, BiomeSpecialization::WebDevelopment));
    assert!(matches!(gaming, BiomeSpecialization::Gaming));
    assert!(matches!(scientific, BiomeSpecialization::Scientific));
    assert!(matches!(enterprise, BiomeSpecialization::Enterprise));
    assert!(matches!(edge, BiomeSpecialization::Edge));
    assert!(matches!(custom, BiomeSpecialization::Custom(_)));
}

#[test]
fn test_mycorrhiza_config_default() {
    let config = MycorrhizaConfig::default();

    assert!(matches!(config.energy_flow, EnergyFlowState::Closed));
    assert!(config.personal_ai.enabled);
    assert!(config.enforcement.deep_packet_inspection);
    assert!(config.enforcement.api_signature_detection);
    assert!(config.enforcement.behavioral_analysis);
    assert!(matches!(
        config.enforcement.threat_response,
        ThreatResponse::BlockAndPreserve
    ));
}

#[test]
fn test_energy_flow_states() {
    let closed = EnergyFlowState::Closed;
    let private_open = EnergyFlowState::PrivateOpen;
    let commercial_open = EnergyFlowState::CommercialOpen;

    assert!(matches!(closed, EnergyFlowState::Closed));
    assert!(matches!(private_open, EnergyFlowState::PrivateOpen));
    assert!(matches!(commercial_open, EnergyFlowState::CommercialOpen));
}

#[test]
fn test_threat_response_types() {
    let block = ThreatResponse::Block;
    let warn = ThreatResponse::Warn;
    let preserve = ThreatResponse::Preserve;
    let block_preserve = ThreatResponse::BlockAndPreserve;

    assert!(matches!(block, ThreatResponse::Block));
    assert!(matches!(warn, ThreatResponse::Warn));
    assert!(matches!(preserve, ThreatResponse::Preserve));
    assert!(matches!(block_preserve, ThreatResponse::BlockAndPreserve));
}

#[test]
fn test_volume_types() {
    let empty_dir = VolumeType::EmptyDir;
    let host_path = VolumeType::HostPath;
    let config_map = VolumeType::ConfigMap;
    let secret = VolumeType::Secret;

    assert!(matches!(empty_dir, VolumeType::EmptyDir));
    assert!(matches!(host_path, VolumeType::HostPath));
    assert!(matches!(config_map, VolumeType::ConfigMap));
    assert!(matches!(secret, VolumeType::Secret));
}

#[test]
fn test_auth_types() {
    let ssh_key = AuthType::SshKey;
    let token = AuthType::Token;
    let certificate = AuthType::Certificate;
    let oauth = AuthType::OAuth;

    assert!(matches!(ssh_key, AuthType::SshKey));
    assert!(matches!(token, AuthType::Token));
    assert!(matches!(certificate, AuthType::Certificate));
    assert!(matches!(oauth, AuthType::OAuth));
}

#[test]
fn test_repository_types() {
    let git = RepositoryType::Git;
    let mercurial = RepositoryType::Mercurial;
    let svn = RepositoryType::Svn;
    let custom = RepositoryType::Custom("custom-repo".to_string());

    assert!(matches!(git, RepositoryType::Git));
    assert!(matches!(mercurial, RepositoryType::Mercurial));
    assert!(matches!(svn, RepositoryType::Svn));
    assert!(matches!(custom, RepositoryType::Custom(_)));
}

#[test]
fn test_validation_results() {
    let mut results = ValidationResults::new();

    assert!(results.is_valid());
    assert_eq!(results.errors.len(), 0);
    assert_eq!(results.warnings.len(), 0);

    results.add_error("Test error".to_string());
    assert!(!results.is_valid());
    assert_eq!(results.errors.len(), 1);

    results.add_warning("Test warning".to_string());
    assert_eq!(results.warnings.len(), 1);
}

#[test]
fn test_sharing_config() {
    let sharing = SharingConfig {
        public: true,
        license: Some("MIT".to_string()),
        repository: Some("https://github.com/test/biome".to_string()),
        registry: Some("biomeos.registry".to_string()),
        fork_permissions: ForkPermissions::Public,
        attribution_required: true,
    };

    assert!(sharing.public);
    assert!(sharing.license.is_some());
    assert!(sharing.repository.is_some());
    assert!(sharing.registry.is_some());
    assert!(matches!(sharing.fork_permissions, ForkPermissions::Public));
    assert!(sharing.attribution_required);
}

#[test]
fn test_fork_permissions() {
    let public = ForkPermissions::Public;
    let authenticated = ForkPermissions::Authenticated;
    let restricted = ForkPermissions::Restricted(vec!["user1".to_string(), "user2".to_string()]);
    let none = ForkPermissions::None;

    assert!(matches!(public, ForkPermissions::Public));
    assert!(matches!(authenticated, ForkPermissions::Authenticated));
    assert!(matches!(restricted, ForkPermissions::Restricted(_)));
    assert!(matches!(none, ForkPermissions::None));
}

#[test]
fn test_template_metadata() {
    let template = TemplateMetadata {
        is_template: true,
        category: Some("development".to_string()),
        difficulty: Some(TemplateDifficulty::Beginner),
        parameters: vec![TemplateParameter {
            name: "app_name".to_string(),
            description: "Application name".to_string(),
            param_type: ParameterType::String,
            required: true,
            default: None,
            validation: Some(ParameterValidation {
                min: Some(json!(1)),
                max: Some(json!(50)),
                pattern: Some("^[a-zA-Z0-9-]+$".to_string()),
                message: Some("Invalid app name".to_string()),
            }),
        }],
        examples: vec![TemplateExample {
            name: "Basic App".to_string(),
            description: "A basic application".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("app_name".to_string(), json!("my-app"));
                params
            },
        }],
    };

    assert!(template.is_template);
    assert!(template.category.is_some());
    assert!(matches!(
        template.difficulty,
        Some(TemplateDifficulty::Beginner)
    ));
    assert_eq!(template.parameters.len(), 1);
    assert_eq!(template.examples.len(), 1);
    assert_eq!(template.parameters[0].name, "app_name");
    assert!(template.parameters[0].required);
}

#[test]
fn test_template_difficulty_levels() {
    let beginner = TemplateDifficulty::Beginner;
    let intermediate = TemplateDifficulty::Intermediate;
    let advanced = TemplateDifficulty::Advanced;
    let expert = TemplateDifficulty::Expert;

    assert!(matches!(beginner, TemplateDifficulty::Beginner));
    assert!(matches!(intermediate, TemplateDifficulty::Intermediate));
    assert!(matches!(advanced, TemplateDifficulty::Advanced));
    assert!(matches!(expert, TemplateDifficulty::Expert));
}

#[test]
fn test_parameter_types() {
    let string = ParameterType::String;
    let number = ParameterType::Number;
    let boolean = ParameterType::Boolean;
    let array = ParameterType::Array;
    let object = ParameterType::Object;
    let choice = ParameterType::Choice(vec!["option1".to_string(), "option2".to_string()]);

    assert!(matches!(string, ParameterType::String));
    assert!(matches!(number, ParameterType::Number));
    assert!(matches!(boolean, ParameterType::Boolean));
    assert!(matches!(array, ParameterType::Array));
    assert!(matches!(object, ParameterType::Object));
    assert!(matches!(choice, ParameterType::Choice(_)));
}

#[test]
fn test_schedule_config() {
    let schedule = ScheduleConfig {
        active: "0 9 * * MON-FRI".to_string(), // 9 AM weekdays
        timezone: Some("UTC".to_string()),
        config_overrides: Some(json!({"performance_mode": "high"})),
        primal_overrides: {
            let mut overrides = HashMap::new();
            overrides.insert("toadstool".to_string(), json!({"cpu_limit": "80%"}));
            overrides
        },
        service_overrides: {
            let mut overrides = HashMap::new();
            overrides.insert("webserver".to_string(), json!({"replicas": 3}));
            overrides
        },
        enabled: true,
    };

    assert_eq!(schedule.active, "0 9 * * MON-FRI");
    assert!(schedule.timezone.is_some());
    assert!(schedule.config_overrides.is_some());
    assert_eq!(schedule.primal_overrides.len(), 1);
    assert_eq!(schedule.service_overrides.len(), 1);
    assert!(schedule.enabled);
}

#[test]
fn test_environment_config() {
    let environment = EnvironmentConfig {
        description: Some("Production environment".to_string()),
        extends: Some("base".to_string()),
        primals: HashMap::new(),
        services: HashMap::new(),
        environment: {
            let mut env = HashMap::new();
            env.insert("NODE_ENV".to_string(), "production".to_string());
            env.insert("LOG_LEVEL".to_string(), "info".to_string());
            env
        },
        resources: None,
        security: None,
        mycorrhiza: None,
    };

    assert!(environment.description.is_some());
    assert!(environment.extends.is_some());
    assert_eq!(environment.environment.len(), 2);
    assert!(environment.environment.contains_key("NODE_ENV"));
    assert!(environment.environment.contains_key("LOG_LEVEL"));
}

#[test]
fn test_dependency_config() {
    let dependency = DependencyConfig {
        requires: vec![DependencySpec {
            name: "postgres".to_string(),
            version: Some(">=13.0".to_string()),
            source: None,
            optional: false,
            reason: Some("Database storage".to_string()),
        }],
        suggests: vec![DependencySpec {
            name: "redis".to_string(),
            version: Some(">=6.0".to_string()),
            source: None,
            optional: true,
            reason: Some("Caching".to_string()),
        }],
        conflicts: vec![],
        features: {
            let mut features = HashMap::new();
            features.insert(
                "monitoring".to_string(),
                FeatureSpec {
                    description: "Enable monitoring".to_string(),
                    dependencies: vec!["prometheus".to_string()],
                    services: vec!["metrics".to_string()],
                    config: Some(json!({"metrics_port": 9090})),
                    default_enabled: false,
                },
            );
            features
        },
    };

    assert_eq!(dependency.requires.len(), 1);
    assert_eq!(dependency.suggests.len(), 1);
    assert_eq!(dependency.conflicts.len(), 0);
    assert_eq!(dependency.features.len(), 1);
    assert!(dependency.features.contains_key("monitoring"));
}

#[test]
fn test_serialization_deserialization() {
    let mut manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: "test-biome".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test biome".to_string()),
            tags: Some(vec!["test".to_string()]),
            author: Some("Test Author".to_string()),
            created: Some(chrono::Utc::now()),
            specialization: Some(BiomeSpecialization::AiResearch),
            repository: None,
            license: None,
            created_by: None,
            forked_from: None,
            sharing: None,
            niches: None,
            template: None,
            custom: None,
        },
        sources: SourceConfig::default(),
        primals: HashMap::new(),
        services: HashMap::new(),
        mycorrhiza: MycorrhizaConfig::default(),
        volumes: HashMap::new(),
        networks: HashMap::new(),
        networking: None,
        security: None,
        resources: None,
        schedules: HashMap::new(),
        environments: HashMap::new(),
        dependencies: None,
        extensions: None,
        biomes: None,
        nested_biomes: None,
        topology: None,
        iterative: None,
        templates: None,
        monitoring: None,
        scaling: None,
    };

    // Add a primal for testing
    manifest.primals.insert(
        "toadstool".to_string(),
        PrimalSpec {
            enabled: true,
            primal_type: PrimalType::Toadstool,
            priority: 10,
            version: Some("1.0.0".to_string()),
            source: None,
            depends_on: vec![],
            startup_timeout: None,
            config: None,
            networking: None,
            resources: None,
            extensions: None,
        },
    );

    // Test serialization
    let serialized = serde_json::to_string(&manifest).unwrap();
    assert!(serialized.contains("test-biome"));
    assert!(serialized.contains("Toadstool"));

    // Test deserialization
    let deserialized: BiomeManifest = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.metadata.name, "test-biome");
    assert_eq!(deserialized.primals.len(), 1);
    assert!(deserialized.primals.contains_key("toadstool"));
}

#[test]
fn test_complex_biome_specializations() {
    let gaming_server = BiomeSpecialization::GamingServer;
    let biocomputation = BiomeSpecialization::Biocomputation;
    let devops = BiomeSpecialization::DevOps;
    let edge_computing = BiomeSpecialization::EdgeComputing;
    let content_creation = BiomeSpecialization::ContentCreation;
    let dual_purpose = BiomeSpecialization::DualPurpose;
    let dynamic_shift = BiomeSpecialization::DynamicShift;

    assert!(matches!(gaming_server, BiomeSpecialization::GamingServer));
    assert!(matches!(
        biocomputation,
        BiomeSpecialization::Biocomputation
    ));
    assert!(matches!(devops, BiomeSpecialization::DevOps));
    assert!(matches!(edge_computing, BiomeSpecialization::EdgeComputing));
    assert!(matches!(
        content_creation,
        BiomeSpecialization::ContentCreation
    ));
    assert!(matches!(dual_purpose, BiomeSpecialization::DualPurpose));
    assert!(matches!(dynamic_shift, BiomeSpecialization::DynamicShift));
}

#[test]
fn test_topology_patterns() {
    let ring = TopologyPattern::Ring;
    let mesh = TopologyPattern::Mesh;
    let cluster = TopologyPattern::Cluster;
    let hierarchy = TopologyPattern::Hierarchy;
    let singleton = TopologyPattern::Singleton;
    let custom = TopologyPattern::Custom("custom-topology".to_string());

    assert!(matches!(ring, TopologyPattern::Ring));
    assert!(matches!(mesh, TopologyPattern::Mesh));
    assert!(matches!(cluster, TopologyPattern::Cluster));
    assert!(matches!(hierarchy, TopologyPattern::Hierarchy));
    assert!(matches!(singleton, TopologyPattern::Singleton));
    assert!(matches!(custom, TopologyPattern::Custom(_)));
}

#[test]
fn test_default_functions() {
    assert_eq!(default_api_version(), "v1");
    assert_eq!(default_kind(), "Biome");
    assert_eq!(default_true(), true);
    assert_eq!(default_priority(), 100);
}
