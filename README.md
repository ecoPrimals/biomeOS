# biomeOS - Autonomous Federation Platform

**NUCLEUS Architecture** | **Neural API** | **Universal IPC v3.0** | **ecoBin v2.0**

---

## Status: Production Ready

| Metric | Value |
|--------|-------|
| Primals | 6/6 ecoBin v2.0 compliant |
| IPC | Universal IPC v3.0 (Unix + Abstract + TCP) |
| Security | A++ LEGENDARY |
| Code Quality | A |
| Tests | 822 passing |
| Unsafe Code | 0 blocks |
| Deployment | USB + Pixel + Cross-Device |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              NUCLEUS                                     │
├─────────────────────────────────────────────────────────────────────────┤
│  Neural API Layer                                                        │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  capability.call → semantic translation → route to provider      │   │
│  └─────────────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────────────┤
│  Atomics Layer                                                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
│  │  Tower   │  │   Node   │  │   Nest   │  │ Squirrel │              │
│  │ BearDog  │  │  Tower + │  │  Tower + │  │   AI     │              │
│  │ Songbird │  │ Toadstool│  │ NestGate │  │          │              │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘              │
├─────────────────────────────────────────────────────────────────────────┤
│  Primals Layer (evolve independently via capability.call)               │
└─────────────────────────────────────────────────────────────────────────┘
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
Squirrel → capability.call("http", "request", ...) → Neural API
    ↓
Neural API translates: http.request → secure_http (Tower Atomic)
    ↓
Songbird (via BearDog TLS 1.3) → External API
```

Primals don't know about each other - they discover capabilities at runtime.

### TRUE Dark Forest Security (A++ LEGENDARY)

```
Before: { "family_id": "...", "payload": "..." }  ← metadata leaks
After:  [0x4a, 0x8f, 0x2c, ...]                   ← pure noise
```

- Zero metadata leaks
- Genetic lineage = decryption key
- Better than Signal/Tor for metadata privacy

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
| [specs/PRIMAL_DEPLOYMENT_STANDARD.md](specs/PRIMAL_DEPLOYMENT_STANDARD.md) | Deployment spec |
| [specs/EVOLUTION_PATH.md](specs/EVOLUTION_PATH.md) | Scripts → Graphs |

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
cargo test --workspace --lib
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
├── crates/                 # Rust workspace crates
│   ├── biomeos-core/       # Core orchestration
│   ├── biomeos-types/      # Shared types
│   ├── biomeos-graph/      # Graph execution
│   ├── biomeos-spore/      # Deployment packaging
│   └── ...
├── livespore-usb/          # USB deployment
│   ├── x86_64/             # Intel/AMD binaries
│   └── aarch64/            # ARM64 binaries
├── pixel8a-deploy/         # Pixel 8a deployment
├── specs/                  # Standards & specs
├── docs/                   # Documentation
│   ├── handoffs/           # Evolution reports
│   └── sessions/           # Session archives
└── graphs/                 # Deployment graphs
```

---

## License

AGPL-3.0-only

---

## Philosophy

> "Primals evolve independently. They discover each other at runtime through capabilities, not hardcoded knowledge. biomeOS orchestrates without controlling."

### Principles

1. **Capability-based**: Primals discover, don't hardcode
2. **Pure Rust**: Zero C dependencies
3. **Deterministic**: Same behavior across architectures
4. **Autonomous**: Self-extracting, self-discovering
5. **Secure**: TRUE Dark Forest (A++ LEGENDARY)

---

**Status**: Production Ready  
**Version**: January 29, 2026  
**Compliance**: ecoBin v2.0, Universal IPC v3.0, PRIMAL_DEPLOYMENT_STANDARD v1.0
