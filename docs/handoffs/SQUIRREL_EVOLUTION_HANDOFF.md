# Squirrel Evolution Handoff
## biomeOS Neural API Integration

**Date**: January 28, 2026 (Final)  
**From**: biomeOS Team  
**To**: Squirrel Team  
**Status**: ✅ **ALL 4 FIXES COMPLETE** (commit 28e59176)

---

## Summary

We've been testing Squirrel's integration with the biomeOS Neural API capability routing system. The Tower Atomic stack (BearDog + Songbird + Neural API) is fully operational, and we've identified some evolution opportunities in Squirrel's capability discovery that would enable seamless AI provider initialization.

## Current State

### Working
- ✅ Squirrel starts and listens on Unix socket
- ✅ JSON-RPC methods respond correctly (`list_providers`, `query_ai`, etc.)
- ✅ `CAPABILITY_REGISTRY_SOCKET` env var is read correctly
- ✅ `ANTHROPIC_API_KEY` env var is read correctly

### Not Working
- ❌ Anthropic/OpenAI adapters fail to initialize (no providers found)
- ❌ `http.request` capability discovery times out

## Root Cause Analysis

### 🚨 CRITICAL: Issue 0: HTTP Body Format Mismatch

**File**: `crates/main/src/api/ai/adapters/anthropic.rs` (and `openai.rs`)  
**Function**: `delegate_http()` (lines 112-151)

**Problem**: Squirrel sends `body` as a JSON object, but Songbird expects a **string**.

**Squirrel sends**:
```json
{
  "method": "http.request",
  "params": {
    "method": "POST",
    "url": "https://api.anthropic.com/v1/messages",
    "headers": {...},
    "body": {"model": "claude-3-opus", ...}  // ❌ OBJECT!
  }
}
```

**Songbird expects**:
```json
{
  "method": "http.request",
  "params": {
    "method": "POST", 
    "url": "https://api.anthropic.com/v1/messages",
    "headers": {...},
    "body": "{\"model\": \"claude-3-opus\", ...}"  // ✅ STRING!
  }
}
```

**Error**: `Invalid params: invalid type: map, expected a string`

**Fix in `delegate_http()`** (line 120):
```rust
// BEFORE (broken):
"body": body,

// AFTER (fixed):
"body": match body {
    serde_json::Value::String(s) => serde_json::Value::String(s),
    serde_json::Value::Null => serde_json::Value::Null,
    other => serde_json::Value::String(serde_json::to_string(&other)?),
},
```

**Verification Test**:
```bash
# This works (body as string):
python3 -c "
import socket, json
sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
sock.connect('/run/user/1000/biomeos/songbird-nat0.sock')
req = {'jsonrpc': '2.0', 'method': 'http.request', 'params': {
    'method': 'POST',
    'url': 'https://api.anthropic.com/v1/messages',
    'headers': {'Content-Type': 'application/json'},
    'body': '{\"test\": true}'  # STRING
}, 'id': 1}
sock.sendall((json.dumps(req) + '\n').encode())
print(sock.recv(4096).decode())
"
```

---

### Issue 1: Registry Query Missing Timeout

**File**: `crates/main/src/capabilities/discovery.rs`  
**Function**: `query_registry()` (lines 303-356)

The registry query has no timeout on the `read_line()` call:

```rust
// Line 331 - NO TIMEOUT
reader.read_line(&mut response_line).await?;
```

Compare with `probe_socket()` which correctly uses a timeout:

```rust
// Lines 243-247 - HAS TIMEOUT
match tokio::time::timeout(
    std::time::Duration::from_secs(2),
    reader.read_line(&mut response_line),
)
```

**Recommendation**: Add 2s timeout to registry query read_line:

```rust
match tokio::time::timeout(
    std::time::Duration::from_secs(2),
    reader.read_line(&mut response_line),
).await {
    Ok(Ok(_)) => { /* parse response */ }
    Ok(Err(e)) => return Err(DiscoveryError::ProbeFailed(format!("Read error: {}", e))),
    Err(_) => return Err(DiscoveryError::ProbeFailed("Registry query timeout".to_string())),
}
```

### Issue 2: Explicit Env Var Requires Probe

**File**: `crates/main/src/capabilities/discovery.rs`  
**Function**: `try_explicit_env()` (lines 103-133)

When `HTTP_REQUEST_PROVIDER_SOCKET=/tmp/songbird-nat0.sock` is set, the discovery:
1. Connects to the socket
2. Calls `discover_capabilities` method
3. Waits for response with capabilities list

**Problem**: Songbird doesn't implement `discover_capabilities`. The probe times out after 2s.

**Recommendation**: Trust explicit env vars without probing:

```rust
async fn try_explicit_env(capability: &str) -> Result<Option<CapabilityProvider>, DiscoveryError> {
    let env_var = format!(
        "{}_PROVIDER_SOCKET",
        capability.to_uppercase().replace('.', "_")
    );

    if let Ok(socket_path) = std::env::var(&env_var) {
        let path = PathBuf::from(&socket_path);

        // Verify socket exists
        if path.exists() {
            info!("✅ Found {} via env var {} = {}", capability, env_var, socket_path);
            
            // Trust the env var - operator knows what they're doing
            // Skip probe since not all primals support discover_capabilities
            return Ok(Some(CapabilityProvider {
                id: format!("{}-provider", capability),
                capabilities: vec![capability.to_string()],
                socket: path,
                metadata: HashMap::new(),
                discovered_via: format!("env:{}", env_var),
            }));
        }
    }

    Ok(None)
}
```

### Issue 3: Adapter Timeout Budget

**File**: `crates/main/src/api/ai/router.rs`  
**Function**: AI router initialization (lines 91-174)

The adapter has a 2s timeout for `is_available()`:

```rust
if let Ok(available) = tokio::time::timeout(
    std::time::Duration::from_secs(2),
    adapter.is_available()
).await {
```

But `is_available()` calls `discover_capability("http.request")` which tries:
1. Env var check (fast)
2. Registry query (can hang if no timeout)
3. Socket scan (up to 5s)

**Recommendation**: Increase timeout to 5s or add early-exit on registry success:

```rust
if let Ok(available) = tokio::time::timeout(
    std::time::Duration::from_secs(5),  // Increased from 2s
    adapter.is_available()
).await {
```

## Quick Fix for Testing

Until these changes are made, you can work around by using `AI_PROVIDER_SOCKETS`:

```bash
# Skip capability discovery entirely - use direct socket connection
AI_PROVIDER_SOCKETS=/tmp/ai-provider.sock \
ANTHROPIC_API_KEY=sk-... \
squirrel server --socket /tmp/squirrel-nat0.sock
```

This path doesn't use capability discovery and connects directly.

## Neural API Integration Notes

### Capability Registration

The Neural API now registers `http.request` (not just `http`) for Songbird:

```bash
# Query works correctly:
echo '{"jsonrpc":"2.0","method":"neural_api.discover_capability","params":{"capability":"http.request"},"id":1}' | nc -U /tmp/neural-api.sock

# Response:
{
  "result": {
    "capability": "http.request",
    "primary_socket": "/tmp/songbird-nat0.sock",
    "primals": [{"name": "songbird", "socket": "/tmp/songbird-nat0.sock"}]
  }
}
```

### Graph Configuration

The `tower_atomic_bootstrap.toml` graph includes Squirrel capability translations:

```toml
[[nodes]]
id = "register_squirrel"
capabilities = ["ai", "inference", "learning"]

[nodes.capabilities_provided]
"ai.query" = "query_ai"
"ai.list_providers" = "list_providers"
"ai.health" = "health"
"ai.ping" = "ping"
```

## Testing Commands

```bash
# Start the ecosystem
./deploy_tower_atomic.sh

# Verify Neural API has http.request registered
echo '{"jsonrpc":"2.0","method":"capability.list","id":1}' | nc -U /tmp/neural-api.sock

# Test Squirrel with workaround
CAPABILITY_REGISTRY_SOCKET=/tmp/neural-api.sock \
ANTHROPIC_API_KEY=... \
./squirrel server --socket /tmp/squirrel-nat0.sock

# Test AI query
echo '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"Hello!","provider":"anthropic"},"id":1}' | nc -U /tmp/squirrel-nat0.sock
```

## Files to Review

1. `crates/main/src/capabilities/discovery.rs` - Main discovery logic
2. `crates/main/src/api/ai/router.rs` - AI adapter initialization
3. `crates/main/src/api/ai/adapters/anthropic.rs` - Anthropic adapter

## Priority

**High** - These changes enable Squirrel to work seamlessly with the biomeOS Neural API ecosystem for AI workloads.

## Contact

For questions about the Neural API or capability routing, reach out to the biomeOS team.

---

*This document was generated during the biomeOS Deep Debt Evolution session.*

