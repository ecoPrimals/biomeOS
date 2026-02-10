# biomeOS Evolution Handoff - February 9, 2026

**Team**: biomeOS (phase2/biomeOS)
**Priority**: HIGH - Orchestrator evolution enables all primal teams
**Codebase**: `ecoPrimals/phase2/biomeOS/`

---

## Context

biomeOS is the ecosystem orchestrator -- it doesn't do the work itself; it coordinates
primals that do. The Neural API provides semantic capability routing (121 translations),
the Plasmodium module provides distributed collective queries, and `start_nucleus.sh`
orchestrates startup.

All major subsystems are validated:
- AI Bridge (Squirrel -> Songbird -> Cloud/Local AI)
- Nest Atomic (Tower + gate2 NestGate)
- Plasmodium collective (2-gate SSH-based queries)
- Neural API (capability.call, proxy_http, capability.register)

---

## Evolution Items

### 1. Multi-Family Architecture (HIGH PRIORITY)

**Decision**: Option A -- primals create family-suffixed sockets.

**biomeOS role**:
- `start_nucleus.sh` passes `--family-id` to each primal
- Neural API discovers family-specific sockets
- Remove socket nucleation symlinks
- Support multiple NUCLEUS instances on one machine

**New capability**: A primal instance can participate in multiple families.
When a primal creates `songbird-family_a.sock` AND `songbird-family_b.sock`,
it serves both families. biomeOS manages this:

```
biomeos nucleus start --family-id alpha --family-id beta
  -> BearDog instance for alpha (beardog-alpha.sock)
  -> BearDog instance for beta  (beardog-beta.sock)
  -> Songbird shared instance   (songbird-alpha.sock, songbird-beta.sock)
```

BearDog MUST have separate instances per family (key material is family-specific).
Songbird CAN share instances (HTTP has no family-specific state).
NestGate CAN share with namespaced keys.
Toadstool CAN share (GPU compute is family-agnostic).
Squirrel CAN share (AI routing is family-agnostic).

**Estimated**: 200 lines (biomeOS) + per-primal socket changes

### 2. Plasmodium Agent Model (HIGH PRIORITY)

**Current**: Plasmodium spawns SSH queries to gather collective state.
**Target**: Neural API creates lightweight agents that leverage existing primals.

A Plasmodium agent is NOT a new process -- it's a Neural API routing context:

```
Plasmodium Agent = {
  family_id: "nat0",
  gates: ["tower", "gate2"],
  role: "hpc_coordinator",
  primals: {
    "songbird": "tower:songbird-nat0.sock",  // local Songbird
    "toadstool": "gate2:toadstool-nat0.sock", // remote Toadstool
    "nestgate": "gate2:nestgate-nat0.sock",   // remote NestGate
  }
}
```

The agent reuses existing primal instances. It routes `capability.call` to the
appropriate gate's primal. No new processes are spawned.

**Key architectural insight**: A Songbird on Tower can be:
- Part of the local Tower Atomic (serving local HTTP, crypto)
- Part of the HPC coordinator agent (routing mesh traffic to gate2)
- Part of multiple families simultaneously

The same binary instance handles all roles. The Neural API maintains separate
routing tables for each context.

**Meld/Split/Mix**:
- **Meld**: Two gates' capabilities merge into one agent view
- **Split**: An agent splits into per-gate agents for independent operation
- **Mix**: Selective capabilities from different gates compose a custom agent

**Estimated**: 300 lines in `biomeos-atomic-deploy/src/neural_api_server/`

### 3. Pure Rust System Calls -- COMPLETED

All production shell-outs replaced with pure Rust:

| Was | Now | Status |
|-----|-----|--------|
| `Command::new("nvidia-smi")` | `/proc/driver/nvidia/gpus/` + `/sys/bus/pci/devices/` | **DONE** |
| `Command::new("df")` | `/proc/mounts` + `nix::sys::statvfs` | **DONE** |
| `Command::new("ip")` (query) | `/sys/class/net/` + `operstate` | **DONE** |
| `Command::new("kill")` | `nix::sys::signal::kill` | **DONE** |
| `Command::new("which")` | Pure Rust `PATH` scan | **DONE** |
| `Command::new("ssh")` | Songbird mesh RPC first, SSH deprecated fallback | **DONE** |
| `Command::new("ip")` (bridge mgmt) | Requires `sudo` -- documented, netlink evolution path | N/A |

### 4. Internalize `start_nucleus.sh` -- COMPLETED

Implemented as `biomeos nucleus start --mode full --node-id tower1` in
`crates/biomeos/src/modes/nucleus.rs`:

- Pure Rust primal binary discovery (scans `livespore-usb/`, `plasmidBin/`, `target/release/`, `$PATH`)
- Dependency-ordered startup (BearDog -> Songbird -> NestGate -> ...)
- Socket creation and health check validation (JSON-RPC ping)
- Graceful shutdown (SIGTERM -> timeout -> SIGKILL via `nix`)
- Family ID derivation from env var or `.family.seed`
- Zero unsafe code, zero shell-outs

### 5. API Route Completion -- COMPLETED

Wired previously dead-code handler modules into the API router:
- `GET /api/v1/capabilities` -- list all discoverable capabilities
- `POST /api/v1/capabilities/discover` -- find primals by capability
- `POST /api/v1/genome/build` -- build new genomeBin
- `GET /api/v1/genome/:id/info` -- get genomeBin metadata
- `POST /api/v1/genome/verify-file` -- verify genomeBin integrity

### 6. Deep Debt Cleanup -- COMPLETED

- Production `unwrap()` in `model_cache.rs` replaced with graceful `continue`
- All `SporeConfig` doc tests and test files updated with required `family_id` field
- `deploy/network.rs` bridge management shell-outs documented with netlink evolution path
- Hardcoded `/run/user/1000` eliminated everywhere -- uses `SystemPaths` + env fallbacks
- All `#[allow(dead_code)]` audited -- each is justified (JSON-RPC spec fields, forward-looking fields, utility modules)

### 7. Bootstrap Mode Detection -- COMPLETED

Integrated ecosystem detection into `biomeos nucleus start`:

- **Bootstrap mode**: No existing primal sockets found. biomeOS starts all
  primals from scratch as the genesis orchestrator.
- **Coordinated mode**: Existing primal sockets detected and responding.
  biomeOS only starts supplementary primals that aren't already running,
  allowing additive scaling (e.g., add Toadstool to existing Tower).
- Stale sockets (file exists but no response) are detected and replaced.

### 8. HealthChecker Evolution -- COMPLETED

- `HealthChecker` now derives `Clone` properly (no more lossy manual impl)
- Added `HealthChecker::new_default()` using `SystemPaths::new_lazy().runtime_dir()`
- Eliminated hardcoded `/tmp` in `LifecycleManager` and `Clone` impl
- Clippy warnings in `model_cache.rs` fixed (`print_literal`, `&PathBuf` -> `&Path`)

### 9. Lifecycle Integration -- COMPLETED

- Nucleus mode now creates `LifecycleManager` and registers all started primals
- Background health monitoring starts automatically after primal startup (10s interval)
- Active primals receive **deep JSON-RPC health checks** (not just socket existence)
- Incubating primals get lighter socket-only checks during startup
- Coordinated shutdown via `LifecycleManager::shutdown_all()` -- dependency-aware ordering
- Auto-resurrection pipeline: degraded primals are automatically restarted with exponential backoff

### 10. SystemPaths Consolidation -- COMPLETED

Eliminated all duplicate path resolution logic in favor of centralized `SystemPaths`:
- `nucleus.rs` `resolve_socket_dir()` simplified to delegate to `SystemPaths::new_lazy()`
- `doctor.rs` config path uses `SystemPaths::config_dir()` instead of manual `etcetera` calls
- `doctor.rs` primal discovery uses `SystemPaths::runtime_dir()` + `HealthChecker` for deep checks
- `trust.rs` fallback uses `SystemPaths::primal_socket()` instead of hardcoded `/tmp`
- `topology.rs` socket endpoints use `SystemPaths::primal_socket()` instead of `/tmp/biomeos/sockets/`
- `genome.rs` storage dir uses `SystemPaths::data_dir()` instead of manual `XDG_DATA_HOME` chain

### 11. Capability-Based Plasmodium -- COMPLETED

Refactored `plasmodium.rs` to eliminate hardcoded primal discovery:
- `query_local_gate()` now dynamically scans the runtime socket directory
  for family-matching sockets instead of hardcoding 5 primal names
- Falls back to env-based discovery only if socket scan finds nothing
- `aggregate_capabilities()` now uses `capability_taxonomy::capabilities_for_primal()`
  from `biomeos-types` instead of a hardcoded primal-name-to-capability match block
- Added `capabilities_for_primal()` function to `biomeos-types::capability_taxonomy`
  as the canonical primal-to-capability mapping (single source of truth)

### 12. Capability Translation Socket Consolidation -- COMPLETED

- `resolve_primal_socket()` reduced from 45-line manual 5-tier fallback to
  5-line delegation to `SystemPaths::new_lazy().primal_socket()`
- Updated module docs to reflect simplified resolution chain

### 13. biomeos-boot Doc Compliance -- COMPLETED

- Fixed all 39 missing-docs warnings across `init_error.rs`, `boot_logger/mod.rs`,
  `bootable.rs`, `initramfs.rs`, and `rootfs.rs`
- Replaced 5 `unwrap()` calls with safe alternatives (`unwrap_or`, `if let`, `match`)
- Zero warnings across entire workspace (including `biomeos-boot`)

### 14. ARM64 biomeOS genomeBin (LOW PRIORITY)

**What**: Cross-compile biomeOS binary to aarch64 for Pixel 8a.

**How**: Add aarch64 target to build pipeline, same as other primals.

**Estimated**: Build configuration

---

## Plasmodium Agent Architecture (Detailed)

### The Problem

Current Plasmodium uses SSH to query remote gates. This works but:
- Requires SSH keys configured manually
- Doesn't leverage the existing Songbird mesh
- Creates new processes per query
- Not capability-based

### The Solution: Neural API Agent Routing

```
+------------------------------------------------------------------+
|                     Plasmodium Collective                          |
+------------------------------------------------------------------+
|                                                                    |
|  Agent: "hpc_coordinator"                                         |
|  +--------------------------------------------------------------+ |
|  | Neural API routing context                                    | |
|  |                                                               | |
|  | capability.call("compute.submit", job)                        | |
|  |   -> route to: gate2:toadstool (RTX 3090, 24GB VRAM)         | |
|  |                                                               | |
|  | capability.call("storage.retrieve", key)                      | |
|  |   -> route to: gate2:nestgate (ZFS, dedup)                   | |
|  |                                                               | |
|  | capability.call("http.request", req)                          | |
|  |   -> route to: tower:songbird (local network)                | |
|  +--------------------------------------------------------------+ |
|                                                                    |
|  Transport: Songbird mesh relay (TCP)                             |
|  Auth: BearDog family seed verification                           |
+------------------------------------------------------------------+
```

### Meld Example

Tower has: BearDog, Songbird, NestGate (filesystem), Toadstool (RTX 4070)
gate2 has: NestGate (ZFS), Toadstool (RTX 3090)

**Melded agent sees**:
- `crypto.*` -> Tower:BearDog
- `http.*` -> Tower:Songbird
- `storage.*` -> gate2:NestGate (prefer ZFS for dedup)
- `compute.*` -> gate2:Toadstool (prefer RTX 3090 for large models)
- `compute.quick` -> Tower:Toadstool (prefer RTX 4070 for fast inference)

### Split Example

The melded agent splits when gates disconnect:
- Tower agent: BearDog + Songbird + NestGate(fs) + Toadstool(4070)
- gate2 agent: NestGate(ZFS) + Toadstool(3090)

Each operates independently until mesh reconnects and they re-meld.

---

## Code Debt Metrics (Post-Evolution)

| Metric | Value |
|--------|-------|
| TODO markers | 1 (intentional design note in metrics.rs) |
| FIXME/HACK/WORKAROUND | 0 |
| Unsafe code | 1 (`Mmap::map` in genome-deploy -- properly documented, OS-level) |
| Clippy warnings | 0 (entire workspace, including biomeos-boot) |
| Production unwrap() | 0 (all in test code) |
| Shell-outs from Rust | 3 production (`sudo ip link/addr/set` in deploy/network.rs for bridge mgmt -- requires root, documented netlink evolution path) |
| Mocks in production | 0 (all isolated to `#[cfg(test)]` and test files) |
| `deny(unsafe_code)` crates | 8 (core, ui, types, atomic-deploy, graph, deployment-mode, genome-extract, nucleus) |
| Hardcoded `/tmp` in production | 0 (all production paths via SystemPaths) |
| Workspace tests passing | 1,789 |
| Workspace test failures | 0 |
