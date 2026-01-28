# NUCLEUS Deployment Specification

> **Complete deployment orchestration for Tower, Node, and Nest patterns.**

## Overview

NUCLEUS coordinates the deployment of biomeOS atomics with:
- **Genetic Lineage**: Cryptographic family trust via BearDog
- **Capability Discovery**: Runtime primal discovery via Neural API
- **Lifecycle Management**: Health monitoring, resurrection, apoptosis
- **Socket Discovery**: No hardcoded paths

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         NUCLEUS                                  │
│                  (Secure Discovery Protocol)                     │
├─────────────────────────────────────────────────────────────────┤
│ Layer 1: Physical Discovery       → Songbird (UDP multicast)    │
│ Layer 2: Identity Verification    → BearDog (Ed25519 sig)       │
│ Layer 3: Capability Verification  → Direct socket query         │
│ Layer 4: Trust Evaluation         → BearDog (lineage)           │
│ Layer 5: Registration & Tracking  → Neural API registry         │
└─────────────────────────────────────────────────────────────────┘
```

## Atomic Types

### Tower Atomic (Foundation)
```
Tower = BearDog + Songbird
- Security foundation (crypto, TLS, genetic lineage)
- Network foundation (HTTP, discovery, mesh)
- Generation 0 - genesis of trust
```

### Node Atomic (Application)
```
Node = Tower + Squirrel + [Application Primals]
- Inherits from Tower (security)
- AI orchestration via Squirrel
- Generation 1+
```

### Nest Atomic (Persistence)
```
Nest = Node + NestGate + Toadstool
- Full ecosystem deployment
- Persistent storage via NestGate
- Compute orchestration via Toadstool
- Generation 2+
```

## Deployment Flow

```
┌──────────────────────────────────────────────────────────────┐
│                    NUCLEUS DEPLOYMENT FLOW                    │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  1. GENESIS (Tower Atomic)                                   │
│     ├─ Germinate BearDog (crypto foundation)                │
│     ├─ Wait for socket (/tmp/beardog-{family}.sock)         │
│     ├─ Germinate Songbird (bonded to BearDog)               │
│     └─ Register capabilities with Neural API                 │
│                                                              │
│  2. EXPANSION (Node/Nest Atomic)                             │
│     ├─ Verify Tower Atomic health                           │
│     ├─ Germinate additional primals                         │
│     ├─ Register with lifecycle manager                      │
│     └─ Imprint ecosystem structure                          │
│                                                              │
│  3. OPERATION                                                │
│     ├─ Continuous health monitoring (10s interval)          │
│     ├─ Auto-resurrection on failure                         │
│     └─ Capability-based request routing                     │
│                                                              │
│  4. TERMINATION                                              │
│     ├─ Graceful apoptosis (dependency order)                │
│     ├─ State persistence (NestGate)                         │
│     └─ Clean socket removal                                 │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

## Deployment Graph

### Tower Atomic Bootstrap (`tower_atomic_bootstrap.toml`)

```toml
[graph]
id = "tower_atomic_bootstrap"
version = "2.0.0"
description = "Bootstrap Tower Atomic (BearDog + Songbird)"

[config]
max_parallelism = 2
rollback_on_failure = true
timeout_ms = 60000

# Phase 1: BearDog (Security Foundation)
[[nodes]]
id = "germinate_beardog"
capabilities = ["crypto", "tls_crypto", "genetic", "security"]

[nodes.primal]
by_capability = "security"

[nodes.operation]
name = "start"
[nodes.operation.params]
mode = "server"
family_id = "nat0"

[nodes.capabilities_provided]
"crypto.sign" = "crypto.sign_ed25519"
"crypto.verify" = "crypto.verify_ed25519"
# ... 39 total methods

# Phase 2: Songbird (Network Foundation)
[[nodes]]
id = "germinate_songbird"
depends_on = ["germinate_beardog"]
capabilities = ["http", "discovery", "secure_http"]

[nodes.primal]
by_capability = "discovery"

[nodes.operation]
name = "start"
[nodes.operation.params]
mode = "server"
family_id = "nat0"

[nodes.operation.environment]
SONGBIRD_SECURITY_PROVIDER = "/tmp/beardog-nat0.sock"
```

## Neural API Methods

### Deployment Methods

| Method | Description |
|--------|-------------|
| `graph.execute` | Execute a deployment graph |
| `graph.status` | Get execution status |
| `primal.germinate` | Spawn a new primal |
| `primal.terraria` | Create test environment |

### Lifecycle Methods

| Method | Description |
|--------|-------------|
| `lifecycle.status` | All primal states |
| `lifecycle.register` | Register for management |
| `lifecycle.resurrect` | Force resurrection |
| `lifecycle.apoptosis` | Graceful shutdown |

### Capability Methods

| Method | Description |
|--------|-------------|
| `capability.discover` | Find primal by capability |
| `capability.list` | List all capabilities |
| `capability.call` | Route request to provider |

## Socket Conventions

### Path Format
```
/tmp/{primal}-{family_id}.sock
Example: /tmp/beardog-nat0.sock
```

### XDG-Compliant (Preferred)
```
$XDG_RUNTIME_DIR/biomeos/{primal}-{family_id}.sock
Example: /run/user/1000/biomeos/beardog-nat0.sock
```

### Discovery Order
1. Environment hint (`BEARDOG_SOCKET`)
2. XDG runtime dir
3. Family-scoped /tmp
4. Neural API registry query

## Lifecycle States

```
GERMINATING → INCUBATING → ACTIVE ←→ DEGRADED → APOPTOSIS
      ↑                        ↓                    ↓
      └────── RESURRECTION ←───┴────────────────────┘
```

| State | Description | Action |
|-------|-------------|--------|
| Germinating | Spawning | Wait |
| Incubating | Waiting for socket | Monitor |
| Active | Healthy | Normal operation |
| Degraded | Unhealthy | Attempt resurrection |
| Apoptosis | Shutting down | Clean up |
| Dead | Terminated | Remove from registry |

## Health Checking

### Levels
1. **Socket Existence** (fast) - File exists
2. **Socket Type** (fast) - Is Unix socket
3. **RPC Ping** (deep) - JSON-RPC health method responds

### Configuration
```rust
HealthConfig {
    check_interval: Duration::from_secs(30),
    timeout: Duration::from_secs(5),
    failure_threshold: 3,
    health_method: "health",
}
```

## Resurrection

### Trigger
- 3+ consecutive health failures
- Socket disappears
- Process dies

### Process
1. Mark as DEGRADED
2. Wait backoff delay (exponential)
3. Kill old process
4. Remove old socket
5. Respawn from deployment graph
6. Mark as INCUBATING

### Configuration
```rust
ResurrectionConfig {
    enabled: true,
    max_attempts: 5,
    base_delay: Duration::from_secs(2),
    max_delay: Duration::from_secs(60),
}
```

## Dependency Graph

```
BearDog (gen 0)
    ↓
Songbird (gen 0, depends: BearDog)
    ↓
Neural API (gen 1, depends: Tower)
    ↓
┌───────────────────────────────────┐
│                                   │
Squirrel (gen 2)    Toadstool (gen 2)
    │                   │
    └───────┬───────────┘
            ↓
       NestGate (gen 2)
```

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `FAMILY_ID` | Yes | Family identifier |
| `NODE_ID` | No | Node identifier (default: hostname) |
| `NEURAL_API_SOCKET` | No | Neural API socket path |
| `BIOMEOS_MODE` | No | `bootstrap` or `coordinated` |

## CLI Commands

```bash
# Deploy Tower Atomic
neural-api --socket /tmp/neural-api.sock \
  --graph-dir ./graphs \
  --family-id nat0

# Query lifecycle status
echo '{"jsonrpc":"2.0","method":"lifecycle.status","params":{},"id":1}' \
  | nc -U /tmp/neural-api.sock

# Execute deployment graph
echo '{"jsonrpc":"2.0","method":"graph.execute","params":{"graph_id":"tower_atomic_bootstrap"},"id":1}' \
  | nc -U /tmp/neural-api.sock
```

## Files

| File | Purpose |
|------|---------|
| `graphs/tower_atomic_bootstrap.toml` | Tower deployment graph |
| `graphs/node_deploy.toml` | Node deployment graph |
| `graphs/nest_deploy.toml` | Nest deployment graph |
| `lifecycle_manager.rs` | Lifecycle state machine |
| `socket_discovery.rs` | Runtime socket discovery |
| `neural_api_server.rs` | Orchestration server |

## Deep Debt Principles

1. **No Hardcoding**: All paths via SocketDiscovery
2. **Capability-Based**: Discover by capability, not name
3. **Self-Knowledge**: Primals only know themselves
4. **Runtime Discovery**: No compile-time dependencies
5. **Modern Rust**: Async/await, no unsafe code
6. **Graceful Degradation**: Continue with degraded primals

---

*Version: 2.0.0*
*Created: January 28, 2026*
*Status: Production Ready*

