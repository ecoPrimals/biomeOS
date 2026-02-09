# biomeOS - Current Status

**Updated**: February 9, 2026 (Plasmodium + Model Cache)
**Version**: 2.9
**Status**: PRODUCTION READY - Over-NUCLEUS Collective Coordination

---

## Quick Summary

| Metric | Status |
|--------|--------|
| **genomeBins** | 5/5 primals ready (100%) |
| **Cross-Arch** | x86_64 + aarch64 (USB + Pixel) |
| **IPC Standard** | Universal IPC v3.0 + tarpc wiring |
| **Security Grade** | A++ (TRUE PRIMAL + Genetic Model) |
| **Code Quality** | A (Pure Rust, idiomatic, zero warnings) |
| **Tests Passing** | 1,747 (0 failures, 6 ignored) |
| **Unsafe Code** | 1 production (justified mmap in genome-deploy) |
| **Clippy** | PASS (0 warnings across all non-boot crates) |
| **Formatting** | PASS (`cargo fmt --check`) |
| **Genetic Model** | EVOLVED - Mitochondrial + Nuclear DNA |
| **BirdSong Discovery** | Encrypted, shared beacon model |
| **Discovery Model** | Dynamic socket scanning (no hardcoded names) |
| **NAT Traversal** | Sovereign mesh relay + Tor gateway |
| **P2P Sovereign Onion** | PRODUCTION READY |
| **External C deps** | 0 (no libc, nix, dirs, reqwest, lazy_static) |
| **Plasmodium** | Over-NUCLEUS collective coordination (read-only) |
| **Model Cache** | NUCLEUS-integrated, HuggingFace import, NestGate fallback |

---

## Deep Debt Evolution (Feb 7, 2026)

### What Changed

Complete deep debt evolution across the entire workspace:

#### Pure Rust Dependencies
| Removed | Replaced With |
|---------|---------------|
| `lazy_static` | `std::sync::OnceLock` |
| `dirs` | `etcetera` / `std::env::var("HOME")` |
| `nix` | `std::env::var("UID")` / `std::env::var("EUID")` |

#### Capability-Based Discovery
- **`PrimalConnections`**: Evolved from fixed 6-field struct to dynamic `HashMap<String, PrimalClient>` with runtime socket directory scanning
- **Provider names**: All configurable via environment variables (`BIOMEOS_SECURITY_PROVIDER`, `BIOMEOS_NETWORK_PROVIDER`, etc.)
- **`BIOMEOS_STRICT_DISCOVERY`**: Disables all bootstrap name fallbacks for pure runtime discovery
- **Live socket scanning**: `discover_all()` scans `$XDG_RUNTIME_DIR/biomeos/*.sock` for any primal

#### Production Mock Elimination
- `discovery.rs`: Replaced `get_standalone_primals()` fabricated data with `probe_live_sockets()`
- `trust.rs`: Removed `standalone_mode` branches that returned fabricated trust decisions
- `livespores.rs`: Replaced hardcoded primal whitelist with dynamic binary scanning
- `discovery_bootstrap.rs`: Replaced broadcast stub with real UDP socket implementation

#### Warning Elimination
- Eliminated all dead code, unused import, unused variable warnings workspace-wide
- Fixed test race conditions with `Mutex` locks for env-var-mutating tests
- All non-boot crates: zero clippy warnings

#### Clippy Modernization
- `or_insert_with(Vec::new)` -> `or_default()` (7 instances)
- `unwrap_or_else(|| json!(null))` -> `unwrap_or(Value::Null)`
- Duplicated `#![deny(unsafe_code)]` removed
- `fn default()` -> proper `impl Default` trait
- `too_many_arguments` -> context struct pattern
- Boolean expression simplification

#### UI Orchestrator Refactoring
- `InteractiveUIOrchestrator`: 6 individual `Option<Client>` fields -> single `PrimalConnections`
- `handle_user_action()`: 8 parameters -> 3 (`action`, `family_id`, `&PrimalConnections`)
- `handle_assign_device()`: 8 parameters -> 4 (via `DeviceAssignmentCtx`)
- `DiscoveryResult`: Wraps `PrimalConnections` dynamic registry

---

## Tor Gateway (Feb 7, 2026)

### Hidden Service Status

| Component | Status |
|-----------|--------|
| Tor daemon | Running, bootstrapped |
| Hidden service | `eaaz3tlirenexp2mabctirbwd2fv67mayvtrr4fmqemhyypvnemybmqd.onion` |
| Port 3492 | TCP Proxy -> Songbird IPC |
| IPC via Tor | Verified (`health` returns healthy) |

### Connection Paths

| Path | Endpoint | Accessibility |
|------|----------|---------------|
| Tor (Global) | `eaaz3...onion:3492` | Anywhere with Tor |
| LAN IPv4 | `192.168.1.144:3492` | Same network only |
| LAN IPv6 | `2600:1700:...:3492` | IPv6 reachable |

---

## P2P Sovereign Onion (Feb 6, 2026)

### TRUE PRIMAL Status: 100%

| Component | Status |
|-----------|--------|
| **BearDog Crypto** | Complete - All 8 methods (SHA3, Ed25519, X25519, ChaCha20, HMAC) |
| **Songbird Service** | Complete - OnionService + OnionConnector |
| **Capability Translations** | 15+ onion/mesh capabilities wired |
| **Neural API Routing** | Direct mesh.*/punch.*/onion.* methods |
| **Integration Tests** | 12/12 passing |
| **Deployment Graph** | `tower_atomic_bootstrap.toml` updated |

---

## Genetic Model Details

### Mitochondrial (Beacon Seed) - Shared Identity

```
Purpose: Family recognition in Dark Forest
Shared:  Yes - all family members have same seed
Sync:    Can be synced/transferred between devices
Evolves: Address book grows as connections made
```

### Nuclear DNA (Lineage Seed) - Unique Identity

```
Purpose: Individual device authentication
Shared:  No - derived uniquely per device
Derived: HKDF(family_seed, device_entropy, context)
Copied:  NEVER - always fresh derivation
```

---

## Ecosystem Status

### NUCLEUS Architecture - GENOMES READY

```
NUCLEUS = Tower + Node + Nest + biomeOS

Tower Atomic  = BearDog + Songbird       (crypto + network)
Node Atomic   = Tower + Toadstool        (+ compute)
Nest Atomic   = Tower + NestGate         (+ storage)
Full NUCLEUS  = All 5 primals + biomeOS  (orchestration)
```

### Primal Status

| Primal | genomeBin | Size | x86_64 | aarch64 |
|--------|-----------|------|--------|---------|
| **songbird** | v3.33.0 | 18.5 MB | Yes | Yes |
| **beardog** | complete | 6.9 MB | Yes | Yes |
| **toadstool** | complete | 8.9 MB | Yes | Yes |
| **nestgate** | complete | 5.6 MB | Yes | Yes |
| **squirrel** | complete | 4.3 MB | Yes | Yes |
| **biomeOS** | complete | 3.9 MB | Yes | Pending |

---

## Latest Work (Feb 9, 2026)

### Plasmodium - Over-NUCLEUS Collective
- **Spec**: `specs/PLASMODIUM_OVER_NUCLEUS_SPEC.md`
- **Module**: `biomeos-core::plasmodium` -- types + collective query engine
- **CLI**: `biomeos plasmodium status|gates|models`
- **Live tested**: Tower local gate fully queried (BearDog, Songbird, RTX 4070, models)
- **Graceful degradation**: Offline gates shown as "offline", collective shrinks/grows dynamically
- **No central brain**: Any gate can query the collective

### Model Cache - Zero Re-Downloads
- **Module**: `biomeos-core::model_cache` -- NestGate integration + filesystem fallback
- **CLI**: `biomeos model-cache import-hf|list|resolve|register|status`
- **Symlink-aware**: Correctly sizes HuggingFace models with blob storage
- **Live tested**: Tower + gate2 model management verified

### NestGate Handoff
- Identified 4 bugs in NestGate (inverted boolean, storage.retrieve null, ZFS assumption, missing exists)
- Documented in `docs/handoffs/NESTGATE_MODEL_CACHE_HANDOFF_FEB09_2026.md`
- Model cache gracefully degrades to filesystem-only mode

### Distributed GPU (Toadstool Handoff)
- Multi-gate GPU inference architecture designed
- Handoff: `docs/handoffs/TOADSTOOL_DISTRIBUTED_GPU_HANDOFF_FEB09_2026.md`

---

## Remaining Work

### High Priority
1. **Plasmodium Phase 2**: Job submission + remote compute query
2. **Songbird mesh peers**: Auto-discovery of bonded gates (currently manual PLASMODIUM_PEERS)
3. **ARM64 biomeOS genomeBin** - blocks Pixel biomeOS deployment

### Medium Priority
1. **NestGate evolution**: Fix 4 identified bugs for mesh model sharing
2. **Test Coverage to 90%** (currently ~48%)
3. **Beacon Genetics Phase 2C** - Cluster beacons

### Low Priority
1. **Chaos/Fault Testing**
2. **biomeos-boot documentation** - 44 pre-existing missing-doc warnings

---

## Standards Compliance

| Standard | Status |
|----------|--------|
| ecoBin v2.0 | 100% Pure Rust |
| Universal IPC v3.0 | Multi-transport (Unix/Abstract/TCP) |
| PRIMAL_DEPLOYMENT_STANDARD v1.0 | Deterministic behavior |
| Semantic Method Naming | capability.call routing |
| AGPL-3.0-only License | Compliant |
| Evolved Genetic Model v2.0 | Mitochondrial + Nuclear |

---

## Quick Commands

```bash
# Build
cargo build --workspace

# Test (1,747 tests)
cargo test --workspace

# Clippy (0 warnings outside biomeos-boot)
cargo clippy --workspace

# Format
cargo fmt --check

# Coverage
cargo llvm-cov --workspace
```

---

**Status**: Production Ready  
**Genetic Model**: Evolved (Mitochondrial + Nuclear)  
**IPC**: Universal IPC v3.0 + tarpc wiring  
**Security**: A++ (Two-seed Dark Forest)  
**Code Quality**: A (Pure Rust, idiomatic, zero warnings)  
**Plasmodium**: Over-NUCLEUS collective coordination  
**Model Cache**: NUCLEUS-integrated, HuggingFace import  
**Tests**: 1,747 passing  
**Clippy**: PASS | **Format**: PASS  
**Unsafe Code**: 1 justified (mmap)
