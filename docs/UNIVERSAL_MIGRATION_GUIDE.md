# Universal BiomeOS Migration Guide

## Overview

This guide helps you migrate from the hard-coded Primal-specific approach to the new universal, capability-based architecture. The universal approach makes biomeOS truly agnostic to specific Primal implementations while maintaining all functionality.

## Key Architectural Changes

### Before: Hard-Coded Primal Dependencies
```yaml
# Old approach - hard-coded primal names
primals:
  toadstool:
    enabled: true
    source: "ecoprimals/toadstool:v1.0.0"
  songbird:
    enabled: true
    source: "ecoprimals/songbird:v1.3.0"
  nestgate:
    enabled: true
    source: "ecoprimals/nestgate:v1.1.0"
  beardog:
    enabled: true
    source: "ecoprimals/beardog:v1.2.0"
  squirrel:
    enabled: true
    source: "ecoprimals/squirrel:v1.0.0"
```

### After: Universal Capability-Based Requirements
```yaml
# New approach - capability-based requirements
apiVersion: biomeOS/v1
kind: Biome
requirements:
  required:
    - capability: compute.container_orchestration
      min_version: 1.0.0
    - capability: networking.service_discovery
      min_version: 1.0.0
    - capability: storage.persistent_volumes
      min_version: 1.0.0
    - capability: security.authentication
      min_version: 1.0.0
    - capability: ai.model_inference
      min_version: 1.0.0
  optional:
    - capability: monitoring.metrics_collection
      min_version: 1.0.0
    - capability: storage.caching
      min_version: 1.0.0
```

## Migration Steps

### 1. Update Dependencies

Replace hard-coded primal imports with universal interfaces:

```rust
// Before
use biomeos_core::{
    ToadStoolManifest, SongBirdConfig, NestGateConfig, 
    BearDogConfig, SquirrelConfig, PrimalConfig
};

// After
use biomeos_core::{
    UniversalBiomeManifest, UniversalPrimalProvider, 
    CapabilityRequirement, UniversalBiomeCoordinator
};
```

### 2. Convert Manifest Structure

Replace primal-specific manifests with universal manifests:

```rust
// Before: ToadStool-specific manifest
let manifest = ToadStoolManifest {
    name: "my-biome".to_string(),
    primals: {
        let mut primals = HashMap::new();
        primals.insert("toadstool".to_string(), PrimalConfig { ... });
        primals.insert("songbird".to_string(), PrimalConfig { ... });
        primals
    },
    services: vec![...],
    // ... other fields
};

// After: Universal manifest
let manifest = UniversalBiomeManifest {
    api_version: "biomeOS/v1".to_string(),
    kind: "Biome".to_string(),
    metadata: UniversalBiomeMetadata {
        name: "my-biome".to_string(),
        description: "My universal biome".to_string(),
        version: "1.0.0".to_string(),
        // ... other metadata
    },
    requirements: BiomeRequirements {
        required: vec![
            CapabilityRequirement {
                capability: "compute.container_orchestration".to_string(),
                min_version: "1.0.0".to_string(),
                // ... other fields
            },
            // ... other requirements
        ],
        // ... other requirement types
    },
    // ... other fields
};
```

### 3. Update Primal Discovery

Replace hard-coded primal lookups with capability-based discovery:

```rust
// Before: Hard-coded primal lookup
let toadstool_primal = primals.get("toadstool").unwrap();
let songbird_primal = primals.get("songbird").unwrap();

// After: Capability-based discovery
let coordinator = UniversalBiomeCoordinator::new();
let discovered_primals = coordinator.discover_primals().await?;

// Find primals by capability
let container_orchestrators = discovered_primals
    .iter()
    .filter(|p| p.capabilities.iter().any(|c| c.name == "compute.container_orchestration"))
    .collect::<Vec<_>>();

let service_discovery_providers = discovered_primals
    .iter()
    .filter(|p| p.capabilities.iter().any(|c| c.name == "networking.service_discovery"))
    .collect::<Vec<_>>();
```

### 4. Update Service Configuration

Replace primal-specific service configurations:

```rust
// Before: Hard-coded primal assignments
let services = vec![
    ServiceConfig {
        name: "web-app".to_string(),
        primal: "toadstool".to_string(),  // Hard-coded
        runtime: RuntimeType::Container,
        // ... other fields
    },
    ServiceConfig {
        name: "discovery".to_string(),
        primal: "songbird".to_string(),  // Hard-coded
        runtime: RuntimeType::Native,
        // ... other fields
    },
];

// After: Capability-based service requirements
let services = vec![
    ServiceDefinition {
        name: "web-app".to_string(),
        description: "Web application service".to_string(),
        service_type: "web".to_string(),
        required_capabilities: vec![
            CapabilityRequirement {
                capability: "compute.container_orchestration".to_string(),
                min_version: "1.0.0".to_string(),
                // ... other fields
            }
        ],
        config: ServiceConfig {
            source: "myapp/web:v1.0.0".to_string(),
            runtime: RuntimeSpec {
                runtime_type: UniversalRuntimeType::Container,
                version: Some("latest".to_string()),
                options: HashMap::new(),
            },
            // ... other fields
        },
        // ... other fields
    },
];
```

### 5. Update Deployment Logic

Replace primal-specific deployment with universal coordination:

```rust
// Before: Hard-coded primal deployment
async fn deploy_biome(manifest: &ToadStoolManifest) -> Result<(), BiomeError> {
    // Start toadstool
    let toadstool = ToadStoolPrimal::new(manifest.primals.get("toadstool").unwrap());
    toadstool.start().await?;
    
    // Start songbird
    let songbird = SongBirdPrimal::new(manifest.primals.get("songbird").unwrap());
    songbird.start().await?;
    
    // Start other primals...
    
    Ok(())
}

// After: Universal deployment
async fn deploy_biome(manifest: &UniversalBiomeManifest) -> Result<EcosystemInstance, BiomeError> {
    let coordinator = UniversalBiomeCoordinator::new();
    let instance = coordinator.bootstrap_ecosystem(manifest.clone()).await?;
    Ok(instance)
}
```

## Capability Mapping

Map existing primal-specific functionality to universal capabilities:

### ToadStool → Compute Capabilities
```rust
// Before: ToadStool-specific
"toadstool" => {
    container_orchestration: true,
    wasm_runtime: true,
    process_management: true,
}

// After: Universal capabilities
vec![
    "compute.container_orchestration",
    "compute.wasm_runtime", 
    "compute.process_management",
]
```

### SongBird → Networking Capabilities
```rust
// Before: SongBird-specific
"songbird" => {
    service_discovery: true,
    load_balancing: true,
    federation: true,
}

// After: Universal capabilities
vec![
    "networking.service_discovery",
    "networking.load_balancing",
    "networking.federation",
]
```

### NestGate → Storage Capabilities
```rust
// Before: NestGate-specific
"nestgate" => {
    persistent_volumes: true,
    object_storage: true,
    backup: true,
}

// After: Universal capabilities
vec![
    "storage.persistent_volumes",
    "storage.object_storage",
    "storage.backup",
]
```

### BearDog → Security Capabilities
```rust
// Before: BearDog-specific
"beardog" => {
    authentication: true,
    authorization: true,
    encryption: true,
}

// After: Universal capabilities
vec![
    "security.authentication",
    "security.authorization", 
    "security.encryption",
]
```

### Squirrel → AI Capabilities
```rust
// Before: Squirrel-specific
"squirrel" => {
    model_inference: true,
    model_training: true,
    embeddings: true,
}

// After: Universal capabilities
vec![
    "ai.model_inference",
    "ai.model_training",
    "ai.embeddings",
]
```

## Creating Custom Primal Implementations

To create a custom primal that works with the universal system:

```rust
use biomeos_core::{
    UniversalPrimalProvider, Capability, CapabilityCategory, 
    PrimalMetadata, CapabilityRequest, CapabilityResponse
};

pub struct MyCustomPrimal {
    id: String,
    capabilities: Vec<Capability>,
    // ... other fields
}

#[async_trait::async_trait]
impl UniversalPrimalProvider for MyCustomPrimal {
    fn primal_id(&self) -> &str {
        &self.id
    }
    
    fn primal_type(&self) -> &str {
        "my_custom_orchestrator"  // Not limited to 5 names!
    }
    
    fn metadata(&self) -> PrimalMetadata {
        PrimalMetadata {
            name: "My Custom Orchestrator".to_string(),
            description: "A custom container orchestrator".to_string(),
            version: "1.0.0".to_string(),
            maintainer: Some("My Team".to_string()),
            tags: HashMap::new(),
        }
    }
    
    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability {
                name: "compute.container_orchestration".to_string(),
                version: "1.0.0".to_string(),
                description: "Container orchestration capability".to_string(),
                category: CapabilityCategory::Compute,
                parameters: HashMap::new(),
                performance: PerformanceSpec {
                    latency_ms: Some((10, 100)),
                    throughput: Some("1000 containers/min".to_string()),
                    // ... other performance specs
                },
                dependencies: vec![],
            },
            // ... other capabilities
        ]
    }
    
    async fn handle_capability_request(
        &self, 
        request: CapabilityRequest
    ) -> BiomeResult<CapabilityResponse> {
        match request.capability.as_str() {
            "compute.container_orchestration" => {
                // Handle container orchestration request
                self.handle_container_request(request).await
            },
            _ => {
                Err(BiomeError::RuntimeError(
                    format!("Unsupported capability: {}", request.capability)
                ))
            }
        }
    }
    
    // ... implement other required methods
}
```

## Benefits of the Universal Approach

### 1. **Primal Agnostic**
- Same biome can run on any primal implementation
- No vendor lock-in
- Easy to switch between implementations

### 2. **Extensible**
- Add new primal types without code changes
- Support custom implementations
- Community can contribute new primals

### 3. **Capability-Driven**
- Focus on what you need, not how it's implemented
- Better resource matching
- Clearer requirements

### 4. **Network Effects**
- Leverage existing ecosystem
- Bootstrap with reference implementations
- Gradually replace with custom implementations

## Migration Checklist

- [ ] Update imports to use universal types
- [ ] Convert ToadStoolManifest to UniversalBiomeManifest
- [ ] Replace hard-coded primal names with capability requirements
- [ ] Update service configurations to use capability-based requirements
- [ ] Replace primal-specific deployment logic with UniversalBiomeCoordinator
- [ ] Update tests to work with universal interfaces
- [ ] Update documentation to reflect capability-based approach
- [ ] Create custom primal implementations if needed
- [ ] Test with different primal implementations

## Backward Compatibility

The universal approach maintains backward compatibility through:

1. **Reference Implementations**: The original primals (toadstool, songbird, etc.) can implement the UniversalPrimalProvider trait
2. **Capability Mapping**: Old primal names can be mapped to capability providers
3. **Migration Layer**: Temporary adapter layer for gradual migration

## Testing Strategy

1. **Unit Tests**: Test capability matching logic
2. **Integration Tests**: Test with multiple primal implementations
3. **Migration Tests**: Test migration from old to new approach
4. **Custom Primal Tests**: Test with custom implementations

## Performance Considerations

- **Discovery Caching**: Cache discovered primals to avoid repeated lookups
- **Capability Indexing**: Index capabilities for fast matching
- **Lazy Loading**: Load primal implementations on demand
- **Resource Pooling**: Pool primal client connections

## Troubleshooting

### Common Issues

1. **Capability Not Found**: Ensure required capabilities are provided by available primals
2. **Version Mismatch**: Check capability version requirements
3. **Performance Issues**: Verify capability performance specifications
4. **Configuration Errors**: Validate universal manifest structure

### Debug Tools

- Use `UniversalBiomeCoordinator::discover_primals()` to see available primals
- Check `manifest.validate()` for validation errors
- Use capability router for request routing debugging

## Future Enhancements

- **Capability Marketplace**: Registry of available capabilities
- **Auto-Discovery**: Automatic primal discovery on network
- **Performance Profiling**: Capability performance monitoring
- **Multi-Cloud Support**: Cross-cloud capability providers

## Conclusion

The universal approach transforms biomeOS from a hard-coded system into a truly agnostic platform. By focusing on capabilities rather than specific implementations, we enable:

- **Flexibility**: Use any primal implementation
- **Extensibility**: Easy to add new primals
- **Community**: Open ecosystem for contributions
- **Future-Proofing**: Adapt to new technologies

The migration process is straightforward and maintains all existing functionality while opening up new possibilities for customization and extension. 