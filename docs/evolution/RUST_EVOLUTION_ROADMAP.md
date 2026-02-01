# 🦀 Pure Rust Evolution Roadmap

**Status**: ✅ **DEEP DEBT EVOLUTION COMPLETE** (January 28, 2026)  
**Philosophy**: *Physics with gravity and resistance, not jelly strings*

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| **TODOs** | 85 | 3 | **-96%** |
| **Unsafe Code** | 0 | 0 | ✅ Enforced |
| **Bash Dependencies** | bootstrap | 0 in prod | ✅ Pure Rust |
| **Hardcoded Paths** | 43 | 0 | Capability-based |
| **Production Stubs** | 12 | 0 | Complete |
| **Hanging Tests** | several | 0 | Concurrent |
| **Test Suites** | - | 106 | 400+ tests |

---

## Onboarding Architecture

```
Human                    Ingestion                  Runtime
┌──────────┐            ┌──────────────┐           ┌──────────────┐
│   TOML   │ ────────▶  │ Rust Types   │ ────────▶ │  JSON-RPC    │
│  Graphs  │   parse    │ (validated)  │  execute  │  (runtime)   │
└──────────┘            └──────────────┘           └──────────────┘
    ▲                         │                          │
    │                         ▼                          ▼
  Human edits            Compile-time              Language-agnostic
  On-the-fly              guarantees               Inter-primal comms
```

**Three layers, each with purpose:**

| Layer | Format | Why |
|-------|--------|-----|
| **Human ↔ Config** | TOML | Readable, editable, version-controllable |
| **Ingestion** | Rust Types | Fail-fast validation, type safety |
| **Runtime** | JSON-RPC | Language-agnostic, debuggable, flexible |

**Implemented in `biomeos-graph` crate:**
- `GraphLoader` - Parse TOML with validation
- `GraphValidator` - Cycle detection, dependency checks
- `DeploymentGraph` - Type-safe graph representation
- `GraphNode` - Validated node with typed params

---

## The Spectrum

```
Jelly Strings                                    Physics w/ Gravity
(No constraints)                                 (Full type safety)
     │                                                  │
   Bash ─────── Python ─────── C ─────── Rust ─────────┤
     │             │           │           │
   Flexible     Typed but    "Vacuum"    Borrow checker
   Fragile      runtime      physics     Ownership
   No deps      checking     Some        Compile-time
   Easy start   Duck types   safety      Zero-cost
```

**C is physics in vacuum** - constraints exist but no resistance. Memory bugs compile.

**Rust is physics with gravity** - the compiler enforces invariants. If it compiles, it works.

---

## Current State (January 2026)

### Latest Evolution (Jan 28, 2026)

| Improvement | Status | Description |
|-------------|--------|-------------|
| **NUCLEUS Lifecycle** | ✅ Complete | Germination → Apoptosis state machine |
| **Socket Discovery** | ✅ Complete | Capability-based, no hardcoded paths |
| **Concurrent Tests** | ✅ Complete | No sleeps, proper async waits |
| **Test Count** | 400+ | 106 test suites |
| **TODOs** | 3 remaining | External team handoffs only |

**Key accomplishments (Jan 28):**
- ✅ `PrimalLifecycleManager` - Full state machine with health monitoring
- ✅ `LifecycleHandler` - JSON-RPC handlers for lifecycle operations
- ✅ `SocketDiscovery` - Multi-strategy capability-based discovery
- ✅ All test hangs fixed with proper timeouts and RAII guards
- ✅ All `sleep()` calls replaced with `oneshot` channels
- ✅ Deadlock in `register_primal` fixed (nested RwLock issue)
- ✅ Environment variable tests properly isolated

### Previous Evolution (Jan 27, 2026)

| Improvement | Before | After | Change |
|-------------|--------|-------|--------|
| `neural_api_server.rs` | 1,691 | 764 | **-55%** |
| `logs.rs` → `logs/` | 1,039 | 744 | **-28%** |
| `neural_executor.rs` | 1,000 | 816 | **-18%** |
| Formatting issues | 515 | 0 | **100% clean** |
| Hardcoded sockets | 43 | 0 | SystemPaths XDG |
| TODOs resolved | 85 | 10 | **-88%** |
| Production stubs | 12 | 0 | **Complete implementations** |
| Mock isolation | Partial | Complete | **Tests only** |

**Key accomplishments:**
- ✅ Handler decomposition (4 focused modules)
- ✅ SystemPaths XDG-compliant socket discovery (no hardcoding)
- ✅ AtomicClient for Pure Rust JSON-RPC over Unix sockets
- ✅ PrimalClient wrapper for type-safe primal communication
- ✅ NestGate integration via JSON-RPC (templates, storage)
- ✅ Squirrel AI integration (suggestions, learning, feedback)
- ✅ Device management provider with real discovery
- ✅ Interactive UI orchestrator with full primal integration
- ✅ `cargo fmt` clean across workspace
- ✅ `cargo clippy` 0 errors

**Deep Debt Session (Jan 27, 2026 - Latest):**
- ✅ **neural_api_server.rs**: Complete BTSP tunnel & primal health verification
- ✅ **nucleus client.rs**: Family seed from XDG secure storage
- ✅ **executor.rs**: BearDog JSON-RPC seed derivation (no more stubs)
- ✅ **topology.rs**: XDG socket directory discovery (no `/tmp` hardcoding)
- ✅ **graph.rs**: SystemPaths for runtime socket resolution
- ✅ **orchestrator.rs**: Extracted `primal_client` module for reuse
- ✅ **fossil.rs**: Complete fossil cleanup implementation
- ✅ **verify_lineage.rs**: Full cryptographic lineage verification
- ✅ **health_tests.rs**: Evolved from reqwest stubs to JSON-RPC Unix sockets
- ✅ **discovery_tests.rs**: Capability-based discovery tests (complete)
- ✅ **executor.rs**: primal_launch, health_check, lineage_verify (complete implementations)

**Deep Debt Session #2 (Jan 27, 2026 - Extended):**
- ✅ **executor.rs**: Complete rollback strategy (RollbackAction enum, stop/remove/jsonrpc)
- ✅ **neural_executor.rs**: Rollback with graceful shutdown + SIGTERM/SIGKILL
- ✅ **subfederation.rs**: BearDog lineage verification & encryption key derivation
- ✅ **topology.rs (api)**: Node ID extraction & real edge metrics collection
- ✅ **lib.rs (api)**: WebSocket subscription management with filter tracking
- ✅ **nucleus_executor.rs**: Family ID from XDG config/environment
- ✅ **log_session.rs**: Full fossil record archival with TOML serialization

**Deep Debt Session #3 (Jan 27, 2026 - Final Push):**
- ✅ **logs.rs (cli)**: Complete fossil cleanup with size calculation & index updates
- ✅ **discovery.rs (federation)**: Songbird UDP multicast discovery integration
- ✅ **documentation.rs (spore)**: XDG-compliant family ID resolution
- ✅ **standard_api.rs (types)**: Import unified CapabilityTaxonomy (no more placeholder)
- ✅ **device_management_server.rs**: Songbird capability advertisement registration
- ✅ **cli.rs (modes)**: System status summary with primal detection & contextual tips

### Pure Rust (Core) ✅

| Component | Status | Lines |
|-----------|--------|-------|
| BearDog | ✅ Pure Rust | ~15,000 |
| Songbird | ✅ Pure Rust | ~20,000 |
| Neural API | ✅ Pure Rust | ~8,000 |
| biomeos-types | ✅ Pure Rust | ~2,000 |
| biomeos-spore | ✅ Pure Rust | ~1,500 |

### Bootstrap Scripts (To Evolve)

| Script | Purpose | Target Crate |
|--------|---------|--------------|
| `deploy.sh` | LiveSpore deployment | `biomeos-deploy` |
| `validate_spore.sh` | System validation | `biomeos-validate` |
| `create_sibling_spore.sh` | Genetic derivation | `biomeos-lineage` |
| `test_dark_forest_lan.sh` | Dark Forest testing | `biomeos-discovery` |

---

## Evolution Plan

### Phase 1: Core Infrastructure (COMPLETE)

- ✅ BearDog JSON-RPC over Unix sockets
- ✅ Songbird HTTP/TLS via BearDog
- ✅ Neural API graph execution
- ✅ 64-byte genetic seed structure
- ✅ Blake3 deterministic key derivation
- ✅ ChaCha20-Poly1305 encryption

### Phase 2: LiveSpore Rust Migration (IN PROGRESS)

```rust
// Current: deploy.sh (bash)
#!/bin/bash
SEED_B64=$(base64 -w0 .family.seed)
export BEARDOG_FAMILY_SEED=$SEED_B64
./primals/beardog server --socket $SOCKET &

// Future: biomeos-deploy (Rust)
use biomeos_spore::{LiveSpore, DeployConfig};

fn main() -> Result<()> {
    let spore = LiveSpore::discover(".")?;
    let config = DeployConfig::from_env()?;
    
    spore.validate_system(&config.manifest)?;
    spore.start_primals(&config)?;
    
    Ok(())
}
```

#### Migration Targets

**1. `biomeos-deploy` crate**
```toml
[package]
name = "biomeos-deploy"
version = "0.1.0"

[[bin]]
name = "spore"
path = "src/main.rs"

[features]
default = ["livespore", "dark-forest"]
livespore = []
dark-forest = ["chacha20poly1305"]
```

Commands:
- `spore deploy` - Start LiveSpore services
- `spore deploy --dark` - Dark Forest mode
- `spore validate` - Check system files
- `spore validate --update` - Auto-fix outdated files
- `spore info` - Show spore metadata

**2. `biomeos-validate` crate**
```rust
use biomeos_types::Manifest;
use std::path::Path;

pub struct SporeValidator {
    manifest: Manifest,
}

impl SporeValidator {
    pub fn validate(&self, spore_root: &Path) -> ValidationReport {
        let mut report = ValidationReport::new();
        
        for entry in &self.manifest.system {
            let actual = self.checksum(&spore_root.join(&entry.target))?;
            if actual != entry.expected {
                report.add_mismatch(entry, actual);
            }
        }
        
        report
    }
    
    pub fn update(&self, spore_root: &Path, report: &ValidationReport) -> Result<()> {
        for mismatch in &report.mismatches {
            std::fs::copy(&mismatch.source, &spore_root.join(&mismatch.target))?;
        }
        Ok(())
    }
}
```

**3. `biomeos-lineage` crate**
```rust
use blake3::Hasher;

pub struct GeneticLineage {
    genesis: [u8; 32],
    node_key: [u8; 32],
}

impl GeneticLineage {
    pub fn genesis() -> Self {
        let genesis: [u8; 32] = rand::random();
        let node_key = Self::derive_node_key(&genesis, "genesis");
        Self { genesis, node_key }
    }
    
    pub fn derive_sibling(parent: &Self, node_id: &str) -> Self {
        let node_key = Self::derive_node_key(&parent.genesis, node_id);
        Self { genesis: parent.genesis, node_key }
    }
    
    fn derive_node_key(genesis: &[u8; 32], node_id: &str) -> [u8; 32] {
        let mut hasher = Hasher::new_keyed(genesis);
        hasher.update(b"node-identity-v1:");
        hasher.update(node_id.as_bytes());
        *hasher.finalize().as_bytes()
    }
    
    pub fn broadcast_key(&self) -> [u8; 32] {
        let mut hasher = Hasher::new_keyed(&self.genesis);
        hasher.update(b"dark_forest_broadcast_v1");
        *hasher.finalize().as_bytes()
    }
}
```

**4. `biomeos-discovery` (Dark Forest)**
```rust
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use chacha20poly1305::aead::Aead;

pub struct DarkForestBeacon {
    lineage: GeneticLineage,
    cipher: ChaCha20Poly1305,
}

impl DarkForestBeacon {
    pub fn new(lineage: GeneticLineage) -> Self {
        let key = Key::from_slice(&lineage.broadcast_key());
        let cipher = ChaCha20Poly1305::new(key);
        Self { lineage, cipher }
    }
    
    pub fn encrypt_beacon(&self, node_info: &NodeInfo) -> Vec<u8> {
        let nonce = Nonce::from_slice(&rand::random::<[u8; 12]>());
        let plaintext = serde_json::to_vec(node_info).unwrap();
        
        let mut result = nonce.to_vec();
        result.extend(self.cipher.encrypt(nonce, plaintext.as_ref()).unwrap());
        result
    }
    
    pub fn try_decrypt_beacon(&self, encrypted: &[u8]) -> Option<NodeInfo> {
        let nonce = Nonce::from_slice(&encrypted[..12]);
        let ciphertext = &encrypted[12..];
        
        self.cipher.decrypt(nonce, ciphertext)
            .ok()
            .and_then(|p| serde_json::from_slice(&p).ok())
    }
}
```

### Phase 3: Neural API Graph Evolution

Replace TOML graph definitions with Rust macros for compile-time validation:

```rust
// Current: TOML graph
// [[graph.nodes]]
// id = "start-beardog"
// capability = "process.spawn"
// params.binary = "${SPORE_ROOT}/primals/beardog"

// Future: Rust macro
deployment_graph! {
    graph LiveSporeDeployment {
        node start_beardog {
            capability: "process.spawn",
            params: {
                binary: env!("SPORE_ROOT").join("primals/beardog"),
                socket: format!("/tmp/beardog-{}-{}.sock", family_id, node_id),
            },
            depends_on: [],
        }
        
        node start_songbird {
            capability: "process.spawn",
            params: {
                binary: env!("SPORE_ROOT").join("primals/songbird"),
                beardog_socket: start_beardog.socket,
            },
            depends_on: [start_beardog],
        }
    }
}
```

### Phase 4: Full ecoBin Compliance

All tools cross-compile to:
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`

```bash
# Build all targets
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu

# plasmidBin structure
plasmidBin/
├── primals/
│   ├── beardog/
│   │   ├── beardog-linux-x86_64
│   │   ├── beardog-linux-aarch64
│   │   └── beardog-active -> beardog-linux-x86_64
│   └── songbird/
│       └── ...
└── tools/
    ├── spore-linux-x86_64
    ├── spore-linux-aarch64
    └── spore-active -> spore-linux-x86_64
```

---

## Why Pure Rust?

### 1. **Type Safety**
```rust
// Rust: Compile-time error if seed is wrong size
let seed: [u8; 64] = read_seed(path)?;

// Bash: Runtime error, maybe, eventually
SEED=$(cat .family.seed)
```

### 2. **Cross-Platform**
```rust
// Same code works everywhere
#[cfg(target_os = "linux")]
fn socket_path() -> PathBuf { "/tmp/beardog.sock".into() }

#[cfg(target_os = "macos")]
fn socket_path() -> PathBuf { "/var/tmp/beardog.sock".into() }
```

### 3. **Zero-Copy Performance**
```rust
// Rust: Zero-copy deserialization
let beacon: Beacon = serde_json::from_slice(&bytes)?;

// Bash: String manipulation, copies everywhere
BEACON=$(echo "$BYTES" | jq '.beacon')
```

### 4. **Error Handling**
```rust
// Rust: Explicit, composable errors
fn deploy() -> Result<(), DeployError> {
    let spore = LiveSpore::discover(".")?;
    spore.validate()?;
    spore.start()?;
    Ok(())
}

// Bash: set -e and hope
set -euo pipefail
./validate.sh || exit 1
```

### 5. **Testability**
```rust
#[test]
fn test_lineage_derivation() {
    let parent = GeneticLineage::genesis();
    let child = GeneticLineage::derive_sibling(&parent, "node-beta");
    
    // Same genesis
    assert_eq!(parent.genesis, child.genesis);
    
    // Different node keys
    assert_ne!(parent.node_key, child.node_key);
    
    // Same broadcast key (from shared genesis)
    assert_eq!(parent.broadcast_key(), child.broadcast_key());
}
```

---

## Migration Priority

| Priority | Script | Rust Crate | Complexity |
|----------|--------|------------|------------|
| 1 | `validate_spore.sh` | `biomeos-validate` | Low |
| 2 | `deploy.sh` | `biomeos-deploy` | Medium |
| 3 | `create_sibling_spore.sh` | `biomeos-lineage` | Low |
| 4 | `test_dark_forest_lan.sh` | `biomeos-discovery` | Medium |
| 5 | TOML graphs | Rust macros | High |

---

## Compatibility

During migration, maintain both:

```bash
# Bootstrap (bash) - always works
./deploy.sh

# Evolved (Rust) - preferred when available
./primals/spore deploy
```

The Rust version will be placed alongside scripts until fully validated, then scripts move to `archive/`.

---

## Metrics

| Metric | Current | Target |
|--------|---------|--------|
| **Pure Rust** | 100% (core) | 100% (all) |
| **Script Dependencies** | bash, nc | None |
| **Cross-Platform** | Linux only | Linux, macOS, Windows |
| **Compile-Time Validation** | Partial | Full |
| **Tests** | 1,071 passing | 90%+ coverage |
| **Crates** | 21 | Stable |
| **TODOs** | 27 | 0 |
| **Production Stubs** | 0 | 0 ✅ |
| **Mock Isolation** | Complete | Testing only ✅ |
| **Hardcoded Paths** | 0 | 0 ✅ |
| **TODOs** | 52 | 0 |
| **Clippy Errors** | 0 | 0 |

---

*"Scripts are scaffolding. Rust is the building."*

*Updated: January 27, 2026*

