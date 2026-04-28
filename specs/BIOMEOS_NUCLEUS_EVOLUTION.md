# biomeOS as Full NUCLEUS — Evolution Spec

**Status**: Design Spec (v3.29)
**Date**: April 28, 2026
**Origin**: primalSpring v0.9.20 Phase 55 Audit

## Background

biomeOS currently coordinates the NUCLEUS but does not *embody* it. The
architecture delegates trust to Tower primals (BearDog for crypto, Songbird
for discovery, NestGate for access control) via RPC over Unix sockets.
This is sound for separation-of-concern but introduces a runtime dependency
chain: biomeOS cannot sign a deployment graph until BearDog is healthy,
cannot discover peers until Songbird is running, etc.

The primalSpring audit suggests evolving biomeOS into a full NUCLEUS where
Tower is an embedded subsystem — so biomeOS *is* the trust anchor rather
than merely calling one.

## Current State

### How biomeOS delegates today

```text
┌──────────────┐        crypto.sign        ┌──────────────┐
│   biomeOS    │ ────────────────────────▶  │   BearDog    │
│  (coordinator)│        JSON-RPC/UDS       │  (Tower key  │
│              │ ◀──────────────────────── │   manager)   │
└──────────────┘                           └──────────────┘
       │ spawns                                    ▲
       ├────────▶ Songbird (discovery)             │
       ├────────▶ NestGate (access control)        │
       ├────────▶ ToadStool (runtime)              │
       └────────▶ Squirrel (capability store)      │
                                                   │
                 All crypto via RPC ───────────────┘
```

- `biomeos nucleus --mode full` spawns Tower primals as child processes
- Neural API derives `coordination` purpose key from BearDog after startup
- Graph signing delegates to `crypto.sign` over IPC
- BTSP handshake enforcement delegates to BearDog for `ClientHello` auth
- biomeOS holds no long-lived keys; it's sovereign by design

### What works well

- Clear process isolation: a compromised biomeOS cannot extract BearDog keys
- biomeOS can restart without re-deriving genetic material
- The delegation model maps directly to the two-tier crypto spec
- Each Tower primal can be updated independently

## Target State (Phases)

### Phase 1: Coordination Key Caching (v3.29 — implemented)

biomeOS derives the `coordination` purpose key from BearDog at startup and
caches the **public key** in `NeuralApiServer.coordination_pubkey`. This
enables:

- `graph.sign` routes through BearDog but the server knows *which* key signed
- `graph.verify` can verify signatures offline using the cached public key
- The `biomeos graph sign <path>` CLI command delegates signing to BearDog

The private key never leaves BearDog. biomeOS only caches the public half.

### Phase 2: Embedded BearDog Library (future)

Extract BearDog's key derivation and signing as a library crate
(`beardog-crypto`) that biomeOS can link directly. This eliminates the
IPC round-trip for `crypto.sign` and `crypto.verify` while keeping the
same API surface.

**Requires**: BearDog team collaboration to split the binary into
`beardog-crypto` (library) and `beardog` (daemon wrapping the library).

```text
┌──────────────────────────────────┐
│           biomeOS NUCLEUS        │
│  ┌───────────┐  ┌─────────────┐ │
│  │ beardog-  │  │ Neural API  │ │
│  │ crypto    │  │ Server      │ │
│  │ (library) │  │             │ │
│  └───────────┘  └─────────────┘ │
│        ▲              │         │
│        └── signs ─────┘         │
│                                 │
│  Still spawns: Songbird,        │
│  NestGate, ToadStool, Squirrel  │
└─────────────────────────────────┘
```

### Phase 3: Full Tower Embedding (future)

All Tower services (discovery, access control, BTSP) are available as
library crates. biomeOS becomes a single-binary NUCLEUS where everything
runs in-process with shared memory. Process-based Tower primals become
optional for backward compatibility.

**Trade-offs**:

| Aspect | Process Tower | Embedded Tower |
|--------|--------------|----------------|
| Isolation | Strong (process boundary) | Weaker (shared address space) |
| Latency | IPC round-trip (~0.1ms UDS) | Direct function call (~0ns) |
| Update | Per-primal binary update | Single binary recompile |
| Crash impact | Primal-scoped | Full NUCLEUS restart |
| Memory | Higher (N processes) | Lower (shared heap) |

## First Concrete Step (v3.29)

1. `NeuralApiServer` gains `coordination_pubkey: Arc<RwLock<Option<String>>>`
2. After primal discovery, `derive_coordination_key()` calls BearDog's
   `crypto.derive_public_key` with `purpose = "coordination"` and caches result
3. `graph.sign` and `graph.verify` use the cached key for verification
4. If BearDog is unavailable, the key stays `None` and signing degrades
   gracefully (unsigned graphs accepted in dev mode)

## Non-Goals for v3.29

- Embedding BearDog as a library (Phase 2)
- In-process BTSP handshake (Phase 3)
- Removing process-based Tower spawning
- Changing the two-tier crypto model

## Relationship to Existing Specs

- **NUCLEUS_TWO_TIER_CRYPTO_MODEL.md**: Defines the `coordination` purpose
  key that biomeOS uses for graph signing. This spec does not change the
  crypto model — it describes how biomeOS *consumes* the model.
- **DESKTOP_NUCLEUS_DEPLOYMENT.md**: Describes the 12-primal stack. This
  spec envisions biomeOS absorbing Tower into its process boundary, reducing
  the stack to 1 coordinator + N application primals.
- **NUCLEUS_ATOMIC_COMPOSITION.md**: Defines how graphs compose capabilities.
  Graph signing adds an integrity layer on top of composition validation.
