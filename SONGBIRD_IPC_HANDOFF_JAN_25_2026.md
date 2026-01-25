# 🐦 Songbird IPC Integration - Handoff Document

**Date**: January 25, 2026  
**From**: biomeOS Team  
**To**: Songbird Team  
**Priority**: 🔴 **P0** - Blocks GitHub connectivity for all primals

---

## 🎯 **REQUEST SUMMARY**

**What We Need**: Expose Songbird's Pure Rust HTTP/HTTPS client via Unix socket JSON-RPC

**Why**: Enable all primals to make HTTPS requests via Tower Atomic without C dependencies

**Timeline**: 1 day

---

## 📊 **CURRENT STATUS**

### ✅ What Songbird Has (Library Level)

**Achievement**: Songbird v5.24.0 has **complete Pure Rust TLS 1.3 + HTTPS**

**Validation** (Jan 23-25, 2026):
- ✅ TLS 1.3 handshake complete (19-28ms)
- ✅ HTTP 200 OK from google.com, cloudflare.com, example.com
- ✅ BearDog integration via RPC
- ✅ Zero C dependencies (at library level)
- ✅ Application data encryption/decryption working

**Files**:
- `songbird-http-client/src/lib.rs` - Pure Rust HTTP client
- `songbird-http-client/src/tls.rs` - TLS 1.3 implementation
- `songbird-http-client/src/beardog_client.rs` - BearDog crypto integration

**Status**: ✅ **COMPLETE** at library level

---

### ❌ What's Missing (IPC Level)

**Problem**: HTTP client is a **library**, not exposed via **Unix socket JSON-RPC**

**Impact**: Primals can't use Tower Atomic for HTTPS requests

**Current Workaround**: biomeOS was using `reqwest` (C dependencies) - **just removed!**

---

## 🚀 **WHAT WE NEED**

### 1. JSON-RPC Method: `http.request`

**Location**: `songbird-orchestrator/src/ipc/handlers/` (new file: `http.rs`)

**Method Signature**:

```rust
// Method: "http.request"
// Params:
{
    "method": "GET" | "POST" | "PUT" | "DELETE",
    "url": "https://api.github.com/...",
    "headers": {
        "Authorization": "Bearer ...",
        "Content-Type": "application/json"
    },
    "body": null | "base64-encoded-body"
}

// Response:
{
    "status": 200,
    "headers": {
        "content-type": "application/json",
        "content-length": "1234"
    },
    "body": "base64-encoded-response-body"
}
```

---

### 2. Implementation Template

**File**: `songbird-orchestrator/src/ipc/handlers/http.rs`

```rust
use anyhow::{Context, Result};
use serde_json::Value;
use songbird_http_client::{HttpClient, HttpMethod};
use crate::beardog::BearDogClient;

/// Handle HTTP/HTTPS requests via Pure Rust TLS 1.3
///
/// This is the IPC wrapper for Songbird's library-level HTTP client
pub async fn handle_http_request(
    beardog_client: &BearDogClient,
    params: Value,
) -> Result<Value> {
    // 1. Parse parameters
    let method_str = params["method"]
        .as_str()
        .context("Missing 'method' parameter")?;
    let url = params["url"]
        .as_str()
        .context("Missing 'url' parameter")?;
    
    let headers = params
        .get("headers")
        .and_then(|h| h.as_object())
        .map(|h| {
            h.iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                .collect()
        })
        .unwrap_or_default();
    
    let body = params
        .get("body")
        .and_then(|b| b.as_str())
        .map(|b| base64::decode(b))
        .transpose()
        .context("Failed to decode body")?;
    
    // 2. Parse HTTP method
    let method = match method_str {
        "GET" => HttpMethod::Get,
        "POST" => HttpMethod::Post,
        "PUT" => HttpMethod::Put,
        "DELETE" => HttpMethod::Delete,
        _ => return Err(anyhow!("Unsupported HTTP method: {}", method_str)),
    };
    
    // 3. Create HTTP client with BearDog crypto provider
    let client = HttpClient::new(beardog_client.clone());
    
    // 4. Make request
    let response = client
        .request(method, url, headers, body)
        .await
        .context("HTTP request failed")?;
    
    // 5. Return response
    Ok(serde_json::json!({
        "status": response.status,
        "headers": response.headers,
        "body": base64::encode(&response.body)
    }))
}
```

---

### 3. Wire Up in IPC Server

**File**: `songbird-orchestrator/src/ipc/server.rs` (or equivalent)

```rust
// Add to match statement in handle_rpc_request():

match method.as_str() {
    // ... existing methods ...
    
    "http.request" => {
        handle_http_request(&self.beardog_client, params)
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))
    }
    
    // Alternative semantic names (optional):
    "http.get" | "http.post" | "http.put" | "http.delete" => {
        // Extract method from RPC method name
        let http_method = method.split('.').nth(1).unwrap().to_uppercase();
        let mut params_with_method = params.clone();
        params_with_method["method"] = json!(http_method);
        
        handle_http_request(&self.beardog_client, params_with_method)
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))
    }
    
    _ => Err(format!("Unknown method: {}", method))
}
```

---

### 4. Update Capabilities Registration

**File**: `songbird-orchestrator/src/capabilities.rs` (or equivalent)

```rust
pub fn songbird_capabilities() -> Vec<String> {
    vec![
        // ... existing capabilities ...
        "secure_http".to_string(),
        "http.request".to_string(),
        "http.get".to_string(),
        "http.post".to_string(),
        "tls.1.3".to_string(),
    ]
}
```

---

## 🔧 **SEMANTIC METHOD NAMES (BONUS)**

### Current Issue

Songbird's internal BearDog client uses **old method names** (pre-semantic standard):

**File**: `songbird-http-client/src/beardog_client.rs`

**Current** (non-semantic):
```rust
- "x25519_generate_ephemeral"
- "x25519_derive_secret"
- "chacha20_poly1305_encrypt"
- "chacha20_poly1305_decrypt"
- "tls_derive_secrets"
```

**Should be** (semantic):
```rust
+ "crypto.x25519_generate_ephemeral"
+ "crypto.x25519_derive_secret"
+ "crypto.chacha20_poly1305_encrypt"
+ "crypto.chacha20_poly1305_decrypt"
+ "tls.derive_secrets"
```

**Impact**: Low priority, but aligns with `SEMANTIC_METHOD_NAMING_STANDARD.md`

**Estimated Time**: 30 minutes

---

## 📋 **TESTING CHECKLIST**

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_http_get_google() {
        let beardog_client = BearDogClient::new(...);
        let params = json!({
            "method": "GET",
            "url": "https://www.google.com",
            "headers": {},
            "body": null
        });
        
        let response = handle_http_request(&beardog_client, params)
            .await
            .unwrap();
        
        assert_eq!(response["status"], 200);
    }
    
    #[tokio::test]
    async fn test_http_post_with_body() {
        let beardog_client = BearDogClient::new(...);
        let body = base64::encode(b"{\"test\": true}");
        let params = json!({
            "method": "POST",
            "url": "https://httpbin.org/post",
            "headers": {
                "Content-Type": "application/json"
            },
            "body": body
        });
        
        let response = handle_http_request(&beardog_client, params)
            .await
            .unwrap();
        
        assert_eq!(response["status"], 200);
    }
}
```

---

### Integration Tests (via Unix Socket)

```bash
# 1. Start BearDog
FAMILY_ID=test ./beardog-server &

# 2. Start Songbird with IPC
FAMILY_ID=test ./songbird-orchestrator &

# 3. Test http.request via Unix socket
echo '{
    "jsonrpc": "2.0",
    "method": "http.request",
    "params": {
        "method": "GET",
        "url": "https://www.google.com",
        "headers": {},
        "body": null
    },
    "id": 1
}' | nc -U /run/user/$(id -u)/songbird-test.sock

# Expected response:
{
    "jsonrpc": "2.0",
    "result": {
        "status": 200,
        "headers": { ... },
        "body": "..." (base64)
    },
    "id": 1
}
```

---

### End-to-End Test (via Neural API)

**biomeOS Side**:

```rust
// Test via Neural API routing
let response = call_unix_socket_rpc(
    "/run/user/1000/neural-api-test.sock",
    "neural_api.proxy_http",
    json!({
        "method": "GET",
        "url": "https://api.github.com/",
        "headers": {},
        "body": null
    })
).await?;

assert_eq!(response["status"], 200);
```

---

## 📂 **FILES TO MODIFY**

| File | Action | Priority |
|------|--------|----------|
| `songbird-orchestrator/src/ipc/handlers/http.rs` | **CREATE** | 🔴 P0 |
| `songbird-orchestrator/src/ipc/handlers/mod.rs` | Add `pub mod http;` | 🔴 P0 |
| `songbird-orchestrator/src/ipc/server.rs` | Add `http.request` to match | 🔴 P0 |
| `songbird-orchestrator/src/capabilities.rs` | Add `secure_http` | 🔴 P0 |
| `songbird-http-client/src/beardog_client.rs` | Fix semantic names | 🟡 P2 |
| `songbird-orchestrator/Cargo.toml` | Add `base64` dependency | 🔴 P0 |

---

## 🎯 **ACCEPTANCE CRITERIA**

### Must Have (P0)
1. ✅ `http.request` JSON-RPC method works via Unix socket
2. ✅ GET requests to https://www.google.com return 200
3. ✅ POST requests with body work
4. ✅ BearDog integration working (crypto via RPC)
5. ✅ Response includes status, headers, body (base64)
6. ✅ Error handling for invalid URLs, timeouts, etc.

### Nice to Have (P2)
1. ⏳ Semantic method names (`crypto.*`, `tls.*`)
2. ⏳ Alternative methods: `http.get`, `http.post`, etc.
3. ⏳ Request/response logging for metrics
4. ⏳ Timeout configuration

---

## 📈 **TIMELINE**

### Day 1 (Implementation)
- **Morning** (2-3 hours):
  - Create `http.rs` handler
  - Wire up in IPC server
  - Add capabilities registration
  
- **Afternoon** (2-3 hours):
  - Unit tests
  - Integration tests via Unix socket
  - Bug fixes

- **Evening** (1 hour):
  - End-to-end test with biomeOS
  - Documentation

### Day 2 (Optional - Semantic Names)
- **Morning** (30 minutes):
  - Fix semantic method names in `beardog_client.rs`
  - Test

---

## 🔗 **REFERENCES**

### Standards
- **WateringHole**: `wateringHole/PRIMAL_IPC_PROTOCOL.md`
- **Semantic Naming**: `wateringHole/SEMANTIC_METHOD_NAMING_STANDARD.md`
- **ecoBin Standard**: `wateringHole/ECOBIN_ARCHITECTURE_STANDARD.md`

### biomeOS Documentation
- **Tower Atomic Status**: `TOWER_ATOMIC_GITHUB_STATUS_JAN_25_2026.md`
- **Neural API Evolution**: `NEURAL_API_HTTP_EVOLUTION_JAN_25_2026.md`
- **Deep Debt Complete**: `archive/session_jan_25_2026_deep_debt/`

### Songbird Files
- **HTTP Client**: `songbird-http-client/src/lib.rs`
- **TLS Implementation**: `songbird-http-client/src/tls.rs`
- **BearDog Client**: `songbird-http-client/src/beardog_client.rs`

---

## 🤝 **COORDINATION**

### Contact Points
- **biomeOS Team**: Available for testing and integration
- **BearDog Team**: Crypto methods already implemented and working

### Communication
- **Questions**: Ask in WateringHole or direct message
- **Testing Support**: biomeOS can provide test harness
- **Integration Testing**: Joint session when ready

### Handoff Back
When complete, please notify biomeOS team with:
1. ✅ Unix socket path format (e.g., `/run/user/{uid}/songbird-{family}.sock`)
2. ✅ JSON-RPC method name(s) implemented
3. ✅ Parameter/response schema
4. ✅ Any error codes or edge cases
5. ✅ Testing results

---

## 🎉 **IMPACT**

### Unblocks
- ✅ GitHub API access for all primals (Squirrel, biomeOS, etc.)
- ✅ Anthropic API access (Claude, etc.)
- ✅ Any external HTTPS API
- ✅ ecoBin compliance (Pure Rust stack)
- ✅ True primal architecture (capability-based)

### Enables
- ✅ Tower Atomic production deployment
- ✅ Neural API routing layer
- ✅ Semantic translation layer validation
- ✅ Metrics & learning system
- ✅ Fault tolerance & failover

### Long-Term Value
- ✅ **20x faster development** validated (TLS 1.3 in 3 weeks vs 15 months)
- ✅ **Pure Rust** - zero C dependencies
- ✅ **Isomorphic evolution** - change impl without breaking contracts
- ✅ **Self-correcting** - semantic layer catches mismatches

---

## 📝 **SUMMARY**

### What You Need to Do
1. Create `http.rs` handler with `handle_http_request()` function
2. Wire up `http.request` JSON-RPC method in IPC server
3. Add `secure_http` to capabilities registration
4. Test via Unix socket
5. Notify biomeOS when ready

### What biomeOS Will Do
1. Wait for your implementation
2. Test end-to-end via Neural API
3. Deploy Tower Atomic to production
4. Validate GitHub connectivity

### Timeline
- **Your work**: 1 day
- **Integration testing**: 1 day
- **Production deployment**: Same week

**Total: 2-3 days to GitHub connectivity!** 🚀

---

**🦀✨ Pure Rust TLS 1.3 | Tower Atomic | TRUE PRIMAL Pattern ✨🦀**

**Priority**: 🔴 **P0** - Blocks all external API access  
**Estimated Time**: 1 day  
**Impact**: HIGH - Unblocks entire ecosystem

**Questions?** Ask in WateringHole or ping biomeOS team!

---

**Prepared by**: biomeOS Team  
**Date**: January 25, 2026  
**Status**: Ready for implementation
