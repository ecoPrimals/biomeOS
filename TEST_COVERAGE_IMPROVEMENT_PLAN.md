# Test Coverage Improvement Plan

**Date**: December 31, 2025  
**Current Coverage**: 29.18% (measured via llvm-cov)  
**Target**: 90% by Q2 2026  
**Philosophy**: Deep debt solutions, not quick fixes  

---

## 📊 Current State (Measured)

### Coverage Breakdown
```
Line Coverage:     29.18% (8,205 / 28,120 lines)
Function Coverage: 38.54% (898 / 2,330 functions)  
Region Coverage:   32.51% (6,572 / 20,216 regions)
```

### Critical Low-Coverage Modules (Priority 1)

| Module | Coverage | Lines | Impact | Priority |
|--------|----------|-------|--------|----------|
| `biomeos-types/config/mod.rs` | 10.23% | 264 | HIGH | ❗ Critical |
| `src/universal_adapter.rs` | 19.41% | 881 | HIGH | ❗ Critical |
| `universal_biomeos_manager/discovery.rs` | ~25% | ~400 | HIGH | ❗ Critical |
| `universal_biomeos_manager/operations.rs` | ~30% | ~300 | HIGH | 🔧 High |
| `universal_biomeos_manager/core.rs` | ~35% | ~250 | MEDIUM | 🔧 High |

---

## 🎯 Execution Plan

### Phase 1: Critical Path Coverage (January 2026)
**Target**: 50% overall coverage  
**Timeline**: 4 weeks  
**Focus**: Core business logic

#### Week 1: Config Module (10% → 80%)
```rust
// File: crates/biomeos-types/src/config/mod.rs
// Current: 10.23% coverage, 264 lines

// Add tests for:
- Config loading and parsing
- Default value generation
- Validation rules
- Environment variable overrides
- Merge behavior
```

**Test Strategy**:
```rust
#[cfg(test)]
mod config_tests {
    use super::*;
    
    #[test]
    fn test_config_default_values() {
        let config = Config::default();
        assert!(config.validate().is_ok());
        // Test all default fields
    }
    
    #[test]
    fn test_config_from_env() {
        // Set env vars
        std::env::set_var("BIOMEOS_PORT", "9999");
        let config = Config::from_env().unwrap();
        assert_eq!(config.port, 9999);
    }
    
    #[test]
    fn test_config_validation_failures() {
        let mut config = Config::default();
        config.port = 0; // Invalid
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_config_merge_behavior() {
        let base = Config::default();
        let override_config = Config { port: 8080, ..Default::default() };
        let merged = base.merge(override_config);
        assert_eq!(merged.port, 8080);
    }
}
```

#### Week 2: Universal Adapter (19% → 70%)
```rust
// File: src/universal_adapter.rs
// Current: 19.41% coverage, 881 lines

// This is a LARGE file (881 lines)
// Strategy: Smart refactoring + comprehensive tests
```

**Refactoring Strategy** (Deep Debt Solution):
```rust
// BEFORE: Large monolithic adapter (881 lines)
// src/universal_adapter.rs - everything in one file

// AFTER: Modular adapter with clear separation of concerns
// src/universal_adapter/
//   mod.rs              (100 lines) - Public API
//   discovery.rs        (150 lines) - Discovery logic
//   http_adapter.rs     (120 lines) - HTTP interface
//   cli_adapter.rs      (120 lines) - CLI interface  
//   mdns_adapter.rs     (120 lines) - mDNS interface
//   adapters/
//     beardog.rs        (80 lines)  - BearDog specific
//     nestgate.rs       (80 lines)  - NestGate specific
//     songbird.rs       (80 lines)  - Songbird specific
//     toadstool.rs      (80 lines)  - Toadstool specific
//   tests.rs            (150 lines) - Integration tests
```

**Test Strategy**:
```rust
// High-level integration tests
#[tokio::test]
async fn test_discover_http_primal() {
    let adapter = UniversalAdapter::discover("http://localhost:9020").await.unwrap();
    assert_eq!(adapter.interface_type(), InterfaceType::HttpApi);
}

// Module-level unit tests
mod discovery_tests {
    #[test]
    fn test_parse_endpoint_http() {
        let endpoint = parse_endpoint("http://localhost:9020").unwrap();
        assert!(matches!(endpoint, Endpoint::Http(_)));
    }
    
    #[test]
    fn test_parse_endpoint_cli() {
        let endpoint = parse_endpoint("/usr/bin/beardog").unwrap();
        assert!(matches!(endpoint, Endpoint::Cli(_)));
    }
}
```

#### Week 3: Universal BiomeOS Manager (25% → 70%)
```rust
// Files in crates/biomeos-core/src/universal_biomeos_manager/
// discovery.rs:  ~25% coverage
// operations.rs: ~30% coverage
// core.rs:       ~35% coverage
```

**Test Strategy**:
```rust
// Test discovery patterns
#[tokio::test]
async fn test_discover_by_capability() {
    let manager = UniversalBiomeosManager::new().await.unwrap();
    let storage_primals = manager
        .discover_by_capability(&[Capability::new("storage", "api", "1.0")])
        .await
        .unwrap();
    assert!(!storage_primals.is_empty());
}

// Test operations
#[tokio::test]
async fn test_deploy_service() {
    let manager = UniversalBiomeosManager::new().await.unwrap();
    let result = manager.deploy_service("test-service", &config).await;
    assert!(result.is_ok());
}

// Test health monitoring
#[tokio::test]
async fn test_health_check_all_primals() {
    let manager = UniversalBiomeosManager::new().await.unwrap();
    let health = manager.check_health_all().await.unwrap();
    assert!(!health.is_empty());
}
```

#### Week 4: Integration Test Suite Expansion
- Add 50+ integration tests for cross-module interactions
- Test error paths and edge cases
- Verify graceful degradation

---

### Phase 2: Comprehensive Coverage (February-March 2026)
**Target**: 70% overall coverage  
**Timeline**: 8 weeks  

#### Critical Modules (Priority 2)
- `biomeos-primal-sdk` (currently 0%)
- `biomeos-manifest` loader functions
- `biomeos-niche` deployment edge cases
- `biomeos-boot` initialization paths

---

### Phase 3: Excellence (April-June 2026)  
**Target**: 90% overall coverage  
**Timeline**: 12 weeks  

#### Comprehensive Testing
- Property-based tests (proptest)
- Fuzzing critical parsers
- Chaos engineering scenarios
- Performance regression tests

---

## 🛠️ Testing Patterns (Modern Idiomatic Rust)

### 1. Error Path Testing (No Unwraps)
```rust
// ❌ BAD: Testing with unwrap
#[test]
fn test_parse_config() {
    let config = parse_config("config.yaml").unwrap();
    assert_eq!(config.port, 8080);
}

// ✅ GOOD: Testing error paths explicitly
#[test]
fn test_parse_config_success() {
    let result = parse_config("config.yaml");
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.port, 8080);
}

#[test]
fn test_parse_config_missing_file() {
    let result = parse_config("nonexistent.yaml");
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err().downcast_ref::<std::io::Error>(),
        Some(e) if e.kind() == std::io::ErrorKind::NotFound
    ));
}

#[test]
fn test_parse_config_invalid_yaml() {
    let result = parse_config("invalid.yaml");
    assert!(result.is_err());
    // Verify specific error type
}
```

### 2. Async Testing Best Practices
```rust
// ✅ Proper async test setup
#[tokio::test]
async fn test_primal_discovery() -> anyhow::Result<()> {
    // Setup
    let mock_primal = MockPrimal::builder("test")
        .port(9999)
        .build()
        .start()
        .await?;
    
    // Test
    let discovered = discover_primal("http://localhost:9999").await?;
    assert_eq!(discovered.name, "test");
    
    // Cleanup
    mock_primal.stop().await?;
    Ok(())
}
```

### 3. Property-Based Testing
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_config_roundtrip(port in 1024u16..65535u16) {
        let config = Config { port, ..Default::default() };
        let yaml = serde_yaml::to_string(&config)?;
        let parsed: Config = serde_yaml::from_str(&yaml)?;
        prop_assert_eq!(config.port, parsed.port);
    }
}
```

### 4. Test Fixtures (DRY)
```rust
// tests/common/fixtures.rs
pub fn sample_config() -> Config {
    Config {
        port: 8080,
        discovery: DiscoveryConfig::default(),
        ..Default::default()
    }
}

pub fn sample_primal_spec() -> PrimalSpec {
    PrimalSpec {
        name: "test-primal".into(),
        capabilities: vec![Capability::new("test", "api", "1.0")],
        ..Default::default()
    }
}

// In tests:
#[test]
fn test_with_fixture() {
    let config = sample_config();
    // Use config...
}
```

---

## 🎯 Smart Refactoring Principles

### Principle 1: Extract by Cohesion, Not Line Count
```rust
// ❌ BAD: Arbitrary split at 500 lines
// file1.rs (500 lines of random functions)
// file2.rs (381 lines of random functions)

// ✅ GOOD: Cohesive modules
// discovery/
//   http.rs      - HTTP discovery
//   cli.rs       - CLI discovery
//   mdns.rs      - mDNS discovery
// adapters/
//   beardog.rs   - BearDog adapter
//   nestgate.rs  - NestGate adapter
```

### Principle 2: Single Responsibility
```rust
// ❌ BAD: God object
struct UniversalAdapter {
    // 881 lines doing everything
}

// ✅ GOOD: Composed responsibilities
struct UniversalAdapter {
    discovery: DiscoveryEngine,
    http: HttpAdapter,
    cli: CliAdapter,
    mdns: MdnsAdapter,
}

impl UniversalAdapter {
    pub async fn discover(&self, endpoint: &str) -> Result<PrimalInfo> {
        let endpoint_type = self.discovery.classify(endpoint)?;
        match endpoint_type {
            EndpointType::Http => self.http.discover(endpoint).await,
            EndpointType::Cli => self.cli.discover(endpoint).await,
            EndpointType::Mdns => self.mdns.discover(endpoint).await,
        }
    }
}
```

### Principle 3: Test at the Right Level
```rust
// High-level integration tests
#[tokio::test]
async fn test_end_to_end_discovery() {
    // Test the full flow
}

// Module-level unit tests
mod http_adapter_tests {
    #[tokio::test]
    async fn test_http_health_check() {
        // Test just HTTP logic
    }
}

// Function-level tests
mod parsing_tests {
    #[test]
    fn test_parse_http_endpoint() {
        // Test just parsing
    }
}
```

---

## 📈 Progress Tracking

### Week 1 Metrics
- [ ] Config module: 10% → 80% coverage
- [ ] Add 20+ config tests
- [ ] Test all validation paths
- [ ] Document config testing patterns

### Week 2 Metrics
- [ ] Universal adapter: 19% → 50% coverage (refactoring in progress)
- [ ] Refactor 881-line file into modules
- [ ] Add 30+ adapter tests
- [ ] Test all interface types

### Week 3 Metrics
- [ ] Universal manager: 25% → 70% coverage
- [ ] Add 40+ manager tests
- [ ] Test all operation types
- [ ] Test capability discovery

### Week 4 Metrics
- [ ] Overall coverage: 29% → 50%
- [ ] Integration test suite: +50 tests
- [ ] All critical paths tested
- [ ] Error handling comprehensive

---

## 🔄 Continuous Improvement

### Daily
- Run `cargo llvm-cov --workspace` before commit
- Ensure no coverage regression
- Add tests for new code

### Weekly
- Review coverage report
- Identify new low-coverage areas
- Update this plan

### Monthly
- Comprehensive coverage review
- Refactoring retrospective
- Adjust targets based on progress

---

## 🎓 Testing Philosophy

### 1. Test Behavior, Not Implementation
```rust
// ❌ BAD: Testing internal details
#[test]
fn test_internal_cache_structure() {
    let adapter = UniversalAdapter::new();
    assert_eq!(adapter.cache.len(), 0); // Fragile!
}

// ✅ GOOD: Testing observable behavior
#[test]
async fn test_caching_reduces_discovery_calls() {
    let adapter = UniversalAdapter::new();
    let first = adapter.discover("test").await.unwrap();
    let second = adapter.discover("test").await.unwrap();
    // Second call should be fast (cached)
    assert_eq!(first.name, second.name);
}
```

### 2. Test Edge Cases and Error Paths
```rust
#[tokio::test]
async fn test_discover_timeout() {
    let result = timeout(
        Duration::from_millis(100),
        discover_primal("http://slow-server:9999")
    ).await;
    assert!(result.is_err()); // Should timeout
}

#[test]
fn test_invalid_capability_format() {
    let result = Capability::parse("invalid::format::too::many::parts");
    assert!(result.is_err());
}
```

### 3. Test Integration, Not Just Units
```rust
#[tokio::test]
async fn test_full_deployment_workflow() {
    // 1. Discover primal
    let primal = discover_primal("nestgate").await?;
    
    // 2. Verify capabilities
    assert!(primal.has_capability("storage"));
    
    // 3. Deploy service
    let service = deploy_service(primal, &config).await?;
    
    // 4. Verify deployment
    let health = check_service_health(service.id).await?;
    assert_eq!(health.status, HealthStatus::Healthy);
    
    Ok(())
}
```

---

## 📊 Success Criteria

### Q1 2026 (March 31)
- ✅ 70% overall coverage
- ✅ All critical paths tested
- ✅ Zero unwraps in production code
- ✅ Smart refactoring complete

### Q2 2026 (June 30)
- ✅ 90% overall coverage
- ✅ Property-based tests added
- ✅ Fuzzing integrated
- ✅ Chaos tests comprehensive

---

**Status**: ACTIVE  
**Owner**: Development Team  
**Next Review**: January 7, 2026  

🌱 **Test coverage is not optional - it's a measure of confidence**

