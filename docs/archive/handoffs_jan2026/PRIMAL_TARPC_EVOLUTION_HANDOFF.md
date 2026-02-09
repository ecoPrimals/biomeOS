# Primal tarpc Evolution Handoff

**Date**: January 28, 2026  
**From**: biomeOS Neural API Team  
**To**: All Primal Teams (BearDog, Songbird, Squirrel, Toadstool, NestGate)  
**Priority**: 🟡 Medium (Performance Evolution)  
**Status**: Ready for Implementation

---

## Executive Summary

biomeOS Neural API now supports **protocol escalation** - dynamically upgrading connections from JSON-RPC (bootstrap) to tarpc (production) based on runtime metrics.

**Goal**: ~10x latency improvement on hot-paths (e.g., Songbird → BearDog TLS operations)

**Your Task**: Implement 3 new JSON-RPC methods to enable Neural API to orchestrate the escalation.

---

## Background

### Current Architecture (JSON-RPC)

```
Songbird ──JSON-RPC──► BearDog   (~100μs per call)
    │
    └── "crypto.sha256", "tls.derive_handshake_secrets", etc.
```

### Target Architecture (JSON-RPC → tarpc)

```
Bootstrap:  Songbird ──JSON-RPC──► BearDog   (configuration, debugging)
Production: Songbird ══tarpc════► BearDog   (~10μs per call)
```

### Why tarpc?

| Protocol | Latency | Use Case |
|----------|---------|----------|
| JSON-RPC | ~100μs | Bootstrap, config, debugging, human-readable |
| tarpc | ~10μs | Production hot-paths, high-throughput |

---

## Required Methods

### Method 1: `rpc.tarpc_endpoint`

**Purpose**: Neural API queries your primal for its tarpc socket.

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "rpc.tarpc_endpoint",
  "params": {},
  "id": 1
}
```

**Response** (if tarpc available):
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

**Response** (if tarpc NOT available):
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

### Method 2: `rpc.escalate_to`

**Purpose**: Neural API tells your primal (as a **client**) to connect to another primal via tarpc.

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

**Response** (success):
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

**Response** (failure):
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32000,
    "message": "Failed to connect: connection refused"
  },
  "id": 2
}
```

### Method 3: `rpc.fallback_to_json_rpc`

**Purpose**: Neural API tells your primal to fall back to JSON-RPC (after tarpc failure).

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

---

## Implementation Guide

### Step 1: Add tarpc Socket

If your primal doesn't already have tarpc, add a second socket listener:

```rust
// In your main.rs or server setup
let json_rpc_socket = "/run/user/1000/biomeos/{primal}-{family}.sock";
let tarpc_socket = "/run/user/1000/biomeos/{primal}-{family}-tarpc.sock";

// Start both listeners
tokio::spawn(json_rpc_server(json_rpc_socket));
tokio::spawn(tarpc_server(tarpc_socket));
```

### Step 2: Define tarpc Service

```rust
// In your primal's service definition
#[tarpc::service]
pub trait CryptoService {
    async fn sha256(data: Vec<u8>) -> Vec<u8>;
    async fn x25519_generate_ephemeral() -> KeyPair;
    async fn aes_gcm_encrypt(key: Vec<u8>, nonce: Vec<u8>, data: Vec<u8>) -> Vec<u8>;
    // ... other methods
}
```

### Step 3: Implement JSON-RPC Methods

```rust
// In your JSON-RPC handler
match method {
    "rpc.tarpc_endpoint" => {
        Ok(json!({
            "available": true,
            "socket": self.tarpc_socket_path,
            "services": self.tarpc_services.clone(),
        }))
    }
    
    "rpc.escalate_to" => {
        let target = params["target"].as_str().unwrap();
        let tarpc_socket = params["tarpc_socket"].as_str().unwrap();
        
        // Connect to target's tarpc socket
        self.connect_tarpc(target, tarpc_socket).await?;
        
        Ok(json!({
            "status": "connected",
            "target": target,
            "protocol": "tarpc",
        }))
    }
    
    "rpc.fallback_to_json_rpc" => {
        let target = params["target"].as_str().unwrap();
        
        // Disconnect tarpc, revert to JSON-RPC
        self.disconnect_tarpc(target).await;
        
        Ok(json!({
            "status": "fallback_complete",
            "target": target,
            "protocol": "json_rpc",
        }))
    }
    
    // ... existing methods
}
```

### Step 4: Maintain Dual-Mode Client

```rust
pub struct DualModeClient {
    target: String,
    json_rpc_socket: PathBuf,
    tarpc_client: Option<CryptoServiceClient>,
    current_mode: ProtocolMode,
}

impl DualModeClient {
    pub async fn call<T>(&self, method: &str, params: Value) -> Result<T> {
        match self.current_mode {
            ProtocolMode::Tarpc if self.tarpc_client.is_some() => {
                // Use tarpc
                self.call_tarpc(method, params).await
            }
            _ => {
                // Fall back to JSON-RPC
                self.call_json_rpc(method, params).await
            }
        }
    }
    
    pub async fn escalate(&mut self, tarpc_socket: &Path) -> Result<()> {
        let transport = tarpc::serde_transport::unix::connect(tarpc_socket, Json::default).await?;
        self.tarpc_client = Some(CryptoServiceClient::new(Default::default(), transport).spawn());
        self.current_mode = ProtocolMode::Tarpc;
        Ok(())
    }
    
    pub fn fallback(&mut self) {
        self.tarpc_client = None;
        self.current_mode = ProtocolMode::JsonRpc;
    }
}
```

---

## Primal-Specific Notes

### BearDog (Crypto/Identity)

**Priority**: 🔴 High - Most called by Songbird's TLS layer

**Hot-paths to optimize**:
- `crypto.sha256` / `crypto.sha384`
- `tls.derive_handshake_secrets`
- `tls.derive_application_secrets`
- `crypto.aes_gcm_encrypt` / `crypto.aes_gcm_decrypt`
- `crypto.x25519_generate_ephemeral`

**tarpc services**:
```rust
#[tarpc::service]
pub trait CryptoService {
    async fn sha256(data: Vec<u8>) -> Vec<u8>;
    async fn sha384(data: Vec<u8>) -> Vec<u8>;
    async fn aes_gcm_encrypt(key: Vec<u8>, nonce: Vec<u8>, plaintext: Vec<u8>, aad: Vec<u8>) -> Vec<u8>;
    async fn aes_gcm_decrypt(key: Vec<u8>, nonce: Vec<u8>, ciphertext: Vec<u8>, aad: Vec<u8>) -> Vec<u8>;
}

#[tarpc::service]
pub trait TlsService {
    async fn derive_handshake_secrets(params: HandshakeParams) -> HandshakeSecrets;
    async fn derive_application_secrets(params: AppParams) -> AppSecrets;
}

#[tarpc::service]
pub trait IdentityService {
    async fn validate_family_member(family_id: String, node_id: String) -> bool;
    async fn derive_broadcast_key(genesis_seed: Vec<u8>) -> Vec<u8>;
}
```

### Songbird (HTTP/TLS)

**Priority**: 🟡 Medium - Client of BearDog, server for other primals

**As client** (calling BearDog):
- Implement `rpc.escalate_to` to connect to BearDog via tarpc
- Update `BearDogProvider` to use tarpc when available

**As server** (serving Squirrel, etc.):
- Implement `rpc.tarpc_endpoint` to advertise HTTP service
- Create `HttpService` tarpc trait

**tarpc services**:
```rust
#[tarpc::service]
pub trait HttpService {
    async fn request(method: String, url: String, headers: HashMap<String, String>, body: Option<Vec<u8>>) -> HttpResponse;
    async fn get(url: String) -> HttpResponse;
    async fn post(url: String, body: Vec<u8>) -> HttpResponse;
}
```

### Squirrel (AI)

**Priority**: 🟢 Low - Primarily a client

**As client** (calling Songbird):
- Implement `rpc.escalate_to` to connect to Songbird via tarpc
- Update AI adapters to use tarpc when available

**As server** (serving Neural API):
- Implement `rpc.tarpc_endpoint` to advertise AI service

**tarpc services**:
```rust
#[tarpc::service]
pub trait AiService {
    async fn query(provider: String, prompt: String, max_tokens: u32) -> String;
    async fn list_providers() -> Vec<String>;
}
```

### Toadstool (Logging/Telemetry)

**Priority**: 🟢 Low - Receives logs, rarely queries

**As server**:
- Implement `rpc.tarpc_endpoint` for high-throughput log ingestion

**tarpc services**:
```rust
#[tarpc::service]
pub trait LogService {
    async fn log(level: String, message: String, metadata: HashMap<String, String>);
    async fn log_batch(entries: Vec<LogEntry>);
}
```

### NestGate (Networking)

**Priority**: 🟢 Low - Federation layer

**As server**:
- Implement `rpc.tarpc_endpoint` for peer-to-peer operations

---

## Testing Your Implementation

### Test 1: Verify tarpc Endpoint Advertisement

```bash
echo '{"jsonrpc":"2.0","method":"rpc.tarpc_endpoint","params":{},"id":1}' \
  | nc -U /run/user/1000/biomeos/beardog-nat0.sock

# Expected: {"result":{"available":true,"socket":"...tarpc.sock","services":[...]}}
```

### Test 2: Test Escalation (from Songbird to BearDog)

```bash
# First, tell Songbird to escalate to BearDog
echo '{"jsonrpc":"2.0","method":"rpc.escalate_to","params":{
  "target":"beardog",
  "tarpc_socket":"/run/user/1000/biomeos/beardog-nat0-tarpc.sock",
  "services":["CryptoService"]
},"id":1}' | nc -U /run/user/1000/biomeos/songbird-nat0.sock

# Expected: {"result":{"status":"connected","target":"beardog","protocol":"tarpc"}}
```

### Test 3: Test Fallback

```bash
echo '{"jsonrpc":"2.0","method":"rpc.fallback_to_json_rpc","params":{
  "target":"beardog",
  "reason":"manual_test"
},"id":1}' | nc -U /run/user/1000/biomeos/songbird-nat0.sock

# Expected: {"result":{"status":"fallback_complete","target":"beardog","protocol":"json_rpc"}}
```

### Test 4: Verify via Neural API

```bash
# Check protocol status
echo '{"jsonrpc":"2.0","method":"protocol.status","params":{},"id":1}' \
  | nc -U /run/user/1000/biomeos/neural-api-nat0.sock

# Expected: Shows connection protocols (JsonRpc, Tarpc, Degraded)
```

---

## Timeline

| Phase | Milestone | Target |
|-------|-----------|--------|
| 1 | BearDog tarpc server | Week 1 |
| 2 | Songbird dual-mode client | Week 2 |
| 3 | End-to-end escalation test | Week 3 |
| 4 | Other primals (Squirrel, etc.) | Week 4+ |

---

## Questions?

Contact the biomeOS team or review:
- `specs/LIVING_GRAPH_PROTOCOL_ESCALATION_SPEC.md` - Full specification
- `PROTOCOL_ESCALATION_ROADMAP.md` - Implementation roadmap
- `crates/biomeos-atomic-deploy/src/protocol_escalation.rs` - Reference implementation

---

## Dependencies

Your primal needs:
- `tarpc` crate (add to Cargo.toml)
- `tokio` async runtime (already have)
- Unix socket support (already have)

```toml
[dependencies]
tarpc = { version = "0.34", features = ["full"] }
```

---

**Status**: Ready for Implementation  
**Priority**: BearDog (High) → Songbird (Medium) → Others (Low)  
**Contact**: biomeOS Neural API Team

*Last Updated: January 28, 2026*

