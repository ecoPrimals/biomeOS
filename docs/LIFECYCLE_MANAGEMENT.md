# Primal Lifecycle Management

> **Deep Debt Solution**: Robust lifecycle management for NUCLEUS (Tower, Node, Nest) deployments.

## Overview

The Lifecycle Manager provides complete primal lifecycle control:
- **Health Monitoring**: Continuous health checks with JSON-RPC pings
- **Crash Detection**: Socket timeout and process death detection
- **Auto-Resurrection**: Automatic restart from deployment graphs
- **Graceful Apoptosis**: Coordinated shutdown respecting dependencies

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                   PRIMAL LIFECYCLE STATES                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  GERMINATING → INCUBATING → ACTIVE ←→ DEGRADED → APOPTOSIS     │
│       ↑                        ↓                    ↓          │
│       └────── RESURRECTION ←───┴────────────────────┘          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### State Definitions

| State | Description |
|-------|-------------|
| **Germinating** | Birth - primal being spawned with minimal knowledge |
| **Incubating** | Startup monitoring - waiting for socket and health |
| **Active** | Running and healthy |
| **Degraded** | Running but unhealthy - will attempt resurrection |
| **Apoptosis** | Programmed graceful shutdown |
| **Dead** | Process terminated |

## Neural API Integration

### JSON-RPC Methods

#### `lifecycle.status`
Get status of all managed primals.

```json
// Request
{"jsonrpc":"2.0","method":"lifecycle.status","params":{},"id":1}

// Response
{
  "result": {
    "primals": [
      {"name": "beardog", "state": "active", "details": {...}},
      {"name": "songbird", "state": "active", "details": {...}}
    ],
    "count": 2,
    "healthy": 2
  }
}
```

#### `lifecycle.get`
Get detailed info for a specific primal.

```json
// Request
{"jsonrpc":"2.0","method":"lifecycle.get","params":{"name":"beardog"},"id":1}

// Response
{
  "result": {
    "name": "beardog",
    "family_id": "nat0",
    "socket_path": "/tmp/beardog-nat0.sock",
    "pid": 12345,
    "state": "active",
    "metrics": {
      "total_uptime_secs": 3600,
      "resurrection_count": 0,
      "health_failures": 0,
      "last_health_latency_ms": 5
    }
  }
}
```

#### `lifecycle.register`
Register a primal for lifecycle management.

```json
// Request
{
  "jsonrpc": "2.0",
  "method": "lifecycle.register",
  "params": {
    "name": "beardog",
    "socket_path": "/tmp/beardog-nat0.sock",
    "pid": 12345,
    "deployment_node": {...}  // Optional: for resurrection
  },
  "id": 1
}

// Response
{
  "result": {
    "registered": "beardog",
    "state": "incubating"
  }
}
```

#### `lifecycle.resurrect`
Force resurrection of a degraded/dead primal.

```json
// Request
{"jsonrpc":"2.0","method":"lifecycle.resurrect","params":{"name":"beardog"},"id":1}
```

#### `lifecycle.apoptosis`
Initiate graceful shutdown of a primal.

```json
// Request
{
  "jsonrpc": "2.0",
  "method": "lifecycle.apoptosis",
  "params": {
    "name": "squirrel",
    "reason": "resource_pressure"  // user_request, ecosystem_health, resource_pressure, system_shutdown
  },
  "id": 1
}
```

#### `lifecycle.shutdown_all`
Initiate system-wide shutdown (respects dependency order).

```json
// Request
{"jsonrpc":"2.0","method":"lifecycle.shutdown_all","params":{},"id":1}
```

## Configuration

### Health Check Configuration

```rust
HealthConfig {
    check_interval: Duration::from_secs(30),  // How often to check
    timeout: Duration::from_secs(5),          // Timeout for health response
    failure_threshold: 3,                      // Failures before degraded
    health_method: "health".to_string(),      // JSON-RPC method to call
}
```

### Resurrection Configuration

```rust
ResurrectionConfig {
    enabled: true,                            // Auto-resurrection enabled
    max_attempts: 5,                          // Max resurrection attempts
    base_delay: Duration::from_secs(2),       // Base backoff delay
    max_delay: Duration::from_secs(60),       // Max backoff delay
}
```

## Apoptosis Reasons

| Reason | Description |
|--------|-------------|
| `UserRequest` | User explicitly requested shutdown |
| `EcosystemHealth` | Ecosystem health requires shutdown |
| `ResourcePressure` | Memory/CPU pressure |
| `DependencyDeath` | A dependency died |
| `ResurrectionExhausted` | Too many resurrection failures |
| `SystemShutdown` | System-wide shutdown |

## Dependency-Aware Shutdown

When a primal is shut down, the lifecycle manager:

1. Identifies all dependents (primals that depend on this one)
2. Cascades apoptosis to dependents first (reverse dependency order)
3. Shuts down the original primal

```
Tower Atomic shutdown example:
  1. Squirrel (depends on Tower) → apoptosis
  2. Toadstool (depends on Tower) → apoptosis
  3. Songbird → apoptosis
  4. BearDog → apoptosis
```

## Usage Example

```rust
use biomeos_atomic_deploy::{LifecycleManager, ApoptosisReason};

// Create manager
let manager = LifecycleManager::new("nat0");

// Start monitoring
manager.start_monitoring().await?;

// Register a primal
manager.register_primal(
    "beardog",
    PathBuf::from("/tmp/beardog-nat0.sock"),
    Some(12345),
    Some(deployment_node),  // For resurrection
).await?;

// Initiate graceful shutdown
manager.apoptosis("squirrel", ApoptosisReason::UserRequest).await?;

// System-wide shutdown
manager.shutdown_all().await?;
```

## Deep Debt Principles

1. **No Hardcoding**: Socket paths discovered via `SocketDiscovery`
2. **Capability-Based**: Primals discovered by capability, not name
3. **Modern Rust**: Async/await, no unsafe code
4. **Dependency Awareness**: Respects primal dependencies
5. **Graceful Degradation**: Continues operating with degraded primals

## Files

- `crates/biomeos-atomic-deploy/src/lifecycle_manager.rs` - Core state machine
- `crates/biomeos-atomic-deploy/src/handlers/lifecycle.rs` - Neural API handler
- `crates/biomeos-atomic-deploy/src/health_check.rs` - Health checking

---

*Created: January 28, 2026*
*Status: Production Ready*

