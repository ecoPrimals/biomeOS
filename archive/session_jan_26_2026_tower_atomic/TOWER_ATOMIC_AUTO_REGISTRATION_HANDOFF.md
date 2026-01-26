# Tower Atomic Auto-Registration Handoff
**Date**: January 25, 2026  
**For**: Songbird Team & BearDog Team  
**Goal**: Complete TRUE PRIMAL pattern with auto-registration and semantic routing  
**Time**: ~3 hours total (1.5h per team in parallel)

---

## 🎯 Executive Summary

**Current State**: Hardcoded method names between Songbird ↔ BearDog  
**Target State**: Full semantic routing via Neural API's `capability.call`  
**Benefit**: Zero coupling, independent evolution, production-ready architecture

### Why This Matters

```
❌ BEFORE (Hardcoded - breaks on any change):
Songbird → "crypto.x25519_generate_ephemeral" → BearDog

✅ AFTER (Semantic - never breaks):
Songbird → Neural API.capability_call("crypto", "generate_keypair")
  → Neural API translates → BearDog
```

**User Insight**: "API differences should be solved with capability.call from NeuralAPI. Otherwise any change breaks things."

---

## 📋 Phase 1: BearDog Auto-Registration (1.5 hours)

### Overview
BearDog must register its crypto capabilities with Neural API on startup, just like Songbird does for `secure_http`.

### Implementation Steps

#### 1. Create Registration Module (30 min)

**File**: `crates/beardog/src/neural_registration.rs`

```rust
//! Neural API Auto-Registration for BearDog
//!
//! Registers BearDog's crypto capabilities with Neural API on startup.

use anyhow::{Context, Result};
use serde_json::json;
use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, debug, warn};

/// Register BearDog's capabilities with Neural API
pub async fn register_with_neural_api(neural_socket: &str) -> Result<()> {
    info!("🔐 Registering BearDog crypto capabilities with Neural API");
    
    // Capability definitions
    let capabilities = vec![
        // Core crypto capability
        json!({
            "capability": "crypto",
            "provider": "beardog",
            "operations": [
                "generate_keypair",
                "ecdh_derive",
                "encrypt",
                "decrypt",
                "encrypt_aes_128_gcm",
                "decrypt_aes_128_gcm",
                "encrypt_aes_256_gcm",
                "decrypt_aes_256_gcm",
                "sha256",
                "sha384",
                "hkdf_extract",
                "hkdf_expand"
            ],
            "semantic_mappings": {
                // Map semantic names to BearDog's actual method names
                "crypto.generate_keypair": "crypto.x25519_generate_ephemeral",
                "crypto.ecdh_derive": "crypto.x25519_derive_secret",
                "crypto.encrypt": "crypto.chacha20_poly1305_encrypt",
                "crypto.decrypt": "crypto.chacha20_poly1305_decrypt",
                "crypto.encrypt_aes_128_gcm": "crypto.aes128_gcm_encrypt",
                "crypto.decrypt_aes_128_gcm": "crypto.aes128_gcm_decrypt",
                "crypto.encrypt_aes_256_gcm": "crypto.aes256_gcm_encrypt",
                "crypto.decrypt_aes_256_gcm": "crypto.aes256_gcm_decrypt",
                "crypto.sha256": "crypto.sha256",
                "crypto.sha384": "crypto.sha384",
                "crypto.hkdf_extract": "crypto.hkdf_extract",
                "crypto.hkdf_expand": "crypto.hkdf_expand"
            }
        }),
        // TLS-specific crypto
        json!({
            "capability": "tls_crypto",
            "provider": "beardog",
            "operations": [
                "derive_handshake_secrets",
                "derive_application_secrets",
                "compute_finished_verify_data"
            ],
            "semantic_mappings": {
                "tls.derive_handshake_secrets": "tls.derive_handshake_secrets",
                "tls.derive_application_secrets": "tls.derive_application_secrets",
                "tls.compute_finished_verify_data": "tls.compute_finished_verify_data"
            }
        }),
        // Genetic lineage
        json!({
            "capability": "genetic_lineage",
            "provider": "beardog",
            "operations": [
                "verify_lineage",
                "generate_lineage_proof"
            ]
        })
    ];

    // Register each capability
    for cap in capabilities {
        register_capability(neural_socket, cap).await
            .context("Failed to register capability")?;
    }

    info!("✅ BearDog capabilities registered with Neural API");
    Ok(())
}

async fn register_capability(neural_socket: &str, capability: serde_json::Value) -> Result<()> {
    let cap_name = capability["capability"].as_str().unwrap_or("unknown");
    debug!("📤 Registering capability: {}", cap_name);

    let request = json!({
        "jsonrpc": "2.0",
        "method": "capability.register",
        "params": capability,
        "id": 1
    });

    let request_str = serde_json::to_string(&request)?;
    
    // Connect to Neural API
    let mut stream = UnixStream::connect(neural_socket).await
        .context(format!("Failed to connect to Neural API at {}", neural_socket))?;

    // Send registration request
    stream.write_all(request_str.as_bytes()).await?;
    stream.write_all(b"\n").await?;

    // Read response
    let mut response = String::new();
    stream.read_to_string(&mut response).await?;

    let response_json: serde_json::Value = serde_json::from_str(&response)
        .context("Failed to parse Neural API response")?;

    if response_json.get("error").is_some() {
        warn!("⚠️  Registration warning for {}: {:?}", cap_name, response_json["error"]);
    } else {
        info!("✅ Registered: {}", cap_name);
    }

    Ok(())
}

/// Discover Neural API socket from environment
pub fn discover_neural_api_socket() -> Option<String> {
    std::env::var("NEURAL_API_SOCKET").ok()
        .or_else(|| std::env::var("NEURALS_SOCKET").ok())
        .or_else(|| Some("/tmp/neural-api-nat0.sock".to_string()))
}
```

#### 2. Integrate into Server Startup (15 min)

**File**: `crates/beardog/src/bin/server.rs` or main server initialization

```rust
use beardog::neural_registration::{register_with_neural_api, discover_neural_api_socket};

#[tokio::main]
async fn main() -> Result<()> {
    // ... existing initialization ...

    // Auto-register with Neural API if available
    if let Some(neural_socket) = discover_neural_api_socket() {
        info!("🌐 Neural API detected at: {}", neural_socket);
        
        match register_with_neural_api(&neural_socket).await {
            Ok(_) => info!("✅ BearDog registered with Neural API"),
            Err(e) => warn!("⚠️  Neural API registration failed (non-fatal): {}", e),
        }
    } else {
        info!("ℹ️  No Neural API detected - running in standalone mode");
    }

    // ... rest of server startup ...
}
```

#### 3. Update Module Exports (5 min)

**File**: `crates/beardog/src/lib.rs`

```rust
pub mod neural_registration;
```

#### 4. Test Auto-Registration (30 min)

**Test Script**: `test_beardog_registration.sh`

```bash
#!/bin/bash
set -e

echo "🧪 Testing BearDog Auto-Registration"
echo "===================================="

# Start Neural API
echo "1. Starting Neural API..."
cd ~/Development/ecoPrimals/phase2/biomeOS
./target/release/biomeos neural-api &
NEURAL_PID=$!
sleep 3

# Start BearDog with auto-registration
echo "2. Starting BearDog with auto-registration..."
cd ~/Development/ecoPrimals/phase1/beardog
export NEURAL_API_SOCKET="/tmp/neural-api-nat0.sock"
./target/release/beardog server --socket /tmp/beardog-nat0.sock &
BEARDOG_PID=$!
sleep 3

# Test capability.call
echo "3. Testing capability.call via Neural API..."
echo '{
  "jsonrpc": "2.0",
  "method": "capability.call",
  "params": {
    "capability": "crypto",
    "operation": "generate_keypair",
    "args": {"algorithm": "x25519"}
  },
  "id": 1
}' | nc -U /tmp/neural-api-nat0.sock

echo ""
echo "4. Cleaning up..."
kill $BEARDOG_PID $NEURAL_PID 2>/dev/null || true

echo "✅ Test complete!"
```

---

## 📋 Phase 2: Songbird Migration to capability.call (1.5 hours)

### Overview
Songbird must stop hardcoding BearDog method names and use Neural API's `capability.call` for all crypto operations.

### Implementation Steps

#### 1. Update BearDogClient to Use capability.call (45 min)

**File**: `crates/songbird-http-client/src/beardog_client.rs`

Replace the entire `call` method with semantic routing:

```rust
/// Call BearDog via Neural API's capability.call (semantic routing)
async fn call(&self, capability: &str, args: Value) -> Result<Value> {
    let id = self.request_id.fetch_add(1, Ordering::SeqCst);

    match &self.mode {
        BearDogMode::Direct { socket_path } => {
            // DEPRECATED: Direct mode for testing only
            // TODO: Remove after migration complete
            warn!("⚠️  Using deprecated direct mode - migrate to Neural API");
            
            let method = self.semantic_to_actual(capability)?;
            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                method: method.to_string(),
                params: args,
                id,
            };

            // ... existing direct RPC code ...
        }
        BearDogMode::NeuralApi { socket_path } => {
            // TRUE PRIMAL: Use capability.call for semantic routing
            let request = json!({
                "jsonrpc": "2.0",
                "method": "capability.call",
                "params": {
                    "capability": capability.split('.').next().unwrap_or("crypto"),
                    "operation": capability.split('.').nth(1).unwrap_or(capability),
                    "args": args
                },
                "id": id
            });

            trace!("→ Neural API capability.call: {} (id={})", capability, id);

            // Connect to Neural API
            let mut stream = UnixStream::connect(socket_path).await.map_err(|e| {
                Error::BearDogRpc(format!("Failed to connect to Neural API at {}: {}", socket_path, e))
            })?;

            // Send request
            let request_str = serde_json::to_string(&request)
                .map_err(|e| Error::BearDogRpc(format!("Failed to serialize request: {}", e)))?;
            
            stream.write_all(request_str.as_bytes()).await
                .map_err(|e| Error::BearDogRpc(format!("Failed to write request: {}", e)))?;
            stream.write_all(b"\n").await
                .map_err(|e| Error::BearDogRpc(format!("Failed to write newline: {}", e)))?;

            // Read response
            let mut response = String::new();
            stream.read_to_string(&mut response).await
                .map_err(|e| Error::BearDogRpc(format!("Failed to read response: {}", e)))?;

            // Parse response
            let response_json: Value = serde_json::from_str(&response)
                .map_err(|e| Error::BearDogRpc(format!("Invalid JSON response: {}", e)))?;

            // Check for error
            if let Some(error) = response_json.get("error") {
                return Err(Error::BearDogRpc(format!("Neural API error: {}", error)));
            }

            // Extract result
            response_json.get("result")
                .cloned()
                .ok_or_else(|| Error::BearDogRpc("Missing result in response".to_string()))
        }
    }
}
```

#### 2. Remove Hardcoded Method Mappings (15 min)

**File**: `crates/songbird-http-client/src/crypto/beardog_provider.rs`

```rust
/// DEPRECATED: Remove after migration to capability.call
/// 
/// This semantic_to_actual mapping is NO LONGER NEEDED.
/// Neural API now handles all semantic translation via capability.call.
#[deprecated(since = "0.2.0", note = "Use Neural API's capability.call instead")]
fn semantic_to_actual<'a>(&self, method: &'a str) -> &'a str {
    // Keep for backward compatibility with direct mode only
    // Will be removed in v0.3.0
    match method {
        // These mappings are now in Neural API's capability registry
        "crypto.generate_keypair" => "crypto.x25519_generate_ephemeral",
        "crypto.ecdh_derive" => "crypto.x25519_derive_secret",
        // ... etc (keep for direct mode compatibility)
        other => other,
    }
}
```

#### 3. Update Default Mode to Neural API (5 min)

**File**: `crates/songbird-http-client/src/beardog_client.rs`

```rust
pub fn from_env() -> Self {
    let mode = std::env::var("BEARDOG_MODE").unwrap_or_else(|_| "neural".to_string());

    match mode.as_str() {
        "direct" => {
            warn!("⚠️  BEARDOG_MODE=direct is DEPRECATED - use 'neural' for production");
            let socket = std::env::var("BEARDOG_SOCKET")
                .unwrap_or_else(|_| "/tmp/beardog.sock".to_string());
            Self::new_direct(socket)
        }
        _ => {
            // Default to Neural API (TRUE PRIMAL pattern)
            let socket = std::env::var("NEURAL_API_SOCKET")
                .unwrap_or_else(|_| "/tmp/neural-api-nat0.sock".to_string());
            info!("🌐 BearDogClient: Neural API mode (TRUE PRIMAL) → {}", socket);
            Self::new_neural_api(socket)
        }
    }
}
```

#### 4. Test Migration (25 min)

**Test Script**: `test_songbird_capability_call.sh`

```bash
#!/bin/bash
set -e

echo "🧪 Testing Songbird capability.call Migration"
echo "=============================================="

# Start full stack
echo "1. Starting Neural API..."
cd ~/Development/ecoPrimals/phase2/biomeOS
./target/release/biomeos neural-api &
NEURAL_PID=$!
sleep 3

echo "2. Starting BearDog (with auto-registration)..."
cd ~/Development/ecoPrimals/phase1/beardog
export NEURAL_API_SOCKET="/tmp/neural-api-nat0.sock"
./target/release/beardog server --socket /tmp/beardog-nat0.sock &
BEARDOG_PID=$!
sleep 3

echo "3. Starting Songbird (Neural API mode)..."
cd ~/Development/ecoPrimals/phase1/songbird
export BEARDOG_MODE="neural"
export NEURAL_API_SOCKET="/tmp/neural-api-nat0.sock"
./target/release/songbird server --socket /tmp/songbird-nat0.sock &
SONGBIRD_PID=$!
sleep 4

echo "4. Testing GitHub API via Tower Atomic..."
echo '{
  "jsonrpc": "2.0",
  "method": "http.request",
  "params": {
    "method": "GET",
    "url": "https://api.github.com/zen",
    "headers": {"User-Agent": "ecoPrimals/1.0"}
  },
  "id": 1
}' | nc -U /tmp/songbird-nat0.sock

echo ""
echo "5. Cleaning up..."
kill $SONGBIRD_PID $BEARDOG_PID $NEURAL_PID 2>/dev/null || true

echo "✅ Test complete!"
```

---

## 📋 Phase 3: Validation & Documentation (30 min)

### Validation Checklist

- [ ] BearDog registers capabilities on startup
- [ ] Songbird uses `capability.call` for all crypto operations
- [ ] Neural API routes requests correctly
- [ ] No hardcoded method names between primals
- [ ] GitHub API returns 200 OK via Pure Rust TLS 1.3
- [ ] All tests pass

### Success Criteria

```bash
# This should work end-to-end:
1. Start Neural API
2. Start BearDog (auto-registers)
3. Start Songbird (uses capability.call)
4. Call Songbird's http.request
5. Get 200 OK from GitHub

# Zero hardcoding:
- Songbird doesn't know BearDog's method names
- BearDog can evolve its API freely
- Neural API handles all translation
```

---

## 🎯 Timeline Summary

| Team | Task | Time | Dependencies |
|------|------|------|--------------|
| **BearDog** | Create registration module | 30 min | None |
| **BearDog** | Integrate into startup | 15 min | Registration module |
| **BearDog** | Update module exports | 5 min | - |
| **BearDog** | Test auto-registration | 30 min | Neural API running |
| **Songbird** | Update BearDogClient | 45 min | None |
| **Songbird** | Remove hardcoded mappings | 15 min | BearDogClient update |
| **Songbird** | Update default mode | 5 min | - |
| **Songbird** | Test migration | 25 min | BearDog registered |
| **Both** | Integration validation | 30 min | Both complete |
| **TOTAL** | **Parallel execution** | **2 hours** | Teams work in parallel |
| **TOTAL** | **Sequential execution** | **3 hours** | If one team at a time |

---

## 🚀 Deployment Strategy

### Development Phase (Now)
```bash
# Test with all components local
NEURAL_API_SOCKET=/tmp/neural-api-nat0.sock
BEARDOG_MODE=neural  # Songbird uses Neural API
```

### Production Phase (After validation)
```bash
# Full Tower Atomic via graph deployment
biomeos deploy graphs/tower_atomic_bootstrap.toml

# Automatic:
- Neural API starts
- BearDog auto-registers
- Songbird auto-registers
- All routing via capability.call
```

---

## 📊 Benefits Summary

### Before (Hardcoded)
- ❌ Tight coupling between primals
- ❌ Every API change breaks consumers
- ❌ Manual coordination required
- ❌ Cannot evolve independently

### After (TRUE PRIMAL)
- ✅ Zero coupling between primals
- ✅ API changes are transparent
- ✅ No coordination needed
- ✅ Independent evolution
- ✅ Neural API provides versioning & fallbacks
- ✅ Production-ready architecture

---

## 🆘 Troubleshooting

### BearDog won't register
```bash
# Check Neural API is running
ls -la /tmp/neural-api-nat0.sock

# Check environment variable
echo $NEURAL_API_SOCKET

# Check logs
tail -f /tmp/beardog.log | grep -i "register\|neural"
```

### Songbird can't find BearDog
```bash
# Check BearDog registered
echo '{"jsonrpc":"2.0","method":"capability.list","id":1}' | \
  nc -U /tmp/neural-api-nat0.sock

# Should show "crypto" capability from beardog provider
```

### capability.call returns error
```bash
# Check semantic mapping
echo '{
  "jsonrpc": "2.0",
  "method": "capability.get",
  "params": {"capability": "crypto"},
  "id": 1
}' | nc -U /tmp/neural-api-nat0.sock

# Should show operations and mappings
```

---

## 📚 References

- [PRIMAL_IPC_PROTOCOL.md](../../wateringHole/PRIMAL_IPC_PROTOCOL.md)
- [SEMANTIC_METHOD_NAMING_STANDARD.md](../../wateringHole/SEMANTIC_METHOD_NAMING_STANDARD.md)
- [CAPABILITY_CALL_EVOLUTION_JAN_25_2026.md](./CAPABILITY_CALL_EVOLUTION_JAN_25_2026.md)
- [SONGBIRD_AUTO_REGISTRATION_HANDOFF.md](./SONGBIRD_AUTO_REGISTRATION_HANDOFF.md)

---

## ✅ Completion Criteria

When both teams complete this handoff:

1. ✅ **BearDog**: Auto-registers on startup, no code changes needed for new consumers
2. ✅ **Songbird**: Uses `capability.call`, no hardcoded BearDog knowledge
3. ✅ **Neural API**: Routes all crypto operations semantically
4. ✅ **Tower Atomic**: GitHub API returns 200 OK via Pure Rust TLS 1.3
5. ✅ **TRUE PRIMAL**: Zero coupling, ready for production deployment

---

**Expected Completion**: 2-3 hours (teams working in parallel)  
**Impact**: Production-ready Tower Atomic with TRUE PRIMAL architecture  
**Next Phase**: Comprehensive validation against 60+ HTTPS endpoints

---

*Created by biomeOS Architect | January 25, 2026*  
*For Tower Atomic Evolution | Phase 1 → Phase 2 Migration*

