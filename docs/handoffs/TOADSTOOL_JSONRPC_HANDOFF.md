# Toadstool JSON-RPC Handler Evolution Handoff

**Date**: January 29, 2026  
**From**: biomeOS Team  
**To**: Toadstool Team  
**Priority**: Medium  
**Status**: JSON-RPC socket not responding

---

## Issue Summary

Toadstool's JSON-RPC socket (`*.jsonrpc.sock`) is not responding to requests. Connections are accepted but responses are never sent, resulting in timeouts and "Broken pipe" errors.

### Error Observation

```
2026-01-29T14:05:40.160167Z ERROR toadstool_server::manual_jsonrpc: Connection error: Broken pipe (os error 32)
```

### Test

```python
import socket, json
sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
sock.settimeout(5.0)
sock.connect('/run/user/1000/toadstool-default.jsonrpc.sock')
request = json.dumps({"jsonrpc": "2.0", "method": "toadstool.health", "params": {}, "id": 1}) + "\n"
sock.sendall(request.encode())
response = sock.recv(4096)  # TIMES OUT - no response received
```

### Expected Behavior

```json
{"jsonrpc":"2.0","result":{"healthy":true,"service":"toadstool","version":"0.1.0"},"id":1}
```

---

## Root Cause Analysis

Likely issues in `crates/server/src/manual_jsonrpc.rs`:

1. **Response not being flushed** - The `write_all` may not be followed by a flush
2. **Connection closed before write completes** - Async timing issue
3. **Handler not awaiting response** - The response generation may be running but not completing before connection close

### Suggested Investigation

Check `handle_jsonrpc_request` and the socket write path:

```rust
// In manual_jsonrpc.rs, ensure:
// 1. Response is serialized
// 2. Response is written to socket
// 3. Socket is flushed before allowing close
// 4. No early return before write
```

---

## Working Paths

### tarpc Socket (Working)

Toadstool's tarpc socket is working for binary RPC:
- Socket: `/run/user/1000/toadstool-default.sock`
- Protocol: tarpc (binary RPC)

This is the PRIMARY protocol and should work for inter-primal communication.

### Socket Locations

Toadstool creates two sockets:
1. **tarpc (PRIMARY)**: `{XDG_RUNTIME_DIR}/toadstool-{family}.sock`
2. **JSON-RPC (FALLBACK)**: `{XDG_RUNTIME_DIR}/toadstool-{family}.jsonrpc.sock`

---

## biomeOS Integration Plan

Once JSON-RPC is working, Toadstool can be integrated via:

### Graph Entry (node_atomic_compute.toml)

```toml
[[nodes]]
id = "germinate_toadstool"
depends_on = ["germinate_beardog", "germinate_songbird"]
output = "toadstool_genesis"
capabilities = ["compute", "workload", "orchestration", "ai_local"]

[nodes.capabilities_provided]
"compute.health" = "toadstool.health"
"compute.version" = "toadstool.version"
"compute.capabilities" = "toadstool.query_capabilities"
"compute.estimate" = "resources.estimate"
"compute.validate" = "resources.validate_availability"
"compute.optimize" = "resources.suggest_optimizations"
```

### Environment Variables

```bash
TOADSTOOL_FAMILY_ID=nat0
TOADSTOOL_NODE_ID=node-alpha
TOADSTOOL_SOCKET=/run/user/1000/biomeos/toadstool-node-alpha.jsonrpc.sock
BEARDOG_SOCKET=/run/user/1000/biomeos/beardog-node-alpha.sock
SONGBIRD_SOCKET=/run/user/1000/biomeos/songbird-node-alpha.sock
```

---

## Expected JSON-RPC Methods

Based on `manual_jsonrpc.rs`:

| Method | Description |
|--------|-------------|
| `toadstool.health` | Health check |
| `toadstool.version` | Version info |
| `toadstool.query_capabilities` | List compute capabilities |
| `resources.estimate` | Estimate resources for workload |
| `resources.validate_availability` | Check if resources available |
| `resources.suggest_optimizations` | Suggest workload optimizations |

---

## Testing After Fix

```bash
# Test health
echo '{"jsonrpc":"2.0","method":"toadstool.health","params":{},"id":1}' | \
  nc -U /run/user/1000/toadstool-default.jsonrpc.sock

# Expected:
# {"jsonrpc":"2.0","result":{"healthy":true,"service":"toadstool","version":"0.1.0"},"id":1}
```

---

## Impact

- **Blocked**: Node Atomic Compute deployment via Neural API
- **Working**: tarpc protocol (use for inter-primal hot paths)
- **Workaround**: Use tarpc socket directly for now

---

## Files to Investigate

1. `crates/server/src/manual_jsonrpc.rs` - Main JSON-RPC handler
2. `crates/server/src/unibin.rs` - Server startup and socket binding

---

**Generated**: 2026-01-29  
**biomeOS Version**: Protocol Escalation Phase 1  
**Toadstool Version**: 0.1.0 (commit 0089fc7a)

