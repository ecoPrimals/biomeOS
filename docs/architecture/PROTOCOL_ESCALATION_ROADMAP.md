# Protocol Escalation Roadmap

**Status**: 📋 PLANNING  
**Created**: January 28, 2026  
**Spec**: [specs/LIVING_GRAPH_PROTOCOL_ESCALATION_SPEC.md](specs/LIVING_GRAPH_PROTOCOL_ESCALATION_SPEC.md)

---

## Overview

biomeOS is a **JSON-RPC and tarpc first ecosystem**:
- **JSON-RPC**: Runtime flexibility, human-readable, debugging, bootstrap
- **tarpc**: Performance-critical paths after stable startup

Neural API manages the **Living Graph** - tracking protocol state per connection and orchestrating escalation from JSON-RPC → tarpc.

---

## Current State

### ✅ Completed (Jan 28, 2026)

| Component | Status | Notes |
|-----------|--------|-------|
| Primals have tarpc | ✅ | BearDog, Songbird already have tarpc integrations |
| JSON-RPC working | ✅ | Full Neural API orchestration via JSON-RPC |
| HTTP headers fixed | ✅ | Songbird v8.14.0 complete |
| Tower Atomic | ✅ | BearDog ↔ Songbird working via JSON-RPC |

### ✅ Living Graph Protocol Escalation - Phase 1 Complete

| Phase | Component | Status | ETA |
|-------|-----------|--------|-----|
| 1 | Living Graph infrastructure | ✅ **Complete** | Jan 28 |
| 2 | Protocol Escalation Manager | ✅ **Complete** | Jan 28 |
| 3 | JSON-RPC Handlers | ✅ **Complete** | Jan 28 |
| 4 | Primal method spec handoffs | 🔲 Pending | Week 2 |
| 5 | Neural API Integration | ✅ **Complete** | Jan 28 |

### Files Created

- `crates/biomeos-atomic-deploy/src/living_graph.rs` (~450 lines)
- `crates/biomeos-atomic-deploy/src/protocol_escalation.rs` (~450 lines)
- `crates/biomeos-atomic-deploy/src/handlers/protocol.rs` (~600 lines)

### Tests Added

- **14 unit tests** for LivingGraph + ProtocolEscalationManager + ProtocolHandler
- **All 153 tests passing** ✅ (entire package)

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    NEURAL API (biomeOS)                         │
│                                                                 │
│  ┌─────────────────┐    ┌──────────────────────────────────┐   │
│  │  Living Graph   │    │  Protocol Escalation Manager     │   │
│  │                 │    │                                  │   │
│  │  • Node state   │◄──►│  • Auto-escalation (metrics)     │   │
│  │  • Connections  │    │  • Manual escalation (API)       │   │
│  │  • Metrics      │    │  • Fallback handling             │   │
│  └─────────────────┘    └──────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ Orchestrates
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         PRIMALS                                  │
│                                                                 │
│  ┌──────────┐ JSON-RPC (bootstrap) ┌──────────┐                │
│  │ BearDog  │◄────────────────────►│ Songbird │                │
│  │          │                      │          │                │
│  │  tarpc ══╪══════════════════════╪══ tarpc  │                │
│  │  socket  │  tarpc (production)  │  socket  │                │
│  └──────────┘                      └──────────┘                │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Protocol Flow

### 1. Bootstrap (JSON-RPC)

```
Neural API ──JSON-RPC──► BearDog: lifecycle.register
Neural API ──JSON-RPC──► Songbird: lifecycle.register
Songbird ──JSON-RPC──► BearDog: crypto.sha256 (initial calls)
```

### 2. Metrics Collection

```
Neural API monitors:
  - Request count per connection
  - Latency (avg, p99)
  - Error rate
```

### 3. Escalation Decision

```
IF requests > 100 AND latency > 500μs AND both healthy:
  THEN escalate(songbird, beardog)
```

### 4. Escalation Execution

```
Neural API ──JSON-RPC──► BearDog: rpc.tarpc_endpoint
BearDog ──JSON-RPC──► Neural API: { socket: "...tarpc.sock" }
Neural API ──JSON-RPC──► Songbird: rpc.escalate_to(beardog, tarpc_socket)
Songbird ══tarpc══► BearDog: Connected!
```

### 5. Production (tarpc)

```
Songbird ══tarpc══► BearDog: x25519_generate_ephemeral() [~10μs]
Songbird ══tarpc══► BearDog: aes_gcm_encrypt() [~10μs]
```

### 6. Fallback (if needed)

```
IF tarpc failures > 3:
  Neural API ──JSON-RPC──► Songbird: rpc.fallback_to_json_rpc
  Songbird ──JSON-RPC──► BearDog: (degraded mode)
```

---

## New Neural API Methods

| Method | Description |
|--------|-------------|
| `protocol.status` | Get all connection protocol states |
| `protocol.escalate` | Manually escalate a connection |
| `protocol.fallback` | Manually fallback a connection |
| `protocol.metrics` | Get metrics for a connection |
| `graph.protocol_map` | Get full Living Graph snapshot |

---

## New Primal Methods (Spec for Teams)

| Method | Description | Required By |
|--------|-------------|-------------|
| `rpc.tarpc_endpoint` | Advertise tarpc socket | All primals with tarpc |
| `rpc.escalate_to` | Connect to target via tarpc | Source primals |
| `rpc.fallback_to_json_rpc` | Fall back from tarpc | Source primals |

---

## Files to Create

### biomeOS (Neural API)

```
crates/biomeos-atomic-deploy/src/
├── living_graph.rs           # Living Graph struct + state
├── protocol_escalation.rs    # Escalation Manager
├── handlers/
│   └── protocol.rs           # JSON-RPC handlers
└── tests/
    ├── living_graph_tests.rs
    └── escalation_tests.rs
```

### Shared (primal-rpc crate - future)

```
crates/primal-rpc/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── transport.rs          # PrimalTransport trait
│   ├── json_rpc.rs           # JSON-RPC transport
│   ├── tarpc_adapter.rs      # tarpc transport
│   └── services/
│       ├── mod.rs
│       ├── crypto.rs         # CryptoService (BearDog)
│       └── http.rs           # HttpService (Songbird)
└── tests/
```

---

## Performance Targets

| Metric | JSON-RPC | tarpc | Improvement |
|--------|----------|-------|-------------|
| Latency (avg) | ~100μs | ~10μs | 10x |
| Latency (p99) | ~200μs | ~50μs | 4x |
| Throughput | 10K rps | 100K rps | 10x |
| Serialization | ~50μs | ~5μs | 10x |

---

## Dependencies

### Existing

- `tarpc` - Already in primal dependencies
- `tokio` - Async runtime
- `serde` - Serialization

### New

- `bincode` - Binary serialization for tarpc (optional, can use JSON)

---

## Milestones

### M1: Living Graph (Week 1)
- [ ] `LivingGraph` struct
- [ ] `ConnectionState` with metrics
- [ ] `protocol.status` method
- [ ] Basic tests

### M2: Escalation Manager (Week 2)
- [ ] `ProtocolEscalationManager`
- [ ] Auto-escalation logic
- [ ] `protocol.escalate` method
- [ ] `protocol.fallback` method
- [ ] Background monitoring loop

### M3: Primal Integration (Week 3)
- [ ] Spec handoff to primal teams
- [ ] Test with BearDog tarpc endpoint
- [ ] Test with Songbird tarpc client
- [ ] End-to-end escalation test

### M4: Production Ready (Week 4)
- [ ] Chaos tests (failure → fallback → recovery)
- [ ] Performance benchmarks
- [ ] Documentation complete
- [ ] Deploy to Tower Atomic

---

## Related Documents

- [LIVING_GRAPH_PROTOCOL_ESCALATION_SPEC.md](specs/LIVING_GRAPH_PROTOCOL_ESCALATION_SPEC.md) - Full specification
- [NUCLEUS_DEPLOYMENT_SPEC.md](specs/NUCLEUS_DEPLOYMENT_SPEC.md) - Neural API deployment
- [docs/handoffs/](docs/handoffs/) - Primal team handoffs

---

## Questions / Decisions

1. **tarpc socket naming**: `{primal}-{family}-tarpc.sock` or separate convention?
2. **Hybrid mode priority**: Implement now or defer?
3. **Cross-node tarpc**: Wait for Songbird TCP support?

---

*Last Updated: January 28, 2026*

