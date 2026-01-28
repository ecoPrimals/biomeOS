# Songbird Evolution Handoff - Comprehensive

**Date**: January 28, 2026 (Final)
**From**: biomeOS Team (Neural API)  
**To**: Songbird Team
**Version**: v8.14.0 (Final - Commit: a2ece6489)
**Status**: 🎊 **ALL PRIORITIES COMPLETE** 🎊

---

## Executive Summary

🎉 **MAJOR UPDATE**: Songbird v8.14.0 has resolved all critical blocking issues!

The Songbird team completed a comprehensive 9-hour session on Jan 28, 2026, addressing:
- Port:0 beacon fix (config validation)
- XDG socket discovery in BOTH HTTP client AND TLS layer
- Dual-mode architecture (External TCP + Internal Unix)
- `--federation-port` CLI flag
- Concurrent test evolution (0 `#[ignore]` flags)

### Current Status

| Feature | Status | Notes |
|---------|--------|-------|
| HTTP Client Socket Discovery | ✅ Complete | `socket_discovery.rs` works |
| TLS Layer Socket Discovery | ✅ Complete | XDG discovery + `EnvReader` trait |
| LAN Discovery | ✅ Fixed | Port:0 validation added |
| STUN Client | ✅ Complete | Pure Rust RFC 5389 |
| Neural API Integration | ✅ Working | Via `BEARDOG_MODE=neural` |
| Method Mapping | ✅ Resolved | Via Neural API translations |
| Dual-Mode Architecture | ✅ Documented | External TCP + Internal Unix |

---

## Priority 1: LAN Discovery - Port:0 Beacon Fix ✅ RESOLVED

### The Problem (FIXED in v8.14.0)

Songbird was broadcasting discovery beacons with `port: 0`, causing peers to reject them.

### Solution Implemented (Commit: d4cccba53)

1. **Configuration Validation**: Added validation in `CanonicalSongbirdConfig::validate()` that rejects `network.base_port = 0` when `discovery.mode` is enabled.

2. **CLI Enhancement**: Added `--federation-port` flag (alias for `--port`) with clear help text explaining dual-mode architecture.

3. **Documentation**: Created comprehensive dual-mode architecture documentation.

### Dual-Mode Architecture (Now Documented)

```
┌─────────────────────────────────────────────────────────────────┐
│                 SONGBIRD DUAL-MODE OPERATION                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  EXTERNAL GATEWAY (TCP Port 8080)     INTERNAL IPC (Unix:0)    │
│  ────────────────────────────────     ─────────────────────    │
│  • LAN beacon broadcasts              • Inter-primal JSON-RPC  │
│  • Initial peer handshake             • BearDog ↔ Songbird     │
│  • Federation discovery               • Squirrel ↔ Neural API  │
│  • External API gateway               • Zero network exposure  │
│                                                                 │
│  ESCALATION: TCP discovery → Unix secure RPC                   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Implementation

```rust
// songbird-orchestrator/src/app/config.rs
struct SongbirdConfig {
    /// Internal: Unix socket (always, for IPC)
    internal_socket: PathBuf,  // /run/user/1000/biomeos/songbird-nat0.sock
    
    /// External: TCP for LAN discovery (optional)
    external_beacon_port: Option<u16>,  // Some(8080) when federation enabled
    
    /// Federation mode
    federation_enabled: bool,
}

// songbird-orchestrator/src/app/core.rs
async fn start_http_server(&self) -> Result<u16> {
    if self.config.federation_enabled {
        // Bind TCP for external discovery
        let port = self.config.external_beacon_port.unwrap_or(8080);
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
        info!("🌐 External gateway bound to port {}", port);
        Ok(port)
    } else {
        // Unix sockets only (current behavior)
        info!("🔒 Unix sockets ONLY (federation disabled)");
        Ok(0)
    }
}
```

### Files to Modify

1. `songbird-orchestrator/src/app/core.rs` - Add TCP binding when federation enabled
2. `songbird-orchestrator/src/app/config.rs` - Add `external_beacon_port` config
3. `songbird-discovery/src/anonymous/broadcaster.rs` - Use external port in beacons
4. `songbird-types/src/config/discovery.rs` - Add federation config options

### CLI Addition

```bash
# Enable federation with external port
./songbird server --socket /run/user/1000/biomeos/songbird-nat0.sock \
    --federation-port 8080  # NEW: External beacon port
```

---

## Priority 2: TLS Layer Socket Discovery ✅ RESOLVED

### The Problem (FIXED in v8.14.0)

The TLS layer was using hardcoded `/tmp` paths.

### Solution Implemented (Commit: 5b1e50e03)

1. **XDG Discovery**: Applied XDG-compliant socket discovery pattern to `songbird-tls/src/crypto.rs`
2. **EnvReader Trait**: Implemented `EnvReader` trait for concurrent testing without global state pollution
3. **Test Evolution**: Eliminated `#[ignore]` flags through proper concurrent test design

### Implementation (Now Complete)

```rust
// songbird-tls/src/crypto.rs
use songbird_http_client::crypto::socket_discovery::{
    discover_beardog_socket,
    discover_neural_api_socket,
};

fn discover_crypto_socket() -> Result<String> {
    // Priority 1: Explicit env vars (keep existing)
    if let Ok(path) = std::env::var("SONGBIRD_CRYPTO_SOCKET") { return Ok(path); }
    if let Ok(path) = std::env::var("BEARDOG_CRYPTO_SOCKET") { return Ok(path); }

    // Priority 2: XDG-compliant discovery (NEW!)
    let beardog = discover_beardog_socket();
    if std::path::Path::new(&beardog).exists() {
        return Ok(beardog);
    }
    
    let neural = discover_neural_api_socket();
    if std::path::Path::new(&neural).exists() {
        return Ok(neural);
    }

    // Priority 3: Legacy fallback (keep for backward compat)
    for path in ["/tmp/beardog.sock", "/tmp/neural-api.sock"] {
        if std::path::Path::new(path).exists() {
            warn!("⚠️ Using legacy /tmp socket: {}", path);
            return Ok(path.to_string());
        }
    }
    
    Err(TlsError::CryptoError("Could not discover crypto socket".into()))
}
```

### Files to Modify

1. `songbird-tls/src/crypto.rs` - Import and use `socket_discovery`
2. Or copy `socket_discovery.rs` to `songbird-tls/src/`

---

## Priority 3: Method Mapping Optimization (Optional)

### Current State: Working via Neural API

Songbird with `BEARDOG_MODE=neural` routes crypto calls through Neural API, which applies semantic translations:

```toml
# biomeOS/graphs/tower_atomic_bootstrap.toml
"x25519_generate_ephemeral" = "crypto.x25519_generate_ephemeral"
"x25519_diffie_hellman" = "crypto.x25519_derive_secret"
"derive_handshake_secrets" = "tls.derive_handshake_secrets"
# ... 74 total translations
```

**Test Result**: `https://api.github.com/zen → 200 OK (389ms)`

### Optional Optimization: Direct Mode Mapping

If Songbird wants to work in `BEARDOG_MODE=direct` without Neural API overhead, update the internal mapping:

```rust
// songbird-http-client/src/crypto/beardog_provider.rs
fn semantic_to_actual<'a>(&self, method: &'a str) -> &'a str {
    match method {
        // Current (wrong for direct mode):
        "crypto.generate_keypair" => "x25519_generate_ephemeral",
        
        // Fix (correct for BearDog):
        "crypto.generate_keypair" => "crypto.x25519_generate_ephemeral",
        
        // ... update all mappings to include crypto. prefix
    }
}
```

**Recommendation**: Keep using Neural API mode (`BEARDOG_MODE=neural`) for automatic API evolution. The translation layer handles method name changes without Songbird updates.

---

## Priority 4: Multi-Transport Discovery (Future)

### Problem

UDP multicast doesn't reliably cross wifi/ethernet boundaries on consumer routers.

### Recommended Transports

| Transport | Same Interface | Cross Interface | Through NAT | Corporate |
|-----------|---------------|-----------------|-------------|-----------|
| Multicast | ✅ | ❌ | ❌ | ❌ |
| Subnet Broadcast | ✅ | ✅ | ❌ | ⚠️ |
| mDNS (5353) | ✅ | ✅ | ❌ | ✅ |
| TCP Rendezvous | ✅ | ✅ | ✅ | ✅ |
| STUN/TURN | ✅ | ✅ | ✅ | ✅ |

### Evolution Path

1. **Add subnet broadcast fallback** (quick win)
2. **Leverage new STUN client for NAT detection** (v8.13.0 has this!)
3. **mDNS backend** (high compatibility)
4. **HTTP Bootstrap → UDP Escalation** (architecturally correct)

---

## Verification Tests

After implementing these evolutions:

```bash
# Test 1: XDG Socket Discovery
XDG_RUNTIME_DIR=/run/user/1000 FAMILY_ID=nat0 ./songbird server
# Should auto-discover BearDog at /run/user/1000/biomeos/beardog-nat0.sock

# Test 2: Federation Discovery
./songbird server --federation-port 8080
# Should broadcast beacons with port:8080 (not port:0)

# Test 3: LAN Discovery
# Tower A (ethernet): ./songbird server --federation-port 8080
# Tower B (wifi): ./songbird server --federation-port 8080
# Both should discover each other within 30 seconds

# Test 4: HTTPS via Neural API
BEARDOG_MODE=neural ./songbird server
curl -X POST --unix-socket /run/user/1000/biomeos/songbird-nat0.sock \
  -d '{"jsonrpc":"2.0","method":"http.get","params":{"url":"https://api.github.com/zen"},"id":1}'
# Should return 200 OK
```

---

## Summary: Files to Modify

| Priority | File | Change |
|----------|------|--------|
| P1 | `songbird-orchestrator/src/app/core.rs` | Add TCP binding for federation |
| P1 | `songbird-discovery/src/anonymous/broadcaster.rs` | Use external port in beacons |
| P2 | `songbird-tls/src/crypto.rs` | Use XDG socket discovery |
| P3 | `songbird-http-client/src/crypto/beardog_provider.rs` | Fix direct mode mappings (optional) |

---

## Current Working Configuration

Until evolutions are complete:

```bash
# Full working setup with all workarounds
BEARDOG_MODE=neural \
NEURAL_API_SOCKET=/run/user/1000/biomeos/neural-api-nat0.sock \
BEARDOG_SOCKET=/run/user/1000/biomeos/beardog-nat0.sock \
BEARDOG_CRYPTO_SOCKET=/run/user/1000/biomeos/beardog-nat0.sock \
SONGBIRD_SECURITY_PROVIDER=beardog \
FAMILY_ID=nat0 \
NODE_ID=tower0 \
./songbird server \
    --socket /run/user/1000/biomeos/songbird-nat0.sock \
    --beardog-socket /run/user/1000/biomeos/beardog-nat0.sock
```

---

## 🆕 Issues: HTTP Headers Not Reaching Server (Jan 28, 2026)

**Discovered During**: Squirrel AI integration testing (commit 28e59176)

### Issue 1: `http.post` Loses Headers at IPC Layer

**Symptom**: When using `http.post`, headers are empty in `HttpRequestParams`:
```
handle_post → HttpRequestParams { ..., headers: {}, ... }  // EMPTY!
```

**Expected**: Headers should be forwarded to `handle_request`.

**Location**: `crates/songbird-orchestrator/src/ipc/handlers/http.rs` (lines 41-45)

### Issue 2: `http.request` Receives Headers but Server Returns 401

**Symptom**: `http.request` shows correct headers in logs, but Anthropic returns 401:
```
handle_request → HttpRequestParams { ..., headers: {"x-api-key": "..."}, ... }  // CORRECT!
BUT server returns: 401 "x-api-key header is required"
```

**Root Cause**: Headers are received by IPC handler but NOT included in actual HTTP request.

**Location**: Somewhere in `songbird-http-client` request building logic.

### Test Commands to Reproduce

```bash
# Issue 1: http.post (headers lost at IPC layer)
echo '{"jsonrpc":"2.0","method":"http.post","params":{"url":"https://api.anthropic.com/v1/messages","headers":{"x-api-key":"test"},"body":"{}"}}' | nc -U /path/to/songbird.sock

# Issue 2: http.request (headers received but not sent to server)
echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"POST","url":"https://api.anthropic.com/v1/messages","headers":{"x-api-key":"test"},"body":"{}"}}' | nc -U /path/to/songbird.sock
```

### Impact
- 🔴 **HIGH** - Blocks Squirrel AI provider integration
- All AI API calls (Anthropic, OpenAI) fail with 401

---

## Contact

For questions or pairing:
- biomeOS team available for implementation support
- Can provide test fixtures and XDG discovery code
- Willing to review PRs

---

---

## 🎉 UPDATE: IPC Layer Fix CONFIRMED Working (Commit 53a45b625)

The Songbird team fixed Issue 1 (`http.post` losing headers). The IPC layer now correctly preserves `caller_headers`:

```
caller_headers={"anthropic-version": "2023-06-01", "x-api-key": "...", "content-type": "application/json"}
HttpRequestParams { headers: {"anthropic-version": "2023-06-01", "x-api-key": "...", ...} }  // ✅ PRESENT!
```

---

## 🔴 NEW ISSUE: HTTP Client Layer Drops `caller_headers` (Jan 28, 2026 - Night)

**Severity**: 🔴 CRITICAL - Blocks all authenticated API calls

### The Problem

The `caller_headers` are correctly passed through the IPC layer, but they are **NOT being merged into the final HTTP request** that goes over the wire.

### Evidence from Songbird Logs

**IPC Layer** (✅ WORKING):
```
handle_post{caller_headers={"x-api-key": "...", "anthropic-version": "2023-06-01"}}
handle_request{params=HttpRequestParams { headers: {"x-api-key": "...", ...} }}
```

**Final HTTP Request** (❌ HEADERS MISSING):
```
🔍 Final HTTP request (10 lines):
   1: POST /v1/messages HTTP/1.1
   2: Host: api.anthropic.com
   3: Accept: application/json
   4: Accept-Language: en-US,en;q=0.9
   5: Connection: keep-alive
   6: Content-Type: application/json      ← Default, not caller's!
   7: User-Agent: Songbird/0.1.0 ...
   8: Content-Length: 2                   ← WRONG! Should be ~100 bytes
   9: 
   10: ""
```

### What's Missing

1. ❌ `x-api-key` header not in final request
2. ❌ `anthropic-version` header not in final request  
3. ❌ Body appears to be `{}` (2 bytes) instead of decoded base64 JSON

### Root Cause Location

**File**: `crates/songbird-http-client/src/request.rs`

The `caller_headers` are passed to `RequestBuilder`, but the `write_headers()` method only writes default headers, not the caller's headers.

### Proposed Fix

```rust
// In RequestBuilder
impl RequestBuilder {
    fn write_headers(&self, writer: &mut impl Write) -> Result<()> {
        // Write default headers
        writeln!(writer, "Host: {}", self.host)?;
        writeln!(writer, "Accept: application/json")?;
        // ... other defaults ...
        
        // ✅ CRITICAL: Write CALLER headers (override defaults)
        for (key, value) in &self.caller_headers {
            writeln!(writer, "{}: {}", key, value)?;
        }
        
        Ok(())
    }
}
```

### Also: Body Decoding Issue

The body in `http.post` is **base64 encoded**:
```json
"body": "eyJtb2RlbCI6ImNsYXVkZS0zLWhhaWt1LTIwMjQwMzA3Ii4uLn0="
```

It should be decoded before inclusion in the HTTP request. Currently `Content-Length: 2` suggests only `{}` is being sent.

### Test Command

```bash
# Test with httpbin.org - should echo back headers
echo '{"jsonrpc":"2.0","method":"http.post","params":{"url":"https://httpbin.org/post","headers":{"X-Custom":"test123"},"body":"eyJ0ZXN0Ijp0cnVlfQ=="},"id":1}' | nc -U /run/user/1000/biomeos/songbird-nat0.sock

# Success criteria: Response should contain "X-Custom": "test123"
```

### Impact

- 🔴 Blocks Squirrel AI integration (Anthropic, OpenAI)
- 🔴 Blocks any external API requiring custom headers
- 🔴 Blocks any API requiring authentication

---

## 🎉 UPDATE: HTTP Client Layer Fix COMPLETE (Commit 2fec947bc)

**Status**: ✅ **ALL HTTP HEADER ISSUES RESOLVED**

The Songbird team completed both fixes:

| Issue | Commit | Status |
|-------|--------|--------|
| IPC Layer (http.post → handle_request) | a6d702dcd | ✅ FIXED |
| HTTP Client Layer (caller_headers → wire) | 2fec947bc | ✅ FIXED |

### Fix Summary

**Commit 2fec947bc** - HTTP client wrapper now preserves caller headers:
- `caller_headers` are now correctly merged into the final HTTP request
- Headers appear in the raw bytes sent over the wire
- Base64 body decoding is working correctly

### Test Coverage

68 comprehensive HTTP header tests added (commit a75356812):
- Header propagation through all layers
- Base64 body encoding/decoding
- Multiple header types (auth, content-type, custom)

### Verification

```bash
# Test with httpbin.org - should echo back custom headers
echo '{"jsonrpc":"2.0","method":"http.post","params":{"url":"https://httpbin.org/post","headers":{"X-Custom":"test123"},"body":"eyJ0ZXN0Ijp0cnVlfQ=="},"id":1}' | nc -U /run/user/1000/biomeos/songbird-nat0.sock

# Expected: Response contains "X-Custom": "test123"
```

---

*Generated: January 28, 2026 (Final)*  
*Songbird Version: v8.14.0 (Commit: f6cb661b4)*  
*biomeOS Neural API: 74 translations registered*  
*IPC Fix: ✅ COMPLETE*  
*HTTP Client Fix: ✅ COMPLETE*  
*Total Tests: 68 HTTP header tests*  
*Status: 🚀 PRODUCTION READY*

