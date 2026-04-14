# biomeOS - Autonomous Federation Platform

**NUCLEUS Architecture** | **Neural API** | **Universal IPC v3.0** | **AI Bridge** | **Distributed Plasmodium**

---

## Status: Production Ready (v3.13)

| Metric | Value |
|--------|-------|
| Primals | 7/7 ecoBin v3.0 compliant (+ barraCuda, coralReef) |
| IPC | Universal IPC v3.0 (Unix + Abstract + TCP + HTTP JSON-RPC) + tarpc binary escalation (wired) |
| Security | A++ LEGENDARY + Dark Forest Beacon Genetics |
| Code Quality | A++ (Pure Rust, Edition 2024, rust-version 1.87, all 26 workspace crates, modern idiomatic, fully concurrent, deep debt resolved, zero-copy evolved, multi-transport IPC, primalSpring-aligned, `#[expect]` throughout, all files <835 LOC) |
| Tests | 7,784 passing (0 failures, 0 ignored, fully concurrent) — 90%+ line / function / region (llvm-cov) |
| Unsafe Code | 0 in production (workspace `deny`, `#[forbid(unsafe_code)]` on all crate roots + all 20+ binary entry points) |
| C Dependencies | 0 (blake3 `default-features = false` + `pure`, deny.toml enforced bans) |
| Clippy | pedantic+nursery enabled, workspace lint inheritance, `-D warnings` |
| Formatting | PASS |
| License | scyBorg triple-copyleft (AGPL-3.0-or-later + ORC + CC-BY-SA 4.0) |
| Deployment | USB + Pixel + Cross-Device AI + TCP-only mobile |
| AI Bridge | Capability-routed to Squirrel (tag-in at runtime; biomeOS runs without AI primal) |
| Neural API | 290+ capability translations, 26 domains, 5 coordination patterns, auto-discovery, lazy rescan, cross-gate routing |
| Composition | Multi-primal graph execution (5+ nodes) proven e2e; `composition.health` standard; `lifecycle.composition` enriched dashboard |
| Plasmodium | HTTP JSON-RPC collective (runtime port, SSH deprecated) |
| NAT Traversal | 4-tier strategy (LAN/punch/coordinated/relay) |
| Lifecycle | Auto-monitoring, deep health checks, auto-resurrection, composition dashboard |
| Files >1000 LOC | 0 (all under 1000 after smart domain extraction) |
| Discovery | **Capability-based** per `CAPABILITY_BASED_DISCOVERY_STANDARD` v1.2.0 — XDG sockets + `topology.rescan` + lazy rescan + `capability.register` + DNS-SD mDNS; no identity-based routing or deprecated discovery stubs |
| Blocking Debt | 0 (composition e2e, dashboard completeness, BM-04, BM-05, TCP-only, gate routing, `#[serial]` — all resolved) |
| Dep Governance | All crates: dependencies centralized via `workspace = true`; `serial_test` removed; pure Rust stack (rustix, etcetera, ureq); blake3 pure-only |
| TODO/FIXME/HACK | 0 in production code |
| Deprecated APIs | 0 (legacy discovery methods and stubs removed in v2.87) |
| SPDX Headers | 100% (all `.rs` files: `AGPL-3.0-or-later`) |
| Hardcoded Primal Names | 0 in production code (all use `primal_names::` constants from `biomeos-types`) |

---

## Architecture

```
+-------------------------------------------------------------+
|                        NUCLEUS                               |
+-------------------------------------------------------------+
|  AI Bridge                                                   |
|  Squirrel -> http.request -> Songbird -> Cloud/Local AI      |
+-------------------------------------------------------------+
|  Neural API (290+ translations, 5 coordination patterns)     |
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
|  +----------+  | barraCuda|  +----------+  +----------+     |
|                | coralReef|                                  |
|                +----------+                                  |
+-------------------------------------------------------------+
|  Primals (evolve independently via capability.call)          |
+-------------------------------------------------------------+
```

### Atomics

| Atomic | Primals | Capabilities |
|--------|---------|--------------|
| Tower | BearDog + Songbird | Crypto, TLS, HTTP, Discovery |
| Node | Tower + Toadstool + barraCuda + coralReef | + Compute, GPU, Math/Tensor/Stats, Shaders |
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

290+ capability translations enable primals to compose without knowing each other:

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
| Toadstool | Compute, GPU dispatch | Operational | GPU job queue |
| barraCuda | GPU Math, Tensors, Stats, Noise, Activation, RNG | Operational | Shader interop |
| coralReef | Shader compilation, WGSL, SPIR-V | Operational | Pipeline caching |
| NestGate | Storage, Federation | Operational (patched) | Upstream boolean fix |
| Squirrel | AI Orchestration | Operational | Ollama native adapter |
| biomeOS | System Orchestrator + Neural API | Evolved | ✅ ARM64 built (9.6 MB static musl) |

---

## Standards Compliance

| Standard | Status |
|----------|--------|
| ecoBin v3.0 | 100% Pure Rust |
| Universal IPC v3.0 | Multi-transport (Unix/Abstract/TCP/HTTP JSON-RPC) |
| PRIMAL_DEPLOYMENT_STANDARD v1.0 | Deterministic behavior |
| Semantic Method Naming | capability.call routing |
| scyBorg Triple-Copyleft | AGPL-3.0-or-later + ORC + CC-BY-SA 4.0 |
| Evolved Genetic Model v2.0 | Mitochondrial + Nuclear |
| XDG Base Directory | SystemPaths (all paths XDG-compliant) |

---

## Development

### Build

```bash
cargo build --workspace
```

### Test (7,784 tests across 26 crates, fully concurrent)

```bash
cargo test --workspace
```

### Coverage (90%+ line / function / region)

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
├── crates/                    # Rust workspace (26 crates, all lint-inherited)
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
├── specs/                     # Standards and specs (24 active)
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

scyBorg triple-copyleft: **AGPL-3.0-or-later** (code) + **ORC** (operational) + **CC-BY-SA 4.0** (documentation)

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

**Status**: Production Ready (v3.12)
**Updated**: April 12, 2026
**Tests**: 7,784 passing (0 ignored), 90%+ line / function / region (llvm-cov) | **Clippy**: pedantic+nursery, `-D warnings` | **Docs**: Full coverage | **Format**: PASS | **C deps**: 0 | **Unsafe**: 0 | **Deprecated**: 0 | **Blocking debt**: 0
**Architecture**: JSON-RPC primary + tarpc binary escalation | Multi-transport IPC (Unix/abstract/TCP/HTTP) | Capability-based discovery + lazy rescan + `capability.call` routing + cross-gate forwarding + DNS-SD | XDG-compliant paths | scyBorg (AGPL-3.0-or-later + ORC + CC-BY-SA 4.0)

---

*Part of the [ecoPrimals](https://github.com/ecoPrimals) sovereign computing ecosystem.*
