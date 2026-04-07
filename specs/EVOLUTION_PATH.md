# biomeOS Evolution Path: Scripts → Graphs → Pure Rust

**Date**: February 3, 2026  
**Status**: Active Evolution  
**Principle**: Scripts are scaffolding; biomeOS is the primal that orchestrates.

---

## 1. Core Insight

**biomeOS is not just infrastructure—it is a primal.**

Like BearDog, Songbird, and Toadstool, biomeOS:
- Provides capabilities (ecosystem management, deployment, orchestration)
- Exposes a JSON-RPC API (via Neural API)
- Can be discovered by other primals
- Evolves independently

The shell scripts (`start_tower.sh`, `start_nest.sh`) are **temporary scaffolding**—initial solutions that will be replaced by graph-based deployment executed by biomeOS's Neural API.

---

## 2. Evolution Phases

### Phase 1: Shell Scripts (Current - Scaffolding)

```
User → ./start_tower.sh → spawns BearDog, Songbird
```

**Characteristics**:
- Bash scripts manage process spawning
- Environment variables configure behavior
- Sequential, imperative startup
- Easy to understand and debug
- **Not deterministic** (race conditions possible)

**Files**:
- `livespore-usb/*/scripts/start_tower.sh`
- `livespore-usb/*/scripts/start_node.sh`
- `livespore-usb/*/scripts/start_nest.sh`
- ~~`scripts/start_nucleus.sh`~~ → evolved to `biomeos nucleus start` (Rust)

**Limitations**:
- No self-healing
- No adaptive optimization
- Manual process management
- Platform-specific quirks

---

### Phase 2: Graph Deployment (Target - Declarative)

```
User → biomeos deploy graphs/tower_atomic_xdg.toml
         │
         ▼
    Neural API
         │ graph.execute
         ▼
    ┌─────────────────────────────────────────┐
    │ Graph Executor                          │
    │  1. Parse graph nodes                   │
    │  2. Resolve capabilities                │
    │  3. Spawn primals (sequential/parallel) │
    │  4. Register capabilities               │
    │  5. Validate health                     │
    └─────────────────────────────────────────┘
```

**Characteristics**:
- TOML graphs declare WHAT, not HOW
- Neural API executes graphs
- Capability-based primal resolution
- Sequential/parallel coordination built-in
- **Deterministic** (same graph = same result)

**Existing Graphs**:
- `tower_atomic_xdg.toml` - Tower with XDG socket paths
- `node_atomic_compute.toml` - Node atomic
- `nest_deploy.toml` - Nest atomic
- `nucleus_complete.toml` - Full NUCLEUS

**Execution**:
```bash
# Via Neural API
echo '{"jsonrpc":"2.0","method":"graph.execute","params":{"graph_id":"tower_atomic_xdg"},"id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/neural-api-ecosystem.sock
```

---

### Phase 3: Living Graphs (Evolution - Adaptive)

```
Neural API
    │
    │ graph.execute + metrics
    ▼
┌─────────────────────────────────────────────┐
│ Learning Engine                             │
│  - Collects execution metrics               │
│  - Learns optimal deployment strategies     │
│  - Adapts to hardware characteristics       │
│  - Self-heals failed primals                │
└─────────────────────────────────────────────┘
```

**Characteristics**:
- Graphs learn from execution
- Adaptive timeouts (based on hardware)
- Automatic retry strategies
- Self-healing (restart failed primals)
- Performance optimization

**Already Implemented** (crates/biomeos-atomic-deploy):
- `living_graph.rs` - Living graph execution
- `neural_router.rs` - Capability-based routing
- `capability_translation.rs` - Semantic translation
- `health_check.rs` - Primal health monitoring
- `lifecycle_manager.rs` - Primal lifecycle

---

## 3. biomeOS as Primal

### Capability Registration

biomeOS provides these capabilities to the ecosystem:

| Capability | Method | Description |
|------------|--------|-------------|
| `graph.execute` | Execute deployment graph | Deploy atomics |
| `graph.list` | List available graphs | Discovery |
| `primal.spawn` | Spawn a primal | Low-level control |
| `primal.health` | Check primal health | Monitoring |
| `capability.register` | Register capability | Dynamic registration |
| `capability.discover` | Discover provider | Service discovery |
| `ecosystem.status` | Get ecosystem status | Observability |

### Neural API as biomeOS Interface

```
┌─────────────────────────────────────────────────────────────────┐
│                         Neural API                              │
│  (biomeOS's external interface - JSON-RPC over Unix socket)     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  capability.call("graph", "execute", {graph_id: "tower_atomic"})│
│       │                                                         │
│       ▼                                                         │
│  ┌─────────────────────────────────────────┐                   │
│  │ Graph Executor (Pure Rust)              │                   │
│  │  - biomeos-graph crate                  │                   │
│  │  - biomeos-atomic-deploy crate          │                   │
│  └─────────────────────────────────────────┘                   │
│       │                                                         │
│       ▼                                                         │
│  Spawns: BearDog → Songbird → [Toadstool | NestGate | Squirrel]│
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 4. Migration Path

### Step 1: Validate Graph Execution (Now)

```bash
# Test existing graph execution
cargo run --bin neural-api-server -- \
  --socket /run/user/$(id -u)/biomeos/neural-api.sock \
  --graphs-dir graphs/

# Execute tower atomic via graph (not script)
echo '{"jsonrpc":"2.0","method":"graph.execute","params":{"graph_id":"tower_atomic_xdg"},"id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/neural-api.sock
```

### Step 2: Update Scripts to Call Graphs

```bash
#!/bin/bash
# start_tower.sh - NOW calls graph instead of spawning directly

SOCKET_DIR="$(resolve_socket_dir)"
FAMILY_ID="${FAMILY_ID:-ecosystem}"

# Start Neural API first (the orchestrator)
neural-api-server \
  --socket "$SOCKET_DIR/neural-api-$FAMILY_ID.sock" \
  --graphs-dir graphs/ &

sleep 2

# Deploy via graph (not direct spawning)
echo "{\"jsonrpc\":\"2.0\",\"method\":\"graph.execute\",\"params\":{\"graph_id\":\"tower_atomic_xdg\",\"family_id\":\"$FAMILY_ID\"},\"id\":1}" | \
  nc -U "$SOCKET_DIR/neural-api-$FAMILY_ID.sock"
```

### Step 3: Remove Scripts (Future)

Once graph execution is fully validated:
1. Remove shell scripts from livespore-usb
2. Deploy graphs directly via Neural API
3. biomeOS binary becomes the single entry point

```bash
# Future: No scripts needed
biomeos deploy tower --family-id ecosystem

# Or via CLI
biomeos graph execute tower_atomic_xdg
```

---

## 5. Architecture: biomeOS as Ecosystem Manager

```
┌───────────────────────────────────────────────────────────────────────────┐
│                              NUCLEUS                                       │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │                         biomeOS (Primal)                            │  │
│  │  ┌─────────────────────────────────────────────────────────────┐   │  │
│  │  │ Neural API Server                                           │   │  │
│  │  │  - graph.execute                                            │   │  │
│  │  │  - capability.call                                          │   │  │
│  │  │  - primal.spawn                                             │   │  │
│  │  └─────────────────────────────────────────────────────────────┘   │  │
│  │                              │                                      │  │
│  │                    ┌─────────┴─────────┐                           │  │
│  │                    ▼                   ▼                           │  │
│  │           ┌─────────────┐      ┌─────────────┐                     │  │
│  │           │ Graph       │      │ Capability  │                     │  │
│  │           │ Executor    │      │ Registry    │                     │  │
│  │           └─────────────┘      └─────────────┘                     │  │
│  │                    │                   │                           │  │
│  │                    ▼                   ▼                           │  │
│  │           ┌─────────────────────────────────────┐                  │  │
│  │           │ Primal Spawner / Lifecycle Manager  │                  │  │
│  │           └─────────────────────────────────────┘                  │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                    │                                       │
│                    ┌───────────────┼───────────────┐                      │
│                    ▼               ▼               ▼                      │
│             ┌──────────┐    ┌──────────┐    ┌──────────┐                  │
│             │ BearDog  │    │ Songbird │    │ Toadstool│  ...             │
│             │ (Crypto) │    │ (Network)│    │ (Compute)│                  │
│             └──────────┘    └──────────┘    └──────────┘                  │
│                                                                           │
│             Layer 0: Primals (spawned and managed by biomeOS)             │
│                                                                           │
└───────────────────────────────────────────────────────────────────────────┘
```

---

## 6. Key Implementation Points

### Existing Pure Rust Components

| Crate | Purpose | Status |
|-------|---------|--------|
| `biomeos-atomic-deploy` | Graph execution, primal spawning | ✅ Implemented |
| `biomeos-graph` | Graph parsing, execution engine | ✅ Implemented |
| `neural-api-client` | Client library for capability.call | ✅ Implemented |
| `biomeos-types` | Socket paths, constants | ✅ Implemented |
| `biomeos-spore` | Primal identity, dark forest | ✅ Implemented |

### What Scripts Provide (to be absorbed)

1. **Socket directory resolution** → Already in `biomeos-types::SystemPaths`
2. **Environment setup** → Graph variables (`${FAMILY_ID}`, `${XDG_RUNTIME_DIR}`)
3. **Sequential startup** → Graph `depends_on` / coordination mode
4. **Health validation** → Graph validation nodes + health_check handlers

---

## 7. Graph Deployment Standard

All graphs MUST follow PRIMAL_DEPLOYMENT_STANDARD:

```toml
[graph]
id = "tower_atomic_bootstrap"
coordination = "Sequential"  # or "Parallel" where safe

[nodes.operation.environment]
# Socket paths via 5-tier resolution
BEARDOG_SOCKET = "${XDG_RUNTIME_DIR}/biomeos/beardog-${FAMILY_ID}.sock"
```

### Graph Variables

| Variable | Source | Example |
|----------|--------|---------|
| `${FAMILY_ID}` | Environment or .family.seed | `ecosystem_alpha` |
| `${NODE_ID}` | Environment or hostname | `pixel8a` |
| `${XDG_RUNTIME_DIR}` | System | `/run/user/1000` |
| `${ARCH}` | Runtime detection | `aarch64` |

---

## 8. Summary

| Phase | Mechanism | Status |
|-------|-----------|--------|
| **Phase 1** | Shell scripts | Current (scaffolding) |
| **Phase 2** | Graph deployment | Ready (graphs exist) |
| **Phase 3** | Adaptive execution | Implemented (living_graph.rs) |

**Evolution Principle**: 
> Scripts prove the pattern. Graphs declare the pattern. biomeOS executes the pattern.

**Goal**: biomeOS as the single orchestration primal—no shell scripts, pure Rust, graph-driven deployment.

---

**Version**: 1.0  
**Standard**: PRIMAL_DEPLOYMENT_STANDARD v1.0  
**Crates**: biomeos-atomic-deploy, biomeos-graph, neural-api-client
