# 🧬 biomeOS Primal Integration Specification

**Version**: 1.0.0  
**Date**: January 25, 2026  
**Status**: 📋 **LIVING DOCUMENT** - Updated as ecosystem evolves  
**Authority**: biomeOS Team (Implementation of WateringHole Standards)

---

## 📜 Purpose

This document specifies **exactly what biomeOS expects from primals** to enable:
- ✅ Graph-based deployment via Neural API
- ✅ Capability-based discovery and routing
- ✅ Semantic translation of RPC calls
- ✅ TRUE PRIMAL architecture (no cross-primal knowledge)

**Audience**: Primal development teams implementing biomeOS compatibility.

---

## 🏛️ Foundation: WateringHole Standards

biomeOS enforces compliance with these ecosystem standards:

| Standard | Document | Required For |
|----------|----------|--------------|
| **UniBin** | `UNIBIN_ARCHITECTURE_STANDARD.md` | Binary structure |
| **ecoBin** | `ECOBIN_ARCHITECTURE_STANDARD.md` | Cross-compilation |
| **Primal IPC** | `PRIMAL_IPC_PROTOCOL.md` | Communication |

**Summary**:
- UniBin = Single binary with subcommands
- ecoBin = UniBin + Pure Rust (universal portability)
- IPC Protocol = JSON-RPC 2.0 over Unix sockets

---

## ✅ MANDATORY: Binary Structure (UniBin)

### **1. Binary Naming**

```
✅ CORRECT          ❌ INCORRECT
primal              primal-server
                    primal-client
                    primal-daemon
```

**biomeOS discovery expects**: Binary named after primal (no suffixes).

### **2. Subcommand Pattern**

```bash
<primal> <mode> [options]
```

**biomeOS expects these modes**:

| Mode | Purpose | Required |
|------|---------|----------|
| `server` | Long-running service | ✅ **MANDATORY** |
| `doctor` | Health diagnostics | ✅ **MANDATORY** |
| `--help` | Show all modes | ✅ **MANDATORY** |
| `--version` | Show version | ✅ **MANDATORY** |

**Example**:
```bash
beardog server --socket /tmp/beardog-nat0.sock --family-id nat0
beardog doctor --comprehensive
beardog --help
beardog --version
```

---

## ✅ MANDATORY: Server Mode CLI Flags

biomeOS spawns primals with specific CLI flags. **All primals MUST support these**:

### **Universal Flags (All Primals)**

| Flag | Purpose | Example |
|------|---------|---------|
| `--socket <path>` | Unix socket path | `--socket /tmp/beardog-nat0.sock` |
| `--family-id <id>` | Deployment family | `--family-id nat0` |

### **Implementation Pattern (Clap)**

```rust
#[derive(Parser)]
struct ServerCmd {
    /// Unix socket path for IPC
    #[arg(long)]
    socket: Option<String>,
    
    /// Family ID for multi-instance deployments
    #[arg(long, default_value = "nat0")]
    family_id: String,
    
    /// HTTP port (optional, for legacy compatibility)
    #[arg(long, short = 'p')]
    port: Option<u16>,
}
```

### **Socket Path Resolution**

biomeOS uses this precedence:

1. `--socket` CLI flag (highest priority)
2. `{PRIMAL}_SOCKET` environment variable
3. `/tmp/{primal}-{family_id}.sock` (default)

**Example**: For `beardog` with family `nat0`:
```bash
# CLI flag (preferred)
beardog server --socket /tmp/beardog-nat0.sock --family-id nat0

# Environment variable fallback
export BEARDOG_SOCKET=/tmp/beardog-nat0.sock
beardog server --family-id nat0

# Default (if no flag/env)
beardog server  # Uses /tmp/beardog-nat0.sock
```

---

## ✅ MANDATORY: IPC Protocol (JSON-RPC 2.0)

### **Transport**

- **Always**: Unix domain sockets
- **API**: `tokio::net::UnixListener` / `UnixStream`
- **Never**: HTTP for inter-primal communication (only Songbird for external)

### **Message Format**

**Request**:
```json
{
    "jsonrpc": "2.0",
    "method": "namespace.method_name",
    "params": {
        "param1": "value1",
        "param2": "value2"
    },
    "id": 1
}
```

**Success Response**:
```json
{
    "jsonrpc": "2.0",
    "result": {
        "output1": "value1"
    },
    "id": 1
}
```

**Error Response**:
```json
{
    "jsonrpc": "2.0",
    "error": {
        "code": -32601,
        "message": "Method not found",
        "data": null
    },
    "id": 1
}
```

### **Method Naming Convention**

```
{namespace}.{action}
```

**Examples**:
- `crypto.encrypt` - Encryption operation
- `crypto.x25519_generate_ephemeral` - Specific crypto operation
- `http.request` - HTTP request
- `tls.derive_secrets` - TLS key derivation

---

## 🔄 Capability Registration

### **On Startup**

Primals SHOULD register their capabilities with the discovery service (Songbird or Neural API).

**Registration Request** (to `/primal/songbird` or Neural API):
```json
{
    "jsonrpc": "2.0",
    "method": "ipc.register",
    "params": {
        "name": "beardog",
        "endpoint": "/tmp/beardog-nat0.sock",
        "capabilities": ["crypto", "tls", "security"],
        "version": "2.0.0",
        "methods": [
            "crypto.x25519_generate_ephemeral",
            "crypto.aes128_gcm_encrypt",
            "crypto.aes128_gcm_decrypt",
            "tls.derive_secrets",
            "tls.derive_handshake_secrets",
            "tls.derive_application_secrets"
        ]
    },
    "id": 1
}
```

### **Capability Categories**

| Capability | Description | Provider |
|------------|-------------|----------|
| `security` | Cryptographic operations | BearDog |
| `crypto` | Low-level crypto primitives | BearDog |
| `tls` | TLS-specific operations | BearDog |
| `http` | HTTP/HTTPS client | Songbird |
| `discovery` | Service discovery | Songbird |
| `ai` | AI/LLM operations | Squirrel |
| `storage` | Persistent storage | NestGate |
| `compute` | Computation tasks | ToadStool |

---

## 🔄 Semantic Translation (Neural API)

### **Purpose**

Neural API can translate **semantic capability names** to **primal-specific method names**.

This enables TRUE PRIMAL architecture where callers don't know specific primal implementations.

### **Translation Example**

**Caller sends** (semantic):
```json
{
    "method": "crypto.encrypt",
    "params": {"data": "...", "key": "..."}
}
```

**Neural API translates to** (primal-specific):
```json
{
    "method": "crypto.aes128_gcm_encrypt",
    "params": {"plaintext": "...", "key": "...", "nonce": "...", "aad": "..."}
}
```

### **Primal Capability Declarations**

Primals declare their semantic mappings in the deployment graph:

```toml
# In tower_atomic_bootstrap.toml

[nodes.capabilities_provided]
# Semantic → Actual method mappings
"crypto.encrypt" = "crypto.aes128_gcm_encrypt"
"crypto.decrypt" = "crypto.aes128_gcm_decrypt"
"crypto.generate_keypair" = "crypto.x25519_generate_ephemeral"

[nodes.parameter_mappings]
# Semantic → Actual parameter mappings
"crypto.ecdh_derive" = { "private_key" = "our_secret", "public_key" = "their_public" }
```

### **biomeOS Graph Node Structure**

```toml
[[nodes]]
id = "germinate_beardog"
output = "beardog_genesis"

[nodes.primal]
by_capability = "security"  # Discover by capability, not name!

[nodes.operation]
name = "start"

[nodes.operation.params]
mode = "server"
family_id = "nat0"

[nodes.operation.environment]
RUST_LOG = "info"

[nodes.constraints]
timeout_ms = 30000

# Semantic capabilities this primal provides
[nodes.capabilities_provided]
"crypto.encrypt" = "crypto.aes128_gcm_encrypt"
"crypto.decrypt" = "crypto.aes128_gcm_decrypt"
"crypto.hash" = "crypto.blake3_hash"
```

---

## 📡 biomeOS Environment Variables

### **Primal-Specific Socket Variables**

biomeOS sets these environment variables when spawning primals:

| Variable | Purpose | Example |
|----------|---------|---------|
| `BEARDOG_SOCKET` | BearDog socket path | `/tmp/beardog-nat0.sock` |
| `SONGBIRD_SOCKET` | Songbird socket path | `/tmp/songbird-nat0.sock` |
| `SQUIRREL_SOCKET` | Squirrel socket path | `/tmp/squirrel-nat0.sock` |
| `NESTGATE_SOCKET` | NestGate socket path | `/tmp/nestgate-nat0.sock` |
| `TOADSTOOL_SOCKET` | ToadStool socket path | `/tmp/toadstool-nat0.sock` |
| `NEURAL_API_SOCKET` | Neural API socket path | `/tmp/neural-api-nat0.sock` |

### **General Variables**

| Variable | Purpose | Example |
|----------|---------|---------|
| `FAMILY_ID` | Deployment family | `nat0` |
| `BIOMEOS_FAMILY_ID` | Alternative family ID | `nat0` |
| `RUST_LOG` | Log level | `info`, `debug`, `trace` |
| `SSLKEYLOGFILE` | TLS key log (debug) | `/tmp/sslkeys.log` |

### **Security Provider Variables**

For primals that need cryptographic services:

| Variable | Purpose | Example |
|----------|---------|---------|
| `SECURITY_ENDPOINT` | Security provider socket | `/tmp/beardog-nat0.sock` |
| `SONGBIRD_SECURITY_PROVIDER` | Songbird → BearDog | `/tmp/beardog-nat0.sock` |
| `BEARDOG_MODE` | BearDog client mode | `direct`, `neural_api` |

---

## 🐻 BearDog Integration (Reference Implementation)

### **Gold Standard CLI**

BearDog is the reference implementation for UniBin + IPC:

```bash
beardog server --socket /tmp/beardog-nat0.sock --family-id nat0
```

### **JSON-RPC Methods**

| Method | Description | Params |
|--------|-------------|--------|
| `crypto.x25519_generate_ephemeral` | Generate X25519 keypair | none |
| `crypto.x25519_derive_secret` | ECDH key exchange | `our_secret`, `their_public` |
| `crypto.aes128_gcm_encrypt` | AES-128-GCM encrypt | `key`, `nonce`, `plaintext`, `aad` |
| `crypto.aes128_gcm_decrypt` | AES-128-GCM decrypt | `key`, `nonce`, `ciphertext`, `aad` |
| `crypto.blake3_hash` | BLAKE3 hash | `data` |
| `tls.derive_handshake_secrets` | TLS 1.3 handshake keys | `shared_secret`, `transcript_hash` |
| `tls.derive_application_secrets` | TLS 1.3 app keys | `handshake_secret`, `transcript_hash` |

---

## 🐦 Songbird Integration (HTTP/TLS Primal)

### **Required Evolution** ⚠️

Songbird currently needs to add Unix socket IPC for HTTPS:

```bash
# Target CLI (needs implementation)
songbird server --socket /tmp/songbird-nat0.sock --port 8080 --family-id nat0
```

### **Expected JSON-RPC Methods**

| Method | Description | Params |
|--------|-------------|--------|
| `http.request` | Generic HTTPS request | `url`, `method`, `headers`, `body`, `timeout_ms` |
| `http.get` | HTTP GET convenience | `url` |
| `http.post` | HTTP POST convenience | `url`, `body`, `content_type` |
| `discovery.announce` | Announce capabilities | `name`, `endpoint`, `capabilities` |
| `discovery.query` | Find by capability | `capability` |

### **See**: `SONGBIRD_IPC_EVOLUTION_REQUIRED_JAN_25_2026.md`

---

## 🧪 Health Check Protocol

### **Doctor Subcommand**

All primals MUST implement `doctor` subcommand:

```bash
beardog doctor
# Output: Health status, diagnostics, connectivity
```

### **Health JSON-RPC Method**

Primals SHOULD expose health check via IPC:

```json
// Request
{"jsonrpc": "2.0", "method": "health.check", "params": {}, "id": 1}

// Response
{
    "jsonrpc": "2.0",
    "result": {
        "status": "healthy",
        "version": "2.0.0",
        "uptime_seconds": 3600,
        "capabilities": ["crypto", "tls"],
        "connections": {
            "active": 5,
            "total": 100
        }
    },
    "id": 1
}
```

---

## 📊 Compliance Checklist

### **Level 1: UniBin Compliance** (Minimum)

- [ ] Single binary named after primal
- [ ] `server` subcommand for service mode
- [ ] `doctor` subcommand for diagnostics
- [ ] `--help` shows all modes
- [ ] `--version` shows version

### **Level 2: IPC Compliance** (Required for biomeOS)

- [ ] `--socket <path>` CLI flag
- [ ] `--family-id <id>` CLI flag
- [ ] Unix socket listener in server mode
- [ ] JSON-RPC 2.0 protocol
- [ ] Proper error responses

### **Level 3: Capability Compliance** (Full Integration)

- [ ] Capability registration on startup
- [ ] Semantic method naming (`namespace.action`)
- [ ] Parameter mapping support
- [ ] Health check endpoint

### **Level 4: ecoBin Compliance** (Optional, Recommended)

- [ ] Pure Rust (zero C dependencies)
- [ ] musl cross-compilation works
- [ ] Universal portability

---

## 🔧 biomeOS Spawning Behavior

### **Binary Discovery**

biomeOS searches for binaries in this order:

1. `BIOMEOS_PLASMID_BIN_DIR` environment variable
2. `./plasmidBin` directory
3. `../plasmidBin` directory
4. `../../plasmidBin` directory

**Binary patterns searched**:
```
{primal}_{arch}_{os}_musl/{primal}  # e.g., beardog_x86_64_linux_musl/beardog
{primal}_{arch}_{os}/{primal}       # e.g., beardog_x86_64_linux/beardog
primals/{primal}/{primal}           # e.g., primals/beardog/beardog
{primal}/{primal}                   # e.g., beardog/beardog
{primal}                            # e.g., beardog
```

### **Process Spawning**

biomeOS spawns primals with:

```rust
// Conceptual spawning logic
let mut cmd = Command::new(&binary_path);
cmd.arg("server");
cmd.arg("--socket").arg(&socket_path);
cmd.arg("--family-id").arg(&family_id);
cmd.env("FAMILY_ID", &family_id);
cmd.env("RUST_LOG", &log_level);

// Primal-specific env vars
match primal_name {
    "songbird" => {
        let beardog_socket = context.get_socket_path("beardog").await;
        cmd.env("BEARDOG_SOCKET", &beardog_socket);
        cmd.env("SECURITY_ENDPOINT", &beardog_socket);
    }
    _ => {}
}

cmd.spawn()?;
```

### **Socket Availability Wait**

biomeOS waits for socket to appear before continuing:

```rust
// Wait up to 3 seconds for socket
for _ in 0..30 {
    if PathBuf::from(&socket_path).exists() {
        return Ok(());
    }
    tokio::time::sleep(Duration::from_millis(100)).await;
}
Err(anyhow!("Timeout waiting for socket"))
```

---

## 📚 Reference Documents

### **WateringHole Standards**
- `wateringHole/UNIBIN_ARCHITECTURE_STANDARD.md`
- `wateringHole/ECOBIN_ARCHITECTURE_STANDARD.md`
- `wateringHole/PRIMAL_IPC_PROTOCOL.md`
- `wateringHole/INTER_PRIMAL_INTERACTIONS.md`

### **biomeOS Implementation**
- `biomeOS/crates/biomeos-atomic-deploy/src/executor/primal_spawner.rs`
- `biomeOS/crates/biomeos-atomic-deploy/src/executor/context.rs`
- `biomeOS/crates/biomeos-types/src/defaults.rs`
- `biomeOS/graphs/tower_atomic_bootstrap.toml`

### **Primal-Specific**
- `biomeOS/SONGBIRD_IPC_EVOLUTION_REQUIRED_JAN_25_2026.md`
- `wateringHole/btsp/BEARDOG_TECHNICAL_STACK.md`

---

## 🎯 Summary

**For a primal to be fully biomeOS-compatible**:

1. **UniBin**: Single binary with `server`, `doctor`, `--help`, `--version`
2. **CLI Flags**: Support `--socket` and `--family-id`
3. **IPC**: JSON-RPC 2.0 over Unix socket
4. **Registration**: Announce capabilities on startup
5. **Methods**: Use `namespace.action` naming convention
6. **Health**: Expose `health.check` method

**biomeOS provides**:
- ✅ Graph-based orchestration
- ✅ Capability-based discovery
- ✅ Semantic translation
- ✅ Socket path management
- ✅ Environment variable injection
- ✅ Process lifecycle management

---

**"Primals speak IPC, biomeOS orchestrates symphonies!"** 🎵🦀

---

*Document Version: 1.0.0*  
*Last Updated: January 25, 2026*  
*Maintainer: biomeOS Team*

