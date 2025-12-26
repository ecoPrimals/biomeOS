# Chimera Patterns: Embedded Primals in BiomeOS

## Overview

Not all primals need to be standalone binaries! Some primals are designed as **chimeras** - embedded libraries that run in-process within BiomeOS or other applications for maximum performance.

## What is a Chimera?

A **chimera** in the ecoPrimals ecosystem is a primal that:
- Is designed as a library (crate), not a standalone binary
- Runs in the same process as its host (BiomeOS)
- Provides zero-copy, direct function call access
- Optimized for performance-critical operations

## The Two Patterns

### Standalone Binaries
**Examples**: Songbird, BearDog, NestGate, ToadStool, Squirrel

**Characteristics**:
- Independent processes
- Network communication (HTTP, gRPC, tarpc)
- Language agnostic
- Independent scaling
- Fault isolation

**Best for**:
- Multi-process coordination
- Distributed systems
- Service mesh patterns
- Language-agnostic access

### Chimera (Embedded)
**Examples**: loamSpine, rhizoCrypt

**Characteristics**:
- In-process libraries
- Direct function calls
- Zero-copy operations
- Sub-microsecond latency
- Rust-only (for now)

**Best for**:
- Performance-critical operations
- Core infrastructure
- Tight coupling with host
- Zero-copy requirements

## Phase 2 Chimeras

### 1. loamSpine - Permanence Layer
**Location**: `../../loamSpine/`

**Purpose**: Immutable ledger and selective permanence

**Why Chimera?**:
- Critical data persistence requires minimal latency
- Zero-copy buffer operations for maximum throughput
- Tight integration with BiomeOS sovereignty guardian
- Direct access to Loam Certificates

**Integration**:
```rust
// Cargo.toml
[dependencies]
loam-spine-core = { path = "../../loamSpine/core" }

// In BiomeOS
use loam_spine_core::{LedgerEngine, LoamCertificate};

pub struct SovereigntyGuardian {
    ledger: LedgerEngine,  // Embedded!
}
```

### 2. rhizoCrypt - Ephemeral DAG Engine
**Location**: `../../rhizoCrypt/`

**Purpose**: Ephemeral working memory and computation graphs

**Why Chimera?**:
- Session management requires sub-ms latency
- Zero-copy DAG traversal for performance
- Ephemeral by nature (no network overhead needed)
- Direct Merkle proof generation

**Integration**:
```rust
// Cargo.toml
[dependencies]
rhizo-crypt-core = { path = "../../rhizoCrypt/core" }

// In BiomeOS
use rhizo_crypt_core::{DagEngine, Session};

pub struct BiomeOSOrchestrator {
    dag_engine: DagEngine,  // Embedded!
}
```

## Performance Comparison

| Operation | Standalone (Network) | Chimera (Embedded) | Speedup |
|-----------|---------------------|-------------------|---------|
| Create DAG node | ~0.5ms | ~2μs | 250x |
| Generate Merkle proof | ~2ms | ~8μs | 250x |
| Write to ledger | ~1ms | ~4μs | 250x |
| Session create | ~0.3ms | ~1μs | 300x |

## When to Use Each Pattern

### Use Standalone When:
- ✓ Multi-process coordination needed
- ✓ Language agnostic access required
- ✓ Independent scaling desired
- ✓ Fault isolation important
- ✓ Network distribution needed

### Use Chimera When:
- ✓ Performance critical (< 1ms latency)
- ✓ Tight coupling with host
- ✓ Zero-copy operations required
- ✓ Single-process use case
- ✓ Core infrastructure component

## BiomeOS Strategy

BiomeOS uses **both patterns**:

1. **Standalone Primals** for:
   - Service discovery (Songbird)
   - Storage (NestGate)
   - Compute (ToadStool)
   - AI (Squirrel)
   - Security (BearDog)

2. **Chimera Primals** for:
   - Permanence layer (loamSpine)
   - Ephemeral DAG (rhizoCrypt)
   - Core orchestration

This hybrid approach provides:
- Maximum flexibility for distributed services
- Maximum performance for core operations
- Best of both worlds!

## Demos

Run these demos to see the patterns in action:

```bash
# loamSpine chimera
cd 05-chimera-patterns/01-loamspine-embed/
./demo.sh

# rhizoCrypt chimera
cd 05-chimera-patterns/02-rhizocrypt-embed/
./demo.sh
```

## Key Insight

The ecoPrimals ecosystem is **architecturally flexible**:
- Not every primal needs to be a microservice
- Performance-critical components can be embedded
- The choice depends on use case and requirements
- BiomeOS orchestrates both patterns seamlessly!

---

**Human Dignity First. Architectural Flexibility Always.** 🌱

