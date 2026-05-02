# Start Here - biomeOS

**Last Updated**: April 30, 2026
**Status**: Production Ready (v3.35) — 8,064+ tests (0 failures, fully concurrent), Edition 2024, rust-version 1.87, all 25 workspace crates, 90%+ line / function / region (llvm-cov), Clippy PASS (0 warnings, pedantic+nursery), 0 C deps (blake3 pure-only), 0 unsafe prod (`#[forbid(unsafe_code)]` all crate roots + all 20+ binaries), 0 deprecated APIs, 0 TODO/FIXME, 0 hardcoded values in production, `#[expect(reason)]` throughout, capability-based discovery compliant, `primal.list` on neural-api + API socket, graph executor operation fallback, bootstrap tolerance for optional nodes, scyBorg triple-copyleft (AGPL-3.0-or-later)

---

## What is biomeOS?

biomeOS is the **ecosystem orchestrator** for ecoPrimals - a federation of autonomous Rust programs (primals) that communicate via capability-based discovery and Universal IPC v3.0.

### Key Concepts

- **Primals**: Self-contained Rust binaries with specific capabilities
- **Atomics**: Primal combinations (Tower = BearDog + Songbird)
- **NUCLEUS**: Complete system (Tower + Node + Nest + Squirrel)
- **Neural API**: Semantic routing via `capability.call` (320+ translations, 27 domains incl. tensor, part of biomeOS)
- **Universal IPC v3.0**: Multi-transport communication (Unix/Abstract/TCP/HTTP JSON-RPC)
- **Dark Forest**: Zero-metadata beacon discovery using genetic lineage
- **Plasmodium**: Over-NUCLEUS collective coordination (slime mold pattern)
- **AI Bridge**: Squirrel -> Songbird HTTP -> Cloud/Local AI
- **LifecycleManager**: Deep health monitoring, auto-resurrection, coordinated shutdown
- **SystemPaths**: XDG-compliant path resolution (no hardcoded paths)
- **Capability Taxonomy**: Canonical primal-to-capability mapping (single source of truth)
- **NAT Traversal**: 4-tier connection strategy (LAN/punch/coordinated/relay)
- **Plasmodium Agents**: Dynamic routing contexts that compose capabilities across gates
- **Provenance Trio**: loamSpine (permanence) + rhizoCrypt (ephemeral DAG) + sweetGrass (attribution) — 4 deploy graphs, 35+ capability translations

---

## Quick Start

### 1. Deploy Full NUCLEUS

```bash
biomeos nucleus start --mode full --node-id tower1
```

### 2. Verify AI Bridge

```bash
# Local AI (Ollama via Songbird)
echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"POST","url":"http://localhost:11434/v1/chat/completions","headers":{"content-type":"application/json"},"body":"{\"model\":\"tinyllama\",\"messages\":[{\"role\":\"user\",\"content\":\"hello\"}],\"max_tokens\":10}"},"id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/songbird.sock -w 15 -q 1

# Cloud AI (Anthropic via Squirrel)
echo '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"hello","model":"claude-3-haiku-20240307","max_tokens":10},"id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/squirrel.sock -w 15 -q 1
```

### 3. Use capability.call (Neural API)

```bash
# Discover who provides a capability
echo '{"jsonrpc":"2.0","method":"capability.discover","params":{"capability":"crypto"},"id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/neural-api.sock -w 2 -q 1

# List all capability translations
echo '{"jsonrpc":"2.0","method":"capability.list_translations","params":{},"id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/neural-api.sock -w 2 -q 1
```

---

## Architecture

```
+-------------------------------------------------------------+
|                        NUCLEUS                               |
+-------------------------------------------------------------+
|  Layer 3: AI Bridge                                          |
|  Squirrel -> http.request -> Songbird -> Cloud/Local AI      |
+-------------------------------------------------------------+
|  Layer 2: Neural API (320+ semantic translations)            |
|  capability.call -> translate -> route to provider           |
+-------------------------------------------------------------+
|  Layer 1: Atomics                                            |
|  +----------+  +----------+  +----------+  +----------+     |
|  |  Tower   |  |   Node   |  |   Nest   |  | Squirrel |     |
|  | BearDog  |  |  Tower   |  |  Tower   |  |  AI/MCP  |     |
|  | Songbird |  | Toadstool|  | NestGate |  |          |     |
|  +----------+  +----------+  +----------+  +----------+     |
+-------------------------------------------------------------+
|  Layer 0: Primals (evolve independently via capability.call) |
+-------------------------------------------------------------+
```

### How Primals Compose

Primals evolve independently. They don't know about each other -- they discover
capabilities at runtime. The Neural API's semantic translation layer is the glue:

```
Squirrel wants: "http.request"
  -> Neural API translation: http.request -> songbird.http_request
  -> Songbird provides: http.request on its JSON-RPC socket
  -> BearDog provides: TLS crypto for HTTPS (discovered by Songbird)
```

No primal imports another primal's code. They compose through sockets and JSON-RPC.

### Deep Debt Principles

1. **Pure Rust**: Zero C dependencies (safe `rustix` crate for POSIX syscalls)
2. **Capability-based**: Primals discover each other at runtime, not by name
3. **XDG-compliant**: All paths via `SystemPaths` -- no hardcoded `/tmp` or `/run/user/1000`
4. **No production mocks**: Stubs replaced with real implementations or honest errors
5. **Idiomatic Rust**: Edition 2024, modern patterns (LazyLock, let-chains, native async traits path)
6. **Zero warnings**: Clippy pedantic+nursery clean, full doc coverage, 0 production files >800 lines
7. **Self-healing**: LifecycleManager auto-resurrects degraded primals
8. **Tested**: 7,814+ tests (0 failures), 90%+ line / function / region (llvm-cov), fully concurrent suite
9. **Concurrent**: All non-chaos tests run in parallel — dependency injection, `tokio::time::pause()`, and `ReadySender`/`ReadyReceiver` eliminate global state races and sleep-before-connect patterns (zero production/test sleeps for timing hacks)

---

## Validated Chains

| Chain | Path | Status |
|-------|------|--------|
| Local AI | Songbird -> HTTP -> Ollama (phi3) | Validated |
| Cloud AI | Squirrel -> Songbird -> BearDog TLS -> Anthropic | Validated |
| Nest Atomic | NestGate storage.exists/store/retrieve | Validated (Tower + gate2) |
| Plasmodium | HTTP JSON-RPC 2-gate collective | Validated |
| Covalent Bond | HTTP transport to gate2:8080 | Validated (beacon discovery pending) |
| Device Enrollment | Blake3-Lineage-KDF (Tower + gate2) | Validated |
| Neural API proxy | proxy_http -> Songbird -> HTTPS | Validated |
| Tower Atomic | BearDog + Songbird health/crypto/JWT | Validated |

---

## Key Documents

| Document | Purpose |
|----------|---------|
| `CURRENT_STATUS.md` | Validated systems, bypasses, evolution needs |
| `QUICK_START.md` | 5-minute deployment |
| `CHANGELOG.md` | Version history |
| `DOCUMENTATION.md` | Full documentation index |
| `wateringHole/handoffs/` | Per-session evolution reports (in ecoPrimals/) |
| `specs/EVOLUTION_ROADMAP.md` | Bypass elimination and evolution waves |

---

## Primal Status

| Primal | Purpose | Status | Next Evolution |
|--------|---------|--------|----------------|
| BearDog | Crypto, Genetics | Reference | Stable |
| Songbird | HTTP, TLS, Discovery, Mesh | 90% | Mesh state fix, UDP discovery |
| Toadstool | Compute, GPU | Operational | GPU job queue |
| NestGate | Storage, Federation | Operational (patched) | Upstream boolean fix |
| Squirrel | AI Orchestration | Operational | Ollama native adapter |
| biomeOS | System Orchestrator + Neural API | Evolved | ARM64 genomeBin |

---

## Standards

| Standard | Description |
|----------|-------------|
| **ecoBin v3.0** | 100% Pure Rust, zero C deps |
| **Universal IPC v3.0** | Multi-transport (Unix/Abstract/TCP/HTTP) |
| **PRIMAL_DEPLOYMENT_STANDARD** | Deterministic cross-platform |
| **Semantic Method Naming** | capability.call routing |
| **scyBorg Triple-Copyleft** | AGPL-3.0-or-later + ORC + CC-BY-SA 4.0 |
| **XDG Base Directory** | All paths via SystemPaths |

---

**Status**: Production Ready (v3.35 — zero blocking debt, deep debt audit CLEAN, all primalSpring audit gaps addressed)
**Discovery**: Capability-based per `CAPABILITY_BASED_DISCOVERY_STANDARD` v1.2.0 — no identity-based routing, no deprecated discovery stubs
**AI Bridge**: Capability-routed to Squirrel at runtime (tag-in on demand)
**Plasmodium**: HTTP JSON-RPC collective (runtime port) + Agent Model
**Neural API**: 320+ semantic translations, 27 capability domains (+ tensor + shader), `primal.list` + `topology.primals`, lazy rescan, cross-gate routing, post-spawn auto-registration (part of biomeOS)
**Composition**: Multi-primal graph execution (5+ nodes) proven e2e; `composition.health` standard; `rpc_call` operation fallback; bootstrap tolerance (`fallback = "skip"`)
**NAT Traversal**: 4-tier strategy (LAN/punch/coordinated/relay)
**Lifecycle**: Deep health monitoring + auto-resurrection + composition dashboard
**IPC**: Universal IPC v3.0 + HTTP JSON-RPC (inter-gate) + TCP-only mode (mobile) + UDS dual-protocol auto-detect + BTSP ClientHello recognition
**Primals**: 7/7 ecoBin v3.0 compliant
**Cross-Arch**: x86_64 + aarch64 + armv7 (32-bit safe)
**Tests**: 8,064+ passing (0 failures), 90%+ line / function / region (llvm-cov) | **Clippy**: PASS (0 warnings, pedantic+nursery) | **Docs**: Full coverage | **C deps**: 0 | **Unsafe**: 0 (`#[forbid(unsafe_code)]` all roots + binaries) | **Deprecated**: 0 | **TODO/FIXME**: 0 | **Blocking debt**: 0 | **Hardcoded values**: 0 (IPs, paths, primal names all centralized) | **Box\<dyn Error\>**: 0 | **Production files >800L**: 0
**Updated**: April 30, 2026
