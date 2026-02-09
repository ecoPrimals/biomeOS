# Plasmodium: Over-NUCLEUS Coordination Specification

**Date**: February 9, 2026  
**Status**: Specification  
**Version**: 1.0.0  
**Depends on**: NUCLEUS_BONDING_MODEL.md, NUCLEUS_ATOMIC_COMPOSITION.md, MESH_IPC_METHODS_SPEC.md

---

## Abstract

Plasmodium is the emergent coordination layer that forms when two or more NUCLEUS instances bond covalently. Named after the multinucleate organism *Physarum polycephalum* (slime mold), it provides a collective view of distributed capabilities, unified job submission, and decentralized resource routing -- without introducing a central controller.

Each NUCLEUS remains fully autonomous. The Plasmodium is an **observation and routing layer** in biomeOS that aggregates state from bonded NUCLEUS instances into a collective view. It is not a new service or primal -- it is a coordination pattern that emerges from existing primitives.

---

## 1. Design Principles

### 1.1 No Central Brain

A slime mold has no nervous system. Each cell pulses independently. The collective behavior -- finding shortest paths, distributing nutrients, avoiding obstacles -- emerges from local interactions.

Plasmodium follows this pattern:
- No master node. Any gate can query the collective.
- No election protocol. No leader.
- Each NUCLEUS publishes its state via existing Songbird mesh primitives.
- biomeOS on any gate can aggregate the collective view.

### 1.2 Pulsing Coordination

Slime molds coordinate via rhythmic chemical pulses. In Plasmodium:
- **Pulse** = Songbird `mesh.announce` heartbeat (already implemented)
- **Pulse payload** = Gate capabilities, load, available models, running jobs
- **Pulse interval** = Configurable, default 30 seconds
- **Pulse propagation** = BirdSong encrypted UDP multicast on LAN, TCP mesh for WAN

### 1.3 Graceful Degradation

Cut a slime mold in half and both halves continue functioning independently.
- If gate2 goes offline, Tower's Plasmodium view shrinks to Tower-only
- If Tower goes offline, gate2's Plasmodium view shrinks to gate2-only
- When they reconnect, the collective reforms automatically via BirdSong discovery
- No state to reconcile -- each pulse carries the full gate state

### 1.4 Capability-Based, Not Name-Based

Plasmodium routes workloads to capabilities, not to named gates:
- "I need a GPU with 24GB VRAM" -> routes to the gate that has one
- "I need model TinyLlama" -> routes to the gate that has it cached
- "I need 256GB RAM" -> routes to the gate that has it

---

## 2. Architecture

### 2.1 Layer Diagram

```
+---------------------------------------------------------------+
|  biomeos plasmodium CLI / API                                  |
|  (aggregate view, job submission, routing)                     |
+---------------------------------------------------------------+
|  Plasmodium Module (biomeos-core::plasmodium)                  |
|  - Queries local Songbird mesh.peers                           |
|  - Connects to peer NUCLEUS via Songbird TCP                   |
|  - Aggregates PlasmodiumState                                  |
+---------------------------------------------------------------+
|  Covalent Bond Layer (Songbird mesh)                           |
|  - BirdSong encrypted heartbeats                               |
|  - Dark Forest verified trust                                  |
|  - TCP JSON-RPC for data plane                                 |
+---------------------------------------------------------------+
|  NUCLEUS A           |  NUCLEUS B           |  NUCLEUS C       |
|  (Tower gate)        |  (gate2)             |  (future gate)   |
|  BearDog+Songbird    |  BearDog+Songbird    |  BearDog+...     |
|  Toadstool+NestGate  |  Toadstool+NestGate  |  ...             |
|  Squirrel            |  Squirrel            |  ...             |
+---------------------------------------------------------------+
```

### 2.2 Data Flow

```
biomeos plasmodium status
    |
    v
Query local Songbird: mesh.peers
    |
    v
For each peer:
    Connect via Songbird TCP (peer_ip:3492)
    Query: lifecycle.status, toadstool.query_capabilities
    Query: model-cache manifest (via biomeos API or direct file)
    |
    v
Aggregate into PlasmodiumState
    |
    v
Display unified collective view
```

### 2.3 Query Methods Used

All queries use existing primal JSON-RPC methods. No new primal methods needed.

| Method | Primal | Purpose |
|--------|--------|---------|
| `mesh.peers` | Songbird | Discover bonded gates |
| `mesh.status` | Songbird | Local mesh status |
| `lifecycle.status` | Neural API | Primal health per gate |
| `toadstool.query_capabilities` | Toadstool | GPU/compute info per gate |
| `storage.list` | NestGate | Storage info per gate |

---

## 3. Types

### 3.1 PlasmodiumState

The aggregate snapshot of the collective.

```rust
pub struct PlasmodiumState {
    /// All gates in the collective
    pub gates: Vec<GateInfo>,

    /// Timestamp of this snapshot
    pub snapshot_at: chrono::DateTime<chrono::Utc>,

    /// Family ID binding this plasmodium
    pub family_id: String,

    /// Total capabilities across all gates
    pub collective_capabilities: CollectiveCapabilities,
}
```

### 3.2 GateInfo

Per-gate status within the collective.

```rust
pub struct GateInfo {
    /// Gate identifier (hostname)
    pub gate_id: String,

    /// Network address (from mesh.peers)
    pub address: String,

    /// Is this the local gate?
    pub is_local: bool,

    /// Primal status (which primals are running)
    pub primals: Vec<PrimalStatus>,

    /// GPU/compute capabilities
    pub compute: ComputeInfo,

    /// Cached models
    pub models: Vec<String>,

    /// Current load (0.0 - 1.0)
    pub load: f64,

    /// Whether this gate is reachable right now
    pub reachable: bool,

    /// Bond type to this gate
    pub bond_type: BondType,
}
```

### 3.3 CollectiveCapabilities

Union of capabilities across all gates.

```rust
pub struct CollectiveCapabilities {
    /// Total GPUs across all gates
    pub total_gpus: usize,

    /// GPU details per gate
    pub gpus: Vec<GpuInfo>,

    /// Total RAM across all gates (bytes)
    pub total_ram_bytes: u64,

    /// Total storage across all gates (bytes)
    pub total_storage_bytes: u64,

    /// All models available across the mesh
    pub models: Vec<ModelAvailability>,

    /// All capabilities available (union)
    pub capabilities: Vec<String>,
}
```

---

## 4. CLI Interface

### 4.1 `biomeos plasmodium status`

Shows the collective state at a glance.

```
  Plasmodium Status - Family: nat0
  =================================

  Collective: 2 gates bonded (covalent)

  GATE              PRIMALS     GPUs    RAM       LOAD   MODELS
  ---------------------------------------------------------------
  Tower (local)     5/5         1       64 GB     0.12   2
  gate2             5/5         2       256 GB    0.05   2
  ---------------------------------------------------------------
  TOTAL             10          3       320 GB           3 unique

  Capabilities: crypto, discovery, compute, ai, storage, network
  Bond: covalent (shared family seed, genetic trust)
```

### 4.2 `biomeos plasmodium gates`

Detailed per-gate hardware and status.

```
  Gate: Tower (local)
    Address:  192.168.1.100
    Primals:  BearDog, Songbird, Toadstool, NestGate, Squirrel
    GPUs:     NVIDIA RTX 4070 (12 GB)
    RAM:      64 GB
    Load:     12%
    Models:   TinyLlama/1.1B-Chat, runwayml/stable-diffusion-v1-5

  Gate: gate2
    Address:  192.168.1.132
    Primals:  BearDog, Songbird, Toadstool, NestGate, Squirrel
    GPUs:     NVIDIA RTX 3090 (24 GB), AMD Radeon RX 6950 XT (16 GB)
    RAM:      256 GB
    Load:     5%
    Models:   TinyLlama/1.1B-Chat, mistralai/Mistral-7B-Instruct-v0.2
```

### 4.3 `biomeos plasmodium models`

Aggregate model view across all gates.

```
  MODEL                              GATES        SIZE       FORMAT
  -------------------------------------------------------------------
  TinyLlama/TinyLlama-1.1B-Chat     Tower,gate2  2.1 GB     safetensors
  runwayml/stable-diffusion-v1-5     Tower        4.1 GB     huggingface
  mistralai/Mistral-7B-Instruct      gate2        13.8 GB    safetensors
  -------------------------------------------------------------------
  Total: 3 unique models, 20.0 GB across 2 gates
```

---

## 5. Job Submission (Phase 2)

### 5.1 `biomeos plasmodium submit <graph>`

Submit a workload graph to the collective. The Plasmodium routes it to the best gate(s).

### 5.2 Routing Strategy

1. **Capability match**: Filter gates that have required capabilities
2. **Resource match**: Filter gates that have required resources (GPU VRAM, RAM, model)
3. **Load balance**: Prefer gates with lower load
4. **Locality**: Prefer local gate when resources are equal
5. **Model affinity**: Prefer gates that already have the model cached

### 5.3 `biomeos plasmodium jobs`

List running and completed jobs across the collective.

---

## 6. Relationship to Existing Specs

| Spec | Relationship |
|------|-------------|
| NUCLEUS_BONDING_MODEL.md | Plasmodium IS the covalent bond collective |
| NUCLEUS_ATOMIC_COMPOSITION.md | Each gate runs a complete NUCLEUS |
| MESH_IPC_METHODS_SPEC.md | Plasmodium queries mesh.peers for gate discovery |
| ECOSYSTEM_ARCHITECTURE.md S4.1 | Covalent bonding section describes the behavior |
| INTER_PRIMAL_INTERACTIONS.md | Plasmodium coordinates inter-gate interactions |

---

## 7. Biological Mapping

| Slime Mold | Plasmodium |
|------------|------------|
| Cell | NUCLEUS instance on a gate |
| Nucleus | BearDog (identity, genetic trust) |
| Cytoplasm | Songbird mesh (communication substrate) |
| Nutrient flow | Workload routing |
| Pulse | BirdSong heartbeat / mesh.announce |
| Pseudopod | New gate joining the collective |
| Sclerotium | Gate going offline (dormant, will rejoin) |
| Fruiting body | Published result / completed job |

---

## 8. Security Model

Plasmodium inherits the covalent bond security model:
- **Trust**: Genetic lineage via shared family seed
- **Verification**: BearDog Dark Forest challenge-response
- **Encryption**: All inter-gate traffic via BearDog ChaCha20-Poly1305
- **Discovery**: BirdSong encrypted UDP (indistinguishable from noise)
- **No credential exchange**: Trust is genetic, not credential-based

A gate that cannot prove lineage cannot join the Plasmodium. Period.

---

## 9. Implementation Notes

### 9.1 biomeOS Module Location

`crates/biomeos-core/src/plasmodium.rs` -- core types and collective query logic  
`crates/biomeos/src/modes/plasmodium.rs` -- CLI handler

### 9.2 Dependencies

- `AtomicClient` for JSON-RPC to local and remote primals
- `ModelCache` for local model inventory
- Songbird mesh for peer discovery

### 9.3 No New Primals

Plasmodium is a biomeOS coordination pattern, not a new primal. It uses existing primal primitives exclusively. No primal needs modification to support Plasmodium.
