# Living Graph Protocol Escalation Specification

**Version**: 1.0.0  
**Date**: January 28, 2026  
**Status**: DRAFT  
**Authors**: biomeOS Team  

---

## 1. Overview

### 1.1 Purpose

This specification defines how Neural API manages **protocol escalation** between primals - transitioning from JSON-RPC (bootstrap/debug) to tarpc (production/performance) based on runtime conditions.

### 1.2 Design Principles

1. **JSON-RPC First**: Bootstrap, configuration, and debugging always use JSON-RPC for human readability
2. **tarpc for Performance**: Hot-paths escalate to tarpc for ~10x latency improvement
3. **Graceful Degradation**: Automatic fallback to JSON-RPC if tarpc fails
4. **Living Graph**: Runtime protocol state tracked per-connection, not per-primal

### 1.3 Ecosystem Context

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         PRIMAL ECOSYSTEM                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌──────────────┐     JSON-RPC (orchestration)                         │
│  │  Neural API  │◄─────────────────────────────────────────────────┐   │
│  │ (biomeOS)    │                                                   │   │
│  └──────┬───────┘                                                   │   │
│         │                                                           │   │
│         │ Bootstrap: JSON-RPC                                       │   │
│         │ Runtime: Manage escalation                                │   │
│         ▼                                                           │   │
│  ┌──────────────┐      ┌──────────────┐      ┌──────────────┐      │   │
│  │   BearDog    │      │   Songbird   │      │   Squirrel   │      │   │
│  │  (Identity)  │      │    (HTTP)    │      │     (AI)     │──────┘   │
│  └──────┬───────┘      └──────┬───────┘      └──────────────┘          │
│         │                     │                                         │
│         │  JSON-RPC → tarpc   │                                         │
│         │  (TLS hot-path)     │                                         │
│         └─────────════════════┘                                         │
│                  ▲                                                       │
│                  │ Production: tarpc/bincode (~10μs)                    │
│                  │ Fallback: JSON-RPC (~100μs)                          │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Protocol Modes

### 2.1 Mode Definitions

| Mode | Description | Use Case |
|------|-------------|----------|
| `JsonRpc` | JSON-RPC over Unix socket | Bootstrap, config, debugging |
| `Tarpc` | tarpc/bincode over Unix socket | Production hot-paths |
| `Hybrid` | JSON-RPC for control, tarpc for data | Mixed workloads |
| `Degraded` | Fell back from tarpc to JSON-RPC | After tarpc failure |

### 2.2 Mode Transitions

```
                    ┌──────────────┐
                    │   JsonRpc    │ ◄── Initial state
                    └──────┬───────┘
                           │
                           │ escalate()
                           │ (metrics threshold OR manual)
                           ▼
                    ┌──────────────┐
         ┌─────────│    Tarpc     │─────────┐
         │         └──────────────┘         │
         │                                   │
         │ fallback()                        │ healthy
         │ (tarpc failure)                   │
         ▼                                   │
  ┌──────────────┐                          │
  │   Degraded   │──────────────────────────┘
  └──────────────┘    recover()
                      (after stable period)
```

---

## 3. Architecture

### 3.1 Components

#### 3.1.1 Living Graph

Tracks runtime state of all primal connections.

```rust
pub struct LivingGraph {
    /// Static deployment definition (from TOML)
    deployment: DeploymentGraph,
    
    /// Runtime state: primal → protocol state
    protocol_state: RwLock<HashMap<String, PrimalProtocolState>>,
    
    /// Active connections between primals  
    connections: RwLock<HashMap<ConnectionId, ConnectionState>>,
    
    /// Metrics collector
    metrics: MetricsCollector,
}

pub struct PrimalProtocolState {
    pub primal_id: String,
    pub json_rpc_socket: PathBuf,
    pub tarpc_socket: Option<PathBuf>,
    pub current_mode: ProtocolMode,
    pub health: HealthStatus,
    pub capabilities: Vec<String>,
}

pub struct ConnectionState {
    pub id: ConnectionId,
    pub from: String,
    pub to: String,
    pub protocol: ProtocolMode,
    pub established_at: Instant,
    pub metrics: ConnectionMetrics,
}

pub struct ConnectionMetrics {
    pub request_count: u64,
    pub error_count: u64,
    pub total_latency_us: u64,
    pub avg_latency_us: f64,
    pub p99_latency_us: u64,
}
```

#### 3.1.2 Protocol Escalation Manager

Decides when and how to escalate/fallback.

```rust
pub struct ProtocolEscalationManager {
    graph: Arc<LivingGraph>,
    config: EscalationConfig,
}

pub struct EscalationConfig {
    /// Minimum requests before considering escalation
    pub min_requests: u64,                    // default: 100
    
    /// Latency threshold to trigger escalation (μs)
    pub latency_threshold_us: u64,            // default: 500
    
    /// Stable health duration before auto-escalate
    pub stable_health_duration: Duration,     // default: 30s
    
    /// tarpc failures before fallback
    pub tarpc_failure_threshold: u32,         // default: 3
    
    /// Time between auto-escalation checks
    pub check_interval: Duration,             // default: 10s
    
    /// Cooldown after failed escalation attempt
    pub escalation_cooldown: Duration,        // default: 60s
}
```

### 3.2 Data Flow

#### 3.2.1 Bootstrap Flow (JSON-RPC)

```
1. Neural API loads deployment graph (TOML)
2. Neural API spawns primals via JSON-RPC
3. Each primal registers via JSON-RPC:
   - Advertises JSON-RPC socket
   - Advertises tarpc socket (if available)
   - Lists capabilities
4. Neural API updates Living Graph
5. Primals communicate via JSON-RPC initially
```

#### 3.2.2 Escalation Flow

```
1. Protocol Manager monitors connection metrics
2. When threshold met (latency > 500μs, requests > 100):
   a. Query target primal: rpc.tarpc_endpoint
   b. Notify source primal: rpc.escalate_to
   c. Source connects tarpc socket
   d. Verify with test call
   e. Update Living Graph
3. Connection now uses tarpc
```

#### 3.2.3 Fallback Flow

```
1. tarpc call fails
2. Increment failure counter
3. If failures > threshold:
   a. Notify source primal: rpc.fallback_to_json_rpc
   b. Update Living Graph (mode = Degraded)
   c. Log warning
4. Connection uses JSON-RPC
5. After stable period, attempt re-escalation
```

---

## 4. API Specification

### 4.1 Neural API Methods (JSON-RPC)

#### 4.1.1 Protocol Status

```json
// Request
{
  "jsonrpc": "2.0",
  "method": "protocol.status",
  "params": {},
  "id": 1
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "connections": [
      {
        "from": "songbird",
        "to": "beardog",
        "protocol": "Tarpc",
        "requests": 15420,
        "avg_latency_us": 12.5,
        "p99_latency_us": 45
      },
      {
        "from": "squirrel",
        "to": "songbird",
        "protocol": "JsonRpc",
        "requests": 234,
        "avg_latency_us": 95.2,
        "p99_latency_us": 180
      }
    ],
    "summary": {
      "json_rpc": 2,
      "tarpc": 1,
      "degraded": 0
    }
  },
  "id": 1
}
```

#### 4.1.2 Manual Escalation

```json
// Request
{
  "jsonrpc": "2.0",
  "method": "protocol.escalate",
  "params": {
    "from": "songbird",
    "to": "beardog"
  },
  "id": 2
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "status": "escalated",
    "from": "songbird",
    "to": "beardog",
    "previous_mode": "JsonRpc",
    "current_mode": "Tarpc",
    "tarpc_socket": "/run/user/1000/biomeos/beardog-nat0-tarpc.sock"
  },
  "id": 2
}
```

#### 4.1.3 Manual Fallback

```json
// Request
{
  "jsonrpc": "2.0",
  "method": "protocol.fallback",
  "params": {
    "from": "songbird",
    "to": "beardog",
    "reason": "manual"
  },
  "id": 3
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "status": "degraded",
    "from": "songbird",
    "to": "beardog",
    "previous_mode": "Tarpc",
    "current_mode": "Degraded"
  },
  "id": 3
}
```

#### 4.1.4 Connection Metrics

```json
// Request
{
  "jsonrpc": "2.0",
  "method": "protocol.metrics",
  "params": {
    "from": "songbird",
    "to": "beardog"
  },
  "id": 4
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "connection": {
      "from": "songbird",
      "to": "beardog",
      "protocol": "Tarpc"
    },
    "metrics": {
      "request_count": 15420,
      "error_count": 0,
      "total_latency_us": 192750,
      "avg_latency_us": 12.5,
      "p50_latency_us": 10,
      "p95_latency_us": 25,
      "p99_latency_us": 45,
      "max_latency_us": 120
    },
    "history": {
      "escalated_at": "2026-01-28T21:30:00Z",
      "escalation_count": 1,
      "fallback_count": 0
    }
  },
  "id": 4
}
```

#### 4.1.5 Living Graph Snapshot

```json
// Request
{
  "jsonrpc": "2.0",
  "method": "graph.protocol_map",
  "params": {},
  "id": 5
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "nodes": [
      {
        "id": "beardog",
        "json_rpc_socket": "/run/user/1000/biomeos/beardog-nat0.sock",
        "tarpc_socket": "/run/user/1000/biomeos/beardog-nat0-tarpc.sock",
        "health": "healthy",
        "capabilities": ["crypto", "tls", "identity"]
      },
      {
        "id": "songbird",
        "json_rpc_socket": "/run/user/1000/biomeos/songbird-nat0.sock",
        "tarpc_socket": "/run/user/1000/biomeos/songbird-nat0-tarpc.sock",
        "health": "healthy",
        "capabilities": ["http", "discovery", "federation"]
      }
    ],
    "edges": [
      {
        "from": "songbird",
        "to": "beardog",
        "protocol": "Tarpc",
        "latency_us": 12.5,
        "requests": 15420
      }
    ]
  },
  "id": 5
}
```

### 4.2 Primal Methods (JSON-RPC)

Primals must implement these methods to support protocol escalation:

#### 4.2.1 Advertise tarpc Endpoint

```json
// Request (from Neural API)
{
  "jsonrpc": "2.0",
  "method": "rpc.tarpc_endpoint",
  "params": {},
  "id": 1
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "available": true,
    "socket": "/run/user/1000/biomeos/beardog-nat0-tarpc.sock",
    "services": ["CryptoService", "IdentityService"]
  },
  "id": 1
}
```

#### 4.2.2 Escalate Connection (Source Primal)

```json
// Request (from Neural API to source primal)
{
  "jsonrpc": "2.0",
  "method": "rpc.escalate_to",
  "params": {
    "target": "beardog",
    "tarpc_socket": "/run/user/1000/biomeos/beardog-nat0-tarpc.sock",
    "services": ["CryptoService"]
  },
  "id": 2
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "status": "connected",
    "target": "beardog",
    "protocol": "tarpc"
  },
  "id": 2
}
```

#### 4.2.3 Fallback to JSON-RPC (Source Primal)

```json
// Request (from Neural API to source primal)
{
  "jsonrpc": "2.0",
  "method": "rpc.fallback_to_json_rpc",
  "params": {
    "target": "beardog",
    "reason": "tarpc_failure"
  },
  "id": 3
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "status": "fallback_complete",
    "target": "beardog",
    "protocol": "json_rpc"
  },
  "id": 3
}
```

---

## 5. Deployment Graph Extension

### 5.1 Protocol Configuration (TOML)

```toml
# graphs/tower_atomic_bootstrap.toml

# Global escalation settings
[protocol]
auto_escalate = true
check_interval_secs = 10
min_requests = 100
latency_threshold_us = 500
tarpc_failure_threshold = 3
stable_health_duration_secs = 30

# Per-primal protocol settings
[[nodes]]
id = "germinate_beardog"
# ... existing config ...

[nodes.protocol]
json_rpc_socket = "${XDG_RUNTIME_DIR}/biomeos/beardog-${FAMILY_ID}.sock"
tarpc_socket = "${XDG_RUNTIME_DIR}/biomeos/beardog-${FAMILY_ID}-tarpc.sock"
tarpc_services = ["CryptoService", "IdentityService", "TlsService"]

[[nodes]]
id = "germinate_songbird"
# ... existing config ...

[nodes.protocol]
json_rpc_socket = "${XDG_RUNTIME_DIR}/biomeos/songbird-${FAMILY_ID}.sock"
tarpc_socket = "${XDG_RUNTIME_DIR}/biomeos/songbird-${FAMILY_ID}-tarpc.sock"
tarpc_services = ["HttpService", "DiscoveryService"]

# Connection-specific settings
[[edges]]
from = "songbird"
to = "beardog"
auto_escalate = true
priority = "critical"  # Escalate first (TLS hot-path)
latency_threshold_us = 100  # Lower threshold for critical path

[[edges]]
from = "squirrel"
to = "songbird"
auto_escalate = true
priority = "normal"
```

---

## 6. Implementation Plan

### Phase 1: Living Graph Infrastructure (Week 1)

- [ ] Create `LivingGraph` struct with protocol state tracking
- [ ] Add `ConnectionState` with metrics collection
- [ ] Implement `protocol.status` JSON-RPC method
- [ ] Implement `graph.protocol_map` JSON-RPC method
- [ ] Add protocol config parsing to deployment graph loader

### Phase 2: Protocol Escalation Manager (Week 2)

- [ ] Create `ProtocolEscalationManager` struct
- [ ] Implement escalation decision logic
- [ ] Implement `protocol.escalate` JSON-RPC method
- [ ] Implement `protocol.fallback` JSON-RPC method
- [ ] Add background auto-escalation loop

### Phase 3: Primal Integration (Week 3)

- [ ] Define `rpc.tarpc_endpoint` method spec for primals
- [ ] Define `rpc.escalate_to` method spec for primals
- [ ] Define `rpc.fallback_to_json_rpc` method spec for primals
- [ ] Create handoff documents for primal teams

### Phase 4: Testing & Validation (Week 4)

- [ ] Unit tests for Living Graph
- [ ] Unit tests for Protocol Escalation Manager
- [ ] Integration tests with BearDog + Songbird
- [ ] Chaos tests (tarpc failure → fallback → recovery)
- [ ] Performance benchmarks (JSON-RPC vs tarpc latency)

---

## 7. Security Considerations

### 7.1 Socket Permissions

- tarpc sockets must have same permission model as JSON-RPC sockets
- Family-scoped: Only same-family primals can connect
- XDG_RUNTIME_DIR ensures user-only access

### 7.2 Escalation Authorization

- Only Neural API can trigger escalation
- Primals cannot self-escalate without Neural API coordination
- All escalation events are logged

### 7.3 Fallback Safety

- Automatic fallback ensures availability over performance
- Degraded mode is fully functional, just slower
- No data loss during protocol transitions

---

## 8. Metrics & Observability

### 8.1 Metrics to Collect

| Metric | Type | Description |
|--------|------|-------------|
| `protocol_escalations_total` | Counter | Total escalation attempts |
| `protocol_escalations_success` | Counter | Successful escalations |
| `protocol_fallbacks_total` | Counter | Total fallbacks |
| `connection_latency_us` | Histogram | Request latency by connection |
| `connection_requests_total` | Counter | Total requests by connection |
| `connection_errors_total` | Counter | Errors by connection |

### 8.2 Log Events

```
INFO  🚀 Escalating songbird → beardog (JSON-RPC → tarpc)
INFO  ✅ Escalation complete: songbird ═tarpc═► beardog (12μs avg)
WARN  ⚠️ tarpc failure on songbird → beardog (attempt 2/3)
WARN  ⚠️ Falling back: songbird → beardog (tarpc → JSON-RPC)
INFO  🔄 Recovery attempt: songbird → beardog (JSON-RPC → tarpc)
```

---

## 9. Future Enhancements

### 9.1 Hybrid Mode

Allow connections to use both protocols simultaneously:
- JSON-RPC for control plane (config, health, lifecycle)
- tarpc for data plane (crypto operations, HTTP requests)

### 9.2 Protocol Negotiation

Primals negotiate best protocol at connection time:
- Check tarpc compatibility
- Compare serialization formats (bincode vs JSON)
- Select based on workload type

### 9.3 Cross-Node tarpc

Extend tarpc support to cross-node connections:
- tarpc over TCP (not just Unix sockets)
- TLS-encrypted tarpc for network connections
- Songbird-mediated tarpc tunneling

---

## 10. References

- [tarpc Documentation](https://docs.rs/tarpc)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- [biomeOS Neural API Spec](../NUCLEUS_DEPLOYMENT_SPEC.md)
- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)

---

*Last Updated: January 28, 2026*

