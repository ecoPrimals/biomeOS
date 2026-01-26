# 🎵 Songbird TLS Handshake Fix - Handoff

**Date**: January 26, 2026 (Updated 13:45 UTC)  
**Priority**: HIGH  
**Status**: 🟡 In Progress - Auth Tag Failure

---

## 🆕 LATEST UPDATE (13:45 UTC)

### Commits Applied
- **BearDog `fb7513739`**: RFC 8446 compliant `derive_application_secrets` API
- **Songbird `73431b6db`**: Pass `cipher_suite` to `tls_derive_application_secrets`

### Progress
| Issue | Status |
|-------|--------|
| BearDog API (`pre_master_secret` → `handshake_secret`) | ✅ FIXED |
| Key length (32 bytes → 16 bytes for AES-128-GCM) | ✅ FIXED |
| Auth tag verification | ❌ FAILING |

### Current Error
```
AES-128-GCM decryption failed: authentication tag verification failed 
(data may be tampered)
```

This means:
1. ✅ Cipher suite is correctly passed (0x1301 / AES-128-GCM)
2. ✅ Key derivation returns 16-byte keys (correct for AES-128)
3. ❌ But the derived keys don't match what the server encrypted with

### Investigation Needed
- Verify `handshake_secret` passed to `derive_application_secrets` is correct
- Verify `transcript_hash` is computed correctly (must include all messages through ServerHello finished)
- Check if `cipher_suite` value is correctly passed (should be `4865` / `0x1301`)

---

## 🚨 PREVIOUS: BearDog API Mismatch (RESOLVED)

BearDog's `tls.derive_application_secrets` has wrong API:

| Parameter | Songbird Sends | BearDog Expects |
|-----------|----------------|-----------------|
| Input Secret | `handshake_secret` ✅ RFC 8446 | `pre_master_secret` ❌ Wrong! |
| Random | (not needed) | `client_random`, `server_random` |
| Transcript | `transcript_hash` | `transcript_hash` (optional) |

**BearDog Fix Required**: `crates/beardog-tunnel/src/unix_socket_ipc/handlers/crypto/tls/key_derivation.rs` line 495

Change from `pre_master_secret` → `handshake_secret` as input.

---

## 🐛 ROOT CAUSE FOUND!

The bug is in **TCP connection reuse during retry attempts**, NOT in the TLS extensions!

### Evidence from Logs

```
17:04:24.405083Z  ✅ Received ServerHello: type=0x16, 90 bytes  ← SUCCESS!
17:04:24.405134Z     Server negotiated cipher suite: 0x1301     ← TLS 1.3!
17:04:24.405865Z  📝 Adding ClientHello  ← NEW RETRY ATTEMPT (same TCP stream!)
17:04:24.405943Z  ❌ Expected 0x16, got 0x14 (Change Cipher Spec) ← READING OLD DATA!
```

The handshake **actually succeeds** on first try, but then the code retries on the SAME TCP stream, which still has buffered data from the previous server response.

---

## The Bug

**File**: `crates/songbird-http-client/src/client.rs`  
**Function**: `attempt_handshake_with_fallback()`

```rust
// BUG: This function receives &mut TcpStream and reuses it across retries
async fn attempt_handshake_with_fallback(
    &self,
    tcp_stream: &mut TcpStream,  // ← Same stream used for all attempts!
    host: &str,
) -> Result<SessionKeys> {
    // ...
    for strategy in strategies_to_try {
        // Tries handshake on SAME tcp_stream
        // First attempt: sends ClientHello, receives ServerHello + CCS + Encrypted
        // Second attempt: reads stale CCS/ApplicationData from buffer!
    }
}
```

---

## The Fix (30 minutes)

**Option A: Create new TCP connection per retry** (RECOMMENDED)

```rust
async fn attempt_handshake_with_fallback(
    &self,
    addr: &str,       // ← Pass address instead of stream
    host: &str,
) -> Result<(TcpStream, SessionKeys)> {  // ← Return the successful stream
    for strategy in strategies_to_try {
        // Create FRESH TCP connection for each attempt
        let mut tcp_stream = TcpStream::connect(addr).await?;
        
        match self.try_handshake(&mut tcp_stream, host, strategy).await {
            Ok(keys) => return Ok((tcp_stream, keys)),
            Err(e) => {
                last_error = Some(e);
                // tcp_stream dropped here, connection closed
            }
        }
    }
    Err(last_error.unwrap())
}
```

**Option B: Disable retry mechanism** (QUICK FIX)

```rust
// In TlsConfig::default() or config initialization:
fallback_strategy: FallbackStrategy::None,  // Single attempt only
```

---

## Why The First Handshake Actually Works

Looking at the logs, the **first attempt succeeds**:
- Received valid ServerHello (0x16, 90 bytes)
- Negotiated TLS_AES_128_GCM_SHA256 (0x1301)
- Server is responding correctly!

The failure happens ONLY on retry attempts because they read stale data.

---

## Testing After Fix

```bash
cd /home/eastgate/Development/ecoPrimals/phase1/songbird
cargo build --release
cp target/release/songbird /home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin/primals/songbird/

# Test
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
./deploy_tower_atomic.sh

echo '{"jsonrpc":"2.0","method":"capability.call","params":{"capability":"secure_http","operation":"http.get","args":{"url":"https://api.github.com/zen"}},"id":1}' | nc -U /tmp/neural-api.sock
```

---

## Contact

Bug discovered: Jan 26, 2026, 12:15 UTC
Root cause: TCP stream reuse during TLS handshake retry
Fix complexity: LOW (30 min)

---

## Update (12:30 UTC): Parameter Mismatch Issue

After fixing the TCP reuse bug, a new issue emerged:

**`tls_derive_handshake_secrets` parameter mismatch:**

| Songbird sends | BearDog expects |
|----------------|-----------------|
| `shared_secret` | `pre_master_secret` |
| `transcript_hash` | `client_random` |
| (none) | `server_random` |
| (none) | `transcript_hash` |
| (none) | `cipher_suite` |

**Fix required in Songbird:**

The `tls_derive_handshake_secrets()` call in `handshake_flow.rs` needs to pass all required parameters:

```rust
// Current (incomplete):
.tls_derive_handshake_secrets(&shared_secret, &handshake_transcript_hash)

// Required (all params):
.tls_derive_handshake_secrets(
    &shared_secret,         // → pre_master_secret
    &client_random,
    &server_random,
    &handshake_transcript_hash,
    cipher_suite,
)
```

This requires updating the `CryptoCapability` trait and `BearDogProvider` implementation.

---

## Location of Issue

```
crates/songbird-http-client/src/tls/handshake_legacy.rs
```

Key areas to check:

1. **ClientHello construction** - Ensure `supported_versions` lists TLS 1.3 only
2. **Session resumption handling** - We should NOT include PSK or session tickets on fresh connections
3. **Extension ordering** - Some servers are sensitive to extension order

---

## Proposed Fix

### Option A: Ensure Clean ClientHello (Recommended)

In `handshake_legacy.rs`, verify the ClientHello:

```rust
// ClientHello should NOT include:
// - psk_key_exchange_modes (unless we want PSK)
// - pre_shared_key extension (unless resuming)
// - Any session ticket data

// ClientHello MUST include:
// - supported_versions = [0x0304] (TLS 1.3 only)
// - key_share with X25519 public key
// - signature_algorithms
// - supported_groups
```

### Option B: Debug ClientHello Bytes

Add hex dump of the actual ClientHello being sent:

```rust
info!("📤 ClientHello hex dump:");
for (i, chunk) in client_hello.chunks(16).enumerate() {
    info!("{:04x}: {}", i * 16, hex::encode(chunk));
}
```

Compare against a known-good TLS 1.3 ClientHello (e.g., from `openssl s_client`).

---

## Verification Steps

### 1. Capture Working ClientHello

```bash
openssl s_client -connect api.github.com:443 -tls1_3 -msg 2>&1 | head -50
```

### 2. Compare Extensions

The working ClientHello should have these extensions:
- `supported_versions` (type 0x002b)
- `key_share` (type 0x0033)
- `signature_algorithms` (type 0x000d)
- `supported_groups` (type 0x000a)

### 3. Test After Fix

```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
./deploy_tower_atomic.sh

# Test GitHub
echo '{"jsonrpc":"2.0","method":"capability.call","params":{"capability":"secure_http","operation":"http.get","args":{"url":"https://api.github.com/zen"}},"id":1}' | nc -U /tmp/neural-api.sock
```

---

## What's Already Working

The Tower Atomic architecture is **fully validated**:

- ✅ Neural API capability.call routing
- ✅ Graph-based semantic translation (39 mappings)
- ✅ BearDog crypto operations via capability.call
- ✅ plasmidBin binary harvesting
- ✅ Explicit coordinated mode (BIOMEOS_MODE=coordinated)

The TLS handshake fix is the **last step** for full HTTPS connectivity.

---

## Files to Review

```
crates/songbird-http-client/src/tls/
├── handshake_legacy.rs   ← Main handshake logic
├── client_hello.rs       ← ClientHello construction
├── extensions.rs         ← TLS extensions
└── record.rs             ← TLS record layer
```

---

## Contact

This handoff was created during biomeOS session on Jan 26, 2026.
Questions: Check archive/ for session history.

