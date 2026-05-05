# Evolution Roadmap - From Bypasses to Pure Rust

**Created**: February 9, 2026
**Updated**: May 5, 2026 (v3.43: discovery schema alignment, primalSpring Phase 59 audit response, 7,867 tests)
**Purpose**: Comprehensive evolution plan for all primals and biomeOS

---

## Current State

biomeOS and the full NUCLEUS stack are **production validated** with **0 active bypasses**.
All 6 original bypasses have been evolved to production-quality solutions. This roadmap
is preserved as a fossil record of the evolution paths taken.

**Post-cleanup**: 22 active specs + 3 lifecycle, 4 active scripts.
47 specs, 14 handoffs, 35 scripts archived to `ecoPrimals/archive/biomeos-feb09-2026/`.

---

## 1. Bypasses and Their Evolution Paths

### ~~Bypass 1: HTTP_REQUEST_PROVIDER_SOCKET env var~~ (EVOLVED Mar 19)

**Was**: Squirrel discovered Songbird via explicit `HTTP_REQUEST_PROVIDER_SOCKET` env var.
**Now**: `http_bridge` registered as Songbird capability in `capability_sockets.rs` and `CapabilityTaxonomy`. Squirrel uses `BIOMEOS_DISCOVERY_SOCKET` and capability-based discovery. Env var removed from nucleus spawn.

### ~~Bypass 2: Socket nucleation symlinks~~ (EVOLVED)

**Was**: `start_nucleus.sh` created symlinks: `songbird.sock -> songbird-{family_id}.sock`
**Now**: Shell script fully replaced by `biomeos nucleus start` (Rust). All primals accept `--family-id` and create `{primal}-{family_id}.sock`. `detect_ecosystem()` dynamically scans socket directories — no hardcoded primal list.
See Section 7 for full multi-family spec.

### ~~Bypass 3: NestGate inverted boolean~~ (EVOLVED Mar 19)

**Was**: biomeOS passed `--socket-only` with inverted semantics.
**Now**: biomeOS omits the flag entirely; with the inversion, this yields socket-only mode. Documented in nucleus.rs.

### ~~Bypass 4: Squirrel default model override~~ (EVOLVED Mar 19)

**Was**: Had to pass model explicitly.
**Now**: `AI_DEFAULT_MODEL` env var passed through to Squirrel in nucleus spawn and graph deployment.

### Bypass 5: SSH-based Plasmodium queries

**Evolution**: Songbird auto mesh discovery + Plasmodium agents via Neural API.
See Section 8 for full Plasmodium agent architecture.

---

## 2. Per-Primal Evolution Summary

Detailed handoffs in `ecoPrimals/infra/wateringHole/handoffs/`:

| Primal | Handoff | Priority Items |
|--------|---------|----------------|
| **Songbird** | `SONGBIRD_EVOLUTION_HANDOFF_FEB09_2026.md` | `discover_capabilities`, TLS socket alignment, auto mesh, multi-family |
| **Squirrel** | `SQUIRREL_EVOLUTION_HANDOFF_FEB09_2026.md` | Default model, Ollama adapter, provider health, multi-family |
| **BearDog** | `BEARDOG_EVOLUTION_HANDOFF_FEB09_2026.md` | Multi-family sockets (must be separate instances per family) |
| **NestGate** | `NESTGATE_EVOLUTION_HANDOFF_FEB09_2026.md` | 4 bug fixes, model cache methods, cross-gate replication, multi-family |
| **Toadstool** | `TOADSTOOL_EVOLUTION_HANDOFF_FEB09_2026.md` | GPU job queue, Ollama integration, cross-gate compute, multi-family |
| **biomeOS** | `BIOMEOS_EVOLUTION_HANDOFF_FEB09_2026.md` | Multi-family orchestration, Plasmodium agents, pure Rust, startup internalization |

---

## 3. Pure Rust System Calls

| Was | Now | Status |
|-----|-----|--------|
| `Command::new("nvidia-smi")` | `/proc/driver/nvidia/gpus/` + `/sys/bus/pci/devices/` | **DONE** |
| `Command::new("df")` | `/proc/mounts` + `nix::sys::statvfs` | **DONE** |
| `Command::new("ip")` (query) | `/sys/class/net/` + `operstate` | **DONE** |
| `Command::new("kill")` | `nix::sys::signal::kill` | **DONE** |
| `Command::new("which")` | Pure Rust `PATH` scan | **DONE** |
| `Command::new("ssh")` | Songbird mesh RPC first, SSH deprecated fallback | **DONE** |
| `Command::new("ip")` (bridge mgmt) | Requires `sudo` -- netlink evolution path documented | Accepted |

**All production shell-outs eliminated** except bridge management (requires root).

---

## 4. Shell Scripts (Post-Cleanup)

Active scripts (shell scripts that remain in the repository):

| Script | Purpose | Status |
|--------|---------|--------|
| ~~`start_nucleus.sh`~~ | ~~NUCLEUS startup~~ | **Archived** → `biomeos nucleus start` (Rust) |
| ~~`build-genome.sh`~~ | ~~genomeBin build~~ | **Archived** Mar 14, 2026 |
| `build_primals_for_testing.sh` | Test builds | Keep (dev tooling) |
| `create_sibling_spore.sh` | Spore creation | Keep (dev tooling) |
| `create_livespore.sh` | LiveSpore USB creation | Keep (dev tooling) |
| `test_provenance_trio_e2e.sh` | Provenance E2E test | Keep (test tooling) |
| `bootstrap-selector.sh` | genomeBin v3 bootstrap | Keep (embedded in binary) |
| `livespore-usb/*/scripts/*.sh` | USB deploy/start scripts | Keep (deployment tooling) |
| `pixel8a-deploy/*.sh` | Mobile deployment | Keep (device tooling) |

35+ scripts archived to fossil record.

---

## 5. Deep Debt Metrics (Updated Apr 21, 2026 — v3.23)

| Metric | Value |
|--------|-------|
| TODO markers in Rust source | 0 |
| TODO in config (deny.toml) | 0 (bincode v1 NOTE remains — blocked by tarpc upstream) |
| FIXME/HACK/WORKAROUND/XXX | 0 |
| Unsafe code | 0 (`#[forbid(unsafe_code)]` on all crate roots + all 20+ binary roots) |
| Clippy warnings | 0 (entire workspace, pedantic+nursery, all 25 crates via workspace lint inheritance, `-D warnings`) |
| Production unwrap() | 0 (all in test code) |
| Shell-outs from Rust | 3 (`sudo ip link/addr/set` in deploy/network.rs — requires root) |
| `forbid(unsafe_code)` crates | all production crates + binary roots |
| Mocks in production | 0 (test_support gated behind feature flag; all stubs resolved) |
| Proptest IPC fuzz tests | 8 |
| C-dep crates banned (deny.toml) | 15 |
| Tests | 7,859 (0 failures, fully concurrent) |
| Coverage | 90%+ line / function / region (llvm-cov) |
| Production files >800 LOC | 0 (all 5 files >800L are test-only) |
| Hardcoded primal strings | 0 (centralized `primal_names` constants) |
| Hardcoded paths/ports | 0 (centralized in `biomeos-types::constants`) |
| Zero-copy payloads | `bytes::Bytes` with base64 serde |
| Bypasses | 0 blocking (Tower routing is documented evolution target) |
| Box\<dyn Error\> in production | 0 (all anyhow::Result) |
| Deprecated APIs | 0 |
| Capability translations | 320+ across 27 domains (+ shader) |
| BTSP wire-format | ClientHello recognition on API socket + Neural API handshake verified |

---

## 6. Evolution Waves

### Wave 1: Quick Wins (hours)
1. NestGate inverted boolean fix (1 LOC upstream)
2. Squirrel configurable default model (15 LOC)
3. All primals: `discover_capabilities` method (30 LOC each)

### Wave 2: Multi-Family Foundation (days)
4. All primals: `--family-id` flag, family-suffixed sockets (10-50 LOC each) -- *per-primal teams*
5. Songbird TLS BearDog socket alignment (30 LOC) -- *Songbird team*
6. ~~biomeOS: update `nucleation.rs` to match new socket convention~~ **DONE** (can_share, assign_multi_family, plan_multi_family)
7. ~~biomeOS: update `start_nucleus.sh` for multi-family~~ **DONE** (`biomeos nucleus start` replaces shell script)
8. ~~`df` -> `statvfs`, `ip` -> `/proc/net`~~ **DONE** (nix::sys::statvfs, /sys/class/net/)

### Wave 3: Plasmodium Agents (weeks)
9. ~~Neural API agent routing contexts~~ **DONE** (agents.rs: PlasmodiumAgent, AgentRegistry, meld/split)
10. Songbird auto mesh peer discovery (200 LOC) -- *Songbird team*
11. ~~Plasmodium via Neural API agents (replace SSH)~~ **DONE** (mesh-first, SSH deprecated fallback)
12. ~~`nvidia-smi` -> `/proc/driver/nvidia/`~~ **DONE** (plasmodium.rs + provider.rs)
13. ~~Nucleus + LifecycleManager integration~~ **DONE** (deep JSON-RPC health monitoring, auto-resurrection, coordinated shutdown)
14. ~~SystemPaths consolidation~~ **DONE** (doctor, trust, topology, genome, nucleus all use SystemPaths)

### Wave 4: Full Federation (weeks)
15. Toadstool GPU job queue (300 LOC) -- *Toadstool team*
16. NestGate cross-gate replication (500 LOC) -- *NestGate team*
17. API key encryption via BearDog + NestGate (200 LOC) -- *BearDog/NestGate teams*
18. Squirrel Ollama native adapter (150 LOC) -- *Squirrel team*
19. ~~`start_nucleus.sh` -> `biomeos nucleus start`~~ **DONE** (nucleus.rs: pure Rust, zero shell-outs)
20. ARM64 biomeOS genomeBin (build config) -- *biomeOS team*

---

## 7. Multi-Family Architecture (Option A)

### Decision

All primals accept `--family-id` and create family-suffixed sockets.
This is the foundation for primal sharing, Plasmodium agents, and meld/split.

### Socket Convention

```
$XDG_RUNTIME_DIR/biomeos/{primal}-{family_id}.sock

Examples:
  /run/user/1000/biomeos/beardog-alpha.sock    # BearDog for family alpha
  /run/user/1000/biomeos/beardog-beta.sock     # BearDog for family beta
  /run/user/1000/biomeos/songbird-alpha.sock   # Songbird for family alpha
  /run/user/1000/biomeos/songbird-alpha.sock   # Same Songbird serves alpha
  /run/user/1000/biomeos/songbird-beta.sock    # ALSO serves beta (symlink or multi-listen)
```

### Primal Sharing Rules

Not all primals can be shared across families. The key constraint is
**family-specific state**:

| Primal | Shareable? | Reason |
|--------|-----------|--------|
| **BearDog** | NO | Key material derived from family seed. Each family MUST have its own BearDog. |
| **Songbird** | YES | HTTP/network has no family-specific state. One instance can route for multiple families. |
| **NestGate** | YES (namespaced) | Storage can be namespaced by family. One instance with `{family_id}/{key}` prefixing. |
| **Toadstool** | YES | GPU compute is family-agnostic. One instance can process jobs for any family. |
| **Squirrel** | YES | AI routing is family-agnostic. One instance can handle queries for any family. |

### Multi-Family Startup

```bash
# Family alpha: full NUCLEUS
biomeos nucleus start --family-id alpha --node-id tower1

# Family beta: share existing Songbird, Toadstool, Squirrel
biomeos nucleus start --family-id beta --node-id tower1 \
  --share songbird --share toadstool --share squirrel
```

biomeOS creates:
- New BearDog instance for beta (separate keys)
- New NestGate instance for beta (separate namespace)
- Symlinks/multi-listen for shared primals:
  - `songbird-beta.sock -> songbird-alpha.sock`
  - `toadstool-beta.sock -> toadstool-alpha.sock`
  - `squirrel-beta.sock -> squirrel-alpha.sock`

### Separate Families

Two completely independent ecosystems on one machine:

```
Family alpha: beardog-alpha + songbird-alpha + nestgate-alpha + toadstool-alpha + squirrel-alpha
Family beta:  beardog-beta  + songbird-beta  + nestgate-beta  + toadstool-beta  + squirrel-beta
```

No shared state. No shared processes. Complete isolation.

### Mixed Families

Selective sharing for efficiency:

```
Family alpha: beardog-alpha + songbird-shared + nestgate-alpha + toadstool-shared + squirrel-shared
Family beta:  beardog-beta  + songbird-shared + nestgate-beta  + toadstool-shared + squirrel-shared
```

Songbird, Toadstool, and Squirrel are shared (family-agnostic).
BearDog and NestGate are separate (family-specific state).

---

## 8. Plasmodium Agent Architecture

### Problem

Current Plasmodium spawns SSH processes to query remote gates. This:
- Doesn't leverage existing primals
- Creates new processes per query
- Isn't capability-based
- Can't meld/split dynamically

### Solution: Neural API Agent Routing

A **Plasmodium agent** is NOT a new process. It's a **Neural API routing context**
that maps capability requests to the best available primal across the mesh.

```
Agent = {
  name: "hpc_coordinator",
  family_id: "cf7e8729",           // Derived from .family.seed (not a plaintext tag)
  beacon_tags: ["gaming", "research"],  // Beacon tags (behavioral realms)
  routing_table: {
    "crypto.*":   -> tower:beardog-cf7e8729.sock      (local)
    "http.*":     -> tower:songbird-cf7e8729.sock     (local)
    "storage.*":  -> gate2:nestgate-cf7e8729.sock     (remote via mesh)
    "compute.*":  -> gate2:toadstool-cf7e8729.sock    (remote via mesh)
    "ai.*":       -> tower:squirrel-cf7e8729.sock     (local)
  }
}
```

The agent reuses existing primal instances. Primals don't know they're part of an agent.
They just receive `capability.call` requests via their normal JSON-RPC socket.

### Meld: Combining Gate Capabilities

When two gates are bonded (shared `.family.seed`, Songbird mesh connected),
their capabilities can be **melded** into a single agent view:

```
Tower primals:  BearDog, Songbird, NestGate(fs), Toadstool(RTX 4070)
gate2 primals:  NestGate(ZFS), Toadstool(RTX 3090)

Melded agent routing:
  crypto.*     -> tower:beardog        (only source)
  http.*       -> tower:songbird       (only source)
  storage.*    -> gate2:nestgate       (prefer ZFS for dedup/compression)
  compute.big  -> gate2:toadstool      (prefer RTX 3090 for large models)
  compute.fast -> tower:toadstool      (prefer RTX 4070 for low-latency)
  ai.*         -> tower:squirrel       (only source)
```

**Routing selection**: based on capability metadata (VRAM size, storage backend,
model already loaded, queue depth).

### Split: Disconnecting Gates

When a gate goes offline, the melded agent splits:

```
BEFORE (melded):
  compute.* -> gate2:toadstool (preferred)
  storage.* -> gate2:nestgate (preferred)

AFTER (gate2 offline, split):
  compute.* -> tower:toadstool (fallback)
  storage.* -> tower:nestgate (fallback)
```

The agent automatically reroutes to available primals. No manual intervention.
When gate2 reconnects, the agent re-melds.

### Mix: Selective Composition

A custom agent can mix capabilities from different sources:

```
Agent: "inference_pipeline"
  Step 1: storage.retrieve(model)     -> gate2:nestgate (ZFS, cached models)
  Step 2: compute.inference(model)    -> gate2:toadstool (RTX 3090)
  Step 3: ai.summarize(result)        -> tower:squirrel -> Anthropic
  Step 4: storage.store(summary)      -> tower:nestgate (local cache)
```

This pipeline crosses gates transparently. Each step is a `capability.call`
routed by the Neural API agent.

### Songbird's Role in Agents

A single Songbird instance can play multiple roles:

1. **Local Tower Atomic**: Handle HTTP, TLS, discovery for local primals
2. **HPC Mesh Coordinator**: Route mesh traffic to remote gates
3. **Multi-Family Router**: Serve HTTP for families alpha and beta simultaneously

The Neural API maintains separate routing tables for each context.
Songbird doesn't need to know about families or agents -- it just handles
the traffic that arrives on its socket.

### Implementation Plan

1. **Neural API**: Add `agent.create`, `agent.list`, `agent.routing` methods
2. **Plasmodium**: Create agents from collective state (auto-meld)
3. **Routing**: Agent routing table in `neural_api_server/routing.rs`
4. **Transport**: Remote capability calls forwarded via Songbird mesh relay
5. **Resilience**: Heartbeat monitoring, auto-split on gate disconnect, auto-meld on reconnect

**Estimated**: 300 LOC (Neural API) + 200 LOC (Plasmodium) + 100 LOC (transport)

---

## 9. Architecture Summary

```
+---------------------------------------------------------------------+
|                    Multi-Family NUCLEUS                               |
+---------------------------------------------------------------------+
|  Plasmodium Agents (meld/split/mix routing contexts)                 |
|  +-------------------------------+  +-----------------------------+  |
|  | Agent: local_tower            |  | Agent: hpc_coordinator      |  |
|  | crypto -> local:beardog       |  | compute -> gate2:toadstool  |  |
|  | http   -> local:songbird      |  | storage -> gate2:nestgate   |  |
|  | ai     -> local:squirrel      |  | http    -> local:songbird   |  |
|  +-------------------------------+  +-----------------------------+  |
+---------------------------------------------------------------------+
|  Neural API (121 translations + agent routing)                       |
+---------------------------------------------------------------------+
|  Multi-Family Sockets                                                |
|  beardog-alpha.sock  songbird-shared.sock  nestgate-alpha.sock       |
|  beardog-beta.sock   (serves both)         nestgate-beta.sock        |
+---------------------------------------------------------------------+
|  Primals (evolve independently, shared or isolated)                  |
+---------------------------------------------------------------------+
```

---

**Total estimated evolution**: ~3,000 LOC across all primals + biomeOS
**Current codebase**: Production ready with 0 active bypasses (all evolved)
**Philosophy**: Solve first, evolve solutions, clean all debt
**Architecture**: Primals compose through capabilities, not coupling

---

## Deep Debt Evolution Session (April 1, 2026)

Systematic deep debt resolution across 7 waves:

| Wave | Scope | Result |
|------|-------|--------|
| 1 | Coverage push | 88.95%→89.11% lines, 90.10% functions; `model_cache.rs` consolidated ~170 LOC of untestable code |
| 2 | Large file refactoring | 4 files refactored: `ai_advisor.rs` 956→769, `engine_tests2.rs` 935→707+248, `routing.rs` 921→421+499, `paths.rs` 912→598+319 |
| 3 | Unsafe code elimination | Removed unused `env_helpers.rs`, upgraded `biomeos-test-utils` to `#![forbid(unsafe_code)]` |
| 4a | Hardcoded primal evolution | `enroll.rs` → capability-based `--security-provider-socket`, taxonomy-resolved socket names; `verify_lineage.rs` → `security_client`; `spore.rs` → dynamic `CORE_PRIMALS` |
| 4b | Stub evolution | `PrimalDiscoveryService` stubs marked `#[deprecated]`; `UniversalBiomeOSManager::discover()` wired to real `SocketDiscovery` 5-tier protocol |
| 5a | Dep alignment | tower 0.4→0.5 workspace in `biomeos-api`, tokio workspace dep in `biomeos-graph` |
| 5b | Shell-out elimination | `build.rs` date shell-out → pure Rust `SystemTime` UTC formatting |
