# Mesh IPC Methods Specification

**Created**: February 6, 2026  
**Status**: SPECIFICATION - Ready for Implementation  
**Related**: `wateringHole/handoffs/SOVEREIGN_BEACON_MESH_HANDOFF_FEB06_2026.md`

---

## Overview

This specification defines the JSON-RPC methods for the distributed beacon mesh network. These methods enable biomeOS and other primals to:

1. Query mesh network status
2. Find paths to peers across NAT boundaries
3. Announce as relay nodes
4. Manage distributed relay infrastructure

---

## Methods

### `mesh.status`

Returns current mesh network status.

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "mesh.status",
  "params": {},
  "id": 1
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "node_id": "tower-abc123",
    "onion_address": "xyz...abc.onion",
    "reachable_peers": 5,
    "relay_enabled": true,
    "nat_type": "symmetric",
    "public_address": "203.0.113.50:45678",
    "local_address": "192.0.2.100:34567",
    "uptime_seconds": 3600,
    "paths": {
      "direct": 2,
      "family_relay": 2,
      "onion": 1
    }
  },
  "id": 1
}
```

**Fields**:
| Field | Type | Description |
|-------|------|-------------|
| `node_id` | string | Our unique node identifier |
| `onion_address` | string? | Our .onion address (if onion feature enabled) |
| `reachable_peers` | number | Number of peers we can reach |
| `relay_enabled` | bool | Whether we're acting as a relay |
| `nat_type` | string | Detected NAT type (full_cone, address_restricted, port_restricted, symmetric, unknown) |
| `public_address` | string? | STUN-discovered public address |
| `local_address` | string | Local bind address |
| `uptime_seconds` | number | How long mesh has been running |
| `paths` | object | Breakdown of path types to peers |

---

### `mesh.find_path`

Find the best path to reach a peer.

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "mesh.find_path",
  "params": {
    "target_node_id": "pixel-xyz789",
    "prefer_direct": true
  },
  "id": 2
}
```

**Response** (success):
```json
{
  "jsonrpc": "2.0",
  "result": {
    "found": true,
    "target_node_id": "pixel-xyz789",
    "path_type": "family_relay",
    "path": [
      {
        "node_id": "tower-abc123",
        "type": "local"
      },
      {
        "node_id": "nest-def456",
        "type": "family_relay",
        "address": "nest-def456.onion:3490"
      },
      {
        "node_id": "pixel-xyz789",
        "type": "target"
      }
    ],
    "estimated_latency_ms": 150,
    "hops": 2
  },
  "id": 2
}
```

**Response** (not found):
```json
{
  "jsonrpc": "2.0",
  "result": {
    "found": false,
    "target_node_id": "unknown-peer",
    "reason": "peer_not_discovered"
  },
  "id": 2
}
```

**Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `target_node_id` | string | Yes | Node ID to find path to |
| `prefer_direct` | bool | No | Prefer direct connection over relay (default: true) |

**Path Types** (priority order):
1. `local` - Same LAN, direct local address
2. `direct` - Hole punch succeeded, direct public address
3. `family_relay` - Route through a family member's relay
4. `onion` - Route via .onion service (highest latency)

---

### `mesh.announce`

Announce ourselves as available for relay duties.

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "mesh.announce",
  "params": {
    "as_relay": true,
    "capabilities": ["relay", "stun", "discovery"],
    "max_relay_sessions": 10,
    "bandwidth_limit_kbps": 1000
  },
  "id": 3
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "announced": true,
    "relay_address": "tower-abc123.family.relay:3490",
    "onion_address": "xyz...abc.onion:3490",
    "ttl_seconds": 300
  },
  "id": 3
}
```

**Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `as_relay` | bool | Yes | Whether to announce as relay |
| `capabilities` | string[] | No | Additional capabilities to advertise |
| `max_relay_sessions` | number | No | Max concurrent relay sessions (default: 10) |
| `bandwidth_limit_kbps` | number | No | Bandwidth limit in kbps (default: unlimited) |

---

### `mesh.peers`

List known peers in the mesh.

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "mesh.peers",
  "params": {
    "include_offline": false
  },
  "id": 4
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "peers": [
      {
        "node_id": "pixel-xyz789",
        "public_address": "198.51.100.25:54321",
        "local_address": "192.168.2.50:34567",
        "nat_type": "port_restricted",
        "path_type": "direct",
        "last_seen_ms": 1500,
        "is_relay": false,
        "capabilities": ["discovery"]
      },
      {
        "node_id": "nest-def456",
        "public_address": null,
        "onion_address": "abc...xyz.onion",
        "nat_type": "symmetric",
        "path_type": "onion",
        "last_seen_ms": 5000,
        "is_relay": true,
        "capabilities": ["relay", "stun", "discovery"]
      }
    ],
    "total": 2,
    "online": 2,
    "relays": 1
  },
  "id": 4
}
```

---

### `mesh.health_check`

Check health of connections to specific peers or all peers.

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "mesh.health_check",
  "params": {
    "target_node_ids": ["pixel-xyz789", "nest-def456"],
    "timeout_ms": 5000
  },
  "id": 5
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "results": [
      {
        "node_id": "pixel-xyz789",
        "healthy": true,
        "latency_ms": 45,
        "path_type": "direct"
      },
      {
        "node_id": "nest-def456",
        "healthy": true,
        "latency_ms": 350,
        "path_type": "onion"
      }
    ],
    "all_healthy": true
  },
  "id": 5
}
```

---

### `punch.request`

Initiate hole punch attempt to a peer.

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "punch.request",
  "params": {
    "target_node_id": "pixel-xyz789",
    "timeout_seconds": 10
  },
  "id": 6
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "success": true,
    "target_node_id": "pixel-xyz789",
    "connected_address": "198.51.100.25:54321",
    "latency_ms": 35,
    "attempts": 5
  },
  "id": 6
}
```

**Response** (failure - will use relay):
```json
{
  "jsonrpc": "2.0",
  "result": {
    "success": false,
    "target_node_id": "pixel-xyz789",
    "attempts": 20,
    "reason": "symmetric_nat_both_sides",
    "fallback": "family_relay"
  },
  "id": 6
}
```

---

### `punch.status`

Check status of ongoing or recent punch attempts.

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "punch.status",
  "params": {
    "target_node_id": "pixel-xyz789"
  },
  "id": 7
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "target_node_id": "pixel-xyz789",
    "status": "in_progress",
    "attempts": 8,
    "max_attempts": 20,
    "elapsed_ms": 2500
  },
  "id": 7
}
```

---

## Capability Registration

These methods should be registered via the Neural API capability system:

```toml
# In deployment graph (e.g., sovereign_onion_genome.toml)
[nodes.capabilities_provided]
"mesh.status" = "mesh.status"
"mesh.find_path" = "mesh.find_path"
"mesh.announce" = "mesh.announce"
"mesh.peers" = "mesh.list_peers"
"mesh.health_check" = "mesh.health_check"
"punch.request" = "punch.request"
"punch.status" = "punch.status"
```

---

## Error Codes

| Code | Message | Description |
|------|---------|-------------|
| -32001 | Peer not found | Target peer not in mesh |
| -32002 | Punch timeout | Hole punch exceeded timeout |
| -32003 | No relay available | No family relays reachable |
| -32004 | Mesh not ready | Mesh still initializing |
| -32005 | Onion not enabled | Requires `--features onion` |

---

## Implementation Notes

### For biomeOS Team

1. **Add handler stubs** in `biomeos-atomic-deploy/src/capability_handlers.rs`
2. **Wire to Neural API** via capability translation
3. **Add to deployment graph** `sovereign_onion_genome.toml`

### For Songbird Team

1. **Implement in `songbird-universal-ipc`** or `songbird-mesh`
2. **Delegate path finding** to `songbird-onion-relay::BeaconMesh`
3. **Delegate hole punch** to `songbird-onion-relay::HolePunchCoordinator`

### Thread Safety

All methods must be:
- **Async**: Use `async fn` handlers
- **Thread-safe**: `Arc<RwLock<...>>` for shared state
- **Timeout-aware**: Honor timeout parameters

---

## Testing

### Unit Tests

```rust
#[tokio::test]
async fn test_mesh_status() {
    let mesh = BeaconMesh::new("test-node");
    let status = mesh.status().await;
    assert_eq!(status.node_id, "test-node");
}
```

### Integration Tests

```bash
# Start mesh node
cargo run -- --features mesh

# Query status
echo '{"jsonrpc":"2.0","method":"mesh.status","id":1}' | nc -U /tmp/songbird.sock

# Find path
echo '{"jsonrpc":"2.0","method":"mesh.find_path","params":{"target_node_id":"pixel"},"id":2}' | nc -U /tmp/songbird.sock
```

---

*Specification complete. Ready for implementation.*
