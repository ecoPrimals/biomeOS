# Testing & Code Quality Evolution Plan - Dec 28, 2025

## Current State Analysis

**Rust Files**: 185 files
**Current Tests**: Very minimal (261 passing from previous audit, but need expansion)
**Test Failures**: 2 doctest failures (biomeos-boot, biomeos-core)
**Coverage Goal**: 90% (using llvm-cov)

---

## Phase 1: Fix Existing Issues (30 min)

### 1.1 Fix Doctest Failures ✅
- `biomeos-boot/src/boot_logger/mod.rs`
- `biomeos-core` doctest issues

### 1.2 Run Clippy and Fix Warnings
```bash
cargo clippy --workspace --all-targets --fix --allow-dirty
```

### 1.3 Format All Code
```bash
cargo fmt --all
```

---

## Phase 2: Unit Tests (2 hours)

### Priority Crates for Unit Testing

#### 2.1 biomeos-types (Core Types)
**Files to test**:
- `src/primal/core.rs` - PrimalType, capabilities
- `src/health/core.rs` - Health system
- `src/error/core.rs` - Error handling
- `src/constants.rs` - Constants validation

**Test coverage**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_primal_type_discovery() {
        // Test PrimalType::from_discovered
    }
    
    #[test]
    fn test_capability_validation() {
        // Test capability matching
    }
    
    #[test]
    fn test_health_status_transitions() {
        // Test Health enum
    }
}
```

#### 2.2 biomeos-core (Core Logic)
**Files to test**:
- `src/discovery_bootstrap.rs` - Discovery system
- `src/primal_adapter/discovery.rs` - Adapter discovery
- `src/primal_adapter/lifecycle.rs` - Lifecycle management
- `src/universal_biomeos_manager/core.rs` - Manager core

**Test patterns**:
- Mock-based tests (using test_utils)
- Integration tests with real discovery
- Error path testing

#### 2.3 biomeos-cli (CLI Commands)
**Files to test**:
- `src/commands/discover.rs`
- `src/commands/deploy.rs`
- `src/commands/health.rs`

**Test approach**:
- Command execution tests
- Output formatting tests
- Error handling tests

---

## Phase 3: Integration Tests (2 hours)

### 3.1 Discovery Integration Tests
```rust
// tests/discovery_integration.rs
#[tokio::test]
async fn test_full_discovery_flow() {
    // Start mock primals
    // Run discovery
    // Verify capabilities found
}
```

### 3.2 Deployment Integration Tests
```rust
// tests/deployment_integration.rs
#[tokio::test]
async fn test_niche_deployment() {
    // Deploy a niche
    // Verify all components started
    // Verify health checks pass
}
```

### 3.3 Multi-Primal Coordination Tests
```rust
// tests/coordination_integration.rs
#[tokio::test]
async fn test_primal_coordination() {
    // Start multiple primals
    // Test capability composition
    // Verify cross-primal communication
}
```

---

## Phase 4: E2E Tests (3 hours)

### 4.1 Showcase E2E Tests
Each showcase demo should have automated E2E test:

```bash
showcase/00-substrate/01-hello-biomeos/
├── demo.sh              # Human-facing demo
├── test.sh              # Automated E2E test
└── validate.sh          # benchScale validation
```

**Test structure**:
```bash
#!/usr/bin/env bash
# test.sh - Automated E2E test

set -e

# Setup
source ../../common/discovery.sh
export PRIMALS_DIR=../../../primals

# Test discovery
primals=$(discover_primals)
assert_contains "$primals" "nestgate"
assert_contains "$primals" "beardog"

# Test capabilities
storage=$(discover_capability "storage")
assert_equals "$storage" "http://localhost:9020"

# Cleanup
echo "✅ All E2E tests passed"
```

### 4.2 CLI E2E Tests
```rust
// tests/cli_e2e.rs
#[test]
fn test_cli_discover_command() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "biomeos-cli", "--", "discover"])
        .output()
        .expect("Failed to execute");
    
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout)
        .contains("Discovered"));
}
```

### 4.3 Federation E2E Tests
```rust
// tests/federation_e2e.rs
#[tokio::test]
async fn test_multi_tower_federation() {
    // Start multiple biomeOS instances
    // Verify they discover each other
    // Test cross-tower operations
}
```

---

## Phase 5: Coverage & Quality (2 hours)

### 5.1 Generate Coverage Report
```bash
# Install llvm-cov
cargo install cargo-llvm-cov

# Generate coverage
cargo llvm-cov --workspace --html

# View report
open target/llvm-cov/html/index.html
```

### 5.2 Improve Coverage to 90%
Priority order:
1. Core types (biomeos-types)
2. Discovery system (biomeos-core)
3. CLI commands (biomeos-cli)
4. Deployment (biomeos-deploy)

### 5.3 Add Property-Based Tests
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_primal_type_parsing(name in "[a-z]{3,20}") {
        let result = PrimalType::from_name(&name);
        // Should never panic
    }
}
```

---

## Phase 6: Idiomatic Rust Improvements (3 hours)

### 6.1 Error Handling Improvements
**Before**:
```rust
pub fn do_something() -> Result<(), String> {
    Err("something went wrong".to_string())
}
```

**After**:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BiomeError {
    #[error("Discovery failed: {0}")]
    DiscoveryFailed(String),
}

pub fn do_something() -> BiomeResult<()> {
    Err(BiomeError::DiscoveryFailed("reason".into()))
}
```

### 6.2 Remove Unwraps
```bash
# Find all unwraps
rg "\.unwrap\(\)" --type rust crates/

# Replace with proper error handling
# unwrap() → ? operator
# unwrap_or() → unwrap_or_else()
# expect() → better error context
```

### 6.3 Use Modern Rust Patterns
- Replace `match` with `if let` where appropriate
- Use `?` operator consistently
- Leverage iterator methods over loops
- Use const generics where applicable
- Replace `Clone` with `Copy` for small types

### 6.4 Async/Await Improvements
```rust
// Before
async fn fetch_all() -> Vec<Result<Data>> {
    let mut results = vec![];
    for url in urls {
        results.push(fetch(url).await);
    }
    results
}

// After (parallel execution)
async fn fetch_all() -> Vec<Result<Data>> {
    futures::future::join_all(
        urls.iter().map(|url| fetch(url))
    ).await
}
```

---

## Testing Infrastructure

### Test Utilities (biomeos-test-utils)

#### Mock Primal Server
```rust
// src/mock_primal.rs
pub struct MockPrimal {
    port: u16,
    capabilities: Vec<String>,
}

impl MockPrimal {
    pub async fn start() -> Self {
        // Start HTTP server
        // Return mock primal
    }
    
    pub fn health_endpoint(&self) -> String {
        format!("http://localhost:{}/health", self.port)
    }
}
```

#### Test Fixtures
```rust
// src/fixtures.rs
pub fn sample_manifest() -> BiomeManifest {
    BiomeManifest {
        name: "test-biome".into(),
        version: "1.0.0".into(),
        // ... complete fixture
    }
}

pub fn sample_primal_config() -> PrimalConfiguration {
    // Complete test fixture
}
```

#### Assertions
```rust
// src/assertions.rs
pub fn assert_primal_healthy(endpoint: &str) {
    let response = reqwest::blocking::get(format!("{}/health", endpoint))
        .expect("Failed to get health");
    assert_eq!(response.status(), 200);
}

pub fn assert_capability_available(capability: &str) {
    // Custom assertion for capabilities
}
```

---

## Continuous Integration

### GitHub Actions Workflow
```yaml
name: Test & Quality

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Run tests
        run: cargo test --workspace --all-features
        
      - name: Run clippy
        run: cargo clippy --workspace -- -D warnings
        
      - name: Check formatting
        run: cargo fmt --check
        
      - name: Generate coverage
        run: |
          cargo install cargo-llvm-cov
          cargo llvm-cov --workspace --lcov --output-path lcov.info
          
      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: lcov.info
```

---

## Success Criteria

### Phase 1 (Foundation)
- [  ] All doctests passing
- [  ] Zero clippy warnings
- [  ] All code formatted

### Phase 2 (Unit Tests)
- [  ] biomeos-types: 90%+ coverage
- [  ] biomeos-core: 80%+ coverage
- [  ] biomeos-cli: 80%+ coverage

### Phase 3 (Integration Tests)
- [  ] Discovery integration tests passing
- [  ] Deployment integration tests passing
- [  ] Multi-primal coordination tests passing

### Phase 4 (E2E Tests)
- [  ] All showcase demos have automated tests
- [  ] CLI E2E tests passing
- [  ] Federation E2E tests passing

### Phase 5 (Coverage)
- [  ] Overall workspace coverage: 90%+
- [  ] Coverage reports generated
- [  ] No untested critical paths

### Phase 6 (Quality)
- [  ] Zero unwraps in production code
- [  ] All errors use thiserror
- [  ] Modern Rust patterns throughout
- [  ] Async code optimized

---

## Timeline

**Total Estimated Time**: 12-14 hours

- Phase 1: 30 minutes
- Phase 2: 2 hours
- Phase 3: 2 hours
- Phase 4: 3 hours
- Phase 5: 2 hours
- Phase 6: 3 hours
- Documentation: 1 hour

---

## Next Actions (Immediate)

1. Fix doctest failures
2. Run clippy and fix warnings
3. Add unit tests to biomeos-types
4. Create E2E test for showcase demos
5. Generate coverage report

Let's start! 🚀

