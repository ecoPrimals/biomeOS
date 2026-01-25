# 🚀 DUAL-MODE EVOLUTION HANDOFF - Team Implementation Guide
## January 24, 2026

**Target Teams**: Songbird, BearDog, biomeOS  
**Goal**: Enable direct primal-to-primal RPC while maintaining Neural API orchestration  
**Timeline**: 4-6 hours total  
**Priority**: HIGH - Enables HTTPS validation + TRUE PRIMAL compliance  

---

## 📋 EXECUTIVE SUMMARY

### **The Challenge**:
- Currently: Songbird REQUIRES Neural API to talk to BearDog
- Issue: Can't run simple client/server self-test
- Blocks: HTTPS transcript comparison validation

### **The Solution**:
- Add dual-mode BearDogClient in Songbird
- **Direct mode**: Talk to BearDog directly (testing, simple deployments)
- **Neural API mode**: Use capability discovery (production, orchestration)
- Both modes maintained, fully backward compatible

### **The Benefit**:
1. ✅ Enables client/server self-test (find HTTPS transcript bug!)
2. ✅ TRUE PRIMAL compliance (primals work independently)
3. ✅ Simpler testing (no orchestration needed)
4. ✅ Flexible deployment (simple vs complex)
5. ✅ Evolution support (Neural API for production)

---

## 🏗️ ARCHITECTURE OVERVIEW

### **Current State** (Neural API Required):
```
Songbird
  ↓ calls
  ↓ "crypto.generate_keypair" (semantic)
  ↓
Neural API
  ↓ discovers provider
  ↓ translates semantic → actual
  ↓
BearDog
  ↓ executes
  "x25519_generate_ephemeral" (actual)
```

### **Future State** (Dual-Mode):
```
DIRECT MODE (testing/simple):
  Songbird → "x25519_generate_ephemeral" → BearDog
  (fast, simple, no discovery)

NEURAL API MODE (production/orchestration):
  Songbird → "crypto.generate_keypair" → Neural API → BearDog
  (discovery, translation, scaling)
```

---

## 📊 IMPLEMENTATION BREAKDOWN

### **Team 1: Songbird** (2-3 hours)

**File**: `crates/songbird-http-client/src/beardog_client.rs`

**Task**: Add dual-mode support to BearDogClient

**Changes**:

#### **Step 1: Add Mode Enum** (5 minutes)
```rust
use std::sync::atomic::{AtomicU64, Ordering};

/// BearDog communication mode
#[derive(Debug, Clone)]
pub enum BearDogMode {
    /// Direct RPC to BearDog (testing, simple deployments)
    /// - Fast (no routing overhead)
    /// - Simple (no discovery needed)
    /// - Fixed topology (you know what you need)
    Direct {
        socket_path: String,
    },
    
    /// Via Neural API (production, orchestration, evolution)
    /// - Capability discovery
    /// - Semantic translation
    /// - Evolution support
    /// - Load balancing & failover
    NeuralApi {
        socket_path: String,
    },
}
```

#### **Step 2: Update BearDogClient Struct** (5 minutes)
```rust
/// BearDog RPC client with dual-mode support
#[derive(Debug)]
pub struct BearDogClient {
    mode: BearDogMode,
    request_id: AtomicU64,
}
```

#### **Step 3: Add Constructors** (10 minutes)
```rust
impl BearDogClient {
    /// Create client in Direct mode (testing, simple deployments)
    /// 
    /// Talks directly to BearDog via Unix socket.
    /// Uses actual BearDog method names (e.g., "x25519_generate_ephemeral").
    /// 
    /// # Example
    /// ```rust
    /// let beardog = BearDogClient::new_direct("/tmp/beardog.sock");
    /// ```
    pub fn new_direct(beardog_socket: impl Into<String>) -> Self {
        Self {
            mode: BearDogMode::Direct {
                socket_path: beardog_socket.into(),
            },
            request_id: AtomicU64::new(1),
        }
    }
    
    /// Create client in Neural API mode (production, orchestration)
    /// 
    /// Routes through Neural API for capability discovery and translation.
    /// Uses semantic capability names (e.g., "crypto.generate_keypair").
    /// 
    /// # Example
    /// ```rust
    /// let beardog = BearDogClient::new_neural_api("/tmp/neural-api.sock");
    /// ```
    pub fn new_neural_api(neural_api_socket: impl Into<String>) -> Self {
        Self {
            mode: BearDogMode::NeuralApi {
                socket_path: neural_api_socket.into(),
            },
            request_id: AtomicU64::new(1),
        }
    }
    
    /// Existing constructor (backward compatible)
    /// Defaults to Neural API mode for compatibility
    pub fn new(neural_api_socket: impl Into<String>) -> Self {
        Self::new_neural_api(neural_api_socket)
    }
    
    /// Create from environment variable
    /// Checks BEARDOG_MODE env var to determine mode:
    /// - "direct" → Direct mode (BEARDOG_SOCKET)
    /// - "neural" or default → Neural API mode (NEURAL_API_SOCKET)
    pub fn from_env() -> Self {
        let mode = std::env::var("BEARDOG_MODE")
            .unwrap_or_else(|_| "neural".to_string());
        
        match mode.as_str() {
            "direct" => {
                let socket = std::env::var("BEARDOG_SOCKET")
                    .unwrap_or_else(|_| "/tmp/beardog.sock".to_string());
                Self::new_direct(socket)
            }
            _ => {
                let socket = std::env::var("NEURAL_API_SOCKET")
                    .unwrap_or_else(|_| "/tmp/neural-api.sock".to_string());
                Self::new_neural_api(socket)
            }
        }
    }
}
```

#### **Step 4: Add Semantic → Actual Mapping** (15 minutes)
```rust
impl BearDogClient {
    /// Map semantic capability names to actual BearDog method names
    /// (Used only in Direct mode)
    fn semantic_to_actual(&self, capability: &str) -> Result<&'static str> {
        Ok(match capability {
            // Crypto operations
            "crypto.generate_keypair" => "x25519_generate_ephemeral",
            "crypto.ecdh_derive" => "x25519_compute_shared_secret",
            "crypto.derive_handshake_keys" => "tls_derive_handshake_secrets",
            "crypto.derive_application_keys" => "tls_derive_application_secrets",
            "crypto.compute_finished_verify_data" => "tls_compute_finished_verify_data",
            "crypto.aes128_gcm_encrypt" => "crypto_aes128_gcm_encrypt",
            "crypto.aes256_gcm_encrypt" => "crypto_aes256_gcm_encrypt",
            "crypto.chacha20_encrypt" => "crypto_chacha20_poly1305_encrypt",
            "crypto.aes128_gcm_decrypt" => "crypto_aes128_gcm_decrypt",
            "crypto.aes256_gcm_decrypt" => "crypto_aes256_gcm_decrypt",
            "crypto.chacha20_decrypt" => "crypto_chacha20_poly1305_decrypt",
            
            _ => return Err(Error::BearDogRpc(format!(
                "Unknown capability: {}. Add mapping to semantic_to_actual()", 
                capability
            ))),
        })
    }
    
    /// Get actual method name based on mode
    fn get_method_name(&self, capability: &str) -> Result<String> {
        match &self.mode {
            BearDogMode::Direct { .. } => {
                // Direct mode: use actual BearDog method names
                Ok(self.semantic_to_actual(capability)?.to_string())
            }
            BearDogMode::NeuralApi { .. } => {
                // Neural API mode: use semantic names (Neural API translates)
                Ok(capability.to_string())
            }
        }
    }
}
```

#### **Step 5: Update call() Method** (30 minutes)
```rust
impl BearDogClient {
    /// Call BearDog (direct or via Neural API based on mode)
    async fn call(&self, capability: &str, params: Value) -> Result<Value> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        
        match &self.mode {
            BearDogMode::Direct { socket_path } => {
                // DIRECT RPC to BearDog
                let method = self.semantic_to_actual(capability)?;
                
                let request = JsonRpcRequest {
                    jsonrpc: "2.0".to_string(),
                    method: method.to_string(),
                    params,
                    id,
                };
                
                trace!("→ BearDog direct RPC: {} (id={})", method, id);
                
                // Connect to BearDog directly
                let mut stream = UnixStream::connect(socket_path)
                    .await
                    .map_err(|e| Error::BearDogRpc(format!(
                        "Failed to connect to BearDog at {}: {}", 
                        socket_path, e
                    )))?;
                
                // Send request
                let request_json = serde_json::to_string(&request)?;
                stream.write_all(request_json.as_bytes()).await?;
                stream.write_all(b"\n").await?;
                stream.flush().await?;
                
                // Read response
                let mut buffer = Vec::new();
                stream.read_to_end(&mut buffer).await?;
                
                let response: JsonRpcResponse = serde_json::from_slice(&buffer)
                    .map_err(|e| Error::BearDogRpc(format!("Invalid JSON response: {}", e)))?;
                
                if let Some(error) = response.error {
                    return Err(Error::BearDogRpc(format!(
                        "BearDog error: {} (code: {})", 
                        error.message, error.code
                    )));
                }
                
                response.result.ok_or_else(|| {
                    Error::BearDogRpc("No result in response".to_string())
                })
            }
            
            BearDogMode::NeuralApi { socket_path } => {
                // VIA NEURAL API (existing logic)
                let request = JsonRpcRequest {
                    jsonrpc: "2.0".to_string(),
                    method: "capability.call".to_string(),
                    params: json!({
                        "capability": capability,
                        "args": params
                    }),
                    id,
                };
                
                trace!("→ Neural API capability.call: {} (id={})", capability, id);
                
                // Connect to Neural API
                let mut stream = UnixStream::connect(socket_path)
                    .await
                    .map_err(|e| Error::BearDogRpc(format!(
                        "Failed to connect to Neural API at {}: {}", 
                        socket_path, e
                    )))?;
                
                // Send request
                let request_json = serde_json::to_string(&request)?;
                stream.write_all(request_json.as_bytes()).await?;
                stream.write_all(b"\n").await?;
                stream.flush().await?;
                
                // Shutdown write to signal we're done
                stream.shutdown().await?;
                
                // Read response with JSON-aware reading (Neural API keeps socket open)
                use tokio::time::{timeout, Duration};
                let mut buffer = Vec::new();
                let mut temp_buf = [0u8; 4096];
                
                loop {
                    match timeout(Duration::from_millis(100), stream.read(&mut temp_buf)).await {
                        Ok(Ok(0)) => break,  // EOF
                        Ok(Ok(n)) => {
                            buffer.extend_from_slice(&temp_buf[..n]);
                            // Check if we have a complete JSON response
                            if let Ok(_) = serde_json::from_slice::<JsonRpcResponse>(&buffer) {
                                break;
                            }
                        }
                        Ok(Err(e)) => return Err(Error::Io(e)),
                        Err(_) => break,  // Timeout = assume complete
                    }
                }
                
                let response: JsonRpcResponse = serde_json::from_slice(&buffer)
                    .map_err(|e| Error::BearDogRpc(format!("Invalid JSON response: {}", e)))?;
                
                if let Some(error) = response.error {
                    return Err(Error::BearDogRpc(format!(
                        "Neural API error for {}: {} (code: {})", 
                        capability, error.message, error.code
                    )));
                }
                
                response.result.ok_or_else(|| {
                    Error::BearDogRpc("No result in response".to_string())
                })
            }
        }
    }
}
```

#### **Step 6: Update Public Methods** (10 minutes)

**No changes needed!** All public methods like `generate_keypair()`, `ecdh_derive()`, etc. already call `self.call(capability, params)`, which now handles both modes automatically.

#### **Step 7: Add Tests** (30 minutes)
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_semantic_to_actual_mapping() {
        let client = BearDogClient::new_direct("/tmp/test.sock");
        
        assert_eq!(
            client.semantic_to_actual("crypto.generate_keypair").unwrap(),
            "x25519_generate_ephemeral"
        );
        
        assert_eq!(
            client.semantic_to_actual("crypto.ecdh_derive").unwrap(),
            "x25519_compute_shared_secret"
        );
        
        assert!(client.semantic_to_actual("unknown.capability").is_err());
    }
    
    #[test]
    fn test_mode_creation() {
        let direct = BearDogClient::new_direct("/tmp/beardog.sock");
        assert!(matches!(direct.mode, BearDogMode::Direct { .. }));
        
        let neural = BearDogClient::new_neural_api("/tmp/neural.sock");
        assert!(matches!(neural.mode, BearDogMode::NeuralApi { .. }));
        
        let default = BearDogClient::new("/tmp/neural.sock");
        assert!(matches!(default.mode, BearDogMode::NeuralApi { .. }));
    }
    
    #[tokio::test]
    async fn test_from_env_direct() {
        std::env::set_var("BEARDOG_MODE", "direct");
        std::env::set_var("BEARDOG_SOCKET", "/tmp/test-beardog.sock");
        
        let client = BearDogClient::from_env();
        assert!(matches!(client.mode, BearDogMode::Direct { .. }));
    }
    
    #[tokio::test]
    async fn test_from_env_neural() {
        std::env::set_var("BEARDOG_MODE", "neural");
        std::env::set_var("NEURAL_API_SOCKET", "/tmp/test-neural.sock");
        
        let client = BearDogClient::from_env();
        assert!(matches!(client.mode, BearDogMode::NeuralApi { .. }));
    }
}
```

#### **Step 8: Update Examples** (20 minutes)

**File**: `crates/songbird-http-client/examples/client_test.rs`

```rust
// Change from:
let client = SongbirdHttpClient::new(&beardog_socket);

// To:
use std::sync::Arc;
use songbird_http_client::beardog_client::BearDogClient;

let beardog = Arc::new(BearDogClient::new_direct(beardog_socket.clone()));
let client = SongbirdHttpClient::with_beardog(beardog);
```

**File**: `crates/songbird-http-client/examples/server_test.rs`

```rust
// Same change as above
let beardog = Arc::new(BearDogClient::new_direct(beardog_socket.clone()));
let server = TlsServer::new(beardog, cert_chain, private_key);
```

---

### **Team 2: BearDog** (NO CHANGES NEEDED!)

**Status**: ✅ BearDog already supports direct RPC via JSON-RPC 2.0!

**Verification**:
- ✅ All methods accept JSON-RPC requests on Unix socket
- ✅ Methods like `x25519_generate_ephemeral` already implemented
- ✅ No changes required for dual-mode support

**Action**: None! Just be aware Songbird will call directly in test mode.

---

### **Team 3: biomeOS** (1 hour)

#### **Task 1: Update Test Script** (20 minutes)

**File**: `scripts/test_client_server_self.sh`

**Changes**: Use Direct mode (no Neural API needed!)

```bash
#!/bin/bash
set -e

echo "═══════════════════════════════════════════════════════════"
echo "🔬 TOWER ATOMIC CLIENT/SERVER SELF-TEST (DIRECT MODE)"
echo "═══════════════════════════════════════════════════════════"

# Cleanup
pkill -9 -f "beardog|songbird|server_test|client_test" 2>/dev/null || true
rm -f /tmp/beardog.sock
sleep 2

# Start BearDog
echo "Step 1: Starting BearDog..."
RUST_LOG=info ./plasmidBin/primals/beardog/beardog server \
  --socket /tmp/beardog.sock \
  --family-id test \
  > /tmp/beardog-test.log 2>&1 &
BEARDOG_PID=$!
sleep 3
echo "✅ BearDog started (PID: $BEARDOG_PID)"

# Start Songbird Server (Direct mode!)
echo "Step 2: Starting Songbird TLS Server (Direct mode)..."
RUST_LOG=info BEARDOG_MODE=direct BEARDOG_SOCKET=/tmp/beardog.sock \
  ./plasmidBin/primals/songbird/server_test \
  --port 8443 \
  --beardog-socket /tmp/beardog.sock \
  > /tmp/server-transcript.log 2>&1 &
SERVER_PID=$!
sleep 5
echo "✅ Server started (PID: $SERVER_PID)"

# Run Songbird Client (Direct mode!)
echo "Step 3: Running Songbird TLS Client (Direct mode)..."
RUST_LOG=info BEARDOG_MODE=direct BEARDOG_SOCKET=/tmp/beardog.sock \
  ./plasmidBin/primals/songbird/client_test \
  --url https://localhost:8443 \
  --skip-verify \
  --beardog-socket /tmp/beardog.sock \
  > /tmp/client-transcript.log 2>&1
CLIENT_EXIT=$?
echo "✅ Client complete (exit: $CLIENT_EXIT)"

# Stop servers
kill $SERVER_PID $BEARDOG_PID 2>/dev/null || true
sleep 1

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "📊 TRANSCRIPT COMPARISON"
echo "═══════════════════════════════════════════════════════════"

# Extract and compare transcripts
grep "CLIENT.*0000:" /tmp/client-transcript.log | awk '{print $NF}' > /tmp/client.hex || true
grep "SERVER.*0000:" /tmp/server-transcript.log | awk '{print $NF}' > /tmp/server.hex || true

CLIENT_LINES=$(wc -l < /tmp/client.hex)
SERVER_LINES=$(wc -l < /tmp/server.hex)

echo "Client transcript: $CLIENT_LINES lines"
echo "Server transcript: $SERVER_LINES lines"

if [ "$CLIENT_LINES" -eq 0 ] || [ "$SERVER_LINES" -eq 0 ]; then
    echo "⚠️  No transcript hex dumps found!"
    echo ""
    echo "Client log (last 30 lines):"
    tail -30 /tmp/client-transcript.log
    echo ""
    echo "Server log (last 30 lines):"
    tail -30 /tmp/server-transcript.log
    exit 1
fi

if diff -u /tmp/client.hex /tmp/server.hex > /tmp/transcript-diff.txt 2>&1; then
    echo ""
    echo "✅✅✅ TRANSCRIPTS MATCH PERFECTLY! ✅✅✅"
else
    echo ""
    echo "❌ DIFFERENCES FOUND!"
    echo ""
    echo "First 50 lines of diff:"
    head -50 /tmp/transcript-diff.txt
    echo ""
    echo "Full diff: /tmp/transcript-diff.txt"
fi

echo ""
echo "Logs: /tmp/client-transcript.log, /tmp/server-transcript.log"
```

#### **Task 2: Document Neural API Mode** (20 minutes)

**File**: `docs/DEPLOYMENT_MODES.md` (new)

```markdown
# biomeOS Deployment Modes

## Direct Mode (Testing/Simple)

For testing and simple deployments where primals are known:

```bash
# Set environment
export BEARDOG_MODE=direct
export BEARDOG_SOCKET=/tmp/beardog.sock

# Start BearDog
beardog server --socket /tmp/beardog.sock

# Start Songbird (direct mode)
songbird-http-client ...
```

## Neural API Mode (Production)

For production deployments with capability discovery:

```bash
# Start BearDog
beardog server --socket /tmp/beardog.sock

# Start Neural API with graph
biomeos neural-api --graph graphs/tower_atomic_production.toml

# Start Songbird (Neural API mode)
export BEARDOG_MODE=neural
export NEURAL_API_SOCKET=/tmp/neural-api.sock
songbird-http-client ...
```

Neural API provides:
- Capability discovery
- Semantic translation
- Load balancing
- Failover
- Evolution support
```

#### **Task 3: Update CI/CD** (20 minutes)

**File**: `.github/workflows/test.yml` (if exists)

Add Direct mode tests:
```yaml
- name: Test Direct Mode
  run: |
    export BEARDOG_MODE=direct
    export BEARDOG_SOCKET=/tmp/beardog-ci.sock
    cargo test --features direct-mode
```

---

## 📋 TESTING CHECKLIST

### **Phase 1: Songbird Unit Tests**
- [ ] Build succeeds: `cargo build`
- [ ] All tests pass: `cargo test`
- [ ] Dual-mode tests pass
- [ ] Examples build: `cargo build --examples`

### **Phase 2: Integration Tests**
- [ ] Start BearDog
- [ ] Direct mode: Songbird → BearDog (no Neural API)
- [ ] Neural API mode: Songbird → Neural API → BearDog
- [ ] Both modes produce same results

### **Phase 3: Self-Test**
- [ ] Run `scripts/test_client_server_self.sh`
- [ ] Client and server both start
- [ ] TLS handshake completes (or fails with known issue)
- [ ] Transcripts logged
- [ ] Comparison shows differences (expected!)

---

## 🎯 SUCCESS CRITERIA

### **Immediate** (End of implementation):
1. ✅ Songbird builds with dual-mode support
2. ✅ All tests pass
3. ✅ Examples work in both modes
4. ✅ Self-test script runs

### **Next Steps** (After implementation):
1. Run self-test to compare transcripts
2. Find exact byte differences in handshake messages
3. Fix content construction (likely Certificate message)
4. Test against example.com
5. **HTTP 200 OK!** 🎉

---

## 🚨 KNOWN ISSUES TO ADDRESS LATER

### **Current HTTPS Bug** (NOT blocking dual-mode):
- Server returns `decrypt_error` when trying to decrypt our client Finished
- Root cause: Transcript content differs in encrypted handshake messages
- Most likely: Certificate message content (chain ordering, extensions)
- Impact: HTTPS doesn't work yet
- Status: Will be revealed by self-test transcript comparison!

### **Dual-mode does NOT fix this bug**:
- Dual-mode enables us to FIND the bug (via self-test)
- But we still need to fix the transcript content issue
- Timeline: ~2 hours after we see the exact differences

---

## 📊 TIMELINE

### **Day 1** (4-6 hours):
- Songbird: Implement dual-mode (2-3 hours)
- biomeOS: Update test script (1 hour)
- Testing: Validate both modes (1-2 hours)

### **Day 2** (2-3 hours):
- Run self-test
- Analyze transcript differences
- Fix content construction
- Validate against example.com
- **100% Pure Rust HTTPS!** 🎉

---

## 💡 KEY INSIGHTS

### **Why This Matters**:
1. **TRUE PRIMAL**: Primals work independently
2. **Testing**: Simple self-test without orchestration
3. **Debugging**: Direct transcript comparison
4. **Evolution**: Neural API for production scaling
5. **Flexibility**: Choose mode based on use case

### **What Doesn't Change**:
1. BearDog (already supports direct RPC)
2. Neural API (still works for production)
3. Public APIs (backward compatible)
4. Existing code (defaults to Neural API mode)

### **What's New**:
1. Direct mode option
2. Environment variable control
3. Testing without Neural API
4. Semantic → actual mapping

---

## 📞 TEAM CONTACTS & QUESTIONS

### **Songbird Team**:
- **Q**: "Do we need to change all our code?"
- **A**: No! Existing code works as-is. Just add new constructors and mode logic.

- **Q**: "What if we mess up the mapping?"
- **A**: Tests will catch it. Plus, errors are clear: "Unknown capability: X"

### **BearDog Team**:
- **Q**: "Do we need to change anything?"
- **A**: Nope! You already support direct RPC. Just be aware Songbird will call directly.

### **biomeOS Team**:
- **Q**: "Which mode for production?"
- **A**: Neural API mode (discovery, scaling). Direct mode for testing only.

- **Q**: "Does this fix HTTPS?"
- **A**: Not directly, but enables us to FIND the fix via self-test!

---

## 🎊 FINAL CHECKLIST

### **Before Starting**:
- [ ] Read this entire document
- [ ] Understand both modes
- [ ] Know which files to modify
- [ ] Have test environment ready

### **During Implementation**:
- [ ] Follow steps in order
- [ ] Test after each step
- [ ] Commit frequently
- [ ] Document changes

### **After Implementation**:
- [ ] All tests pass
- [ ] Self-test runs
- [ ] Both modes work
- [ ] Documentation updated

---

**"Dual-mode enables testing AND production!"** 🎯  
**"TRUE PRIMAL: Work independently, orchestrate optionally!"** ✅  
**"Self-test will reveal the HTTPS bug!"** 🔬  
**"Timeline: 4-6 hours to implementation complete!"** 🚀  

---

**Questions?** Check the architecture docs:
- `ARCHITECTURAL_EVOLUTION_PRIMAL_INDEPENDENCE_JAN_24_2026.md`
- `ARCHITECTURAL_CLARITY_NEURAL_API_EVOLUTION_ENGINE_JAN_24_2026.md`

**Ready to start?** Let's build this! 💪✨

