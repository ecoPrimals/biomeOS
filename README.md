# biomeOS - Autonomous Federation Platform

**NUCLEUS Architecture** | **Neural API** | **Universal IPC v3.0** | **AI Bridge** | **Distributed Plasmodium**

---

## Status: Production Ready (v2.50)

| Metric | Value |
|--------|-------|
| Primals | 6/6 ecoBin v3.0 compliant |
| IPC | Universal IPC v3.0 (Unix + Abstract + TCP + HTTP JSON-RPC) + tarpc binary escalation (wired) |
| Security | A++ LEGENDARY + Dark Forest Beacon Genetics |
| Code Quality | A++ (Pure Rust, Edition 2024 all crates, modern idiomatic, fully concurrent, zero warnings, full doc coverage, deep debt audit, zero-copy) |
| Tests | 5,203 passing (0 failures) — fully concurrent (~82% line coverage) |
| Unsafe Code | 0 in production |
| Clippy | PASS (0 warnings, pedantic+nursery, all 23 crates via workspace lint inheritance) |
| Formatting | PASS |
| License | scyBorg triple-copyleft (AGPL-3.0 + ORC + CC-BY-SA 4.0) |
| Deployment | USB + Pixel + Cross-Device AI |
| AI Bridge | Squirrel -> Songbird -> Cloud/Local AI (validated) |
| Neural API | 285+ capability translations, 25 domains, 5 coordination patterns (Sequential, Parallel, ConditionalDag, Pipeline, Continuous) |
| Coordination | Sequential + Parallel + ConditionalDag + Pipeline (streaming) + Continuous (60Hz tick) |
| Streaming | PipelineExecutor (mpsc channels), NDJSON streaming client, JSON-RPC 2.0 notifications |
| Continuous | ContinuousExecutor (60Hz tick), push events, sensor routing |
| XR/VR | Stereo rendering, motion capture, haptic feedback pipeline |
| Surgical Domain | Anatomy models, tissue physics, biosignals, pharmacokinetics |
| Plasmodium | HTTP JSON-RPC collective (runtime port, SSH deprecated) |
| NAT Traversal | 4-tier strategy (LAN/punch/coordinated/relay) |
| Agents | Plasmodium Agent Model (meld/split/mix routing contexts) |
| Lifecycle | Auto-monitoring, deep health checks, auto-resurrection |

---

## Architecture

```
+-------------------------------------------------------------+
|                        NUCLEUS                               |
+-------------------------------------------------------------+
|  AI Bridge                                                   |
|  Squirrel -> http.request -> Songbird -> Cloud/Local AI      |
+-------------------------------------------------------------+
|  Neural API (285+ translations, 5 coordination patterns)     |
|  graph.execute   -> Sequential / Parallel / ConditionalDag   |
|  graph.execute_pipeline -> Pipeline (streaming mpsc channels) |
|  graph.start_continuous -> Continuous (60Hz tick loop)        |
|  capability.call -> translate -> route to provider            |
+-------------------------------------------------------------+
|  Atomics                                                     |
|  +----------+  +----------+  +----------+  +----------+     |
|  |  Tower   |  |   Node   |  |   Nest   |  | Squirrel |     |
|  | BearDog  |  |  Tower + |  |  Tower + |  |   AI     |     |
|  | Songbird |  | Toadstool|  | NestGate |  |          |     |
|  +----------+  +----------+  +----------+  +----------+     |
+-------------------------------------------------------------+
|  Primals (evolve independently via capability.call)          |
+-------------------------------------------------------------+
```

### Atomics

| Atomic | Primals | Capabilities |
|--------|---------|--------------|
| Tower | BearDog + Songbird | Crypto, TLS, HTTP, Discovery |
| Node | Tower + Toadstool | + Compute, GPU |
| Nest | Tower + NestGate | + Storage, Persistence |
| Full | All + Squirrel | + AI Orchestration, Neural API |

---

## Quick Start

### Deploy Full NUCLEUS (Pure Rust)

```bash
biomeos nucleus start --mode full --node-id tower1
```

### Deploy Tower Atomic Only

```bash
biomeos nucleus start --mode tower --node-id tower1
```

### On Pixel 8a

```bash
adb push pixel8a-deploy /data/local/tmp/biomeos
adb shell /data/local/tmp/biomeos/start_nucleus_mobile.sh
```

The `biomeos nucleus start` command:
- Detects if an ecosystem is already running (bootstrap vs. coordinated mode)
- Discovers primal binaries from `livespore-usb/`, `plasmidBin/`, `target/release/`, `$PATH`
- Starts primals in dependency order with family-suffixed sockets
- Integrates with `LifecycleManager` for ongoing deep health monitoring (JSON-RPC ping)
- Auto-resurrects degraded primals with exponential backoff
- Graceful coordinated shutdown via SIGTERM with dependency ordering

---

## Validated AI Bridge

Squirrel discovers HTTP capability via explicit socket path, routes AI queries
through Songbird's HTTP handler, with BearDog providing TLS for HTTPS:

```
Local AI:    Songbird -> HTTP POST -> Ollama (phi3/tinyllama)  ~2s
Cloud AI:    Squirrel -> Songbird -> BearDog TLS -> Anthropic  ~786ms
Neural API:  proxy_http -> Songbird -> BearDog TLS -> HTTPS    ~756ms
```

### Test AI Bridge

```bash
# Local AI via Songbird
echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"POST","url":"http://localhost:11434/v1/chat/completions","headers":{"content-type":"application/json"},"body":"{\"model\":\"tinyllama\",\"messages\":[{\"role\":\"user\",\"content\":\"Name the largest planet. One word.\"}],\"max_tokens\":10}"},"id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/songbird.sock -w 15 -q 1

# Cloud AI via Squirrel
echo '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"Name the largest ocean. One word.","model":"claude-3-haiku-20240307","max_tokens":10},"id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/squirrel.sock -w 15 -q 1
```

---

## Neural API - Semantic Routing

285+ capability translations enable primals to compose without knowing each other:

```
Squirrel -> capability.call("http", "request", ...) -> Neural API
    |
Neural API translates: http.request -> songbird.http_request
    |
Songbird (via BearDog TLS 1.3) -> External API
```

Primals don't know about each other - they discover capabilities at runtime.

---

## Plasmodium (Over-NUCLEUS Collective)

When 2+ gates run a complete NUCLEUS and share a `.family.seed`, they form a
**Plasmodium** -- a decentralized collective named after *Physarum polycephalum*.

```
Tower (RTX 4070, 31 GB RAM, 24 cores)  <-HTTP JSON-RPC->  gate2 (RTX 3090, 251 GB RAM, 128 cores)
                              |
                Collective: 36 GB VRAM, 282 GB RAM, 152 CPU
```

```bash
# Collective status across all bonded gates
biomeos plasmodium status

# Per-gate hardware details
biomeos plasmodium gates

# Aggregate model view across all gates
biomeos plasmodium models
# Family ID is auto-discovered from .family.seed (or FAMILY_ID env var)
```

No central brain. Gates join/leave dynamically. Capabilities aggregate automatically.
Transport: `AtomicClient::http()` → Songbird HTTP JSON-RPC gateway (port 8080).
Port discovery: `mesh.peers` beacon exchange → `SONGBIRD_MESH_PORT` → default 8080.

---

## TRUE Dark Forest Security (A++ LEGENDARY)

```
Before: { "family_id": "...", "payload": "..." }  <- metadata leaks
After:  [0x4a, 0x8f, 0x2c, ...]                   <- pure noise
```

- Zero metadata leaks
- Genetic lineage = decryption key
- Better than Signal/Tor for metadata privacy

---

## Evolved Genetic Model

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

---

## Primal Status

| Primal | Purpose | Status | Next Evolution |
|--------|---------|--------|----------------|
| BearDog | Crypto, Genetics | Reference | Stable |
| Songbird | HTTP, TLS, Discovery, Mesh | 90% | Mesh state fix, UDP discovery fix |
| Toadstool | Compute, GPU | Operational | GPU job queue |
| NestGate | Storage, Federation | Operational (patched) | Upstream boolean fix |
| Squirrel | AI Orchestration | Operational | Ollama native adapter |
| biomeOS | System Orchestrator + Neural API | Evolved | ARM64 genomeBin |

---

## Standards Compliance

| Standard | Status |
|----------|--------|
| ecoBin v3.0 | 100% Pure Rust |
| Universal IPC v3.0 | Multi-transport (Unix/Abstract/TCP/HTTP JSON-RPC) |
| PRIMAL_DEPLOYMENT_STANDARD v1.0 | Deterministic behavior |
| Semantic Method Naming | capability.call routing |
| AGPL-3.0-only License | Compliant |
| Evolved Genetic Model v2.0 | Mitochondrial + Nuclear |
| XDG Base Directory | SystemPaths (all paths XDG-compliant) |

---

## Development

### Build

```bash
cargo build --workspace
```

### Test (5,161+ tests — fully concurrent)

```bash
cargo test --workspace
```

### Coverage (78% line)

```bash
cargo llvm-cov --workspace
```

### Check

```bash
cargo check --workspace
cargo clippy --workspace   # 0 warnings
cargo fmt --check
cargo doc --workspace      # 0 missing_docs warnings
```

---

## Project Structure

```
biomeOS/
├── crates/                    # Rust workspace (25 crates)
│   ├── biomeos/               # Main binary (CLI + nucleus modes)
│   ├── biomeos-core/          # Core orchestration + discovery + plasmodium
│   ├── biomeos-types/         # Shared types, SystemPaths, capability taxonomy
│   ├── biomeos-cli/           # Command-line interface + TUI
│   ├── biomeos-api/           # HTTP/WebSocket API server
│   ├── biomeos-compute/       # Fractal compute architecture
│   ├── biomeos-graph/         # Graph execution engine (sled→redb)
│   ├── biomeos-spore/         # Deployment packaging + beacon genetics
│   ├── biomeos-ui/            # Interactive UI orchestration
│   ├── biomeos-atomic-deploy/ # Atomic deployment + Neural API + Lifecycle
│   ├── biomeos-deploy/        # QEMU/VM deployment
│   ├── biomeos-boot/          # ISO/initramfs builder
│   ├── biomeos-nucleus/       # NUCLEUS lifecycle management
│   ├── biomeos-federation/    # Federation + secure discovery
│   ├── biomeos-genome-factory/# genomeBin build + compose + replicate
│   ├── biomeos-genomebin-v3/  # genomeBin v3.0 binary format
│   ├── biomeos-primal-sdk/    # Primal development SDK
│   ├── biomeos-genome-deploy/  # genomeBin deployment
│   ├── neural-api-client-sync/ # Synchronous Neural API client
│   └── ...                    # + 8 more (manifest, niche, chimera, test-utils, etc.)
├── livespore-usb/             # USB deployment
│   ├── x86_64/                # Intel/AMD binaries
│   └── aarch64/               # ARM64 binaries
├── pixel8a-deploy/            # Pixel 8a deployment
├── specs/                     # Standards and specs (19 active)
├── docs/                      # Architecture docs (handoffs in ecoPrimals/wateringHole/)
├── graphs/                    # Deployment graphs
└── scripts/                   # Startup and build scripts
```

---

## Documentation

| Document | Purpose |
|----------|---------|
| [START_HERE.md](START_HERE.md) | Architecture overview |
| [CURRENT_STATUS.md](CURRENT_STATUS.md) | Validated systems + evolution needs |
| [QUICK_START.md](QUICK_START.md) | 5-minute deployment |
| [CHANGELOG.md](CHANGELOG.md) | Version history |
| [DOCUMENTATION.md](DOCUMENTATION.md) | Full documentation index |

---

## License

AGPL-3.0-only

---

## Philosophy

> "Primals evolve independently. They discover each other at runtime through capabilities, not hardcoded knowledge. biomeOS orchestrates without controlling."

### Principles

1. **Capability-based**: Primals discover, don't hardcode
2. **Pure Rust**: Zero C dependencies (`rustix` for POSIX syscalls, `/proc` for metrics)
3. **XDG-compliant**: All paths via `SystemPaths` -- portable across systems
4. **Deterministic**: Same behavior across architectures
5. **Autonomous**: Self-extracting, self-discovering
6. **Secure**: TRUE Dark Forest (A++ LEGENDARY)
7. **Self-healing**: LifecycleManager auto-resurrects degraded primals

---

**Status**: Production Ready (v2.49)
**Updated**: March 16, 2026
**Deep Audit Evolution**: Edition 2024 (all 25 crates), capability-based discovery, tarpc binary protocol, zero-copy Arc<str>, lint hardening (deny unwrap_used/expect_used), 0 files >1000 lines, circuit-breaker resilient dispatch, cost-aware Pathway Learner, manifest discovery fallback
**Tests**: 5,161+ passing, fully concurrent (78% line, 80% function) | **Clippy**: PASS (0 warnings) | **Docs**: Full coverage | **Format**: PASS | **C deps**: 0 | **Unsafe**: 0
**Architecture**: JSON-RPC primary + tarpc binary escalation | Capability-based discovery | XDG-compliant paths | AGPL-3.0-only
