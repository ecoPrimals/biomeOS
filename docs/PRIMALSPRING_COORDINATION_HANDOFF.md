# primalSpring Coordination Handoff

**From:** biomeOS v2.52  
**To:** primalSpring (next version)  
**Date:** March 18, 2026  
**Status:** Active — biomeOS extricated patterns ready for replication

## Purpose

biomeOS developed the foundational coordination patterns for NUCLEUS atomics,
beacon genetics, concurrent startup, composition validation, and Plasmodium
bonding. These patterns are now being **externalized to primalSpring** so that:

1. biomeOS can focus on evolving the biomeOS primal itself
2. Springs can cross-learn from primalSpring's coordination reference
3. Coordination logic lives where it belongs: in the spring whose domain IS coordination

This document maps every extractable pattern, its location in biomeOS,
its readiness for extraction, and what primalSpring must replicate.

---

## Pattern Inventory

### Tier 1 — Clean, Capability-Based (extract as-is)

These patterns have **zero hardcoded primal names** and are already expressed
in terms of capabilities. primalSpring should absorb these directly.

#### 1.1 Atomic Type System

| File | Lines | Key Types |
|------|-------|-----------|
| `crates/biomeos-types/src/atomic.rs` | 451 | `AtomicTier`, `AtomicCapability`, `PrimalHealth`, `ProviderHealthMap` |
| | | `TowerAtomic`, `NodeAtomic`, `NestAtomic`, `FullNucleus` |

**What it provides:**
- `AtomicTier::required_capabilities()` — what each tier needs (not which primals)
- `ProviderHealthMap` — runtime-discovered health by capability, not by name
- `TowerAtomic::is_healthy()` — checks all providers, not specific primals
- `FullNucleus::degradation_level()` — sovereign degradation (Full → Node+Nest → Node → Tower → Sovereign)

**primalSpring action:** Absorb the type system. Use `AtomicTier` and
`AtomicCapability` as the shared vocabulary. Discover providers via
`capability.discover`, populate `ProviderHealthMap` at runtime.

#### 1.2 Concurrent Startup Waves

| File | Lines | Key Types |
|------|-------|-----------|
| `crates/biomeos-core/src/concurrent_startup.rs` | 213 | `DependencyGraph`, `topological_waves()`, `start_in_waves()` |

**What it provides:**
- `DependencyGraph::build(primals)` — builds from `ManagedPrimal::provides()` and `requires()` (capability names, not primal names)
- `topological_waves()` — Kahn's algorithm, detects cycles, returns `Vec<Vec<PrimalId>>`
- `start_in_waves()` — concurrent execution within each wave, sequential between waves

**primalSpring action:** Reimplement the topological sort over deploy graph
nodes (not `ManagedPrimal` — that's biomeOS-specific). Use `depends_on` edges
from graph TOML. The algorithm is the same; the input type is different.

#### 1.3 Validation Sink

| File | Lines | Key Types |
|------|-------|-----------|
| `crates/biomeos-types/src/validation.rs` | 251 | `ValidationSink`, `ValidationFinding`, `ValidationSeverity` |
| | | `StderrSink`, `BufferSink` |

**What it provides:**
- Pluggable validation output: stderr for interactive, buffer for programmatic, JSON for CI
- `ValidationSeverity` ordering: Info < Warning < Error
- `ValidationFinding` with rule, message, and optional location

**primalSpring action:** Absorb directly. primalSpring already has a
`ValidationResult` type — wire it to emit `ValidationFinding`s through
a `ValidationSink` for structured output.

#### 1.4 Coordination Patterns

| File | Lines | Key Types |
|------|-------|-----------|
| `crates/biomeos-graph/src/graph.rs` | ~200 | `CoordinationPattern`, `TickConfig`, `GraphDefinition` |

**What it provides:**
- `CoordinationPattern` enum: Sequential, Parallel, ConditionalDag, Pipeline, Continuous
- `TickConfig` for fixed-timestep loops (target_hz, max_accumulator_ms)
- `GraphDefinition` with metadata, env, nodes, outputs

**primalSpring action:** Absorb the `CoordinationPattern` enum. primalSpring
already declares coordination patterns in `graphs/mod.rs` — validate that
the semantics match biomeOS's implementation.

#### 1.5 Plasmodium Agent Routing

| File | Lines | Key Types |
|------|-------|-----------|
| `crates/biomeos-atomic-deploy/src/neural_api_server/agents/types.rs` | 185 | `PlasmodiumAgent`, `CapabilityRoute`, `AgentState` |
| `crates/biomeos-atomic-deploy/src/neural_api_server/agents/registry.rs` | 118 | `AgentRegistry` |
| `crates/biomeos-atomic-deploy/src/neural_api_server/agents/rpc.rs` | 278 | `handle_agent_request()` |

**What it provides:**
- `PlasmodiumAgent` with routing table: `HashMap<String, Vec<CapabilityRoute>>`
- `meld()` — merge two agents' capability routing tables
- `split()` — extract a gate from an agent into a new one
- `resolve()` — find best route for a capability domain
- `AgentRegistry` — named agents with RPC interface

**primalSpring action:** Absorb the agent routing model. primalSpring's bonding
module currently has data models only — wire them to live capability routing
using the `PlasmodiumAgent` pattern.

---

### Tier 2 — Beacon Coordination (parameterize before extraction)

These patterns use the `CapabilityCaller` trait abstraction but have
hardcoded fallback paths. The abstraction is correct; the fallbacks
need to become configurable.

#### 2.1 Beacon Genetics Types

| File | Lines | Key Types |
|------|-------|-----------|
| `crates/biomeos-spore/src/beacon_genetics/types.rs` | 506 | `BeaconId`, `MeetingRecord`, `BeaconGeneticsManifest` |
| | | `MeetingRelationship`, `MeetingVisibility`, `ClusterMembership` |
| `crates/biomeos-spore/src/beacon_genetics/capability.rs` | 346 | `CapabilityCaller` trait |
| `crates/biomeos-spore/src/beacon_genetics/manager/mod.rs` | 352 | `BeaconGeneticsManager` |

**What it provides:**
- `CapabilityCaller` — async trait for routing capability calls (generic)
- `NeuralApiCapabilityCaller` — via Neural API socket (biomeOS-specific)
- `DirectBeardogCaller` — direct BearDog socket (biomeOS-specific, **hardcoded**)
- Meeting model: record, relationship, visibility, cluster membership
- `BeaconGeneticsManager` — initiate meetings, sync with peers, decrypt beacons

**primalSpring action:** Absorb the generic types (`BeaconId`,
`MeetingRecord`, `CapabilityCaller`). Replace `DirectBeardogCaller` with
`connect_by_capability("security")`. The manager orchestration logic maps
directly to primalSpring's coordination domain.

#### 2.2 Dark Forest Beacon

| File | Lines | Key Types |
|------|-------|-----------|
| `crates/biomeos-spore/src/dark_forest/beacon.rs` | 556 | `DarkForestBeacon`, `EncryptedBeacon`, `BeaconPlaintext` |

**What it provides:**
- Zero-metadata encrypted beacons (ciphertext indistinguishable from noise)
- `generate_encrypted_beacon()` → `beacon.encrypt` via capability call
- `try_decrypt_beacon()` → `beacon.try_decrypt` via capability call
- Lineage proof generation and verification
- Session key derivation

**Pattern (zero-metadata handshake via Tower Atomic):**
1. BearDog: `beacon.generate` → family-scoped seed (mitochondrial DNA)
2. BearDog: `beacon.encrypt` → ChaCha20-Poly1305 AEAD ciphertext
3. Songbird: `network.beacon_exchange` → relay encrypted beacon over mesh
4. Remote BearDog: `beacon.try_decrypt` → only family members succeed
5. Rendezvous: Tower stores only ciphertext, matches by hashed lineage

**primalSpring action:** Absorb the protocol, not the implementation.
primalSpring validates that the beacon capability chain is available:
`beacon.generate` → `beacon.encrypt` → `network.beacon_exchange` →
`beacon.try_decrypt`. The crypto implementation stays in BearDog.

#### 2.3 Rendezvous Protocol

| File | Lines | Key Types |
|------|-------|-----------|
| `crates/biomeos-api/src/handlers/rendezvous.rs` | 726 | `RendezvousState`, `RendezvousPostRequest/Response` |
| `crates/biomeos-api/src/beacon_verification.rs` | 358 | `verify_dark_forest_token()`, `hash_via_capability()` |

**What it provides:**
- POST beacon → verify Dark Forest token → store encrypted → match by lineage hash
- 5-minute TTL on rendezvous slots
- `PeerConnectionInfo` for NAT traversal strategy
- Fallback: Neural API first, then direct BearDog/Songbird sockets (**hardcoded**)

**primalSpring action:** Reference only. The rendezvous handler is an HTTP
endpoint that runs in biomeOS's API server. primalSpring should validate
that the rendezvous endpoint is reachable and that the beacon verification
chain works, but not reimplement the HTTP handler.

---

### Tier 3 — biomeOS-Specific (reference only)

These patterns contain hardcoded primal names and are specific to
biomeOS's deployment orchestration. primalSpring references them but
does not absorb the code.

#### 3.1 Deployment Orchestrator

| File | Lines | Notes |
|------|-------|-------|
| `crates/biomeos-atomic-deploy/src/orchestrator.rs` | 565 | Hardcodes `beardog-server`, `songbird-orchestrator` |
| `crates/biomeos-atomic-deploy/src/deployment_graph.rs` | 194 | Hardcodes primal names in node configs |

**Why reference only:** These launch actual primal processes with specific
binary names and env vars. primalSpring doesn't launch primals — it validates
that compositions are healthy and coordinates graph execution.

#### 3.2 Capability Translation Registry

| File | Lines | Notes |
|------|-------|-------|
| `crates/biomeos-atomic-deploy/src/capability_translation/defaults.rs` | ~300 | Maps semantic → actual methods per primal |

**Why reference only:** The semantic→actual mapping is how biomeOS's Neural API
routes `beacon.generate` to BearDog's actual method. primalSpring uses
`capability.call` through the Neural API and doesn't need to know the mapping.

---

## Validation Matrix (Capability-Based)

The NUCLEUS spec (`specs/NUCLEUS_ATOMIC_COMPOSITION.md`) defines a validation
matrix using primal names. Here is the **capability-based equivalent** that
primalSpring should implement:

### Tower Atomic

| Test | Capability Call | Expected |
|------|----------------|----------|
| Security provider health | `capability.discover("security")` → `health.check` | `healthy` |
| Discovery provider health | `capability.discover("discovery")` → `health.check` | `healthy` |
| Beacon generation | `capability.call("beacon", "generate")` | seed bytes |
| Beacon encrypt/decrypt | `capability.call("beacon", "encrypt")` + `try_decrypt` | roundtrip OK |
| Network beacon exchange | `capability.call("network", "beacon_exchange")` | delivery OK |
| Lineage verification | `capability.call("beacon", "verify_lineage")` | `is_family: true` |

### Node Atomic (Tower + Compute)

| Test | Capability Call | Expected |
|------|----------------|----------|
| All Tower checks | (above) | (above) |
| Compute provider health | `capability.discover("compute")` → `health.check` | `healthy` |
| GPU detection | `capability.call("compute", "query_capabilities")` | GPU info |
| Resource estimation | `capability.call("compute", "estimate_resources")` | metrics |

### Nest Atomic (Tower + Storage)

| Test | Capability Call | Expected |
|------|----------------|----------|
| All Tower checks | (above) | (above) |
| Storage provider health | `capability.discover("storage")` → `health.check` | `healthy` |
| Store data | `capability.call("storage", "store")` | success |
| List keys | `capability.call("storage", "list")` | array |
| Retrieve data | `capability.call("storage", "retrieve")` | data |

### Full NUCLEUS

| Test | Capability Call | Expected |
|------|----------------|----------|
| All Tower + Node + Nest checks | (above) | (above) |
| AI provider health | `capability.discover("ai")` → `health.check` | `healthy` |
| Provenance provider health | `capability.discover("provenance")` → `health.check` | `healthy` |
| DAG provider health | `capability.discover("dag")` → `health.check` | `healthy` |
| Commit provider health | `capability.discover("commit")` → `health.check` | `healthy` |

---

## Deploy Graph Schema

biomeOS deploy graphs use this TOML schema. primalSpring should support the
same format for BYOB graphs:

```toml
[graph]
name = "tower_atomic_bootstrap"
version = "0.3.0"
description = "Tower Atomic: security + discovery foundation"
coordination = "Sequential"  # Sequential | Parallel | ConditionalDag | Pipeline | Continuous

[graph.metadata]
internal_bond_type = "covalent"
default_interaction_bond = "ionic"
trust_model = "genetic_lineage"

[[graph.node]]
name = "security_provider"
binary = "beardog_primal"          # actual binary (biomeOS resolves at runtime)
order = 1
required = true
depends_on = []
health_method = "health.liveness"
by_capability = "security"          # capability-first discovery
capabilities = ["beacon.generate", "beacon.encrypt", "crypto.sign"]

[[graph.node]]
name = "discovery_provider"
binary = "songbird_primal"
order = 2
required = true
depends_on = ["security_provider"]
health_method = "health.liveness"
by_capability = "discovery"
capabilities = ["network.beacon_exchange", "mesh.join"]
```

Key fields for capability-based graphs:
- `by_capability` — discover provider by what it provides, not who it is
- `capabilities` — what this node provides once healthy
- `depends_on` — dependency edges for topological sorting into startup waves
- `health_method` — JSON-RPC method to call for health probing
- `coordination` — execution pattern for the graph

---

## Bonding Model

biomeOS's `Plasmodium` module defines 4 bond types for multi-NUCLEUS coordination.
primalSpring's bonding module has the data models but needs live validation.

| Bond | Trust Model | Capability Sharing | Discovery |
|------|------------|-------------------|-----------|
| **Covalent** | Shared family seed | Full, bidirectional | BirdSong mesh, mutual |
| **Ionic** | Contract-based | Limited, per-contract | API calls only |
| **Metallic** | Infrastructure | Electron sea, specialized | Centralized orchestration |
| **Weak** | Zero trust | Read-only, minimal | Ephemeral, no auth |

**primalSpring action:** Evolve bonding from data-model-only to live validation.
Use `PlasmodiumAgent::meld()` and `split()` patterns. Validate that capability
routing tables merge correctly when two NUCLEUS instances bond.

---

## What Stays in biomeOS

- Process launching and lifecycle management
- HTTP API endpoints (rendezvous, genome distribution, livespore)
- Neural API server routing internals
- CLI commands (`biomeos validate`, `biomeos deploy`)
- Device-specific deployment scripts (Pixel8a, LiveSpore USB)
- GenomeBin packaging and distribution

## What Moves to primalSpring

- Atomic composition validation (via capability discovery)
- Deploy graph structural and live validation
- Startup wave computation (topological sort)
- Bonding validation (meld/split/resolve)
- Beacon coordination validation (capability chain)
- Cross-spring ecology validation
- BYOB graph template and graduation criteria

---

## Cross-Learning Pattern

The goal: springs learn from each other through primalSpring.

```
biomeOS → primalSpring: "Here are the coordination patterns"
primalSpring → springs: "Here's how to validate your compositions"
springs → primalSpring: "Here's what we learned" (like barraCuda shader growth)
primalSpring → biomeOS: "Here's what evolved"
```

primalSpring is the coordination reference. It doesn't launch primals.
It validates that primals are healthy, compositions are complete,
and coordination patterns work. biomeOS focuses on being the best
biomeOS primal it can be.
