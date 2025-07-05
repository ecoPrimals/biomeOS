use biomeos_manifest::*;

#[test]
fn test_basic_manifest_creation() {
    let manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: "test-biome".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test biome".to_string()),
            tags: None,
            author: None,
            created: None,
            specialization: None,
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
        primals: std::collections::HashMap::new(),
        services: std::collections::HashMap::new(),
        mycorrhiza: MycorrhizaConfig::default(),
        volumes: std::collections::HashMap::new(),
        networks: std::collections::HashMap::new(),
        networking: None,
        security: None,
        resources: None,
        schedules: std::collections::HashMap::new(),
        environments: std::collections::HashMap::new(),
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
}

#[test]
fn test_primal_creation() {
    let primal = PrimalSpec {
        enabled: true,
        primal_type: PrimalType::BearDog,
        priority: 100,
        version: None,
        source: None,
        depends_on: vec![],
        startup_timeout: None,
        config: None,
        networking: None,
        resources: None,
        extensions: None,
    };

    assert_eq!(primal.enabled, true);
    assert_eq!(primal.primal_type, PrimalType::BearDog);
    assert_eq!(primal.priority, 100);
}

#[test]
fn test_serialization() {
    let manifest = BiomeManifest {
        api_version: "v1".to_string(),
        kind: "Biome".to_string(),
        metadata: ManifestMetadata {
            name: "test-biome".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test biome".to_string()),
            tags: None,
            author: None,
            created: None,
            specialization: None,
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
        primals: std::collections::HashMap::new(),
        services: std::collections::HashMap::new(),
        mycorrhiza: MycorrhizaConfig::default(),
        volumes: std::collections::HashMap::new(),
        networks: std::collections::HashMap::new(),
        networking: None,
        security: None,
        resources: None,
        schedules: std::collections::HashMap::new(),
        environments: std::collections::HashMap::new(),
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

    let serialized = serde_json::to_string(&manifest);
    assert!(serialized.is_ok());

    let deserialized: Result<BiomeManifest, _> = serde_json::from_str(&serialized.unwrap());
    assert!(deserialized.is_ok());
}

#[test]
fn test_energy_flow_states() {
    let closed = EnergyFlowState::Closed;
    let private = EnergyFlowState::PrivateOpen;
    let commercial = EnergyFlowState::CommercialOpen;

    assert_eq!(closed, EnergyFlowState::Closed);
    assert_eq!(private, EnergyFlowState::PrivateOpen);
    assert_eq!(commercial, EnergyFlowState::CommercialOpen);
}

#[test]
fn test_primal_types() {
    let beardog = PrimalType::BearDog;
    let songbird = PrimalType::Songbird;
    let nestgate = PrimalType::NestGate;

    assert_eq!(beardog, PrimalType::BearDog);
    assert_eq!(songbird, PrimalType::Songbird);
    assert_eq!(nestgate, PrimalType::NestGate);
}

#[test]
fn test_default_functions() {
    assert_eq!(default_api_version(), "v1");
    assert_eq!(default_kind(), "Biome");
    assert_eq!(default_true(), true);
    assert_eq!(default_priority(), 100);
} 