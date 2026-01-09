# 🔍 Protocol Mismatch Deep Debt Analysis

**Date**: January 6, 2026 - 01:30 EST  
**Severity**: 🔴 **CRITICAL** - Blocks genetic lineage trust  
**Issue**: Songbird sends HTTP, BearDog expects JSON-RPC  
**Scope**: Phase1 primals (Songbird + BearDog teams)

---

## 🎯 Executive Summary

**Problem**: Tower 2 discovered that Songbird and BearDog cannot communicate via Unix sockets due to a protocol mismatch.

**Root Cause**: Architectural assumption mismatch between primals:
- **Songbird**: Uses `reqwest::Client` (HTTP protocol) for ALL endpoints (including Unix sockets)
- **BearDog**: Unix socket IPC server expects pure JSON-RPC 2.0 (no HTTP headers)

**Impact**: Genetic lineage trust cannot function, falling back to anonymous trust.

**Owner**: **Songbird team** (Songbird's SecurityAdapter needs protocol detection)

**Alternative**: BearDog team could add HTTP-over-Unix support (more complex)

---

## 🔬 Technical Analysis

### BearDog's Unix Socket Server

**File**: `phase1/beardog/crates/beardog-tunnel/src/unix_socket_ipc.rs`

**Lines 42-70**: JSON-RPC 2.0 request/response structures

**Expected Format**:
```json
{
  "jsonrpc": "2.0",
  "method": "evaluate_trust",
  "params": { "peer_id": "tower1", "family": "nat0" },
  "id": 1
}
```

**Key Code**:
```rust
/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Option<serde_json::Value>,
    pub id: Option<serde_json::Value>,
}
```

**Protocol**: Pure JSON-RPC 2.0 (NO HTTP headers!)

---

### Songbird's Security Adapter

**File**: `phase1/songbird/crates/songbird-universal/src/adapters/security.rs`

**Lines 200-288**: SecurityAdapter implementation

**Lines 306-315**: collect_metrics() method - sends HTTP GET

**Key Code**:
```rust
pub struct SecurityAdapter {
    /// Endpoint URL for the security capability provider
    endpoint: String,
    /// HTTP client for requests
    client: reqwest::Client,  // ← ALWAYS HTTP!
    /// Request timeout
    timeout: Duration,
}

pub async fn collect_metrics(&self) -> SongbirdResult<SecurityMetrics> {
    let url = format!("{}/metrics/security", self.endpoint);
    
    // ❌ reqwest sends HTTP headers even for unix:// URLs!
    let response = self.client.get(&url)
        .timeout(self.timeout)
        .send().await?;
    // ...
}
```

**Protocol**: HTTP/1.1 or HTTP/2 (always, even over Unix sockets!)

---

## 🚨 The Mismatch

### What Songbird Sends (HTTP over Unix socket):

```
GET /metrics/security HTTP/1.1
Host: unix
User-Agent: reqwest/0.11
Accept: */*
Content-Length: 0

```

### What BearDog Expects (pure JSON-RPC):

```json
{"jsonrpc":"2.0","method":"get_metrics","params":{},"id":1}
```

### What BearDog Receives:

```
GET /metrics/security HTTP/1.1\r\n
Host: unix\r\n
...
```

### What BearDog Does:

```rust
// Tries to parse as JSON-RPC
let request: JsonRpcRequest = serde_json::from_str(&line)?;
// ❌ FAILS: "GET /metrics/security HTTP/1.1" is not valid JSON!
```

### Error Result:

```
ERROR beardog_tunnel::unix_socket_ipc: ❌ Request handler error: 
  Failed to parse JSON-RPC request
```

---

## 💡 Solution Options

### Option A: Songbird Protocol Detection (Recommended) ✅

**Owner**: **Songbird team**

**File**: `songbird-universal/src/adapters/security.rs`

**Change**: Detect `unix://` URLs and use JSON-RPC client instead of HTTP client

**Implementation**:
```rust
pub struct SecurityAdapter {
    endpoint: String,
    protocol: Protocol,  // ← NEW!
    timeout: Duration,
}

enum Protocol {
    Http(reqwest::Client),
    JsonRpc(JsonRpcClient),  // ← NEW!
}

impl SecurityAdapter {
    pub fn new(endpoint: String) -> SongbirdResult<Self> {
        let protocol = if endpoint.starts_with("unix://") {
            // Use JSON-RPC for Unix sockets
            Protocol::JsonRpc(JsonRpcClient::new(endpoint)?)
        } else {
            // Use HTTP for network endpoints
            Protocol::Http(reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()?)
        };
        
        Ok(Self { endpoint, protocol, timeout: Duration::from_secs(5) })
    }
    
    pub async fn collect_metrics(&self) -> SongbirdResult<SecurityMetrics> {
        match &self.protocol {
            Protocol::Http(client) => {
                // Existing HTTP code
                let url = format!("{}/metrics/security", self.endpoint);
                let response = client.get(&url).timeout(self.timeout).send().await?;
                response.json().await
            }
            Protocol::JsonRpc(client) => {
                // NEW: JSON-RPC code
                let request = json!({
                    "jsonrpc": "2.0",
                    "method": "get_metrics",
                    "params": {},
                    "id": 1
                });
                let response = client.call(request).await?;
                serde_json::from_value(response["result"].clone())
            }
        }
    }
}
```

**Pros**:
- ✅ True port-free architecture
- ✅ Songbird becomes protocol-agnostic
- ✅ Matches Songbird's "sovereignty" principle (no vendor hardcoding)
- ✅ BearDog unchanged (minimal disruption)
- ✅ Fractal-safe (Unix sockets scale better than HTTP ports)

**Cons**:
- ⚠️ Requires new JSON-RPC client implementation
- ⚠️ Adds complexity to SecurityAdapter

**Effort**: 2-4 hours (implement JsonRpcClient + tests)

---

### Option B: BearDog HTTP-over-Unix Support (Alternative)

**Owner**: **BearDog team**

**File**: `beardog-tunnel/src/unix_socket_ipc.rs`

**Change**: Parse both HTTP and JSON-RPC in Unix socket server

**Implementation**:
```rust
async fn handle_connection(
    stream: UnixStream,
    btsp_provider: Arc<BeardogBtspProvider>
) -> Result<()> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    
    // ← NEW: Detect protocol
    let response = if line.starts_with("GET ") || line.starts_with("POST ") {
        // HTTP protocol - parse HTTP request, extract method/body
        let http_request = parse_http_request(&mut reader, line).await?;
        handle_http_request(http_request, &btsp_provider).await?
    } else {
        // JSON-RPC protocol (existing code)
        handle_jsonrpc_request(&line, &btsp_provider).await?
    };
    
    writer.write_all(response.as_bytes()).await?;
    Ok(())
}
```

**Pros**:
- ✅ Songbird unchanged (minimal disruption)
- ✅ BearDog becomes protocol-agnostic
- ✅ Matches BearDog's "multi-protocol" principle

**Cons**:
- ⚠️ More complex server (HTTP parsing, headers, etc.)
- ⚠️ Mixes concerns (IPC shouldn't need HTTP overhead)
- ⚠️ Performance overhead (HTTP parsing on every request)

**Effort**: 3-5 hours (implement HTTP parser + tests)

---

### Option C: Temporary HTTP Port Workaround ❌

**NOT RECOMMENDED** - Violates port-free architecture!

**Why NOT**:
- ❌ Defeats port-free architecture
- ❌ Not fractal-safe (port collisions with multiple instances)
- ❌ Not isomorphic (different config for single vs multi-instance)
- ❌ Less secure (TCP sockets vs Unix sockets)

**Only use if**: Neither team can implement A or B, and genetic lineage is urgently needed.

---

## 🎯 Recommendation

**UPDATE**: **Both teams evolving to dual-protocol support!** 🚀

**New Strategy**: tarpc + JSON-RPC
- **tarpc**: Type-safe, high-performance (production)
- **JSON-RPC**: Flexible, debuggable (development)

**Rationale**:
1. **Type Safety**: tarpc provides compile-time contracts (Rust ↔ Rust)
2. **Flexibility**: JSON-RPC enables cross-language, debugging, dynamic discovery
3. **Fractal Deployment**: Choose protocol per layer (core = tarpc, edge = JSON-RPC)
4. **Isomorphic**: Same code, different protocol config

**See**: `DUAL_PROTOCOL_EVOLUTION.md` for complete strategy

---

## 📋 Handoff to Songbird Team

### Task: Implement JSON-RPC Client for Unix Socket Endpoints

**File**: Create `songbird-universal/src/jsonrpc_client.rs`

**Requirements**:
1. Async JSON-RPC 2.0 client over Unix sockets
2. Compatible with `reqwest::Client` interface (drop-in replacement)
3. Support standard methods: `call()`, `call_with_timeout()`
4. Error handling compatible with `SongbirdError`

**Integration Point**: `songbird-universal/src/adapters/security.rs`

**Detection Logic**:
```rust
if endpoint.starts_with("unix://") {
    Protocol::JsonRpc(JsonRpcClient::new(endpoint)?)
} else {
    Protocol::Http(reqwest::Client::new())
}
```

**Testing**: Add tests for both HTTP and JSON-RPC protocols in `security_tests.rs`

**Acceptance Criteria**:
- ✅ `SECURITY_ENDPOINT=unix:///tmp/beardog-nat0-tower1.sock` works
- ✅ `SECURITY_ENDPOINT=http://127.0.0.1:9000` still works
- ✅ Protocol automatically detected (no config flag needed)
- ✅ All existing tests pass
- ✅ New tests for JSON-RPC protocol added

---

## 🧪 Test Scenario

### Setup:
1. Start BearDog with Unix socket: `/tmp/beardog-nat0-tower1.sock`
2. Start Songbird with `SECURITY_ENDPOINT=unix:///tmp/beardog-nat0-tower1.sock`

### Expected Behavior (After Fix):
```
# Songbird detects unix:// scheme
✅ SECURITY_ENDPOINT set: unix:///tmp/beardog-nat0-tower1.sock
✅ Protocol detected: JSON-RPC over Unix socket
🔌 Connecting to BearDog via Unix socket

# Songbird sends JSON-RPC request
→ {"jsonrpc":"2.0","method":"evaluate_trust","params":{...},"id":1}

# BearDog receives and parses successfully
✅ Received JSON-RPC request: evaluate_trust
🧬 Evaluating genetic lineage for peer: tower2

# BearDog responds
← {"jsonrpc":"2.0","result":{"trust_level":2,"reason":"genetic_lineage_verified"},"id":1}

# Songbird parses response
✅ Trust evaluation successful: level 2 (genetic lineage verified)
✅ Trust Decision: ACCEPT (reason: genetic_lineage_verified)
```

---

## 📊 Impact Assessment

### Current State (Without Fix)

| Component | Status | Impact |
|-----------|--------|--------|
| Discovery | ✅ Working | UDP multicast functional |
| Federation | ✅ Working | Anonymous trust fallback |
| BearDog Socket | ✅ Created | Unix socket listening |
| Songbird Connection | ⚠️ Fails | Protocol mismatch |
| Genetic Lineage | ❌ Blocked | Trust evaluation unavailable |
| Trust Level | 1 (Limited) | BirdSong coordination only |

**Result**: 5/6 working, genetic lineage blocked

### After Fix (Option A)

| Component | Status | Impact |
|-----------|--------|--------|
| Discovery | ✅ Working | Same |
| Federation | ✅ Working | Same |
| BearDog Socket | ✅ Created | Same |
| Songbird Connection | ✅ **Working** | **Protocol matched!** |
| Genetic Lineage | ✅ **Working** | **Trust evaluation functional!** |
| Trust Level | 2+ (Full) | **All capabilities enabled!** |

**Result**: 6/6 working, genetic lineage operational

---

## 🔄 Dependency Chain

```
biomeOS Tower
    │
    ├─ BearDog (v0.15.0) ✅ Ready
    │   └─ Unix socket IPC (JSON-RPC) ✅ Working
    │
    └─ Songbird (v3.10.3) ⚠️ Needs Fix
        └─ SecurityAdapter → ❌ HTTP-only
           └─ FIX: Add JSON-RPC support ← BLOCKS genetic lineage
```

**Blocker**: Songbird's SecurityAdapter HTTP-only implementation

**Unblocks**: Full genetic lineage trust validation

---

## 📚 Related Documentation

**BearDog**:
- `phase1/beardog/PORT_FREE_EVOLUTION_COMPLETE.md` - Port-free architecture
- `phase1/beardog/MULTI_PROTOCOL_GUIDE.md` - Multi-protocol support
- `phase1/beardog/crates/beardog-api/src/jsonrpc.rs` - JSON-RPC implementation reference

**Songbird**:
- `phase1/songbird/IPC_INTEGRATION_GUIDE.md` - IPC integration
- `phase1/songbird/crates/songbird-universal/src/adapters/` - Adapter architecture
- `phase1/songbird/DEEP_DEBT_EVOLUTION_PLAN.md` - Deep debt tracking

**biomeOS**:
- `phase2/biomeOS/docs/jan4-session/TOWER2_FEEDBACK_RESPONSE.md` - Tower 2's findings
- `phase2/biomeOS/docs/jan4-session/GENETIC_LINEAGE_READY.md` - Configuration updates

---

## ✅ Acceptance Criteria for Resolution

### Songbird Team Delivers:

1. ✅ `JsonRpcClient` implementation
2. ✅ Protocol detection in `SecurityAdapter::new()`
3. ✅ Both HTTP and JSON-RPC working
4. ✅ Tests for both protocols
5. ✅ Documentation updated

### biomeOS Tests:

1. ✅ Deploy Tower 1 with `SECURITY_ENDPOINT=unix://...`
2. ✅ Deploy Tower 2 with `SECURITY_ENDPOINT=unix://...`
3. ✅ Genetic lineage trust evaluation succeeds
4. ✅ Trust level upgrades from 1 to 2+
5. ✅ Full federation functional

### Success Metrics:

- **Federation time**: ~18-20 seconds (same as before)
- **Trust level**: 2+ (genetic lineage verified)
- **Protocol overhead**: <10ms for Unix socket JSON-RPC (vs 100ms+ for HTTP over TCP)
- **Port usage**: 0 (port-free architecture maintained!)

---

## 🎊 Summary

**Deep Debt Owner**: **Songbird team**

**Issue**: SecurityAdapter HTTP-only, needs JSON-RPC support for Unix sockets

**Fix**: Protocol detection + JsonRpcClient implementation

**Effort**: 2-4 hours

**Impact**: Unblocks genetic lineage trust, completes port-free architecture

**Status**: Ready for handoff to Songbird team

---

**Date**: January 6, 2026 - 01:30 EST  
**Discovered by**: Tower 2 testing team (excellent diagnosis!)  
**Analyzed by**: biomeOS integration team  
**Assigned to**: Songbird development team  
**Priority**: HIGH - Blocks genetic lineage trust validation  
**Architecture Impact**: Enables true port-free fractal deployment

