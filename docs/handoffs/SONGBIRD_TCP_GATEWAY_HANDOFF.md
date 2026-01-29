# Songbird TCP Gateway / HTTP Server Handoff

**Date**: January 29, 2026  
**Status**: 🔴 TCP Gateway Not Binding  
**biomeOS Integration**: ⚠️ UDP Discovery works, TCP/HTTP Federation blocked

---

## Current Behavior

### What Works ✅
- **UDP Discovery**: Port 2300 listening, beacons broadcasting
- **Unix Socket IPC**: JSON-RPC via `/run/user/1000/biomeos/songbird-*.sock`
- **HTTP Client**: External HTTPS requests work via `http.request` method
- **TLS 1.3**: Outbound TLS handshakes complete (~375ms latency)

### What Doesn't Work ❌
- **TCP Port Binding**: `--port 8081` specified but port 8081 not bound
- **HTTP Server**: No inbound HTTP/HTTPS connections accepted
- **Federation Gateway**: No TCP listener for peer-to-peer connections

---

## Investigation Findings

### 1. No TCP Listener
```bash
$ ss -tlnp | grep songbird
# (empty output)

$ ss -ulnp | grep songbird
UNCONN 0 0 0.0.0.0:2300 0.0.0.0:* users:(("songbird",pid=3398607,fd=12))
```

### 2. Startup Errors During HTTP Server Init
During startup, Songbird attempts TLS handshakes that fail with HTTP responses:

```
ERROR songbird_http_client::tls: ❌ Received HTTP response instead of TLS!
   Content type 0x48 = ASCII 'H'
   First 50 bytes: "HTTP/1.1 400 Bad Request..."
   This usually means:
     1. Connected to port 80 instead of 443
     2. Server redirected to HTTP
     3. Server doesn't support TLS
```

This suggests Songbird is trying to reach an external service (ACME or validation endpoint?) during startup and failing.

### 3. Environment Variables Tried

| Variable | Value | Result |
|----------|-------|--------|
| `--port 8081` | CLI argument | No TCP binding |
| `SONGBIRD_NETWORK_BIND_HOST` | `0.0.0.0` | No effect |
| `SONGBIRD_NETWORK_BASE_PORT` | `8081` | No effect |

---

## Expected Behavior

Songbird should bind TCP port for:
1. **HTTP/HTTPS API**: Accept inbound HTTP requests from peers
2. **Federation Gateway**: Accept peer connections for federation protocol
3. **tarpc Server**: High-performance RPC for primal communication

Based on code review (`crates/songbird-orchestrator/src/app/http_server.rs`):
```rust
pub async fn start_http_server(...) -> Result<u16> {
    let (listener, actual_addr) = bind_with_fallback(&bind_addr).await?;
    // ... starts axum server
}
```

The HTTP server SHOULD be starting via `crate::app::http_server::start_http_server()` but isn't reaching that code path.

---

## Root Cause Hypothesis

The startup sequence in `SongbirdOrchestrator::start()` may be failing early due to:
1. **Missing BearDog connectivity** - crypto requests fail
2. **Failed TLS validation** - startup checks require external connectivity
3. **Configuration validation** - port:0 or missing config blocks HTTP server start

---

## biomeOS Integration Requirements

For cross-spore/cross-tower federation, Songbird needs:

### 1. TCP Gateway for Federation
```
Spore Alpha (192.168.1.144) ←→ Spore Gamma (192.168.1.134)
          ↑                              ↑
       TCP/TLS                        TCP/TLS
          ↓                              ↓
   Songbird:8081 ←← Federation ←→ Songbird:8082
```

### 2. JSON-RPC Methods for Gateway Control
| Method | Description |
|--------|-------------|
| `gateway.status` | Returns gateway port, peers connected |
| `tcp.connect` | Initiate outbound TCP to peer |
| `federation.connect` | Start federation with peer |
| `peer.accept` | Accept incoming peer connection |

### 3. TLS Certificate for Server Mode
Songbird client TLS works, but server mode requires certificate:
- Auto-generated self-signed cert (for local/LAN)
- Let's Encrypt integration (for public internet)
- BearDog-delegated cert signing

---

## Validation Script

```bash
#!/bin/bash
SONGBIRD_PORT=8081

# Test 1: TCP listener exists
if ss -tlnp | grep -q ":$SONGBIRD_PORT"; then
    echo "✅ TCP listener on $SONGBIRD_PORT"
else
    echo "❌ No TCP listener on $SONGBIRD_PORT"
fi

# Test 2: HTTP health endpoint responds
if curl -s --connect-timeout 3 http://localhost:$SONGBIRD_PORT/health; then
    echo "✅ HTTP server responding"
else
    echo "❌ HTTP server not responding"
fi

# Test 3: Federation endpoint exists
if curl -s --connect-timeout 3 http://localhost:$SONGBIRD_PORT/api/federation/status; then
    echo "✅ Federation API available"
else
    echo "❌ Federation API not available"
fi
```

---

## Configuration Reference

From Songbird's `CanonicalSongbirdConfig`:

```toml
[network]
bind_host = "0.0.0.0"  # Listen on all interfaces
base_port = 8081       # HTTP/HTTPS port

[discovery]
mode = "Anonymous"     # Discovery mode (enables beacons)

[federation]
cluster_name = "biomeOS"
trust_escalation_policy = "Progressive"
```

---

## Priority

**HIGH** - This blocks:
- Cross-spore communication (local and LAN)
- Peer-to-peer federation
- Tower mesh networking
- Protocol escalation to tarpc over TCP

---

## Handoff Actions

1. **Debug startup sequence**: Why does HTTP server binding fail?
2. **Add graceful fallback**: If startup validation fails, still bind HTTP
3. **Expose gateway methods**: `gateway.status`, `tcp.connect`, `peer.connect`
4. **Document dual-mode**: Unix socket (local) + TCP (network) operation

---

**Handoff from**: biomeOS Neural API Team  
**Handoff to**: Songbird Development Team

---

## Related Handoffs

- `SONGBIRD_STUN_RENDEZVOUS_HANDOFF.md` - STUN method exposure
- `SONGBIRD_EVOLUTION_HANDOFF.md` - HTTP headers fix (✅ COMPLETE)

