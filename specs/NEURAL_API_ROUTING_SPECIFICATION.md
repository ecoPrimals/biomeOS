# Neural API Routing Specification

**Version**: 2.0.0 - **Capability Translation Evolution**  
**Status**: Active Development  
**Last Updated**: January 21, 2026  
**See Also**: `specs/CAPABILITY_TRANSLATION_ARCHITECTURE.md`

---

## Overview

The Neural API serves as the central routing and orchestration layer for biomeOS, enabling primals to discover and communicate with each other without hardcoded dependencies.

**NEW (v2.0.0)**: Neural API now provides **capability translation**, allowing primals to speak in semantic capabilities while the API automatically translates to provider-specific method names. This eliminates all cross-primal API coupling and enables TRUE PRIMAL pattern compliance.

**Key Principle**: **Isomorphic Evolution** - The ecosystem maintains its functional structure even as individual primals change, evolve, swap, or extend. Semantic capabilities remain stable while implementations evolve freely.

**See**: `ISOMORPHIC_EVOLUTION.md` for comprehensive architectural principles.

---

## Core Responsibilities

1. **Capability Registry**: Maintain registry of available primal capabilities
2. **Capability Translation**: Translate semantic capabilities to provider-specific method names ✨ **NEW**
3. **Request Routing**: Route requests to appropriate primals with automatic translation
4. **Service Discovery**: Enable dynamic primal discovery
5. **Load Balancing**: Distribute load across multiple instances
6. **Health Monitoring**: Track primal health and availability
7. **Method Mapping**: Build and maintain semantic → actual method translation maps ✨ **NEW**

---

## Capability Translation (v2.0.0)

### Problem Solved

**Before**: Primals hardcoded other primals' exact method names, causing tight coupling.

```rust
// ❌ BAD: Songbird hardcodes BearDog's API
beardog_client.call("x25519_generate_ephemeral", params).await?
```

**After**: Primals use semantic capabilities, Neural API translates automatically.

```rust
// ✅ GOOD: Songbird uses semantic capability
neural_api.call_capability("crypto.generate_keypair", params).await?
// Neural API translates to "x25519_generate_ephemeral" for BearDog
```

### Translation Registry

```rust
pub struct CapabilityTranslation {
    /// Semantic capability name (what consumers call)
    pub semantic: String,
    
    /// Provider primal ID
    pub provider: String,
    
    /// Actual method name on provider
    pub actual_method: String,
    
    /// Provider socket path
    pub socket: String,
    
    /// Optional metadata
    pub metadata: HashMap<String, Value>,
}
```

### Graph-Based Configuration

Primals self-describe their capability mappings in deployment graphs:

```toml
[[nodes]]
id = "beardog"
binary = "./plasmidBin/primals/beardog/beardog"

[nodes.capabilities_provided]
"crypto.generate_keypair" = "x25519_generate_ephemeral"
"crypto.ecdh_derive" = "x25519_derive_secret"
"crypto.encrypt" = "chacha20_poly1305_encrypt"
"crypto.decrypt" = "chacha20_poly1305_decrypt"
```

Neural API loads this at graph deployment and builds translation map automatically.

---

## New RPC Methods (v2.0.0)

### 1. `capability.call` - Call with Translation

```json
{
  "jsonrpc": "2.0",
  "method": "capability.call",
  "params": {
    "capability": "crypto.generate_keypair",
    "args": {
      "algorithm": "x25519"
    }
  },
  "id": 1
}
```

Response:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "public_key": "base64...",
    "private_key": "base64..."
  },
  "id": 1
}
```

**Flow**:
1. Neural API looks up `crypto.generate_keypair`
2. Finds provider: `beardog` at `/tmp/beardog-nat0.sock`
3. Translates to actual method: `x25519_generate_ephemeral`
4. Routes RPC to BearDog
5. Returns result to caller

### 2. `capability.discover_translation` - Inspect Translation

```json
{
  "jsonrpc": "2.0",
  "method": "capability.discover_translation",
  "params": {
    "capability": "crypto.generate_keypair"
  },
  "id": 2
}
```

Response:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "semantic": "crypto.generate_keypair",
    "provider": "beardog",
    "actual_method": "x25519_generate_ephemeral",
    "socket": "/tmp/beardog-nat0.sock"
  },
  "id": 2
}
```

### 3. `capability.list_translations` - List All Mappings

```json
{
  "jsonrpc": "2.0",
  "method": "capability.list_translations",
  "id": 3
}
```

Response:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "translations": [
      {
        "semantic": "crypto.generate_keypair",
        "provider": "beardog",
        "actual_method": "x25519_generate_ephemeral"
      },
      {
        "semantic": "http.request",
        "provider": "songbird",
        "actual_method": "http_request"
      }
    ]
  },
  "id": 3
}
```

---

## Existing RPC Methods (v1.0.0)

### `register_primal`

Register a primal and its capabilities:

```json
{
  "jsonrpc": "2.0",
  "method": "register_primal",
  "params": {
    "primal_id": "songbird",
    "socket": "/tmp/songbird.sock",
    "capabilities": ["http.request", "http.get", "discovery"],
    "metadata": {
      "version": "0.2.0",
      "family_id": "nat0"
    }
  },
  "id": 1
}
```

### `query_capability`

Find primals providing a capability:

```json
{
  "jsonrpc": "2.0",
  "method": "query_capability",
  "params": {
    "capability": "http.request"
  },
  "id": 2
}
```

Response:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "providers": [
      {
        "primal_id": "songbird",
        "socket": "/tmp/songbird.sock",
        "metadata": {
          "version": "0.2.0"
        }
      }
    ]
  },
  "id": 2
}
```

### `route_request`

Route a request to a capability provider:

```json
{
  "jsonrpc": "2.0",
  "method": "route_request",
  "params": {
    "capability": "http.request",
    "method": "GET",
    "args": {
      "url": "https://api.github.com/zen"
    }
  },
  "id": 3
}
```

---

## Implementation Updates (v2.0.0)

### Capability Registry Structure

```rust
pub struct CapabilityRegistry {
    /// Primal registrations (existing)
    primals: HashMap<String, PrimalInfo>,
    
    /// Capability → Provider mappings (existing)
    capabilities: HashMap<String, Vec<String>>,
    
    /// NEW: Semantic → Actual method translations
    translations: HashMap<String, CapabilityTranslation>,
    
    /// NEW: Provider → All translations
    provider_translations: HashMap<String, Vec<String>>,
}

impl CapabilityRegistry {
    /// NEW: Register translation from graph
    pub fn register_translation(
        &mut self,
        semantic: impl Into<String>,
        provider: impl Into<String>,
        actual_method: impl Into<String>,
        socket: impl Into<String>,
    ) {
        let translation = CapabilityTranslation {
            semantic: semantic.into(),
            provider: provider.into(),
            actual_method: actual_method.into(),
            socket: socket.into(),
            metadata: HashMap::new(),
        };
        
        self.translations.insert(translation.semantic.clone(), translation);
    }
    
    /// NEW: Get translation for capability
    pub fn get_translation(&self, semantic: &str) -> Option<&CapabilityTranslation> {
        self.translations.get(semantic)
    }
    
    /// NEW: Call capability with automatic translation
    pub async fn call_capability(
        &self,
        semantic: &str,
        params: Value,
    ) -> Result<Value> {
        let translation = self.get_translation(semantic)
            .ok_or_else(|| anyhow!("No provider for capability: {}", semantic))?;
        
        // Connect to provider
        let mut stream = UnixStream::connect(&translation.socket).await?;
        
        // Build RPC with ACTUAL method name
        let rpc = json!({
            "jsonrpc": "2.0",
            "method": translation.actual_method,
            "params": params,
            "id": self.next_id()
        });
        
        // Send and receive
        stream.write_all(rpc.to_string().as_bytes()).await?;
        let response = read_json_rpc_response(&mut stream).await?;
        
        Ok(response)
    }
}
```

### Graph Loading Updates

```rust
impl NeuralApiServer {
    /// Load graph and build translation registry
    pub async fn load_graph(&mut self, graph_path: &str) -> Result<()> {
        let graph = Graph::from_toml_file(graph_path)?;
        
        for node in graph.nodes {
            // Existing: Register primal
            self.registry.register_primal(
                &node.id,
                &node.socket_path,
                node.capabilities.clone(),
            )?;
            
            // NEW: Register capability translations
            if let Some(caps_provided) = node.capabilities_provided {
                for (semantic, actual) in caps_provided {
                    self.registry.register_translation(
                        semantic,
                        &node.id,
                        actual,
                        &node.socket_path,
                    );
                }
            }
        }
        
        Ok(())
    }
}
```

---

## Benefits

### 1. Zero Cross-Knowledge ✅

Primals never know other primals' method names. Only semantic capabilities.

### 2. Provider Swapping ✅

Change crypto provider from BearDog to RustCrypto? Just update graph, zero code changes.

### 3. Version Evolution ✅

BearDog renames methods? Update graph, consumers unchanged.

### 4. Multi-Provider Support ✅

Multiple providers for same capability, Neural API routes intelligently.

### 5. TRUE PRIMAL Compliance ✅

Complete elimination of cross-primal hardcoding.

---

## Migration Path

### Phase 1: Add Translation Support (Current)

- Add `capabilities_provided` to graph schema
- Implement translation registry in Neural API
- Add `capability.call` RPC method
- **Status**: In progress

### Phase 2: Update Consumers (Next)

- Songbird: Replace BearDog direct calls with `capability.call`
- Squirrel: Use `capability.call` for HTTP delegation
- Test end-to-end HTTPS with translation
- **ETA**: 2-3 hours

### Phase 3: Generalize (Future)

- All primals use semantic capabilities
- All graphs include capability mappings
- Neural API is universal translation layer
- **Timeline**: Ongoing

---

## Related Specifications

- `specs/CAPABILITY_TRANSLATION_ARCHITECTURE.md` - Full translation design
- `specs/GRAPH_BASED_ORCHESTRATION_SPEC.md` - Graph schema
- `wateringHole/PRIMAL_IPC_PROTOCOL.md` - RPC patterns
- `specs/BIOMEOS_AS_PRIMAL_SPECIALIZATION.md` - Neural API role

---

**Status**: Active Development (v2.0.0)  
**Priority**: Critical (unblocks HTTPS)  
**Next**: Implement translation registry

---

*The neural way: Connect concepts, not implementations* 🧠🦀
