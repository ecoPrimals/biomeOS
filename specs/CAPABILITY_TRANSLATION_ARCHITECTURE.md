# Capability Translation Architecture

**Date**: January 21, 2026  
**Status**: ✅ **ARCHITECTURAL EVOLUTION**  
**Component**: Neural API Capability Registry

---

## Problem Statement

**Discovered during HTTPS debugging**: Songbird calls `crypto.generate_keypair` but BearDog provides `x25519_generate_ephemeral`.

**Initial (wrong) solution**: Force Songbird to hardcode BearDog's exact method names.

**Why wrong**: Violates TRUE PRIMAL pattern - primals should never have specific knowledge of other primals' APIs.

---

## TRUE PRIMAL Solution: Capability Translation

### Core Principle

**Primals speak in semantic capabilities, Neural API handles translation.**

- **Songbird**: "I need `crypto.generate_keypair`" (semantic capability)
- **BearDog**: "I provide `crypto.generate_keypair` via `x25519_generate_ephemeral`" (self-describing)
- **Neural API**: Translates semantic → actual method name automatically

---

## Architecture

### 1. Self-Describing Capabilities in Graphs

Primals declare both **semantic** and **actual** APIs in deployment graphs:

```toml
# graphs/tower_atomic_bootstrap.toml

[[nodes]]
id = "beardog"
binary = "./plasmidBin/primals/beardog/beardog"
mode = "server"
socket_path = "/tmp/beardog-nat0.sock"

# Self-describing: Semantic capability → Actual method mapping
# BearDog v0.9.0: 23 crypto methods (8 core + 8 ECDSA/RSA + 4 genetic + 3 TLS)
[nodes.capabilities_provided]
# Core Crypto Operations
"crypto.generate_keypair" = "crypto.x25519_generate_ephemeral"
"crypto.ecdh_derive" = "crypto.x25519_derive_secret"
"crypto.encrypt" = "crypto.chacha20_poly1305_encrypt"
"crypto.decrypt" = "crypto.chacha20_poly1305_decrypt"
"crypto.hash" = "crypto.blake3_hash"
"crypto.hmac" = "crypto.hmac_sha256"
"crypto.sign" = "crypto.sign_ed25519"
"crypto.verify" = "crypto.verify_ed25519"

# ECDSA Signature Algorithms (96% HTTPS coverage)
"crypto.sign_ecdsa_p256" = "crypto.sign_ecdsa_secp256r1"
"crypto.verify_ecdsa_p256" = "crypto.verify_ecdsa_secp256r1"
"crypto.sign_ecdsa_p384" = "crypto.sign_ecdsa_secp384r1"
"crypto.verify_ecdsa_p384" = "crypto.verify_ecdsa_secp384r1"

# RSA Signature Algorithms (legacy + modern)
"crypto.sign_rsa_pkcs1" = "crypto.sign_rsa_pkcs1_sha256"
"crypto.verify_rsa_pkcs1" = "crypto.verify_rsa_pkcs1_sha256"
"crypto.sign_rsa_pss" = "crypto.sign_rsa_pss_sha256"
"crypto.verify_rsa_pss" = "crypto.verify_rsa_pss_sha256"

# TLS Crypto Operations
"tls.derive_secrets" = "tls.derive_secrets"
"tls.sign_handshake" = "tls.sign_handshake"
"tls.verify_certificate" = "tls.verify_certificate"

# Genetic Crypto Operations (Phase 5)
"genetic.derive_key" = "genetic.derive_lineage_key"
"genetic.mix_entropy" = "genetic.mix_entropy"
"genetic.verify_lineage" = "genetic.verify_lineage"
"genetic.generate_proof" = "genetic.generate_lineage_proof"
```

### 2. Semantic Requirements (No Hardcoding)

Consumers declare what they need **semantically**:

```toml
[[nodes]]
id = "songbird"
binary = "./plasmidBin/primals/songbird/songbird"
mode = "server"
socket_path = "/tmp/songbird-nat0.sock"

[nodes.capabilities_required]
security_provider = [
  "crypto.generate_keypair",  # Semantic - no knowledge of actual method
  "crypto.ecdh_derive",
  "crypto.encrypt",
  "crypto.decrypt"
]

[nodes.environment]
SONGBIRD_SECURITY_PROVIDER = "${neural_api.capability.crypto.generate_keypair.socket}"
```

### 3. Neural API Auto-Translation

Neural API builds translation map from graph and provides capability routing:

```rust
// Neural API Capability Registry (evolved)
pub struct CapabilityRegistry {
    providers: HashMap<String, ProviderInfo>,
    translations: HashMap<String, MethodTranslation>,
}

pub struct MethodTranslation {
    semantic_name: String,      // "crypto.generate_keypair"
    provider_id: String,         // "beardog"
    actual_method: String,       // "x25519_generate_ephemeral"
    socket: String,              // "/tmp/beardog-nat0.sock"
}
```

---

## Implementation Flow

### Graph Loading (Neural API Startup)

```rust
// 1. Neural API reads graph
let graph = Graph::from_toml_file("graphs/tower_atomic_bootstrap.toml")?;

// 2. Extract capability mappings from nodes
for node in graph.nodes {
    if let Some(caps) = node.capabilities_provided {
        for (semantic, actual) in caps {
            registry.register_translation(
                semantic,           // "crypto.generate_keypair"
                node.id,            // "beardog"
                actual,             // "x25519_generate_ephemeral"
                node.socket_path    // "/tmp/beardog-nat0.sock"
            );
        }
    }
}

// 3. Registry now has full translation map
```

### Capability Call (Runtime)

```rust
// Primal calls Neural API with semantic capability
let response = neural_api_client
    .call_capability("crypto.generate_keypair", json!({
        "algorithm": "x25519"
    }))
    .await?;

// Neural API translates and routes:
impl NeuralApiServer {
    async fn call_capability(&self, semantic: &str, params: Value) -> Result<Value> {
        // 1. Lookup translation
        let translation = self.registry.get_translation(semantic)?;
        
        // 2. Connect to provider
        let mut stream = UnixStream::connect(&translation.socket).await?;
        
        // 3. Build RPC with ACTUAL method name
        let rpc_request = json!({
            "jsonrpc": "2.0",
            "method": translation.actual_method,  // "x25519_generate_ephemeral"
            "params": params,
            "id": self.next_id()
        });
        
        // 4. Send and receive
        stream.write_all(rpc_request.to_string().as_bytes()).await?;
        let response = self.read_rpc_response(&mut stream).await?;
        
        Ok(response)
    }
}
```

---

## Benefits

### 1. Zero Cross-Knowledge ✅

**Before** (hardcoded):
```rust
// Songbird knows BearDog's exact API
let response = beardog_client.call("x25519_generate_ephemeral", params).await?;
```

**After** (semantic):
```rust
// Songbird only knows semantic capability
let response = neural_api.call_capability("crypto.generate_keypair", params).await?;
```

### 2. Provider Swapping ✅

Want to use different crypto provider?

```toml
[[nodes]]
id = "rustcrypto-provider"
binary = "./plasmidBin/primals/rustcrypto/rustcrypto"

[nodes.capabilities_provided]
"crypto.generate_keypair" = "generate_x25519_keys"  # Different method name!
"crypto.ecdh_derive" = "derive_shared_secret"       # Different method name!
```

**Zero changes to Songbird code!** Neural API handles translation.

### 3. Version Evolution ✅

BearDog v0.10.0 renames methods:

```toml
# Old graph (v0.9.0)
[nodes.capabilities_provided]
"crypto.generate_keypair" = "x25519_generate_ephemeral"

# New graph (v0.10.0)
[nodes.capabilities_provided]
"crypto.generate_keypair" = "crypto.x25519.generate"  # New method name
```

**Just update graph**, no code changes in consumers.

### 4. Multi-Provider Support ✅

Multiple providers for same capability:

```toml
[[nodes]]
id = "beardog"
[nodes.capabilities_provided]
"crypto.generate_keypair" = "x25519_generate_ephemeral"

[[nodes]]
id = "rustcrypto"
[nodes.capabilities_provided]
"crypto.generate_keypair" = "generate_x25519"

# Neural API can route based on:
# - Load balancing
# - Provider preference
# - Fallback/redundancy
```

---

## Graph Schema Evolution

### Before (No Translation)

```toml
[[nodes]]
id = "beardog"
binary = "./beardog"
```

### After (Self-Describing)

```toml
[[nodes]]
id = "beardog"
binary = "./beardog"
socket_path = "/tmp/beardog-nat0.sock"

# NEW: Capability → Method mapping
[nodes.capabilities_provided]
"crypto.generate_keypair" = "x25519_generate_ephemeral"
"crypto.ecdh_derive" = "x25519_derive_secret"
"crypto.encrypt" = "chacha20_poly1305_encrypt"
"crypto.decrypt" = "chacha20_poly1305_decrypt"
"tls.derive_secrets" = "tls_derive_session_keys"
"tls.sign_handshake" = "tls_sign_handshake_context"
"tls.verify_certificate" = "tls_verify_cert_chain"

# Optional: Capability metadata
[nodes.capabilities_metadata."crypto.generate_keypair"]
algorithm = "x25519"
curve = "Curve25519"
output = "base64"
```

---

## Neural API Extensions

### New RPC Methods

```json
// 1. Call capability (with translation)
{
  "jsonrpc": "2.0",
  "method": "capability.call",
  "params": {
    "capability": "crypto.generate_keypair",
    "args": {"algorithm": "x25519"}
  },
  "id": 1
}

// 2. Discover capability translation
{
  "jsonrpc": "2.0",
  "method": "capability.discover",
  "params": {
    "capability": "crypto.generate_keypair"
  },
  "id": 2
}
// Response:
{
  "result": {
    "provider": "beardog",
    "socket": "/tmp/beardog-nat0.sock",
    "actual_method": "x25519_generate_ephemeral"
  }
}

// 3. List all translations
{
  "jsonrpc": "2.0",
  "method": "capability.list_translations",
  "id": 3
}
```

---

## Migration Path

### Phase 1: Add Translation Registry (Current)

- Add `capabilities_provided` to graph schema
- Implement translation registry in Neural API
- Add `capability.call` RPC method

### Phase 2: Update Songbird (Next)

- Replace direct BearDog RPC with `neural_api.call_capability()`
- Remove hardcoded method names
- Test HTTPS with translation layer

### Phase 3: Generalize Pattern (Future)

- All primals use semantic capabilities
- All graphs include capability mappings
- Neural API becomes universal translation layer

---

## Example: Full Tower Atomic with Translation

```toml
# graphs/tower_atomic_bootstrap.toml

[[nodes]]
id = "beardog"
binary = "./plasmidBin/primals/beardog/beardog"
mode = "server"
socket_path = "/tmp/beardog-nat0.sock"

[nodes.capabilities_provided]
"crypto.generate_keypair" = "x25519_generate_ephemeral"
"crypto.ecdh_derive" = "x25519_derive_secret"
"crypto.encrypt" = "chacha20_poly1305_encrypt"
"crypto.decrypt" = "chacha20_poly1305_decrypt"
"crypto.hash" = "blake3_hash"
"tls.derive_secrets" = "tls_derive_session_keys"
"tls.sign_handshake" = "tls_sign_handshake_context"

[[nodes]]
id = "songbird"
binary = "./plasmidBin/primals/songbird/songbird"
mode = "server"
socket_path = "/tmp/songbird-nat0.sock"

[nodes.capabilities_required]
security = [
  "crypto.generate_keypair",
  "crypto.ecdh_derive",
  "crypto.encrypt",
  "crypto.decrypt",
  "tls.derive_secrets"
]

[nodes.capabilities_provided]
"http.request" = "http_request"
"http.get" = "http_get"
"http.post" = "http_post"

[nodes.environment]
# Neural API injects provider socket via translation lookup
SONGBIRD_SECURITY_PROVIDER = "${neural_api.capability_provider.crypto.generate_keypair}"

[[edges]]
from = "songbird"
to = "beardog"
type = "capability_delegation"
capabilities = ["crypto.*", "tls.*"]
```

---

## Testing Strategy

### 1. Unit Tests (Neural API)

```rust
#[tokio::test]
async fn test_capability_translation() {
    let mut registry = CapabilityRegistry::new();
    
    registry.register_translation(
        "crypto.generate_keypair",
        "beardog",
        "x25519_generate_ephemeral",
        "/tmp/beardog.sock"
    );
    
    let translation = registry.get_translation("crypto.generate_keypair").unwrap();
    assert_eq!(translation.actual_method, "x25519_generate_ephemeral");
}
```

### 2. Integration Tests (Graph + Translation)

```rust
#[tokio::test]
async fn test_graph_capability_loading() {
    let graph = Graph::from_toml_file("test_graph.toml").unwrap();
    let registry = CapabilityRegistry::from_graph(&graph).unwrap();
    
    assert!(registry.has_capability("crypto.generate_keypair"));
}
```

### 3. End-to-End (Songbird → Neural API → BearDog)

```rust
#[tokio::test]
async fn test_https_via_translation() {
    // Start Neural API with translation
    let neural_api = start_neural_api_with_graph("tower_atomic.toml").await;
    
    // Songbird calls semantic capability
    let response = neural_api
        .call_capability("crypto.generate_keypair", json!({"algorithm": "x25519"}))
        .await
        .unwrap();
    
    assert!(response["public_key"].is_string());
}
```

---

## Success Criteria

✅ **Zero hardcoding**: Primals never know provider method names  
✅ **Self-describing**: Graphs contain all translation info  
✅ **Auto-wiring**: Neural API builds translation map from graph  
✅ **Provider agnostic**: Swap providers without code changes  
✅ **Evolution-friendly**: Method renames only affect graphs  

---

## Related Specifications

- `specs/NEURAL_API_CAPABILITY_REGISTRY.md` - Core registry design
- `specs/PRIMAL_IPC_PROTOCOL.md` - RPC/IPC patterns
- `specs/BIOMEOS_AS_PRIMAL_SPECIALIZATION.md` - Neural API role
- `wateringHole/PRIMAL_DISCOVERY_STANDARD.md` - Capability discovery

---

**Status**: Ready for implementation  
**Priority**: Critical (unblocks HTTPS)  
**Complexity**: Medium (2-3 hours)  
**Impact**: Foundational for TRUE PRIMAL pattern

---

*The ecological way: Speak in concepts, not implementations* 🌍🦀

