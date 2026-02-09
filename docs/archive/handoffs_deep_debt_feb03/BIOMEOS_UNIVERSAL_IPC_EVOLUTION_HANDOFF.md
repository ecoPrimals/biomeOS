# biomeOS Universal IPC Evolution Handoff

**Date**: January 29, 2026  
**Status**: Implemented  
**Standard**: Universal IPC Standard v3.0

---

## Summary

Evolved biomeOS IPC system to comply with Universal IPC Standard v3.0, adding multi-transport support (Unix, Abstract, TCP) with automatic Tier 1 → Tier 2 fallback.

---

## Changes Made

### 1. socket_discovery.rs (Major Evolution)

**File**: `crates/biomeos-core/src/socket_discovery.rs`

#### New Types

```rust
/// Transport endpoint for connecting to a primal (Universal IPC v3.0)
pub enum TransportEndpoint {
    UnixSocket { path: PathBuf },      // Tier 1 - Linux, macOS
    AbstractSocket { name: String },    // Tier 1 - Linux, Android
    TcpSocket { host: String, port: u16 }, // Tier 2 - Universal
}
```

#### New Methods

- `discover_with_fallback()` - Multi-transport discovery with automatic fallback
- `discover_endpoint_via_env()` - Environment variable discovery for all transports
- `try_unix_socket_xdg()` - XDG runtime directory discovery
- `try_abstract_socket()` - Abstract socket discovery (Linux/Android)
- `try_tcp_fallback()` - TCP fallback discovery
- `verify_unix_socket()` - Actual connection verification
- `verify_tcp_connection()` - TCP connection verification
- `calculate_primal_port()` - Deterministic port assignment

#### Strategy Updates

```rust
pub struct DiscoveryStrategy {
    // ... existing fields ...
    pub try_abstract_sockets: bool,     // NEW
    pub enable_tcp_fallback: bool,      // NEW
    pub tcp_port_start: u16,            // NEW
    pub tcp_fallback_host: String,      // NEW
}

impl DiscoveryStrategy {
    pub fn android() -> Self { ... }      // NEW
    pub fn cross_device() -> Self { ... } // NEW
}
```

### 2. atomic_client.rs (Major Evolution)

**File**: `crates/biomeos-core/src/atomic_client.rs`

#### New Constructors

```rust
impl AtomicClient {
    pub fn from_endpoint(endpoint: TransportEndpoint) -> Self;
    pub fn unix(socket_path: impl AsRef<Path>) -> Self;
    pub fn tcp(host: impl Into<String>, port: u16) -> Self;
    #[cfg(target_os = "linux")]
    pub fn abstract_socket(name: impl Into<String>) -> Self;
}
```

#### Multi-Transport Call Dispatch

```rust
async fn call_impl(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
    match &self.endpoint {
        TransportEndpoint::UnixSocket { path } => self.call_via_unix(path, request).await,
        TransportEndpoint::TcpSocket { host, port } => self.call_via_tcp(host, *port, request).await,
        TransportEndpoint::AbstractSocket { name } => self.call_via_abstract(name, request).await,
    }
}
```

#### Generic Stream Handler

```rust
async fn send_request<S>(&self, stream: S, request: JsonRpcRequest) -> Result<JsonRpcResponse>
where
    S: AsyncRead + AsyncWrite + Unpin,
```

### 3. lib.rs (Export Updates)

**File**: `crates/biomeos-core/src/lib.rs`

```rust
// Universal IPC v3.0 exports
pub use socket_discovery::{
    discover_endpoint, discover_socket, DiscoveredSocket, DiscoveryMethod,
    DiscoveryStrategy, SocketDiscovery, TransportEndpoint,
};

pub use atomic_client::{
    discover_primal_endpoint, AtomicClient, AtomicPrimalClient,
    ExecutionResult, JsonRpcRequest, JsonRpcResponse,
};
```

---

## Compliance Assessment

| Requirement | Status | Notes |
|-------------|--------|-------|
| Primal Autonomy | ✅ | Own IPC implementation, no cross-primal deps |
| Unix Sockets | ✅ | Fully implemented |
| Abstract Sockets | ✅ | Linux/Android support |
| TCP Fallback | ✅ | Universal Tier 2 support |
| Runtime Discovery | ✅ | 5-tier resolution + fallback |
| Graceful Fallback | ✅ | Tier 1 → Tier 2 automatic |
| JSON-RPC 2.0 | ✅ | Fully implemented |
| No Unsafe Code | ✅ | Pure safe Rust |

---

## Discovery Order (Universal IPC v3.0)

1. **Environment hints**: `{PRIMAL}_SOCKET`, `{PRIMAL}_TCP`, `{PRIMAL}_ENDPOINT`
2. **XDG runtime**: `$XDG_RUNTIME_DIR/biomeos/{primal}-{family}.sock`
3. **Abstract socket**: `@biomeos_{primal}_{family}` (Linux/Android)
4. **Family /tmp**: `/tmp/{primal}-{family}.sock`
5. **Capability registry**: Query Neural API
6. **TCP fallback**: `127.0.0.1:{calculated_port}`

---

## Usage Examples

### Auto-Discovery with Fallback

```rust
use biomeos_core::{AtomicClient, AtomicPrimalClient};

// Automatic transport selection
let client = AtomicClient::discover("beardog").await?;
let result = client.call("ping", serde_json::json!({})).await?;

// High-level client
let primal = AtomicPrimalClient::discover("songbird").await?;
primal.health_check().await?;
```

### Explicit Transport

```rust
// Unix socket (Tier 1)
let client = AtomicClient::unix("/tmp/beardog.sock");

// TCP socket (Tier 2)
let client = AtomicClient::tcp("192.168.1.100", 9100);

// Abstract socket (Linux/Android, Tier 1)
#[cfg(target_os = "linux")]
let client = AtomicClient::abstract_socket("biomeos_beardog_nat0");
```

### Cross-Device Strategy

```rust
use biomeos_core::DiscoveryStrategy;

let strategy = DiscoveryStrategy::cross_device();
let discovery = SocketDiscovery::with_strategy("nat0", strategy);
let endpoint = discovery.discover_with_fallback("beardog").await?;
```

---

## Transport Tier Reference

| Tier | Transport | Platform | Performance | Cross-Device |
|------|-----------|----------|-------------|--------------|
| 1 | Unix Socket | Linux, macOS | Fastest | No |
| 1 | Abstract Socket | Linux, Android | Fast | No |
| 2 | TCP Socket | All | Good | Yes |

---

## Testing

Run the test suite:

```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
cargo test -p biomeos-core socket_discovery
cargo test -p biomeos-core atomic_client
```

### Test Coverage

- `test_transport_endpoint_parse_*` - Endpoint parsing
- `test_transport_endpoint_tier` - Tier classification
- `test_discovery_strategy_*` - Strategy configurations
- `test_discovered_socket_*` - Socket construction
- `test_atomic_client_*` - Client constructors
- `test_calculate_primal_port` - Port calculation

---

## Remaining Work

### biomeos-federation Refactoring (Optional)

The `biomeos-federation` crate has similar discovery patterns that could be migrated to use `SocketDiscovery`:

- `discovery.rs:discover_songbird_socket()` → Use `SocketDiscovery`
- `subfederation.rs:find_beardog_socket()` → Use `SocketDiscovery`

This is not critical since the current implementation follows good patterns (env vars → XDG → fallback).

### Error Handling Improvements (Optional)

The audit identified 141 `.unwrap()` calls. Most are in test code, but production code would benefit from context-rich error handling.

---

## Architecture Notes

### Why Multi-Transport?

1. **Android SELinux**: Blocks filesystem Unix sockets, but abstract sockets work
2. **Cross-Device**: TCP enables Pixel ↔ Desktop coordination
3. **WASM**: Only TCP/HTTP supported in browser environments
4. **Network Isolation**: Containers may only have TCP access

### Primal Autonomy Preserved

biomeOS owns its IPC implementation:
- No imports from phase1 primals (beardog, songbird, etc.)
- References to primals are string names for runtime discovery
- Zero shared IPC crates

---

## Related Documents

- `wateringHole/UNIVERSAL_IPC_STANDARD_V3.md` - Ecosystem standard
- `wateringHole/handoffs/UNIVERSAL_IPC_EVOLUTION_HANDOFF.md` - Primal teams handoff
- `specs/PRIMAL_DEPLOYMENT_STANDARD.md` - Deployment conventions

---

## Conclusion

biomeOS now fully complies with Universal IPC Standard v3.0:
- Multi-transport support (Unix, Abstract, TCP)
- Automatic Tier 1 → Tier 2 fallback
- Platform-agnostic transport selection
- Runtime discovery with no hardcoded primal knowledge
- Pure Rust, safe code, no external dependencies for IPC
