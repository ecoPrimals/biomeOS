# Spring Niche Deployment Guide

**Version**: 1.0.0
**Status**: Active
**Date**: March 15, 2026
**Origin**: Absorbed from wetSpring V114 niche setup guidance + wateringHole standards
**Resolves**: wateringHole SPRING_EVOLUTION_ISSUES ISSUE-014

---

## Overview

This guide walks a spring team through the steps required to become a
deployable niche within biomeOS. It condenses the full
`SPRING_AS_NICHE_DEPLOYMENT_STANDARD` into a practical 7-step checklist,
with concrete file paths and validation commands.

**Reference implementations**:
- wetSpring V114 (life science)
- ludoSpring (game science)
- airSpring V0.8.1 (ecology)

---

## Prerequisites

- Rust toolchain (stable, edition 2024)
- biomeOS workspace checked out
- `#![forbid(unsafe_code)]` in production crate roots (test-utils may use guarded unsafe)
- scyBorg triple-copyleft license headers (AGPL-3.0-or-later + ORC + CC-BY-SA 4.0)

---

## 7-Step Niche Deployment Checklist

### Step 1: UniBin Binary

Build a single binary with standard subcommands:

```bash
<spring> server    # Start IPC server (germination mode)
<spring> status    # Health and capability info
<spring> version   # Version info
```

The binary must be cross-platform (ecoBin: pure Rust, zero C dependencies).

**Validation**:
```bash
cargo build --release -p <spring>
./<spring> version        # prints version
./<spring> status         # prints capabilities JSON
```

### Step 2: Unix Socket IPC Server

Start a JSON-RPC 2.0 server on a Unix domain socket:

```
$XDG_RUNTIME_DIR/biomeos/<spring>-${FAMILY_ID}.sock
```

The server must handle at minimum:
- `health.check` — returns status, version, capabilities, uptime
- `capability.list` — returns all operations this spring provides
- `lifecycle.status` — returns name, state, capabilities

**Validation**:
```bash
export FAMILY_ID=dev
./<spring> server &
echo '{"jsonrpc":"2.0","method":"health.check","id":1}' | \
  socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/biomeos/<spring>-dev.sock
```

### Step 3: Capability Domain Registration

Register your spring's capabilities with biomeOS in three places:

**A. Primal names** (`biomeos-types/src/primal_names.rs`):

Your spring should already be listed in `SPRING_PRIMALS`.

**B. Capability domain** (`biomeos-types/src/constants.rs`):

Your domain constant should exist in the `capabilities` module
(e.g., `ECOLOGY`, `SCIENCE`, `PHYSICS`).

**C. Capability translation** (`config/capability_registry.toml`):

```toml
[domains.<domain>]
provider = "<spring>"
capabilities = ["<domain>", "<sub_cap_1>", "<sub_cap_2>"]

[translations.<domain>]
"<domain>.<method_a>" = { provider = "<spring>", method = "<domain>.<method_a>" }
```

**D. Runtime registration** via `capability.register`:

```json
{
  "jsonrpc": "2.0",
  "method": "capability.register",
  "params": {
    "capability": "<domain>",
    "primal": "<spring>",
    "socket": "$XDG_RUNTIME_DIR/biomeos/<spring>-${FAMILY_ID}.sock",
    "source": "startup",
    "semantic_mappings": {
      "<method_a>": "<domain>.<method_a>"
    }
  },
  "id": 1
}
```

Or from Rust using the SDK:

```rust
use biomeos_primal_sdk::provider::{register_capabilities, CapabilityRegistration};

let reg = CapabilityRegistration {
    capability: "ecology".into(),
    primal: "airspring".into(),
    socket: socket_path.to_string(),
    source: "startup".into(),
    semantic_mappings: mappings,
};
register_capabilities(&neural_api_socket, &reg).await?;
```

**Validation**:
```bash
echo '{"jsonrpc":"2.0","method":"capability.list","id":1}' | \
  socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/biomeos/neural-api.sock
# Should list your domain in the response
```

### Step 4: Deploy Graph

Create `graphs/<spring>_deploy.toml` in the biomeOS workspace:

```toml
[graph]
id = "<spring>_deploy"
description = "<Spring> primal atop Node Atomic"
coordination = "Sequential"

# Phase 1: Tower Atomic
[[nodes]]
id = "germinate_beardog"
# ... security foundation

[[nodes]]
id = "germinate_songbird"
depends_on = ["germinate_beardog"]
# ... discovery foundation

# Phase 2: Optional dependencies (ToadStool for GPU, NestGate for data)

# Phase 3: The spring itself
[[nodes]]
id = "germinate_<spring>"
depends_on = ["germinate_beardog", "germinate_songbird"]
output = "<spring>_genesis"
capabilities = ["<domain>", "<cap1>", "<cap2>"]

[nodes.primal]
by_capability = "<domain>"

[nodes.operation]
name = "start"

[nodes.operation.params]
mode = "server"
family_id = "${FAMILY_ID}"

[nodes.capabilities_provided]
"<domain>.<method1>" = "<domain>.<method1>"

# Phase 4: Validation
[[nodes]]
id = "validate_<spring>_atomic"
depends_on = ["germinate_<spring>"]

[nodes.operation]
name = "health_check"
```

A niche template must also be registered in
`biomeos-atomic-deploy/src/handlers/niche.rs` so that
`niche.list` returns it and `niche.deploy` can execute it.

**Validation**:
```bash
biomeos deploy graphs/<spring>_deploy.toml
```

### Step 5: Provenance Trio Integration

Connect to rootpulse (rhizoCrypt + LoamSpine + sweetGrass) for experiment
lineage tracking. Zero compile-time coupling — use `capability.call`:

```rust
use biomeos_primal_sdk::provider::provenance;

// Begin session
let session = provenance::begin_experiment_session(&provider, "exp-001", agent_did).await?;

// Record steps
provenance::record_step(&provider, &session_id, "compute_et0", data).await?;

// Commit
provenance::complete_experiment(&provider, &session_id).await?;
```

Data flows: experiment input → spring processing → NestGate storage → provenance commit.

### Step 6: Cross-Spring Time Series

Implement the `ecoPrimals/time-series/v1` format for data exchange:

```rust
use biomeos_types::time_series::{CrossSpringTimeSeries, TimeSeriesSource, SCHEMA_V1};

let ts = CrossSpringTimeSeries::new(
    "soil_moisture_vol",
    "m3/m3",
    timestamps,
    values,
)?
.with_source(TimeSeriesSource {
    spring: "airSpring".into(),
    experiment: Some("exp022".into()),
    capability: Some("ecology.water_balance".into()),
});
```

Pass time series as arguments to cross-spring `capability.call` operations.

### Step 7: Workflow Graphs (Optional)

For springs participating in cross-spring pipelines, create workflow graphs:

- `cross_spring_ecology.toml` — airSpring ET₀ → wetSpring diversity → neuralSpring spectral
- `cross_spring_soil_microbiome.toml` — airSpring soil → wetSpring microbiome → provenance

These graphs coordinate multi-spring computations through biomeOS's graph engine.

---

## Compliance Checklist

| # | Requirement | Mandatory |
|---|------------|-----------|
| 1 | UniBin binary (`server`, `status`, `version`) | Yes |
| 2 | JSON-RPC 2.0 over Unix socket | Yes |
| 3 | Socket at `$XDG_RUNTIME_DIR/biomeos/<spring>-${FAMILY_ID}.sock` | Yes |
| 4 | `health.check` and `capability.list` methods | Yes |
| 5 | Capability domain registered in biomeos-types | Yes |
| 6 | Deploy graph at `graphs/<spring>_deploy.toml` | Yes |
| 7 | Niche template in `handlers/niche.rs` | Yes |
| 8 | Provenance trio integration | Recommended |
| 9 | Cross-Spring Time Series v1 support | Recommended |
| 10 | No hardcoded primal names in production code | Yes |
| 11 | `#![forbid(unsafe_code)]` | Yes |
| 12 | scyBorg triple-copyleft license | Yes |
| 13 | Neural API registration (`capability.register`) | Yes |
| 14 | Clean SIGTERM shutdown | Yes |

---

## Current Spring Status

| Spring | Step 1 | Step 2 | Step 3 | Step 4 | Step 5 | Step 6 | Step 7 |
|--------|--------|--------|--------|--------|--------|--------|--------|
| wetSpring | Done | Done | Done | Done | Planned | Planned | Planned |
| airSpring | Done | Done | Done | Done | Done | Done | Done |
| neuralSpring | Done | Done | Done | Done | Planned | Planned | Planned |
| groundSpring | Done | Done | Partial | Done | Planned | Planned | Planned |
| hotSpring | Done | Done | Partial | Done | Planned | Planned | Planned |
| healthSpring | Done | Done | Done | Done | Planned | Planned | Planned |
| ludoSpring | Done | Done | Done | Done | Planned | Planned | Done |

---

## Related Documents

Paths relative to the `ecoPrimals/` monorepo root:

- `wateringHole/SPRING_AS_NICHE_DEPLOYMENT_STANDARD.md` — Full standard
- `wateringHole/SPRING_AS_PROVIDER_PATTERN.md` — Registration pattern
- `wateringHole/CROSS_SPRING_DATA_FLOW_STANDARD.md` — Time series exchange
- `wateringHole/PRIMAL_DEPLOYMENT_STANDARD.md` — Socket resolution
