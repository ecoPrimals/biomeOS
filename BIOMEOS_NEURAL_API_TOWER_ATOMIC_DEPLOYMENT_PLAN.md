# biomeOS Neural API Evolution - Tower Atomic Deployment
**Date**: January 25, 2026  
**Status**: 🎯 **READY TO EXECUTE** - Songbird/BearDog Working, Now Deploy via biomeOS  
**Goal**: TRUE PRIMAL deployment via Neural API with semantic translation

---

## 🎯 THE VISION

### **Current State** (Working, but Coupled):
```
Songbird ──[Direct JSON-RPC]──> BearDog
         hardcoded: "crypto.aes128_gcm_encrypt"
         hardcoded: "/tmp/beardog.sock"
```

**Problem**: Songbird "knows" BearDog - violates TRUE PRIMAL principles!

### **Target State** (TRUE PRIMAL):
```
Songbird ──[Semantic Request]──> Neural API ──[Translation]──> BearDog
         "encrypt_data"          Discovers     "crypto.aes128_gcm_encrypt"
         no hardcoding!          Routes        at runtime path
```

**Goal**: Primals have ONLY self-knowledge, discover capabilities at runtime!

---

## 🏗️ EVOLUTION PATH

### **Phase A: Graph-Based Deployment** (This Week)
**Goal**: Deploy Tower Atomic via Neural API graph, achieve HTTPS

```
biomeOS Neural API
        │
        ▼
┌───────────────────────────────────────┐
│         tower_atomic_https.toml       │
│                                       │
│  [node.beardog]                       │
│  capability = "security"              │
│  mode = "server"                      │
│                                       │
│  [node.songbird]                      │
│  capability = "discovery"             │
│  mode = "server"                      │
│  depends_on = ["beardog"]             │
│                                       │
│  [node.https_test]                    │
│  type = "operation"                   │
│  target = "songbird"                  │
│  method = "https_get"                 │
│  params.url = "https://cloudflare.com"│
│  depends_on = ["songbird"]            │
└───────────────────────────────────────┘
        │
        ▼
  HTTP 200 OK! 🎉
```

**Status**: Ready to implement!

### **Phase B: Capability Registry** (Week 2)
**Goal**: Primals register capabilities, Neural API discovers

```
BearDog ──[register]──> Neural API Capability Registry
         capabilities: [
           "symmetric_encryption",
           "key_exchange",
           "tls_key_derivation"
         ]

Songbird ──[discover]──> Neural API
          "I need: symmetric_encryption"
                    ↓
          "BearDog at /tmp/beardog-nat0.sock"
```

### **Phase C: Semantic Translation** (Week 3-4)
**Goal**: Neural API translates semantic requests to primal-specific RPC

```
Songbird ──[semantic]──> Neural API ──[translate]──> BearDog
         "encrypt_data"              "crypto.aes128_gcm_encrypt"
         algorithm: "AES-128-GCM"    key, nonce, plaintext, aad
         data: {...}
```

### **Phase D: Zero Cross-Primal Knowledge** (Month 2)
**Goal**: Songbird has NO mention of BearDog, BearDog has NO mention of Songbird

```rust
// Songbird code - NO BearDog references!
let crypto = biome.discover_capability("symmetric_encryption")
    .with_algorithm("AES-128-GCM")
    .await?;
    
let ciphertext = crypto.encrypt(plaintext).await?;
```

---

## 📋 IMMEDIATE EXECUTION PLAN

### **Task 1: Create Tower Atomic HTTPS Graph** (2 hours)

**File**: `graphs/tower_atomic_https.toml`

```toml
# Tower Atomic HTTPS Deployment Graph
# Deploys BearDog + Songbird via Neural API for HTTPS testing

[graph]
name = "tower_atomic_https"
description = "Deploy Tower Atomic stack and test HTTPS"
version = "1.0.0"

[environment]
FAMILY_ID = "nat0"
RUST_LOG = "info"
SSLKEYLOGFILE = "/tmp/tls-keys.log"

# Phase 1: Launch BearDog (security provider)
[node.beardog]
type = "primal_launch"
capability = "security"
mode = "server"
config.socket_path = "$BIOMEOS_SOCKET_DIR/beardog-$FAMILY_ID.sock"

# Phase 2: Launch Songbird (TLS/HTTP provider)  
[node.songbird]
type = "primal_launch"
capability = "discovery"
mode = "server"
depends_on = ["beardog"]
config.socket_path = "$BIOMEOS_SOCKET_DIR/songbird-$FAMILY_ID.sock"
environment.BEARDOG_SOCKET = "$BIOMEOS_SOCKET_DIR/beardog-$FAMILY_ID.sock"
environment.BEARDOG_MODE = "direct"

# Phase 3: Wait for services to be ready
[node.wait_ready]
type = "health_check"
target = ["beardog", "songbird"]
depends_on = ["songbird"]
timeout_ms = 5000

# Phase 4: Execute HTTPS test
[node.https_test]
type = "operation"
target = "songbird"
method = "https_get"
depends_on = ["wait_ready"]

[node.https_test.params]
url = "https://www.cloudflare.com"
timeout_ms = 10000

# Phase 5: Validate response
[node.validate]
type = "assertion"
depends_on = ["https_test"]
condition = "outputs.https_test.status_code == 200"
```

### **Task 2: Update Neural Executor for HTTPS Operations** (3 hours)

**File**: `crates/biomeos-atomic-deploy/src/neural_executor.rs`

Add support for `https_get` operation:

```rust
async fn execute_operation(
    &self,
    node: &GraphNode,
    context: &ExecutionContext,
) -> Result<serde_json::Value> {
    let target = node.config.get("target")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing target for operation"))?;
    
    let method = node.config.get("method")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing method for operation"))?;
    
    match method {
        "https_get" => self.execute_https_get(node, context).await,
        "https_post" => self.execute_https_post(node, context).await,
        // ... other operations
        _ => Err(anyhow!("Unknown operation method: {}", method)),
    }
}

async fn execute_https_get(
    &self,
    node: &GraphNode,
    context: &ExecutionContext,
) -> Result<serde_json::Value> {
    let url = context.get_required_param(node, "url")?;
    let timeout_ms = context.get_param(node, "timeout_ms")?
        .and_then(|s| s.parse().ok())
        .unwrap_or(10000);
    
    // Get Songbird socket from context
    let songbird_socket = context.get_socket_path("songbird").await;
    
    // Call Songbird via JSON-RPC
    let client = UnixSocketClient::connect(&songbird_socket).await?;
    let response = client.call("https_get", json!({
        "url": url,
        "timeout_ms": timeout_ms,
    })).await?;
    
    Ok(response)
}
```

### **Task 3: Implement Songbird JSON-RPC Server** (Songbird Team)

**Current**: Songbird is a library/client  
**Needed**: Songbird as JSON-RPC server with `https_get` method

```rust
// Songbird JSON-RPC handler
async fn handle_https_get(params: Value) -> Result<Value> {
    let url = params["url"].as_str()
        .ok_or_else(|| anyhow!("Missing url parameter"))?;
    
    let client = HttpsClient::new(BearDogClient::from_env()?);
    let response = client.get(url).await?;
    
    Ok(json!({
        "status_code": response.status_code,
        "headers": response.headers,
        "body": response.body,
    }))
}
```

### **Task 4: Test Graph Deployment** (1 hour)

```bash
# Deploy Tower Atomic via Neural API
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
cargo build --release

# Run the graph
./target/release/biomeos neural-api \
    --graph graphs/tower_atomic_https.toml \
    --family-id nat0

# Expected output:
# ✅ beardog started (socket: /tmp/biomeos/sockets/beardog-nat0.sock)
# ✅ songbird started (socket: /tmp/biomeos/sockets/songbird-nat0.sock)
# ✅ health check passed
# ✅ https_get https://www.cloudflare.com
# ✅ Response: HTTP 200 OK
```

---

## 🔧 biomeOS WORK BREAKDOWN

### **Immediate (This Week)**

| Task | Owner | Hours | Status |
|------|-------|-------|--------|
| Create tower_atomic_https.toml | biomeOS | 2 | 🔜 |
| Update neural_executor for https_get | biomeOS | 3 | 🔜 |
| Test graph deployment | biomeOS | 1 | 🔜 |
| Coordinate with Songbird on RPC server | All | 1 | 🔜 |

### **Week 2: Capability Registry**

| Task | Owner | Hours | Status |
|------|-------|-------|--------|
| Design capability registry schema | biomeOS | 2 | 🔜 |
| Implement registry in Neural API | biomeOS | 4 | 🔜 |
| Add capability registration to BearDog | BearDog | 2 | 🔜 |
| Add capability discovery to Songbird | Songbird | 2 | 🔜 |
| Test capability-based routing | All | 2 | 🔜 |

### **Week 3-4: Semantic Translation**

| Task | Owner | Hours | Status |
|------|-------|-------|--------|
| Design semantic translation schema | biomeOS | 3 | 🔜 |
| Implement translation layer | biomeOS | 6 | 🔜 |
| Create semantic mappings for crypto | biomeOS | 2 | 🔜 |
| Update Songbird to use semantic requests | Songbird | 4 | 🔜 |
| Test full semantic translation | All | 3 | 🔜 |

### **Month 2: Zero Cross-Primal Knowledge**

| Task | Owner | Hours | Status |
|------|-------|-------|--------|
| Remove BearDog references from Songbird | Songbird | 6 | 🔜 |
| Remove Songbird references from BearDog | BearDog | 2 | 🔜 |
| Implement runtime discovery | biomeOS | 4 | 🔜 |
| Full integration testing | All | 4 | 🔜 |
| Documentation update | All | 2 | 🔜 |

---

## 📐 SEMANTIC TRANSLATION DESIGN

### **Capability Taxonomy**

```rust
// biomeos-types/src/capability_taxonomy.rs

pub enum CapabilityTaxonomy {
    // Security capabilities
    SymmetricEncryption,
    AsymmetricEncryption,
    KeyExchange,
    Hashing,
    Signing,
    TlsKeyDerivation,
    
    // Network capabilities
    HttpClient,
    HttpsClient,
    TcpClient,
    DnsResolver,
    
    // Storage capabilities
    KeyValueStore,
    FileStorage,
    
    // Compute capabilities
    AiInference,
    TaskExecution,
}
```

### **Semantic Request Schema**

```json
{
  "intent": "encrypt_application_data",
  "capability": "symmetric_encryption",
  "context": {
    "protocol": "TLS 1.3",
    "algorithm": "AES-128-GCM"
  },
  "inputs": {
    "key": "<base64>",
    "nonce": "<base64>",
    "plaintext": "<base64>",
    "aad": "<base64>"
  }
}
```

### **Translation Mapping**

```toml
# semantic_mappings/crypto.toml

[symmetric_encryption.AES-128-GCM.encrypt]
provider = "beardog"
method = "crypto.aes128_gcm_encrypt"
parameter_mapping = """
  inputs.key -> key
  inputs.nonce -> nonce
  inputs.plaintext -> plaintext
  inputs.aad -> aad
"""

[symmetric_encryption.AES-128-GCM.decrypt]
provider = "beardog"
method = "crypto.aes128_gcm_decrypt"
parameter_mapping = """
  inputs.key -> key
  inputs.nonce -> nonce
  inputs.ciphertext -> ciphertext
  inputs.aad -> aad
"""
```

### **Neural API Translation Layer**

```rust
// biomeos-atomic-deploy/src/semantic_translator.rs

pub struct SemanticTranslator {
    mappings: HashMap<String, TranslationMapping>,
    registry: CapabilityRegistry,
}

impl SemanticTranslator {
    pub async fn translate(&self, request: SemanticRequest) -> Result<RpcCall> {
        // 1. Find capability provider
        let provider = self.registry
            .find_provider(&request.capability, &request.context)
            .await?;
        
        // 2. Get translation mapping
        let mapping = self.mappings
            .get(&format!("{}.{}.{}", 
                request.capability, 
                request.context.algorithm,
                request.intent
            ))
            .ok_or_else(|| anyhow!("No mapping for request"))?;
        
        // 3. Translate parameters
        let params = mapping.translate_params(&request.inputs)?;
        
        // 4. Return RPC call
        Ok(RpcCall {
            socket: provider.socket_path,
            method: mapping.method.clone(),
            params,
        })
    }
}
```

---

## 🔄 MIGRATION STRATEGY

### **Step 1: Dual-Mode (Current)**
- Songbird can use BearDog directly (testing/simple)
- Songbird can use Neural API (production)
- Both paths work

### **Step 2: Neural API Default (Week 2)**
- Neural API becomes default path
- Direct mode available for debugging
- Capability registry active

### **Step 3: Semantic Translation (Week 3-4)**
- Semantic requests replace method names
- Neural API translates
- Primals remain unchanged

### **Step 4: TRUE PRIMAL (Month 2)**
- Remove all cross-primal references
- Discovery-only architecture
- Full evolution support

---

## 🎯 SUCCESS CRITERIA

### **Phase A Complete When**:
- [x] BearDog binary available
- [x] Songbird binary available  
- [ ] Graph deploys both primals
- [ ] HTTPS test passes via graph
- [ ] HTTP 200 OK achieved

### **Phase B Complete When**:
- [ ] Capability registry implemented
- [ ] BearDog registers capabilities
- [ ] Songbird discovers capabilities
- [ ] Routing works via registry

### **Phase C Complete When**:
- [ ] Semantic translation layer implemented
- [ ] Mappings defined for crypto operations
- [ ] Songbird uses semantic requests
- [ ] Translation verified

### **Phase D Complete When**:
- [ ] Zero BearDog references in Songbird
- [ ] Zero Songbird references in BearDog
- [ ] Runtime discovery works
- [ ] Full HTTPS via semantic translation

---

## 💡 KEY PRINCIPLES

### **TRUE PRIMAL**:
1. **Self-knowledge only** - Primals know themselves
2. **Runtime discovery** - Find capabilities at runtime
3. **No hardcoding** - No cross-primal references
4. **Evolution support** - Primals can change without breaking others

### **Neural API Role**:
1. **Evolution engine** - Enables primal evolution
2. **Discovery** - Capability-based routing
3. **Translation** - Semantic → Primal-specific
4. **Orchestration** - Graph-based deployment

### **Semantic Translation**:
1. **Stable interfaces** - Semantic names don't change
2. **Provider flexibility** - Implementations can evolve
3. **Automatic routing** - Best provider selected
4. **Backward compatible** - Old primals keep working

---

## 🚀 LET'S EXECUTE!

### **Today**:
1. Create `graphs/tower_atomic_https.toml`
2. Update neural_executor for https operations
3. Test basic graph deployment

### **This Week**:
1. Full graph deployment working
2. HTTPS via Neural API
3. Document patterns

### **Next 2 Weeks**:
1. Capability registry
2. Semantic translation
3. TRUE PRIMAL evolution

**The foundation is solid (100% HTTPS working!) - now biomeOS brings orchestration and evolution!** 🎯

---

**"Working binaries + Neural API = TRUE PRIMAL deployment!"** 🚀  
**"Semantic translation = Evolution without breaking!"** 🔄  
**"Graph-based orchestration = Terrariums and incubation!"** 🌱  
**"biomeOS has much work - and it's exciting!"** 💪  

Let's build this! 🎉

