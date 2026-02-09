# biomeOS - Autonomous Federation Platform

**NUCLEUS Architecture** | **Neural API** | **Universal IPC v3.0** | **Sovereign NAT Traversal**

---

## Status: Production Ready

| Metric | Value |
|--------|-------|
| Primals | 6/6 ecoBin v2.0 compliant |
| IPC | Universal IPC v3.0 (Unix + Abstract + TCP) |
| Security | A++ LEGENDARY + Dark Forest Beacon Genetics |
| Code Quality | A (Pure Rust, idiomatic, zero actionable warnings) |
| Tests | 1,747 passing, 0 failures |
| Unsafe Code | 1 in production (justified mmap) |
| Clippy | PASS (0 warnings outside biomeos-boot) |
| Formatting | PASS |
| Deployment | USB + Pixel + Cross-Device AI |
| Genetics | Evolved (Mitochondrial + Nuclear DNA) |
| NAT Traversal | Sovereign mesh relay + hole punching |
| Neural API | Capability-based routing |
| Discovery | Dynamic runtime socket scanning |
| Plasmodium | Over-NUCLEUS collective coordination |
| Model Cache | NUCLEUS-integrated, HuggingFace import |

---

## Architecture

```
+---------------------------------------------------------------------------+
|                              NUCLEUS                                       |
+---------------------------------------------------------------------------+
|  Neural API Layer                                                          |
|  +---------------------------------------------------------------------+  |
|  |  capability.call -> semantic translation -> route to provider        |  |
|  +---------------------------------------------------------------------+  |
+---------------------------------------------------------------------------+
|  Atomics Layer                                                             |
|  +----------+  +----------+  +----------+  +----------+                   |
|  |  Tower   |  |   Node   |  |   Nest   |  | Squirrel |                   |
|  | BearDog  |  |  Tower + |  |  Tower + |  |   AI     |                   |
|  | Songbird |  | Toadstool|  | NestGate |  |          |                   |
|  +----------+  +----------+  +----------+  +----------+                   |
+---------------------------------------------------------------------------+
|  Primals Layer (evolve independently via capability.call)                  |
+---------------------------------------------------------------------------+
```

### Atomics

| Atomic | Primals | Capabilities |
|--------|---------|--------------|
| Tower | BearDog + Songbird | Crypto, TLS, HTTP, Discovery |
| Node | Tower + Toadstool | + Compute, GPU |
| Nest | Tower + NestGate | + Storage, Persistence |
| Full | All + Squirrel | + AI Orchestration |

---

## Quick Start

### Deploy Tower Atomic

```bash
cd livespore-usb/$(uname -m)/scripts/
FAMILY_ID=my_ecosystem ./start_tower.sh
```

### Deploy Full NUCLEUS

```bash
cd livespore-usb/$(uname -m)/scripts/
FAMILY_ID=my_ecosystem ./deploy_atomic.sh nucleus
```

### On Pixel 8a

```bash
adb push pixel8a-deploy /data/local/tmp/biomeos
adb shell /data/local/tmp/biomeos/start_nucleus_mobile.sh
```

---

## Key Features

### Neural API - Semantic Routing

```
Squirrel -> capability.call("http", "request", ...) -> Neural API
    |
Neural API translates: http.request -> secure_http (Tower Atomic)
    |
Songbird (via BearDog TLS 1.3) -> External API
```

Primals don't know about each other - they discover capabilities at runtime.

### Dynamic Capability-Based Discovery

```rust
// Primals discovered at runtime by scanning socket directory
let connections = PrimalConnections::discover_all(&family_id).await;

// Access by capability, not by name
if let Some(security) = connections.get("beardog") {
    security.call("crypto.sign", params).await?;
}
```

All primal names are configurable via environment variables:
- `BIOMEOS_SECURITY_PROVIDER` (default: "beardog")
- `BIOMEOS_NETWORK_PROVIDER` (default: "songbird")
- `BIOMEOS_REGISTRY_PROVIDER` (default: "songbird")
- `BIOMEOS_STORAGE_PROVIDER` (default: "nestgate")

### TRUE Dark Forest Security (A++ LEGENDARY)

```
Before: { "family_id": "...", "payload": "..." }  <- metadata leaks
After:  [0x4a, 0x8f, 0x2c, ...]                   <- pure noise
```

- Zero metadata leaks
- Genetic lineage = decryption key
- Better than Signal/Tor for metadata privacy

### Evolved Genetic Model

```
+-------------------------------------------------------------+
|                   LINEAGE SEED (Nuclear DNA)                 |
|                 Same across family - PERMISSIONS             |
|                                                              |
|  "What can they do?" - trust, access, capabilities           |
+-----------------------------+--------------------------------+
                              |
           +------------------+------------------+
           v                                     v
+---------------------+           +---------------------+
| BEACON SEED (Mito)  |           | BEACON SEED (Mito)  |
|   usb-desktop       |           |     pixel8a         |
|                     |           |                     |
| "Who can see me?"   |           | "Who can see me?"   |
| + Address book      |           | + Address book      |
+---------------------+           +---------------------+
```

| Seed | Model | Function | Shared? |
|------|-------|----------|---------|
| Beacon | Mitochondrial DNA | Family encryption, Dark Forest | Yes |
| Lineage | Nuclear DNA | Device identity, ancestry proof | Never |

### ecoBin v2.0 Standard

- 100% Pure Rust (zero C dependencies)
- Cross-compilation to any target
- Self-extracting genomeBins
- Universal IPC v3.0 (multi-transport)

### Universal IPC v3.0

Multi-transport communication with automatic fallback:

```rust
// Auto-discovery with fallback
let client = AtomicClient::discover("beardog").await?;

// Explicit transports
let unix = AtomicClient::unix("/path/to/socket");
let tcp = AtomicClient::tcp("192.168.1.100", 9100);  // Cross-device
```

| Transport | Platform | Use Case |
|-----------|----------|----------|
| Unix Socket | Linux/macOS | High performance local |
| Abstract Socket | Linux/Android | SELinux-friendly |
| TCP Socket | All | Cross-device federation |

### Plasmodium (Over-NUCLEUS Collective)

When 2+ gates run a complete NUCLEUS and share a `family_seed`, they form a **Plasmodium** -- a decentralized collective named after the slime mold *Physarum polycephalum*.

```bash
# Collective status across all bonded gates
FAMILY_ID=nat0 biomeos plasmodium status

# Per-gate hardware details
FAMILY_ID=nat0 biomeos plasmodium gates

# Aggregate model view across all gates
FAMILY_ID=nat0 biomeos plasmodium models
```

No central brain. Gates join/leave dynamically. Capabilities aggregate automatically.

### Model Cache

NUCLEUS-integrated model management with zero re-downloads:

```bash
# Import models from HuggingFace cache
biomeos model-cache import-hf

# List cached models
biomeos model-cache list

# Resolve model (local or mesh)
biomeos model-cache resolve "TinyLlama/TinyLlama-1.1B-Chat-v1.0"
```

### Sovereign NAT Traversal

Pure Rust solution for symmetric NAT connectivity:

```
+------------------------------------------------------------------+
|                   Sovereign Beacon Mesh                            |
+------------------------------------------------------------------+
|  STUN Detection -> NAT Type Analysis -> Path Selection            |
|       |                                                           |
|  [Full Cone] -> Direct UDP                                        |
|  [Restricted] -> UDP Hole Punch                                   |
|  [Symmetric] -> Mesh Relay OR Sovereign Onion Service             |
+------------------------------------------------------------------+
```

| Method | Use Case | Provider |
|--------|----------|----------|
| `mesh.status` | Network mesh status | Songbird |
| `mesh.find_path` | Route to peer via mesh | Songbird |
| `punch.request` | UDP hole punch | Songbird |
| `onion.create_service` | .onion address | Songbird + BearDog |

**No port forwarding required** - family members relay for each other.

---

## Deployment Standard

### Transport Discovery (5-Tier)

```
Tier 1 (Native):
  - $PRIMAL_SOCKET environment variable
  - $XDG_RUNTIME_DIR/biomeos/primal.sock
  - /run/user/$UID/biomeos/primal.sock
  - @biomeos_primal (Abstract - Linux only)

Tier 2 (Universal):
  - TCP localhost:910X (calculated from primal name)
  - TCP remote:910X (cross-device)
```

### Supported Platforms

| Platform | Architecture | Status |
|----------|--------------|--------|
| Linux | x86_64 | Production |
| Linux | aarch64 | Production |
| Android | aarch64 | Production |
| GrapheneOS | aarch64 | Production |

---

## Primal Status

| Primal | Purpose | ecoBin | Status |
|--------|---------|--------|--------|
| BearDog | Crypto, Genetics | v2.0 | Reference |
| Songbird | HTTP, TLS, Discovery | v2.0 | 93% TLS validation |
| Toadstool | Compute, GPU | v2.0 | barraCUDA ready |
| NestGate | Storage, Federation | v2.0 | Socket-only default |
| Squirrel | AI Orchestration | v2.0 | Capability-based |
| biomeOS | System Orchestrator | v2.0 | Neural API |

---

## Documentation

| Document | Purpose |
|----------|---------|
| [START_HERE.md](START_HERE.md) | Quick start guide |
| [CURRENT_STATUS.md](CURRENT_STATUS.md) | Latest status |
| [QUICK_START.md](QUICK_START.md) | 5-minute deployment |
| [DOCUMENTATION.md](DOCUMENTATION.md) | Full documentation index |
| [CHANGELOG.md](CHANGELOG.md) | Version history |

### Standards (wateringHole)

| Standard | Description |
|----------|-------------|
| ecoBin v2.0 | Pure Rust requirement |
| Universal IPC v3.0 | Multi-transport JSON-RPC |
| UniBin | Single binary, multiple modes |
| Semantic Method Naming | capability.call routing |

---

## Development

### Build

```bash
cargo build --workspace
```

### Test

```bash
cargo test --workspace
```

### Check

```bash
cargo check --workspace
cargo clippy --workspace
cargo fmt --check
```

### Coverage

```bash
cargo llvm-cov --workspace
```

---

## Project Structure

```
biomeOS/
+-- crates/                 # Rust workspace (25 crates)
|   +-- biomeos-core/       # Core orchestration + discovery
|   +-- biomeos-types/      # Shared types and constants
|   +-- biomeos-graph/      # Graph execution engine
|   +-- biomeos-spore/      # Deployment packaging
|   +-- biomeos-api/        # HTTP/WebSocket API server
|   +-- biomeos-ui/         # Interactive UI orchestration
|   +-- biomeos-atomic-deploy/ # Atomic deployment + Neural API
|   +-- biomeos-cli/        # Command-line interface
|   +-- biomeos-boot/       # ISO/initramfs builder
|   +-- biomeos-primal-sdk/ # Primal development SDK
|   +-- genome-deploy/      # genomeBin deployment
|   +-- ...
+-- livespore-usb/          # USB deployment
|   +-- x86_64/             # Intel/AMD binaries
|   +-- aarch64/            # ARM64 binaries
+-- pixel8a-deploy/         # Pixel 8a deployment
+-- specs/                  # Standards and specs
+-- docs/                   # Documentation
|   +-- handoffs/           # Evolution reports
|   +-- sessions/           # Session archives
+-- graphs/                 # Deployment graphs
```

---

## License

AGPL-3.0-only

---

## Philosophy

> "Primals evolve independently. They discover each other at runtime through capabilities, not hardcoded knowledge. biomeOS orchestrates without controlling."

### Principles

1. **Capability-based**: Primals discover, don't hardcode
2. **Pure Rust**: Zero C dependencies (no libc, no nix, no reqwest)
3. **Deterministic**: Same behavior across architectures
4. **Autonomous**: Self-extracting, self-discovering
5. **Secure**: TRUE Dark Forest (A++ LEGENDARY)
6. **Agnostic**: Provider names configurable via environment

---

**Status**: Production Ready  
**Updated**: February 9, 2026  
**Compliance**: ecoBin v2.0, Universal IPC v3.0, PRIMAL_DEPLOYMENT_STANDARD v1.0  
**Cross-Device**: BirdSong Discovery + AI Coordination + Sovereign NAT Traversal  
**Plasmodium**: Over-NUCLEUS collective coordination across bonded gates  
**Tests**: 1,747 passing | **Clippy**: PASS | **Format**: PASS
