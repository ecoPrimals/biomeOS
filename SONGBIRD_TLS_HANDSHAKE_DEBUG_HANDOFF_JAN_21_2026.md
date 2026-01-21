# Songbird TLS Handshake Debugging Handoff

**Date**: January 21, 2026  
**Status**: 🟡 **Issue Isolated** - Ready for TLS debugging  
**For**: Songbird Team  
**From**: biomeOS Capability Translation Integration

---

## 🎯 Issue Summary

**Symptom**: HTTPS requests timeout after ~20 seconds  
**Status**: Songbird now starting successfully, but TLS handshake hangs  
**Impact**: HTTP works, capability translation works, but HTTPS blocked

---

## ✅ What's Fixed

### 1. **Songbird Startup Issue** ✅
**Problem**: Songbird was crashing on startup with:
```
Error: No security provider configured.
Please set one of:
- SONGBIRD_SECURITY_PROVIDER (recommended - generic capability)
```

**Solution**: Added environment variables to deployment graphs:
```toml
[nodes.operation.environment]
NEURAL_API_SOCKET = "/tmp/neural-api-nat0.sock"
BEARDOG_SOCKET = "/tmp/beardog-nat0.sock"
SONGBIRD_SECURITY_PROVIDER = "/tmp/beardog-nat0.sock"
```

**Status**: ✅ Songbird now starts successfully and responds to health checks

### 2. **Capability Translation** ✅
**Status**: Fully functional
- Neural API capability.call works
- BearDog keypair generation via capability translation verified
- 14 translations loading from graphs

---

## 🟡 Remaining Issue: TLS Handshake Timeout

### Symptoms

1. **HTTP works**:
```bash
$ echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"GET","url":"http://httpbin.org/get"},"id":1}' | nc -U /tmp/songbird-nat0.sock
# Returns response (with 400 error, but proves HTTP client works)
```

2. **HTTPS times out**:
```bash
$ echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"GET","url":"https://api.github.com/zen"},"id":2}' | timeout 20 nc -U /tmp/songbird-nat0.sock
# Times out after 20 seconds, no response
```

3. **Capability translation works**:
```bash
$ echo '{"jsonrpc":"2.0","method":"capability.call","params":{"capability":"crypto.generate_keypair","args":{}},"id":3}' | nc -U /tmp/neural-api-nat0.sock
# Returns keypair from BearDog successfully
```

### What This Tells Us

✅ **Network layer**: Working (HTTP succeeds)  
✅ **Songbird startup**: Working (health check passes)  
✅ **BearDog integration**: Working (capability.call succeeds)  
✅ **Neural API routing**: Working (semantic translation functional)  
🟡 **TLS handshake logic**: Hanging somewhere

---

## 🔍 Debugging Steps

### 1. Enable Trace Logging

```bash
# Start Songbird with trace logging
RUST_LOG=trace \
NEURAL_API_SOCKET=/tmp/neural-api-nat0.sock \
BEARDOG_SOCKET=/tmp/beardog-nat0.sock \
./songbird server > /tmp/songbird-tls-debug.log 2>&1 &

# Test HTTPS
echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"GET","url":"https://api.github.com/zen"},"id":1}' | timeout 20 nc -U /tmp/songbird-nat0.sock &

# Wait and analyze logs
sleep 10
grep -E "(handshake|TLS|keypair|Neural)" -i /tmp/songbird-tls-debug.log
```

### 2. Check What Gets Called

Expected flow:
```
1. Songbird receives http.request for HTTPS URL
2. Songbird's HTTP client detects https:// scheme
3. Calls https_request() → TlsHandshake::handshake()
4. TLS handshake calls beardog.generate_keypair()
   → Should route through Neural API
   → Neural API translates crypto.generate_keypair → crypto.x25519_generate_ephemeral
   → Calls BearDog
   → Returns keypair
5. TLS handshake continues with keypair
6. ... (where does it hang?)
```

### 3. Verify BearDog Calls

Since we know `capability.call` works, the issue is likely:
- **A**: Songbird HTTP client isn't using Neural API routing
- **B**: TLS handshake has a logic error (e.g., waiting for wrong message)
- **C**: Socket communication issue in TLS layer

Check the code:
```rust
// In songbird-http-client/src/beardog_client.rs
// Should be routing through Neural API:
let mut stream = UnixStream::connect(&self.neural_api_socket).await?;
// NOT directly to BearDog socket
```

### 4. Compare Working vs Broken

**Working** (via capability.call):
```bash
Client → Neural API → BearDog → Returns result
```

**Not Working** (via HTTPS):
```bash
Songbird HTTP client → ??? → Times out
```

Question: Is the HTTP client actually using the updated `BearDogClient` that routes through Neural API?

---

## 🔧 Potential Issues

### Issue 1: HTTP Client Not Using Updated BearDogClient

**Check**: `songbird-http-client/src/client.rs`

```rust
impl SongbirdHttpClient {
    pub fn new(neural_api_socket: impl Into<String>) -> Self {
        Self {
            beardog: Arc::new(BearDogClient::new(neural_api_socket)), // ✅ Should use neural_api_socket
        }
    }
}
```

**Verify**: Is `NEURAL_API_SOCKET` being passed when creating the HTTP client?

### Issue 2: TLS Handshake Logic Error

**Location**: `songbird-http-client/src/tls/handshake.rs`

Possible hang points:
1. Waiting for ServerHello (never arrives)
2. Reading post-handshake messages (stuck in loop)
3. Waiting for certificate (parsing error)
4. ECDH derivation (BearDog call hangs)

**Add logging**:
```rust
debug!("Step 1: Generating keypair");
let (public, private) = self.beardog.generate_keypair().await?;
debug!("Step 1 complete: got {} byte keypair", public.len());

debug!("Step 2: Building ClientHello");
let client_hello = self.build_client_hello(...)?;
debug!("Step 2 complete: ClientHello {} bytes", client_hello.len());

debug!("Step 3: Sending ClientHello");
stream.write_all(&client_hello).await?;
debug!("Step 3 complete");

debug!("Step 4: Waiting for ServerHello");
let server_hello = timeout(Duration::from_secs(5), read_tls_record(&mut stream)).await??;
debug!("Step 4 complete: ServerHello {} bytes", server_hello.len());

// etc...
```

### Issue 3: Socket Read Timeout

**Check**: Are we using `read_to_end()` anywhere in TLS code?

```rust
// BAD: Hangs waiting for socket to close
stream.read_to_end(&mut buffer).await?;

// GOOD: Read specific number of bytes or use timeout
timeout(Duration::from_secs(5), stream.read_exact(&mut buffer[..len])).await??;
```

---

## 📋 Action Items for Songbird Team

### Immediate (1-2 hours)

1. **Add comprehensive logging to TLS handshake**
   - Log every step with timestamps
   - Log all BearDog calls (should show Neural API routing)
   - Log all network I/O (bytes sent/received)

2. **Verify HTTP client configuration**
   - Confirm `BearDogClient::new()` receives `NEURAL_API_SOCKET`
   - Confirm it's NOT using old direct BearDog connection

3. **Test individual TLS steps**
   ```bash
   # Test just keypair generation
   echo '{"jsonrpc":"2.0","method":"test_tls_keypair","id":1}' | nc -U /tmp/songbird-nat0.sock
   
   # Test just ClientHello construction
   echo '{"jsonrpc":"2.0","method":"test_tls_client_hello","id":2}' | nc -U /tmp/songbird-nat0.sock
   ```

### Investigation (2-4 hours)

1. **Packet capture**
   ```bash
   sudo tcpdump -i any -w tls-debug.pcap host api.github.com and port 443 &
   # Run HTTPS test
   # Analyze with Wireshark
   ```

2. **Compare with working TLS client**
   ```bash
   openssl s_client -connect api.github.com:443 -tls1_3 -showcerts
   # Compare ClientHello format with ours
   ```

3. **Test with simpler endpoint**
   ```bash
   # Try a test server that logs TLS handshake
   echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"GET","url":"https://example.com"},"id":1}' | timeout 10 nc -U /tmp/songbird-nat0.sock
   ```

---

## 🔍 Diagnostic Commands

```bash
# 1. Check Songbird is running
ps aux | grep songbird | grep server

# 2. Check sockets exist
ls -la /tmp/*-nat0.sock

# 3. Test Songbird health
echo '{"jsonrpc":"2.0","method":"health","id":1}' | nc -U /tmp/songbird-nat0.sock

# 4. Test HTTP (should work)
echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"GET","url":"http://httpbin.org/get"},"id":2}' | timeout 5 nc -U /tmp/songbird-nat0.sock

# 5. Test capability translation (should work)
echo '{"jsonrpc":"2.0","method":"capability.call","params":{"capability":"crypto.generate_keypair","args":{}},"id":3}' | nc -U /tmp/neural-api-nat0.sock

# 6. Test HTTPS (will timeout)
echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"GET","url":"https://api.github.com/zen"},"id":4}' | timeout 20 nc -U /tmp/songbird-nat0.sock
```

---

## 📊 Current Status

| Component | Status | Notes |
|-----------|--------|-------|
| Songbird Startup | ✅ Working | Fixed with BEARDOG_SOCKET env var |
| Health Check | ✅ Working | Responds correctly |
| HTTP Requests | ✅ Working | httpbin returns responses |
| Capability Translation | ✅ Working | Neural API routing functional |
| BearDog Integration | ✅ Working | Keypair generation via capability.call works |
| TLS Handshake | 🟡 Hanging | Times out after ~20 seconds |

---

## 🎯 Success Criteria

When TLS handshake is fixed, we should see:

```bash
$ echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"GET","url":"https://api.github.com/zen"},"id":1}' | nc -U /tmp/songbird-nat0.sock
{
  "jsonrpc": "2.0",
  "result": {
    "status": 200,
    "headers": {...},
    "body": "Design for failure."  # Or whatever GitHub's Zen quote is
  },
  "id": 1
}
```

**Timeline**: 2-4 hours for identification and fix  
**Complexity**: Medium (TLS is complex, but framework is in place)  
**Blockers**: None (all infrastructure working)

---

## 💡 Hints

1. **Most likely issue**: TLS handshake is using old direct BearDog connection instead of Neural API routing
2. **Second most likely**: Logic error in post-handshake message reading (stuck in loop)
3. **Third most likely**: Socket read timeout (using `read_to_end()` instead of framed reading)

---

## 📚 References

- **BearDog RPC API**: 47 methods documented (semantic namespaces)
- **Capability Translation**: `CAPABILITY_TRANSLATION_ARCHITECTURE.md`
- **HTTP Client**: `phase1/songbird/crates/songbird-http-client/`
- **TLS Handshake**: `phase1/songbird/crates/songbird-http-client/src/tls/handshake.rs`

---

**Status**: Ready for Songbird team to debug TLS handshake  
**Priority**: Medium (capability translation objective complete, HTTPS is bonus)  
**Impact**: High (enables full Pure Rust HTTPS stack)

---

*Handoff Created: January 21, 2026*  
*From: biomeOS Capability Translation Session*  
*To: Songbird Team*  
*Next: TLS handshake debugging with comprehensive logging*

🐦🔐 **Good hunting! The infrastructure is all working - just need to find the TLS hang point.** 🔐🐦

