# BiomeOS Production Patterns & Idiomatic Rust

**Date**: December 26, 2025  
**Status**: Deep debt solutions, modern idiomatic Rust ✅

This document details the production-quality patterns and idiomatic Rust practices used throughout BiomeOS.

---

## 🎯 Core Principles

### 1. Zero Technical Debt
- No `unwrap()` in production code paths
- Comprehensive error handling with `Result<T>` and `anyhow::Context`
- Proper cleanup with RAII and `Drop` traits
- No panics in production code

### 2. Modern Idiomatic Rust
- `async/await` for all I/O operations
- `Arc` and `RwLock` for shared state
- Trait-based abstractions
- Type-safe APIs throughout
- Proper lifetime management

### 3. Production Quality
- Structured logging with `tracing`
- Graceful degradation
- Proper resource cleanup
- Health checking
- Comprehensive error context

---

## 📚 Pattern Catalog

### Pattern 1: Result-Based Error Handling

**Bad** (technical debt):
```rust
fn get_primal() -> PrimalBinary {
    let path = find_binary().unwrap();  // ❌ Panic on error
    load_binary(path).expect("failed")   // ❌ Vague error
}
```

**Good** (production):
```rust
fn get_primal() -> Result<PrimalBinary> {
    let path = find_binary()
        .context("Failed to find primal binary")?;  // ✅ Contextual error
    load_binary(&path)
        .with_context(|| format!("Failed to load binary from {:?}", path))?  // ✅ Detailed context
}
```

**Used in**: `primal_registry/mod.rs`, `full_integration_test.rs`

---

### Pattern 2: Structured Async with Context

**Bad**:
```rust
async fn deploy() {
    let result = copy_file().await;
    if result.is_err() {
        println!("Failed");  // ❌ Lost error context
    }
}
```

**Good**:
```rust
async fn deploy(node: &str) -> Result<()> {
    copy_file()
        .await
        .with_context(|| format!("Failed to deploy to {}", node))?;  // ✅ Preserves context
    Ok(())
}
```

**Used in**: `full_integration_test.rs` - all async functions

---

### Pattern 3: Trait-Based Abstractions

**Bad** (coupled):
```rust
struct BearDog {
    // Specific implementation details
}

fn use_beardog(bd: &BearDog) {  // ❌ Tightly coupled
    bd.create_tunnel();
}
```

**Good** (abstracted):
```rust
trait SecurityProvider {
    async fn create_tunnel(&self, target: &str) -> Result<TunnelInfo>;
}

fn use_security<S: SecurityProvider>(provider: &S) -> Result<()> {  // ✅ Generic
    provider.create_tunnel("target").await?;
    Ok(())
}
```

**Used in**: `p2p_coordination/mod.rs` - `SecurityProvider`, `DiscoveryProvider`, `RoutingProvider`

---

### Pattern 4: Arc + RwLock for Shared State

**Bad**:
```rust
struct Lab {
    state: LabState,  // ❌ Can't share across threads
}
```

**Good**:
```rust
struct Lab {
    state: Arc<RwLock<LabState>>,  // ✅ Thread-safe shared state
}

impl Lab {
    async fn update(&self) -> Result<()> {
        let mut state = self.state.write().await;  // ✅ Async lock
        state.status = Status::Running;
        Ok(())
    }
}
```

**Used in**: `lab/mod.rs` in benchScale

---

### Pattern 5: Proper Cleanup with RAII

**Bad**:
```rust
async fn test() -> Result<()> {
    let lab = create_lab().await?;
    run_tests(&lab).await?;
    // ❌ If test fails, lab never cleaned up
    destroy_lab(lab).await?;
    Ok(())
}
```

**Good** (RAII):
```rust
struct LabGuard {
    lab: Lab,
}

impl Drop for LabGuard {
    fn drop(&mut self) {
        // ✅ Automatic cleanup even on panic
        tokio::spawn(async move {
            let _ = self.lab.destroy().await;
        });
    }
}

async fn test() -> Result<()> {
    let _guard = LabGuard::new(create_lab().await?);
    run_tests(&_guard.lab).await?;
    Ok(())  // ✅ Cleanup happens automatically
}
```

**Used in**: `benchscale_p2p_test.rs`, `full_integration_test.rs` (cleanup pattern)

---

### Pattern 6: Structured Logging

**Bad**:
```rust
println!("Starting service");  // ❌ No structure, no levels
eprintln!("Error: {}", e);      // ❌ Hard to filter
```

**Good**:
```rust
use tracing::{info, error, warn};

info!("Starting service", service = "beardog", port = 9000);  // ✅ Structured
error!("Service failed", error = %e, node = "node-1");        // ✅ Contextual
warn!("High latency", latency_ms = 500, threshold_ms = 100);  // ✅ Queryable
```

**Used in**: All examples and modules

---

### Pattern 7: Enum for Binary Locations

**Bad**:
```rust
struct Binary {
    path: String,  // ❌ Ambiguous: local file? URL? GitHub?
}
```

**Good**:
```rust
enum BinaryLocation {
    Local(PathBuf),                    // ✅ Clearly a local file
    GitHub { org: String, repo: String, tag: String, asset: String },  // ✅ GitHub release
    Remote(String),                    // ✅ Custom URL
}

impl BinaryLocation {
    async fn fetch(&self) -> Result<PathBuf> {
        match self {
            Self::Local(path) => Ok(path.clone()),
            Self::GitHub { .. } => self.download_from_github().await,
            Self::Remote(url) => self.download_from_url(url).await,
        }
    }
}
```

**Used in**: `primal_registry/mod.rs`

---

### Pattern 8: Graceful Degradation

**Bad**:
```rust
fn main() {
    if !docker_available() {
        panic!("Docker required!");  // ❌ Fails hard
    }
}
```

**Good**:
```rust
async fn main() -> Result<()> {
    let prereqs = check_prerequisites().await?;
    
    if !prereqs.docker_available {
        warn!("Docker not available - running in simulation mode");  // ✅ Graceful fallback
        return run_simulation_mode().await;
    }
    
    run_full_integration().await
}
```

**Used in**: `full_integration_test.rs`, `benchscale_p2p_test.rs`

---

### Pattern 9: Type-Safe Configuration

**Bad**:
```rust
fn start_service(config: HashMap<String, String>) -> Result<()> {
    let port = config.get("port").unwrap().parse::<u16>().unwrap();  // ❌ Multiple unwraps
    // ...
}
```

**Good**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServiceConfig {
    port: u16,                    // ✅ Type-safe
    #[serde(default)]
    timeout_secs: u64,            // ✅ Optional with default
}

fn start_service(config: &ServiceConfig) -> Result<()> {
    listen_on(config.port)?;      // ✅ No parsing needed
    Ok(())
}
```

**Used in**: `topology/mod.rs`, `primal_registry/mod.rs`

---

### Pattern 10: Checksum Verification

**Bad**:
```rust
async fn deploy(path: &Path) -> Result<()> {
    let contents = read(path).await?;
    copy_to_target(contents).await?;  // ❌ No integrity check
    Ok(())
}
```

**Good**:
```rust
async fn deploy(binary: &PrimalBinary) -> Result<()> {
    let contents = read(&binary.path).await?;
    
    if let Some(expected_checksum) = &binary.checksum {
        let actual = compute_sha256(&contents);  // ✅ Verify integrity
        if actual != *expected_checksum {
            return Err(anyhow!("Checksum mismatch"));
        }
    }
    
    copy_to_target(contents).await?;
    Ok(())
}
```

**Used in**: `primal_registry/mod.rs`

---

## 🏗️ Architecture Patterns

### Pattern A: Layered Architecture

```
Application Layer (examples/)
  ↓ uses
Core Layer (biomeos-core/)
  ↓ uses  
Types Layer (biomeos-types/)
```

**Benefits**:
- Clear separation of concerns
- Easy to test each layer
- Prevents circular dependencies

---

### Pattern B: Backend Trait for Extensibility

```rust
trait Backend {
    async fn create_node(...) -> Result<NodeInfo>;
    async fn delete_node(...) -> Result<()>;
    // ...
}

struct DockerBackend { /* ... */ }
impl Backend for DockerBackend { /* ... */ }

struct PodmanBackend { /* ... */ }
impl Backend for PodmanBackend { /* ... */ }
```

**Benefits**:
- Can swap implementations (Docker ↔ Podman ↔ LXD)
- Easy to mock for testing
- Future-proof

**Used in**: benchScale `backend/mod.rs`

---

### Pattern C: Registry Pattern for Discovery

```rust
struct PrimalRegistry {
    local_dir: PathBuf,
    binaries: HashMap<String, Vec<PrimalBinary>>,
}

impl PrimalRegistry {
    async fn scan_local(&mut self) -> Result<()>;
    async fn fetch_from_github(&mut self) -> Result<()>;
    fn get_latest(&self, name: &str) -> Option<&PrimalBinary>;
}
```

**Benefits**:
- Centralized binary management
- Version management
- Multiple sources (local, GitHub, remote)

**Used in**: `primal_registry/mod.rs`

---

## 🧪 Testing Patterns

### Pattern T1: Simulation Mode

```rust
async fn main() -> Result<()> {
    if !prerequisites_met() {
        return run_simulation_mode().await;  // ✅ Can test without Docker
    }
    run_real_test().await
}
```

**Benefits**:
- CI/CD friendly
- Fast iteration
- No infrastructure required

---

### Pattern T2: Mock vs Real

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_with_mock() {
        let backend = MockBackend::new();  // ✅ Fast unit test
        test_logic(&backend).await.unwrap();
    }
}

#[cfg(feature = "integration")]
#[tokio::test]
async fn test_with_real_docker() {
    let backend = DockerBackend::new().unwrap();  // ✅ Real integration test
    test_logic(&backend).await.unwrap();
}
```

**Used in**: benchScale tests

---

## 📊 Performance Patterns

### Pattern P1: Async Batch Operations

**Bad** (sequential):
```rust
for node in nodes {
    deploy(node).await?;  // ❌ One at a time
}
```

**Good** (parallel):
```rust
let futures: Vec<_> = nodes
    .iter()
    .map(|node| deploy(node))
    .collect();
    
futures::future::try_join_all(futures).await?;  // ✅ Parallel
```

**Future improvement** for BiomeOS

---

### Pattern P2: Zero-Copy Where Possible

```rust
// Instead of cloning
fn process(data: Vec<u8>) {  // ❌ Moves data
    // ...
}

// Use references
fn process(data: &[u8]) {  // ✅ Borrows
    // ...
}
```

**Used throughout**: Reference parameters, `Arc` for shared ownership

---

## 🔒 Security Patterns

### Pattern S1: Hardened Images

```rust
impl DockerBackend {
    fn get_image(&self, base: &str) -> String {
        if self.use_hardened {
            match base {
                "ubuntu" => "docker.io/dockerhardened/ubuntu:latest",  // ✅ Hardened
                _ => base,
            }
        } else {
            base
        }
    }
}
```

**Used in**: benchScale `backend/docker.rs`

---

### Pattern S2: Checksum Verification

```rust
async fn verify_binary(binary: &PrimalBinary) -> Result<()> {
    let actual = compute_sha256(&read_file(&binary.path).await?).await?;
    
    if let Some(expected) = &binary.checksum {
        if actual != *expected {
            return Err(anyhow!("Integrity check failed"));  // ✅ Detect tampering
        }
    }
    
    Ok(())
}
```

**Used in**: `primal_registry/mod.rs`

---

## 📝 Documentation Patterns

### Pattern D1: Module-Level Docs

```rust
//! # Primal Registry
//!
//! Discovers and manages primal binaries from multiple sources.
//!
//! ## Example
//!
//! ```rust,no_run
//! let mut registry = PrimalRegistry::new("../phase1bins");
//! registry.scan_local().await?;
//! ```
```

**Used in**: All modules

---

### Pattern D2: Error Context in Messages

```rust
.with_context(|| format!(
    "Failed to deploy {} v{} to {}",
    primal_name,
    version,
    target_node
))?;  // ✅ Full context in error chain
```

**Used in**: All error handling

---

## ✅ Compliance Checklist

For every new module/feature:

- [ ] No `unwrap()` or `expect()` in production paths
- [ ] All I/O operations are `async`
- [ ] Proper error handling with `Result` and `Context`
- [ ] Structured logging with `tracing`
- [ ] Type-safe APIs (no stringly-typed)
- [ ] Graceful degradation where possible
- [ ] Resource cleanup (RAII or explicit)
- [ ] Module documentation
- [ ] Example code
- [ ] Tests (unit + integration where applicable)

---

## 🚀 Examples of Pattern Usage

### Full Integration Test
**File**: `examples/full_integration_test.rs`

**Patterns Used**:
- ✅ Result-based error handling throughout
- ✅ Structured async with context
- ✅ Proper cleanup
- ✅ Graceful degradation (simulation mode)
- ✅ Structured logging
- ✅ Type-safe configuration

### Primal Registry
**File**: `crates/biomeos-core/src/primal_registry/mod.rs`

**Patterns Used**:
- ✅ Enum for binary locations
- ✅ Checksum verification
- ✅ Registry pattern
- ✅ Async I/O
- ✅ Comprehensive error handling

### benchScale
**File**: `benchscale/src/backend/docker.rs`

**Patterns Used**:
- ✅ Trait-based abstractions
- ✅ Hardened images support
- ✅ Arc + RwLock for shared state
- ✅ Proper resource cleanup

---

## 🎯 Summary

BiomeOS uses **production-quality, idiomatic Rust patterns** throughout:

1. **Zero technical debt** - No shortcuts, proper error handling
2. **Type safety** - Leveraging Rust's type system
3. **Async throughout** - Modern async/await patterns
4. **Graceful degradation** - Simulation modes, fallbacks
5. **Security-first** - Checksums, hardened images
6. **Maintainability** - Clear abstractions, good docs

**Result**: A codebase that's production-ready, maintainable, and exemplifies Rust best practices.

---

**Status**: ✅ Deep debt solutions, modern idiomatic Rust achieved!

