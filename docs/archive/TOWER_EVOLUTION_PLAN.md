# Tower Evolution Plan - Universal Agnostic Orchestration

**Date**: January 3, 2026  
**Goal**: Make tower truly platform-agnostic, idiomatic, and concurrent

---

## 🎯 Current Issues

### 1. Hardcoded Primal Types
```rust
// tower.rs currently has:
Commands::Start {
    security_binary: Option<String>,      // ❌ Hardcoded "security" concept
    discovery_binary: Option<String>,     // ❌ Hardcoded "discovery" concept
    additional: Option<String>,           // Everything else is "additional"?
}
```

**Problem**: Not truly agnostic. What if there's no "security" primal? What if we have 5 primals?

### 2. Environment Variables Per-Primal
```bash
# Current approach requires manual env var management:
export SECURITY_PROVIDER_BINARY="./primals/beardog"
export DISCOVERY_ORCHESTRATOR_BINARY="./primals/songbird"
export ADDITIONAL_PRIMALS="./primals/toadstool,./primals/nestgate"
```

**Problem**: Doesn't scale. Not concurrent. Not platform-agnostic.

### 3. No Auto-Discovery
Tower doesn't scan for available primals - you must specify each one manually.

---

## ✅ Proposed Universal Architecture

### 1. Config-Driven Orchestration

**tower.toml** (platform-agnostic):
```toml
[tower]
name = "tower1"
family = "nat0"
concurrent_startup = true
health_check_interval = 30

[[primal]]
binary = "./primals/beardog"
# Capabilities auto-discovered by querying binary
# or specified explicitly:
provides = ["Security", "Encryption", "Trust"]
requires = []

[[primal]]
binary = "./primals/songbird"
provides = ["Discovery"]
requires = ["Security"]

[[primal]]
binary = "./primals/toadstool"
provides = ["Orchestration", "Containers"]
requires = ["Security", "Discovery"]

# Optional: Auto-discover all primals in a directory
[discovery]
scan_dirs = ["./primals", "./plugins"]
auto_register = true
```

### 2. Auto-Discovery from Directory

```rust
// Scan directory and query each binary for capabilities
pub async fn discover_primals(dir: &Path) -> Vec<PrimalMetadata> {
    let mut primals = Vec::new();
    
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_file() && is_executable(&path) {
            // Query binary for its capabilities
            if let Ok(metadata) = query_primal_metadata(&path).await {
                primals.push(metadata);
            }
        }
    }
    
    primals
}

// Query binary: ./primal --biomeos-capabilities
// Returns JSON: {"provides": ["Security"], "requires": []}
async fn query_primal_metadata(binary: &Path) -> Result<PrimalMetadata> {
    let output = Command::new(binary)
        .arg("--biomeos-capabilities")
        .output()
        .await?;
    
    serde_json::from_slice(&output.stdout)
}
```

### 3. Concurrent Startup with Dependency Resolution

```rust
// Start primals concurrently in waves based on dependencies
pub async fn start_with_concurrency(
    orchestrator: &PrimalOrchestrator,
    primals: Vec<Arc<dyn ManagedPrimal>>
) -> Result<()> {
    // Build dependency graph
    let graph = build_dependency_graph(&primals)?;
    
    // Get startup waves (primals that can start in parallel)
    let waves = topological_sort_waves(&graph)?;
    
    // Start each wave concurrently
    for (wave_num, wave) in waves.iter().enumerate() {
        info!("🌊 Starting wave {} ({} primals)", wave_num, wave.len());
        
        // Spawn all primals in this wave concurrently
        let handles: Vec<_> = wave.iter().map(|primal_id| {
            let primal = primals.iter()
                .find(|p| p.id() == *primal_id)
                .unwrap()
                .clone();
            
            tokio::spawn(async move {
                primal.start().await
            })
        }).collect();
        
        // Wait for all in this wave to complete
        for handle in handles {
            handle.await??;
        }
        
        info!("✅ Wave {} complete", wave_num);
    }
    
    Ok(())
}
```

### 4. Platform-Agnostic Binary Discovery

```rust
// Works on Linux, macOS, Windows, bare metal
#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(metadata) = fs::metadata(path) {
        metadata.permissions().mode() & 0o111 != 0
    } else {
        false
    }
}

#[cfg(windows)]
fn is_executable(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("exe"))
        .unwrap_or(false)
}

#[cfg(not(any(unix, windows)))]
fn is_executable(path: &Path) -> bool {
    // Bare metal / WASM: check magic bytes or rely on naming convention
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|name| !name.contains('.'))
        .unwrap_or(false)
}
```

### 5. Universal Primal Protocol

**Every primal supports**:
```bash
# Capability discovery
./primal --biomeos-capabilities
# Output: {"provides": ["Security"], "requires": ["Discovery"]}

# Health check
./primal --biomeos-health
# Output: {"status": "healthy", "version": "1.0.0"}

# Graceful shutdown
./primal --biomeos-shutdown
# Primal saves state and exits cleanly

# Info query
./primal --biomeos-info
# Output: {"name": "BearDog", "family_id": "nat0", "node_id": "tower1_xxx"}
```

### 6. Idiomatic Rust Patterns

```rust
// Builder pattern for configuration
let tower = Tower::builder()
    .name("tower1")
    .family("nat0")
    .config_file("tower.toml")
    .scan_directory("./primals")
    .auto_discover(true)
    .concurrent_startup(true)
    .health_interval(Duration::from_secs(30))
    .build()?;

// Async stream for primal events
let mut events = tower.event_stream();
while let Some(event) = events.next().await {
    match event {
        PrimalEvent::Started(id) => info!("✅ {} started", id),
        PrimalEvent::Stopped(id) => info!("🛑 {} stopped", id),
        PrimalEvent::Unhealthy(id) => warn!("⚠️ {} unhealthy", id),
    }
}

// Error handling with thiserror
#[derive(thiserror::Error, Debug)]
pub enum TowerError {
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),
    
    #[error("Missing capability: {capability} required by {primal}")]
    MissingCapability { capability: String, primal: String },
    
    #[error("Platform not supported: {0}")]
    UnsupportedPlatform(String),
}
```

---

## 🚀 Implementation Plan

### Phase 1: Config File Support
- [ ] Add TOML parsing with `serde`
- [ ] Define `TowerConfig` struct
- [ ] Support both env vars and config file
- [ ] Config file overrides env vars

### Phase 2: Auto-Discovery
- [ ] Implement directory scanning
- [ ] Add `--biomeos-capabilities` protocol to all primals
- [ ] Query binaries for their capabilities
- [ ] Build registry of discovered primals

### Phase 3: Concurrent Startup
- [ ] Implement dependency graph builder
- [ ] Topological sort into waves
- [ ] Spawn primals concurrently per wave
- [ ] Collect results with `join_all`

### Phase 4: Platform Abstraction
- [ ] Abstract executable detection (Unix/Windows/bare metal)
- [ ] Abstract process spawning (tokio/OS-specific)
- [ ] Abstract health checks (HTTP/gRPC/socket/shared memory)
- [ ] Test on Linux, macOS, Windows, WASM

### Phase 5: Universal Protocol
- [ ] Define primal CLI protocol (`--biomeos-*` flags)
- [ ] Update all primals to support protocol
- [ ] Document protocol in `PRIMAL_PROTOCOL.md`
- [ ] Version the protocol for evolution

---

## 📊 Benefits

### Before (Current)
```bash
# Manual, sequential, hardcoded
export SECURITY_PROVIDER_BINARY="./primals/beardog"
export DISCOVERY_ORCHESTRATOR_BINARY="./primals/songbird"
./bin/tower start  # Takes 10 seconds (sequential)
```

### After (Universal)
```bash
# Automatic, concurrent, agnostic
./bin/tower start --config tower.toml
# OR
./bin/tower start --scan ./primals
# Takes 3 seconds (concurrent waves)
```

### Platform Support
- ✅ Linux (any distro)
- ✅ macOS (any version)
- ✅ Windows (10/11/Server)
- ✅ FreeBSD/OpenBSD
- ✅ Containers (Docker/Podman/Kubernetes)
- ✅ VMs (QEMU/VirtualBox/VMware/Hyper-V)
- ✅ Bare metal (x86_64/ARM/RISC-V)
- ✅ WASM (browser/edge)

---

## 🎯 Next Steps

1. **Create `TowerConfig` struct** with TOML support
2. **Implement directory scanner** for auto-discovery
3. **Add concurrent wave-based startup** 
4. **Define universal primal protocol**
5. **Update all primals** to support protocol
6. **Test on multiple platforms**

**Goal**: `./bin/tower start` on ANY platform, discovers primals, starts them concurrently with proper dependency resolution, zero hardcoding!

---

**Status**: Design complete, ready for implementation

