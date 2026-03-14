> **HISTORICAL** — This handoff predates v2.37. See CURRENT_STATUS.md for latest.

# Plasmodium Over-NUCLEUS Coordination - Handoff

**Date**: February 9, 2026  
**Status**: Phase 1 Complete (read-only collective view)

---

## What Was Built

### 1. Specification

**File**: `specs/PLASMODIUM_OVER_NUCLEUS_SPEC.md`

Defines the Plasmodium coordination layer -- the emergent collective organism that forms when 2+ NUCLEUS instances bond covalently. Named after the slime mold *Physarum polycephalum*.

Key design decisions:
- No central brain -- any gate can query the collective
- Uses only existing primal primitives (no new primal methods needed)
- Graceful degradation when gates go offline
- Capability-based routing, not name-based

### 2. Core Module

**File**: `crates/biomeos-core/src/plasmodium.rs`

Types:
- `PlasmodiumState` -- aggregate snapshot of the collective
- `GateInfo` -- per-gate status (hostname, primals, GPUs, RAM, models, load)
- `CollectiveCapabilities` -- union of all gate capabilities
- `ComputeInfo`, `GpuInfo`, `PrimalStatus`, `BondType`, `ModelAvailability`

Query engine (`Plasmodium`):
- `new()` -- creates query engine, reads family ID from env/hostname
- `query_collective()` -- full collective snapshot (local + remote gates)
- `query_local_gate()` -- local primal health, GPU detection, model cache, system load
- `discover_peers()` -- via Songbird `mesh.peers` + `PLASMODIUM_PEERS` env fallback
- `query_remote_gate()` -- connects to remote Songbird TCP, queries health/primals
- `aggregate_capabilities()` -- builds unified capability view from all gates

### 3. CLI Mode

**File**: `crates/biomeos/src/modes/plasmodium.rs`

Subcommands:
- `biomeos plasmodium status` -- collective status table (gates, primals, GPUs, RAM, load, models)
- `biomeos plasmodium gates` -- detailed per-gate hardware and primal info
- `biomeos plasmodium models` -- aggregate model availability across all gates

### 4. Cross-References Updated

- `wateringHole/README.md` -- added Plasmodium to "Composed Systems" section
- `wateringHole/INTER_PRIMAL_INTERACTIONS.md` -- added Plasmodium section
- `specs/NUCLEUS_BONDING_MODEL.md` -- added section 2.2.5 (Plasmodium as covalent collective) + glossary entry
- `specs/COMPLETE_ECOSYSTEM_NUCLEUS_INTEGRATION.md` -- added multi-gate coordination section

---

## Live Test Results

### Tower (local gate)

```
  Plasmodium Status - Family: nat0
  =========================================

  Collective: 1 gate bonded (covalent)

  GATE               PRIMALS       GPUs      RAM     LOAD  MODELS
  -----------------------------------------------------------------
  pop-os (local)     2/2              1    31 GB      12%       2

  Capabilities: crypto, discovery, network, security
```

### Tower + gate2 (gate2 offline)

```
  Collective: 1 gate bonded (covalent)

  GATE               PRIMALS       GPUs      RAM     LOAD  MODELS
  -----------------------------------------------------------------
  pop-os (local)     2/2              1    31 GB      10%       2
  gate2              offline          0        -        -       0

  Capabilities: crypto, discovery, network, security
```

Graceful degradation works correctly -- gate2 shows as offline, collective shrinks to 1 reachable gate.

---

## Configuration

- `FAMILY_ID` or `NODE_FAMILY_ID` env var sets the family identity
- `GATE_ID` or `HOSTNAME` env var sets the local gate name
- `PLASMODIUM_PEERS` env var: comma-separated list for manual peer discovery
  - Format: `name@address` or just `address`
  - Example: `PLASMODIUM_PEERS="gate2@192.168.1.132,gate3@192.168.1.133"`

---

## Phase 2 Evolution (Future)

When Songbird mesh is fully operational across gates:

1. **Auto-discovery**: `mesh.peers` will return bonded gates automatically (no `PLASMODIUM_PEERS` needed)
2. **Remote compute query**: Query `toadstool.query_capabilities` on remote gates for GPU/hardware details
3. **Job submission**: `biomeos plasmodium submit <graph>` to route workloads to best gate
4. **Job tracking**: `biomeos plasmodium jobs` to view running/completed jobs across the collective
5. **Remote model query**: Query model cache manifest on remote gates via API

### Dependencies for Phase 2

- Songbird mesh peers populated (requires Songbird evolution for reliable TCP peer connections)
- Toadstool `toadstool.query_capabilities` method on remote gates
- NestGate storage fixes for remote model manifest sharing
- Neural API running on each gate for `lifecycle.status` queries

---

## Files Modified

| File | Change |
|------|--------|
| `crates/biomeos-core/src/plasmodium.rs` | **NEW** - Core types and query engine |
| `crates/biomeos-core/src/lib.rs` | Added `pub mod plasmodium;` |
| `crates/biomeos/src/modes/plasmodium.rs` | **NEW** - CLI handler |
| `crates/biomeos/src/modes/mod.rs` | Added `pub mod plasmodium;` |
| `crates/biomeos/src/main.rs` | Added `PlasmodiumCommand` enum and `Mode::Plasmodium` |
| `specs/PLASMODIUM_OVER_NUCLEUS_SPEC.md` | **NEW** - Full specification |
| `specs/NUCLEUS_BONDING_MODEL.md` | Added section 2.2.5 + glossary entry |
| `specs/COMPLETE_ECOSYSTEM_NUCLEUS_INTEGRATION.md` | Added multi-gate coordination section |
| `wateringHole/README.md` | Added Plasmodium to Composed Systems |
| `wateringHole/INTER_PRIMAL_INTERACTIONS.md` | Added Plasmodium section |
