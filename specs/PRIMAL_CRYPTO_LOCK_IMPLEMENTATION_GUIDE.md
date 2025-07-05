---
description: Implementation guide for crypto-lock licensing enforcement across all Primals
globs: ["**/*.rs", "**/*.toml", "**/biome.yaml"]
crossRefs:
  - DIGITAL_SOVEREIGNTY_LICENSING.md
  - CROSS_PRIMAL_API_CONTRACTS.md
  - PRIMAL_SERVICE_REGISTRATION_STANDARDS.md
---

# Primal Crypto-Lock Implementation Guide

## Context
- When implementing crypto-lock licensing in individual Primals
- When integrating with BearDog security provider
- When enforcing external dependency access control
- When building local compliance monitoring
- When preparing for biomeOS integration

## Overview

Each Primal must implement crypto-lock enforcement for external dependencies while maintaining digital sovereignty principles. This guide provides specific implementation requirements for each team.

## Universal Requirements (All Primals)

### 1. Sovereign Key Integration
```rust
// All Primals must integrate sovereign key validation
pub trait SovereignKeyIntegration {
    fn validate_sovereign_key(&self, key: &[u8]) -> Result<KeyValidation>;
    fn get_access_level(&self, key: &SovereignKey) -> AccessLevel;
    fn enforce_dependency_access(&self, dep: &str, level: AccessLevel) -> Result<()>;
}
```

### 2. Local Compliance Monitoring
```rust
// No data collection - local monitoring only
pub trait LocalComplianceMonitor {
    fn classify_usage_pattern(&self) -> UsagePattern;
    fn check_compliance_status(&self) -> ComplianceStatus;
    fn handle_compliance_violation(&self, violation: ComplianceViolation);
    
    // FORBIDDEN - no network communication
    // fn report_telemetry() - NOT ALLOWED
    // fn phone_home() - NOT ALLOWED
}
```

### 3. External Dependency Gate
```rust
// Block external dependencies without sovereign key
pub trait ExternalDependencyGate {
    fn is_dependency_locked(&self, dep: &str) -> bool;
    fn require_sovereign_key(&self, dep: &str) -> Result<()>;
    fn block_unauthorized_access(&self, dep: &str) -> Result<()>;
}
```

## Toadstool Implementation Requirements

### Container Runtime Enforcement
```rust
// Toadstool: Container-level access control
pub struct ToadstoolCryptoLock {
    sovereign_validator: SovereignKeyValidator,
    container_gate: ContainerAccessGate,
    manifest_enforcer: BiomeManifestEnforcer,
}

impl ToadstoolCryptoLock {
    pub fn enforce_container_access(&self, container: &Container) -> Result<()> {
        // Check if container uses external dependencies
        let external_deps = self.extract_external_dependencies(container)?;
        
        for dep in external_deps {
            if self.is_crypto_locked(&dep) {
                self.sovereign_validator.require_valid_key()?;
            }
        }
        
        Ok(())
    }
    
    pub fn validate_biome_manifest(&self, manifest: &BiomeManifest) -> Result<()> {
        // Validate licensing configuration in biome.yaml
        let licensing = manifest.licensing.as_ref()
            .ok_or(LicensingConfigurationMissing)?;
        
        self.sovereign_validator.validate_key_path(&licensing.sovereign_key_path)?;
        self.validate_compliance_level(&licensing.compliance_level)?;
        
        Ok(())
    }
}
```

### External Service Discovery Locks
```yaml
# Toadstool: Lock external service discovery
external_service_discovery:
  kubernetes_apis:
    crypto_locked: true
    sovereign_key_required: true
    access_level: "commercial_or_agpl3"
  
  consul_discovery:
    crypto_locked: true
    sovereign_key_required: true
    access_level: "commercial_or_agpl3"
  
  nomad_schedulers:
    crypto_locked: true
    sovereign_key_required: true
    access_level: "commercial_or_agpl3"
```

### BiomeManifest Integration
```yaml
# Toadstool: Extend BiomeManifest with licensing
licensing:
  sovereign_key_path: "/etc/biome/sovereign.key"
  compliance_level: "individual" | "institutional" | "commercial"
  external_dependencies:
    - name: "kubernetes"
      crypto_locked: true
      required_access_level: "commercial"
    - name: "consul"
      crypto_locked: true
      required_access_level: "commercial"
    - name: "docker"
      crypto_locked: false
      required_access_level: "none"
```

## Songbird Implementation Requirements

### Service Discovery Filtering
```rust
// Songbird: Filter service discovery based on sovereign key
pub struct SongbirdCryptoLock {
    sovereign_validator: SovereignKeyValidator,
    discovery_filter: ServiceDiscoveryFilter,
    federation_gate: FederationAccessGate,
}

impl SongbirdCryptoLock {
    pub fn filter_external_registries(&self, registries: &[ServiceRegistry]) -> Vec<ServiceRegistry> {
        registries.iter()
            .filter(|registry| {
                if self.is_external_registry(registry) {
                    self.sovereign_validator.has_valid_key()
                } else {
                    true // Allow internal registries
                }
            })
            .cloned()
            .collect()
    }
    
    pub fn enforce_federation_access(&self, federation: &Federation) -> Result<()> {
        if federation.is_external() {
            self.sovereign_validator.require_commercial_key()?;
        }
        Ok(())
    }
}
```

### External Orchestrator Locks
```yaml
# Songbird: Lock external orchestration systems
external_orchestrators:
  kubernetes:
    crypto_locked: true
    access_requirements: "sovereign_key"
    compliance_level: "commercial_or_agpl3"
    
  nomad:
    crypto_locked: true
    access_requirements: "sovereign_key"
    compliance_level: "commercial_or_agpl3"
    
  docker_swarm:
    crypto_locked: false  # Open source friendly
    access_requirements: "none"
```

### Load Balancer Backend Filtering
```rust
// Songbird: Filter load balancer backends
impl LoadBalancerCryptoLock {
    pub fn validate_backend_access(&self, backend: &Backend) -> Result<()> {
        if self.is_external_backend(backend) && self.is_crypto_locked_service(backend) {
            self.sovereign_validator.require_valid_key()?;
        }
        Ok(())
    }
}
```

## NestGate Implementation Requirements

### Storage Provider Access Control
```rust
// NestGate: Control access to external storage providers
pub struct NestGateCryptoLock {
    sovereign_validator: SovereignKeyValidator,
    storage_gate: StorageProviderGate,
    backup_enforcer: BackupServiceEnforcer,
}

impl NestGateCryptoLock {
    pub fn validate_storage_provider(&self, provider: &StorageProvider) -> Result<()> {
        match provider.provider_type() {
            ProviderType::External => {
                self.sovereign_validator.require_commercial_key()?;
            },
            ProviderType::Local => {
                // Always allowed
            },
            ProviderType::SelfHosted => {
                // Always allowed
            },
        }
        Ok(())
    }
    
    pub fn enforce_backup_compliance(&self, backup: &BackupService) -> Result<()> {
        if backup.is_cloud_service() {
            self.sovereign_validator.require_valid_key()?;
            self.validate_data_sovereignty_compliance(backup)?;
        }
        Ok(())
    }
}
```

### External Object Store Locks
```yaml
# NestGate: Lock external object storage
external_storage_providers:
  aws_s3:
    crypto_locked: true
    sovereign_key_required: true
    compliance_level: "commercial_or_agpl3"
    
  google_cloud_storage:
    crypto_locked: true
    sovereign_key_required: true
    compliance_level: "commercial_or_agpl3"
    
  azure_blob:
    crypto_locked: true
    sovereign_key_required: true
    compliance_level: "commercial_or_agpl3"
    
  minio:
    crypto_locked: false  # Self-hosted friendly
    sovereign_key_required: false
```

### ZFS Integration with Compliance
```rust
// NestGate: ZFS operations with compliance checking
impl ZFSCryptoLock {
    pub fn create_encrypted_dataset(&self, config: &DatasetConfig) -> Result<Dataset> {
        // Check if using external key management
        if config.uses_external_kms() {
            self.sovereign_validator.require_commercial_key()?;
        }
        
        // Proceed with ZFS dataset creation
        self.zfs_manager.create_dataset(config)
    }
}
```

## Squirrel Implementation Requirements

### AI Provider Access Control
```rust
// Squirrel: Control access to external AI providers
pub struct SquirrelCryptoLock {
    sovereign_validator: SovereignKeyValidator,
    ai_provider_gate: AIProviderGate,
    plugin_enforcer: PluginMarketplaceEnforcer,
}

impl SquirrelCryptoLock {
    pub fn validate_ai_provider(&self, provider: &AIProvider) -> Result<()> {
        match provider.provider_type() {
            AIProviderType::External => {
                self.sovereign_validator.require_commercial_key()?;
            },
            AIProviderType::SelfHosted => {
                // Always allowed
            },
            AIProviderType::Local => {
                // Always allowed
            },
        }
        Ok(())
    }
    
    pub fn enforce_plugin_compliance(&self, plugin: &Plugin) -> Result<()> {
        if plugin.is_from_external_marketplace() {
            self.sovereign_validator.require_valid_key()?;
        }
        Ok(())
    }
}
```

### MCP Provider Locks
```yaml
# Squirrel: Lock external MCP providers
external_mcp_providers:
  openai:
    crypto_locked: true
    sovereign_key_required: true
    compliance_level: "commercial_or_agpl3"
    
  anthropic:
    crypto_locked: true
    sovereign_key_required: true
    compliance_level: "commercial_or_agpl3"
    
  google_ai:
    crypto_locked: true
    sovereign_key_required: true
    compliance_level: "commercial_or_agpl3"
    
  ollama:
    crypto_locked: false  # Self-hosted friendly
    sovereign_key_required: false
```

### Plugin Sandbox Enforcement
```rust
// Squirrel: Plugin sandbox with compliance
impl PluginSandboxCryptoLock {
    pub fn validate_plugin_access(&self, plugin: &Plugin, resource: &ExternalResource) -> Result<()> {
        if resource.is_external() && self.is_crypto_locked_resource(resource) {
            self.sovereign_validator.require_valid_key()?;
        }
        Ok(())
    }
}
```

## BearDog Implementation Requirements

### Core Security Provider
```rust
// BearDog: Core crypto-lock security provider
pub struct BearDogCryptoLock {
    key_validator: SovereignKeyValidator,
    crypto_provider: CryptographicProvider,
    access_controller: AccessController,
}

impl BearDogCryptoLock {
    pub fn validate_sovereign_key(&self, key: &[u8]) -> Result<KeyValidation> {
        let signature = self.crypto_provider.verify_signature(key)?;
        let metadata = self.extract_key_metadata(&signature)?;
        
        Ok(KeyValidation {
            valid: true,
            access_level: metadata.access_level,
            expiry: metadata.expiry,
            restrictions: metadata.restrictions,
        })
    }
    
    pub fn enforce_cross_primal_access(&self, 
        source_primal: &str,
        target_dependency: &str,
        key: &SovereignKey
    ) -> Result<AccessGrant> {
        let dependency_requirements = self.get_dependency_requirements(target_dependency)?;
        let key_capabilities = self.extract_key_capabilities(key)?;
        
        if key_capabilities.satisfies(&dependency_requirements) {
            Ok(AccessGrant::new(key_capabilities.access_level))
        } else {
            Err(AccessDenied::InsufficientPrivileges)
        }
    }
}
```

### Cross-Primal Security Coordination
```rust
// BearDog: Coordinate security across all Primals
impl CrossPrimalSecurityCoordinator {
    pub fn register_primal_crypto_locks(&mut self, primal: &str, locks: Vec<CryptoLock>) {
        self.primal_locks.insert(primal.to_string(), locks);
    }
    
    pub fn validate_cross_primal_access(&self, 
        source: &str, 
        target: &str, 
        dependency: &str
    ) -> Result<()> {
        let source_key = self.get_primal_sovereign_key(source)?;
        let target_requirements = self.get_dependency_requirements(target, dependency)?;
        
        self.validate_access_compatibility(source_key, target_requirements)
    }
}
```

## Implementation Checklist by Primal

### Toadstool Team Checklist
- [ ] Container runtime sovereign key validation
- [ ] BiomeManifest licensing configuration
- [ ] External service discovery crypto-locks
- [ ] Container dependency gate implementation
- [ ] Local compliance monitoring (no data collection)
- [ ] Integration with BearDog security provider

### Songbird Team Checklist
- [ ] Service discovery filtering based on sovereign key
- [ ] External orchestrator access control
- [ ] Federation crypto-lock enforcement
- [ ] Load balancer backend filtering
- [ ] Local usage pattern classification
- [ ] Integration with BearDog security provider

### NestGate Team Checklist
- [ ] External storage provider access control
- [ ] Cloud backup service crypto-locks
- [ ] ZFS encryption with compliance checking
- [ ] Data sovereignty validation
- [ ] Storage tiering compliance enforcement
- [ ] Integration with BearDog security provider

### Squirrel Team Checklist
- [ ] AI provider access control implementation
- [ ] External MCP provider crypto-locks
- [ ] Plugin marketplace enforcement
- [ ] Cross-platform sandbox compliance
- [ ] Local AI usage monitoring
- [ ] Integration with BearDog security provider

### BearDog Team Checklist
- [ ] Core sovereign key validation system
- [ ] Cryptographic signature verification
- [ ] Cross-Primal access coordination
- [ ] Key distribution infrastructure
- [ ] Compliance violation handling
- [ ] Security provider API implementation

## Testing Requirements

### Individual Primal Testing
```rust
// Each Primal must implement these tests
#[cfg(test)]
mod crypto_lock_tests {
    #[test]
    fn test_sovereign_key_validation() {
        // Test valid and invalid sovereign keys
    }
    
    #[test]
    fn test_external_dependency_blocking() {
        // Test that external dependencies are blocked without key
    }
    
    #[test]
    fn test_local_compliance_monitoring() {
        // Test usage pattern classification
    }
    
    #[test]
    fn test_beardog_integration() {
        // Test integration with BearDog security provider
    }
}
```

### Cross-Primal Integration Testing
```rust
// Test cross-Primal crypto-lock coordination
#[cfg(test)]
mod integration_tests {
    #[test]
    fn test_cross_primal_access_control() {
        // Test access control between Primals
    }
    
    #[test]
    fn test_unified_compliance_enforcement() {
        // Test consistent compliance across ecosystem
    }
}
```

## Development Timeline by Primal

### Phase 1 (Week 1-2): Core Infrastructure
- **All Teams**: Implement sovereign key validation
- **BearDog**: Build core security provider
- **All Teams**: Basic external dependency gates

### Phase 2 (Week 3-4): Primal-Specific Features
- **Toadstool**: Container runtime enforcement
- **Songbird**: Service discovery filtering
- **NestGate**: Storage provider access control
- **Squirrel**: AI provider crypto-locks
- **BearDog**: Cross-Primal coordination

### Phase 3 (Week 5-6): Integration and Testing
- **All Teams**: Cross-Primal integration testing
- **All Teams**: Local compliance monitoring
- **All Teams**: End-to-end validation

### Phase 4 (Week 7-8): Hardening and Documentation
- **All Teams**: Security hardening
- **All Teams**: Performance optimization
- **All Teams**: Documentation and guides

## Success Criteria

### Technical Success
- 100% external dependency access requires sovereign key
- Zero data collection or phone home functionality
- Consistent enforcement across all Primals
- Seamless BearDog integration

### User Experience Success
- Transparent operation for legitimate users
- Clear error messages for compliance violations
- Easy sovereign key management
- Minimal performance impact

### Business Success
- Effective prevention of unauthorized commercial use
- Clear path for legitimate commercial licensing
- University partnerships remain unrestricted
- Individual researchers have easy access

## Next Steps

1. **Review this specification** with each Primal team
2. **Identify existing capabilities** that can be leveraged
3. **Plan implementation sprints** based on dependencies
4. **Coordinate cross-Primal interfaces** through BearDog
5. **Begin with BearDog core infrastructure** as foundation
6. **Implement Primal-specific features** in parallel
7. **Integrate and test** cross-Primal functionality
8. **Deploy and harden** for production readiness

<version>1.0.0</version> 