# Primal tarpc Method Specification Handoff

**Date**: January 28, 2026  
**Status**: 📋 SPECIFICATION  
**Affects**: All primals (BearDog, Songbird, Squirrel, Toadstool, NestGate)  
**Purpose**: Enable Neural API protocol escalation (JSON-RPC → tarpc)

---

## Overview

Neural API now supports **Living Graph Protocol Escalation** - the ability to dynamically upgrade connections between primals from JSON-RPC (bootstrap/debug) to tarpc (production/performance) based on runtime metrics.

For this to work, **each primal must implement 3 new JSON-RPC methods** that enable Neural API to coordinate the escalation.

---

## Required Methods

### 1. `rpc.tarpc_endpoint`

**Purpose**: Advertise the primal's tarpc socket and available services.

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "rpc.tarpc_endpoint",
  "params": {},
  "id": 1
}
```

**Response (tarpc available)**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "available": true,
    "socket": "/run/user/1000/biomeos/beardog-nat0-tarpc.sock",
    "services": ["CryptoService", "IdentityService", "TlsService"]
  },
  "id": 1
}
```

**Response (tarpc not available)**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "available": false,
    "socket": null,
    "services": []
  },
  "id": 1
}
```

**Implementation Notes**:
- Return `available: false` if tarpc is not implemented yet (graceful degradation)
- `socket` should be the full path to the tarpc Unix socket
- `services` lists the tarpc service traits implemented (for documentation/discovery)

---

### 2. `rpc.escalate_to`

**Purpose**: Tell the primal to connect to a target primal via tarpc instead of JSON-RPC.

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "rpc.escalate_to",
  "params": {
    "target": "beardog",
    "tarpc_socket": "/run/user/1000/biomeos/beardog-nat0-tarpc.sock",
    "services": ["CryptoService"]
  },
  "id": 2
}
```

**Response (success)**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "status": "connected",
    "target": "beardog",
    "protocol": "tarpc"
  },
  "id": 2
}
```

**Response (failure)**:
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32603,
    "message": "Failed to connect to tarpc socket: Connection refused"
  },
  "id": 2
}
```

**Implementation Notes**:
- The primal should establish a tarpc client connection to the target socket
- Store the tarpc client for subsequent calls to the target
- Fall back to JSON-RPC if connection fails (return error, Neural API will handle fallback)

---

### 3. `rpc.fallback_to_json_rpc`

**Purpose**: Tell the primal to fall back from tarpc to JSON-RPC for a target primal.

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "rpc.fallback_to_json_rpc",
  "params": {
    "target": "beardog",
    "reason": "tarpc_failure"
  },
  "id": 3
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "status": "fallback_complete",
    "target": "beardog",
    "protocol": "json_rpc"
  },
  "id": 3
}
```

**Implementation Notes**:
- Close the tarpc client connection to the target
- Revert to using JSON-RPC for subsequent calls
- Log the fallback reason for debugging

---

## Implementation Priority

| Primal | Priority | Reason |
|--------|----------|--------|
| **BearDog** | P0 | TLS hot-path (Songbird → BearDog) |
| **Songbird** | P0 | TLS hot-path + HTTP client |
| **Squirrel** | P1 | AI API calls (Squirrel → Songbird) |
| **Toadstool** | P2 | Config lookups |
| **NestGate** | P2 | Identity verification |

---

## Example Implementation (Rust)

### BearDog Example

```rust
// In beardog's JSON-RPC handler

async fn handle_rpc_tarpc_endpoint(&self) -> Result<Value> {
    // Check if tarpc is enabled
    if self.tarpc_enabled {
        Ok(json!({
            "available": true,
            "socket": self.tarpc_socket_path,
            "services": ["CryptoService", "IdentityService", "TlsService"]
        }))
    } else {
        Ok(json!({
            "available": false,
            "socket": null,
            "services": []
        }))
    }
}

async fn handle_rpc_escalate_to(&self, params: &Value) -> Result<Value> {
    let target = params["target"].as_str()
        .ok_or_else(|| anyhow!("Missing 'target' parameter"))?;
    let tarpc_socket = params["tarpc_socket"].as_str()
        .ok_or_else(|| anyhow!("Missing 'tarpc_socket' parameter"))?;

    // Establish tarpc connection
    let transport = tarpc::serde_transport::unix::connect(tarpc_socket, Json::default).await?;
    let client = CryptoServiceClient::new(Default::default(), transport).spawn();

    // Store client for future calls
    self.tarpc_clients.write().await.insert(target.to_string(), client);

    Ok(json!({
        "status": "connected",
        "target": target,
        "protocol": "tarpc"
    }))
}

async fn handle_rpc_fallback_to_json_rpc(&self, params: &Value) -> Result<Value> {
    let target = params["target"].as_str()
        .ok_or_else(|| anyhow!("Missing 'target' parameter"))?;
    let reason = params["reason"].as_str().unwrap_or("unknown");

    // Remove tarpc client
    self.tarpc_clients.write().await.remove(target);

    warn!("⚠️ Falling back to JSON-RPC for {}: {}", target, reason);

    Ok(json!({
        "status": "fallback_complete",
        "target": target,
        "protocol": "json_rpc"
    }))
}
```

### Songbird Example (Client Side)

```rust
// In Songbird's BearDogProvider

impl BearDogProvider {
    /// Call BearDog - automatically uses tarpc if escalated
    async fn call(&self, method: &str, params: Value) -> Result<Value> {
        // Check if we have a tarpc client for BearDog
        if let Some(client) = self.tarpc_client.read().await.as_ref() {
            // Use tarpc for hot-path
            match method {
                "crypto.sha256" => {
                    let result = client.sha256(context::current(), params).await?;
                    return Ok(result);
                }
                "crypto.x25519_generate_ephemeral" => {
                    let result = client.x25519_generate_ephemeral(context::current()).await?;
                    return Ok(result);
                }
                // ... other methods
                _ => {
                    // Unknown method, fall back to JSON-RPC
                    warn!("Method {} not available via tarpc, using JSON-RPC", method);
                }
            }
        }

        // Fall back to JSON-RPC
        self.json_rpc_call(method, params).await
    }

    /// Handle escalation request from Neural API
    async fn escalate_to(&self, tarpc_socket: &str, services: &[String]) -> Result<()> {
        let transport = tarpc::serde_transport::unix::connect(tarpc_socket, Json::default).await?;
        let client = CryptoServiceClient::new(Default::default(), transport).spawn();

        *self.tarpc_client.write().await = Some(client);
        info!("✅ Escalated to tarpc for BearDog");
        Ok(())
    }

    /// Handle fallback request from Neural API
    async fn fallback_to_json_rpc(&self, reason: &str) -> Result<()> {
        *self.tarpc_client.write().await = None;
        warn!("⚠️ Fell back to JSON-RPC for BearDog: {}", reason);
        Ok(())
    }
}
```

---

## tarpc Service Definitions

### BearDog CryptoService

```rust
#[tarpc::service]
pub trait CryptoService {
    /// SHA-256 hash
    async fn sha256(data: Vec<u8>) -> Vec<u8>;
    
    /// SHA-384 hash
    async fn sha384(data: Vec<u8>) -> Vec<u8>;
    
    /// Generate X25519 ephemeral keypair
    async fn x25519_generate_ephemeral() -> (Vec<u8>, Vec<u8>);  // (public, private)
    
    /// X25519 ECDH
    async fn x25519_ecdh(private_key: Vec<u8>, public_key: Vec<u8>) -> Vec<u8>;
    
    /// AES-GCM encrypt
    async fn aes_gcm_encrypt(key: Vec<u8>, nonce: Vec<u8>, plaintext: Vec<u8>, aad: Vec<u8>) -> Vec<u8>;
    
    /// AES-GCM decrypt
    async fn aes_gcm_decrypt(key: Vec<u8>, nonce: Vec<u8>, ciphertext: Vec<u8>, aad: Vec<u8>) -> Vec<u8>;
    
    /// HKDF expand
    async fn hkdf_expand(secret: Vec<u8>, info: Vec<u8>, length: u32) -> Vec<u8>;
}
```

### Songbird HttpService

```rust
#[tarpc::service]
pub trait HttpService {
    /// HTTP request
    async fn request(
        method: String,
        url: String,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> HttpResponse;
    
    /// HTTP GET convenience
    async fn get(url: String, headers: HashMap<String, String>) -> HttpResponse;
    
    /// HTTP POST convenience
    async fn post(
        url: String,
        headers: HashMap<String, String>,
        body: Vec<u8>,
    ) -> HttpResponse;
}

#[derive(Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}
```

---

## Testing

### Verify Method Implementation

```bash
# Test rpc.tarpc_endpoint
echo '{"jsonrpc":"2.0","method":"rpc.tarpc_endpoint","params":{},"id":1}' \
  | nc -U /run/user/1000/biomeos/beardog-nat0.sock

# Expected: {"result":{"available":true,"socket":"...","services":[...]}}

# Test rpc.escalate_to (Songbird → BearDog)
echo '{"jsonrpc":"2.0","method":"rpc.escalate_to","params":{"target":"beardog","tarpc_socket":"/run/user/1000/biomeos/beardog-nat0-tarpc.sock","services":["CryptoService"]},"id":1}' \
  | nc -U /run/user/1000/biomeos/songbird-nat0.sock

# Expected: {"result":{"status":"connected","target":"beardog","protocol":"tarpc"}}

# Test rpc.fallback_to_json_rpc
echo '{"jsonrpc":"2.0","method":"rpc.fallback_to_json_rpc","params":{"target":"beardog","reason":"test"},"id":1}' \
  | nc -U /run/user/1000/biomeos/songbird-nat0.sock

# Expected: {"result":{"status":"fallback_complete","target":"beardog","protocol":"json_rpc"}}
```

### Integration Test via Neural API

```bash
# Register primals with protocol state
echo '{"jsonrpc":"2.0","method":"protocol.register_primal","params":{"primal_id":"beardog","json_rpc_socket":"/run/user/1000/biomeos/beardog-nat0.sock","tarpc_socket":"/run/user/1000/biomeos/beardog-nat0-tarpc.sock","capabilities":["crypto"]},"id":1}' \
  | nc -U /run/user/1000/biomeos/neural-api-nat0.sock

# Register connection
echo '{"jsonrpc":"2.0","method":"protocol.register_connection","params":{"from":"songbird","to":"beardog"},"id":1}' \
  | nc -U /run/user/1000/biomeos/neural-api-nat0.sock

# Manually trigger escalation
echo '{"jsonrpc":"2.0","method":"protocol.escalate","params":{"from":"songbird","to":"beardog"},"id":1}' \
  | nc -U /run/user/1000/biomeos/neural-api-nat0.sock

# Check status
echo '{"jsonrpc":"2.0","method":"protocol.status","params":{},"id":1}' \
  | nc -U /run/user/1000/biomeos/neural-api-nat0.sock
```

---

## Escalation Criteria

Neural API auto-escalates connections when:

1. **Request volume**: > 100 requests on the connection
2. **Latency threshold**: Average latency > 500μs
3. **Health status**: Both primals healthy
4. **tarpc available**: Target primal supports tarpc

```toml
# Default config (can be overridden in deployment graph)
[protocol]
auto_escalate = true
min_requests = 100
latency_threshold_us = 500
check_interval_secs = 10
tarpc_failure_threshold = 3
```

---

## Graceful Degradation

**If a primal doesn't implement these methods yet:**

1. `rpc.tarpc_endpoint` returns `Method not found` → Neural API treats as `available: false`
2. The connection remains on JSON-RPC (no escalation attempted)
3. No impact on functionality, just no performance boost

**If tarpc connection fails:**

1. Neural API calls `rpc.fallback_to_json_rpc`
2. Connection reverts to JSON-RPC (degraded mode)
3. After stability period, re-escalation may be attempted

---

## Timeline

| Milestone | Date | Team |
|-----------|------|------|
| Spec published | Jan 28, 2026 | biomeOS ✅ |
| BearDog tarpc server | TBD | BearDog team |
| Songbird tarpc client | TBD | Songbird team |
| Integration testing | TBD | biomeOS |
| Production deployment | TBD | Ops |

---

## Questions / Decisions

1. **Binary serialization**: Use `bincode` or stick with JSON over tarpc?
   - Recommendation: Start with JSON for debugging, evolve to bincode

2. **Connection pooling**: Single tarpc client or pooled?
   - Recommendation: Single client initially, pool if needed

3. **Timeout handling**: How long before giving up on tarpc?
   - Recommendation: 100ms timeout, then fallback

---

## Contact

- **biomeOS Neural API**: This repository
- **Spec**: `specs/LIVING_GRAPH_PROTOCOL_ESCALATION_SPEC.md`
- **Roadmap**: `PROTOCOL_ESCALATION_ROADMAP.md`

---

*Last Updated: January 28, 2026*

