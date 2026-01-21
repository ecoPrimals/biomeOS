# 🐦🐕 Songbird + BearDog Tower Atomic HTTP Co-Evolution Handoff

**Date**: January 21, 2026  
**Status**: 🚨 **CRITICAL - BLOCKS AI INTEGRATION**  
**Teams**: Songbird Team + BearDog Team  
**Timeline**: 1-2 weeks (coordinated evolution)

---

## 🎯 MISSION

**Build a Pure Rust HTTP/HTTPS client for Tower Atomic that uses BearDog crypto and Songbird networking.**

Zero C dependencies. Zero OpenSSL. Zero ring. **TRUE Pure Rust.**

---

## 📐 ARCHITECTURE

### Tower Atomic HTTP Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    EXTERNAL AI API                          │
│              (Anthropic, OpenAI, etc.)                      │
└─────────────────────────▲───────────────────────────────────┘
                          │ HTTPS
                          │
┌─────────────────────────┴───────────────────────────────────┐
│                      SONGBIRD                               │
│              (TLS Handshake, Network I/O)                   │
│                                                              │
│  ┌────────────────────────────────────────────────┐        │
│  │  Pure Rust HTTP/HTTPS Client                   │        │
│  │  - hyper (HTTP protocol)                       │        │
│  │  - Custom TLS using BearDog crypto via RPC     │        │
│  │  - Zero C dependencies                         │        │
│  └────────────────────────────────────────────────┘        │
└──────────────────────────▲───────────────────────────────────┘
                           │ Unix Socket RPC
                           │ (JSON-RPC 2.0)
┌──────────────────────────┴───────────────────────────────────┐
│                      BEARDOG                                 │
│              (Pure Rust Crypto Operations)                   │
│                                                              │
│  RPC Methods for TLS:                                       │
│  - crypto.sign          (ed25519 signatures)                │
│  - crypto.verify        (ed25519 verification)              │
│  - crypto.derive_key    (x25519 key exchange)              │
│  - crypto.encrypt       (ChaCha20-Poly1305)                │
│  - crypto.decrypt       (ChaCha20-Poly1305)                │
│  - crypto.hash          (BLAKE3)                            │
│  - tls.handshake_sign   (TLS-specific signing)             │
│  - tls.derive_secrets   (TLS key derivation)               │
└─────────────────────────────────────────────────────────────┘
```

### Why This Architecture?

1. **BearDog**: Pure Rust crypto (ed25519, x25519, ChaCha20, BLAKE3)
2. **Songbird**: Pure Rust networking (TCP, TLS, HTTP)
3. **Separation of Concerns**: Crypto vs. Network
4. **True ecoBin**: Cross-compiles everywhere
5. **Zero Trust**: All crypto delegated to BearDog

---

## 📋 BEARDOG TEAM RESPONSIBILITIES

### 1. Implement TLS-Specific Crypto RPC Methods

**New RPC methods needed** (in addition to existing crypto methods):

#### Method: `tls.derive_secrets`

Derives TLS session secrets from pre-master secret.

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "tls.derive_secrets",
  "params": {
    "pre_master_secret": "base64_encoded_secret",
    "client_random": "base64_encoded_32_bytes",
    "server_random": "base64_encoded_32_bytes",
    "cipher_suite": "TLS_CHACHA20_POLY1305_SHA256"
  },
  "id": 1
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "master_secret": "base64_encoded_48_bytes",
    "client_write_key": "base64_encoded_key",
    "server_write_key": "base64_encoded_key",
    "client_write_iv": "base64_encoded_iv",
    "server_write_iv": "base64_encoded_iv"
  },
  "id": 1
}
```

#### Method: `tls.sign_handshake`

Signs TLS handshake messages.

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "tls.sign_handshake",
  "params": {
    "message": "base64_encoded_handshake_messages",
    "algorithm": "ed25519"
  },
  "id": 1
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "signature": "base64_encoded_signature"
  },
  "id": 1
}
```

#### Method: `tls.verify_certificate`

Verifies TLS certificate chain.

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "tls.verify_certificate",
  "params": {
    "certificate_chain": ["base64_cert1", "base64_cert2", ...],
    "server_name": "api.anthropic.com",
    "current_time_unix": 1737456000
  },
  "id": 1
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "valid": true,
    "public_key": "base64_encoded_public_key",
    "expiry": 1800000000
  },
  "id": 1
}
```

#### Method: `crypto.ecdh_derive`

Performs ECDH key exchange (x25519).

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "crypto.ecdh_derive",
  "params": {
    "our_private_key": "base64_encoded_32_bytes",
    "their_public_key": "base64_encoded_32_bytes"
  },
  "id": 1
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "shared_secret": "base64_encoded_32_bytes"
  },
  "id": 1
}
```

### 2. Document Crypto API for TLS

**File to create**: `phase1/beardog/docs/TLS_CRYPTO_API.md`

Document:
- What crypto operations are needed for TLS 1.3
- How Songbird should call each RPC method
- Performance expectations (< 1ms per crypto operation)
- Error handling
- Key lifecycle management

### 3. Performance Validation

Ensure RPC latency is acceptable for HTTPS:
- Target: < 1ms per crypto operation
- TLS handshake: < 10ms total (all crypto ops combined)
- Measure and document actual performance

### 4. Test Infrastructure

Create test harness:
- Mock TLS handshake sequences
- Verify correct crypto operations
- Stress test: 1000 concurrent TLS handshakes

---

## 📋 SONGBIRD TEAM RESPONSIBILITIES

### 1. Create `songbird-http-client` Crate

**Location**: `phase1/songbird/crates/songbird-http-client/`

**Dependencies** (Pure Rust only):
```toml
[dependencies]
hyper = { version = "1.0", features = ["client", "http1", "http2"] }
hyper-util = "0.1"
tokio = { version = "1.0", features = ["net", "rt"] }
tower = "0.4"
http-body-util = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
thiserror = "1.0"
base64 = "0.21"

# Tower Atomic for BearDog RPC
beardog-tower-atomic = { path = "../../../beardog/crates/beardog-tower-atomic" }
```

**NO `reqwest`, NO `rustls`, NO `ring`!**

### 2. Implement Pure Rust TLS Client

**File**: `phase1/songbird/crates/songbird-http-client/src/tls.rs`

**Key Components**:

```rust
/// Pure Rust TLS client that delegates crypto to BearDog
pub struct BearDogTlsClient {
    /// BearDog RPC client for crypto operations
    beardog_client: beardog_tower_atomic::Client,
    /// TCP stream
    tcp_stream: TcpStream,
}

impl BearDogTlsClient {
    /// Perform TLS 1.3 handshake using BearDog crypto
    pub async fn handshake(&mut self, server_name: &str) -> Result<()> {
        // 1. Send ClientHello
        // 2. Receive ServerHello
        // 3. Call BearDog for key derivation
        // 4. Call BearDog for signature verification
        // 5. Send Finished message
        // 6. Receive Finished message
        // 7. Establish encrypted channel
    }
    
    /// Encrypt application data using BearDog
    pub async fn encrypt(&mut self, plaintext: &[u8]) -> Result<Vec<u8>> {
        // Call BearDog's crypto.encrypt via RPC
    }
    
    /// Decrypt application data using BearDog
    pub async fn decrypt(&mut self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        // Call BearDog's crypto.decrypt via RPC
    }
}
```

### 3. Implement HTTP/HTTPS Client

**File**: `phase1/songbird/crates/songbird-http-client/src/client.rs`

```rust
/// Pure Rust HTTP/HTTPS client using BearDog crypto
pub struct SongbirdHttpClient {
    beardog_socket: PathBuf,
}

impl SongbirdHttpClient {
    /// Make HTTP/HTTPS request
    pub async fn request(
        &self,
        method: &str,
        url: &str,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> Result<HttpResponse> {
        // 1. Parse URL
        let url = url.parse::<hyper::Uri>()?;
        
        // 2. Connect via TCP
        let tcp = TcpStream::connect((host, port)).await?;
        
        // 3. If HTTPS, perform TLS handshake via BearDog
        if url.scheme() == "https" {
            let mut tls = BearDogTlsClient::new(tcp, beardog_socket).await?;
            tls.handshake(host).await?;
            // Use tls for HTTP protocol
        }
        
        // 4. Send HTTP request via hyper
        // 5. Receive HTTP response
        // 6. Return structured response
    }
}
```

### 4. Update `handle_http_request` RPC Method

**File**: `phase1/songbird/crates/songbird-orchestrator/src/ipc/server_pure_rust.rs`

**Replace lines 588-683** with:

```rust
async fn handle_http_request(params: Option<serde_json::Value>) -> Result<serde_json::Value, JsonRpcError> {
    use songbird_http_client::SongbirdHttpClient;
    
    // Parse params (same as before)
    let params: HttpRequestParams = ...;
    
    info!("🌐 HTTP delegation (Pure Rust): {} {}", params.method, params.url);
    
    // Get BearDog socket from environment
    let beardog_socket = std::env::var("SONGBIRD_SECURITY_PROVIDER")
        .unwrap_or_else(|_| "/tmp/beardog-nat0.sock".to_string());
    
    // Create Pure Rust HTTP client
    let client = SongbirdHttpClient::new(&beardog_socket).await
        .map_err(|e| JsonRpcError::internal_error(&format!("Failed to create HTTP client: {}", e)))?;
    
    // Make request
    let response = client.request(
        &params.method,
        &params.url,
        params.headers,
        params.body.map(|b| serde_json::to_vec(&b).unwrap()),
    ).await
    .map_err(|e| JsonRpcError::internal_error(&format!("HTTP request failed: {}", e)))?;
    
    // Return structured response
    Ok(serde_json::json!({
        "status": response.status,
        "headers": response.headers,
        "body": response.body
    }))
}
```

### 5. Remove `reqwest` Dependency

**File**: `phase1/songbird/crates/songbird-orchestrator/Cargo.toml`

**DELETE**:
```toml
reqwest = { version = "0.11", features = ["json"], default-features = false }
```

### 6. Test with httpbin.org

Before integrating with Squirrel, validate:

```bash
# Test via Unix socket
echo '{
  "jsonrpc":"2.0",
  "method":"http.request",
  "params":{
    "method":"GET",
    "url":"https://httpbin.org/get",
    "headers":{}
  },
  "id":1
}' | nc -N -U /tmp/songbird-nat0.sock | jq '.'
```

Expected:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "status": 200,
    "headers": {"content-type": "application/json"},
    "body": {"args": {}, "headers": {...}, "url": "https://httpbin.org/get"}
  },
  "id": 1
}
```

---

## 🤝 CO-EVOLUTION COORDINATION

### Week 1: Design and Contracts

**BearDog Team**:
- ✅ Design TLS crypto RPC API
- ✅ Document expected behavior
- ✅ Create test vectors

**Songbird Team**:
- ✅ Design HTTP/HTTPS client architecture
- ✅ Document RPC call sequences
- ✅ Create integration test plan

**Joint**:
- ✅ Review and align on RPC contracts
- ✅ Agree on error handling
- ✅ Agree on performance targets

### Week 2: Implementation

**BearDog Team**:
- ✅ Implement `tls.derive_secrets`
- ✅ Implement `tls.sign_handshake`
- ✅ Implement `tls.verify_certificate`
- ✅ Implement `crypto.ecdh_derive`
- ✅ Unit tests for each method

**Songbird Team**:
- ✅ Implement `BearDogTlsClient`
- ✅ Implement TLS 1.3 handshake logic
- ✅ Implement `SongbirdHttpClient`
- ✅ Update `handle_http_request`

**Joint**:
- ✅ Integration tests (Songbird calls BearDog)
- ✅ Mock external HTTPS endpoints
- ✅ Measure latency

### Week 3: Integration and Testing

**BearDog Team**:
- ✅ Performance optimization
- ✅ Stress testing (1000 concurrent handshakes)
- ✅ Documentation finalization

**Songbird Team**:
- ✅ Test with real HTTPS endpoints (httpbin.org)
- ✅ Test with Anthropic API
- ✅ Error handling edge cases
- ✅ Remove all `reqwest` code

**Joint**:
- ✅ End-to-end validation: Squirrel → Songbird → BearDog → Anthropic
- ✅ Performance validation (< 5s total latency)
- ✅ ecoBin cross-compilation validation

---

## 📊 SUCCESS CRITERIA

### BearDog

1. ✅ All TLS crypto RPC methods implemented
2. ✅ < 1ms per crypto operation
3. ✅ TLS handshake crypto ops < 10ms total
4. ✅ Zero unsafe code in new RPC methods
5. ✅ Documentation complete
6. ✅ Test coverage > 90%

### Songbird

1. ✅ Pure Rust HTTP/HTTPS client working
2. ✅ Zero C dependencies (no reqwest, no rustls with ring)
3. ✅ TLS 1.3 handshake successful with real servers
4. ✅ HTTP/2 support
5. ✅ `handle_http_request` uses new client
6. ✅ Test coverage > 90%

### Joint (Tower Atomic)

1. ✅ Squirrel → Songbird → BearDog → Anthropic works end-to-end
2. ✅ < 5s total latency for AI query
3. ✅ ecoBin builds for x86_64, ARM, RISC-V
4. ✅ Zero C dependencies confirmed
5. ✅ Production-ready error handling
6. ✅ Logging and observability

---

## 🚧 INTERIM STATE

### While BearDog/Songbird Are Evolving

**biomeOS will**:
1. ✅ Document this blocker
2. ✅ Focus on other primals (NestGate, ToadStool, petalTongue)
3. ✅ Evolve Neural API deployment system
4. ✅ Work on Squirrel's local AI integration (Tier 2)
5. ⏸️ Pause external AI integration (Tier 1) until Tower Atomic ready

**Squirrel will**:
1. ✅ Continue evolution of AI routing logic
2. ✅ Implement Tier 2 (local AI providers like ToadStool)
3. ⏸️ Pause Tier 1 (external APIs via HTTP delegation) until Tower Atomic ready

---

## 📚 REFERENCES

**Architecture**:
- BearDog Tower Atomic: `phase1/beardog/crates/beardog-tower-atomic/src/lib.rs`
- Songbird BTSP Provider: `phase1/songbird/crates/songbird-network-federation/src/btsp/http_provider.rs`

**Current (Wrong) Implementation**:
- `phase1/songbird/crates/songbird-orchestrator/src/ipc/server_pure_rust.rs:588-683` (DELETE)

**Pure Rust HTTP Libraries**:
- `hyper` - HTTP/1.1 and HTTP/2 protocol
- NOT `reqwest` (has C dependencies)
- NOT `rustls` (uses ring, has C dependencies)

**TLS 1.3 References**:
- RFC 8446: https://www.rfc-editor.org/rfc/rfc8446.html
- Handshake flow diagrams
- Crypto operations per handshake

---

## 🎯 NEXT ACTIONS

### Immediate (Today)

**BearDog Team**:
1. Review this handoff
2. Identify existing crypto primitives that can be reused
3. Draft TLS crypto RPC API design
4. Respond with timeline estimate

**Songbird Team**:
1. Review this handoff
2. Research Pure Rust TLS implementations (for reference, not dependency)
3. Draft HTTP client architecture
4. Respond with timeline estimate

**biomeOS (Me)**:
1. ✅ Create this handoff
2. ✅ Update root docs
3. ✅ Archive incorrect implementation
4. ⏸️ Pause Squirrel external AI work
5. 🔄 Continue other primals evolution

### Week 1

**Joint Meeting**:
- Align on RPC contracts
- Review architecture diagrams
- Commit to timeline
- Define communication channels

---

## 🎊 LONG-TERM IMPACT

### When Complete

This will be **THE** reference implementation for:
1. ✅ Pure Rust HTTP/HTTPS client
2. ✅ True Tower Atomic architecture
3. ✅ Crypto delegation pattern
4. ✅ ecoBin compliance at scale
5. ✅ Zero C dependencies in networking stack

**Every primal needing HTTP will use this.**

This is not just "fixing a bug" - this is **architecting the future** of ecoPrimals networking.

---

**🐦🐕 CO-EVOLUTION FOR PURE RUST TOWER ATOMIC 🐕🐦**

---

*Handoff Created: January 21, 2026*  
*Status: Active - Both Teams*  
*Timeline: 1-2 weeks coordinated evolution*  
*Impact: CRITICAL - Enables all external API integration*

