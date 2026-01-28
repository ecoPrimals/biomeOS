# Socket Discovery - Capability-Based Runtime Discovery

> **Deep Debt Solution**: Replaces hardcoded `/tmp/{primal}.sock` paths with capability-based discovery.

## Overview

Socket Discovery provides runtime socket path resolution without hardcoding:
- **No Hardcoded Paths**: Sockets discovered at runtime
- **XDG Compliance**: Respects XDG_RUNTIME_DIR
- **Family Isolation**: Sockets namespaced by family_id
- **Capability Discovery**: Find primals by capability, not location
- **Caching**: Efficient with TTL-based cache

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                   SOCKET DISCOVERY ORDER                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. Environment Hint    → BEARDOG_SOCKET=/path/to/socket        │
│  2. XDG Runtime Dir     → /run/user/1000/biomeos/beardog.sock   │
│  3. Family-Scoped /tmp  → /tmp/beardog-nat0.sock                │
│  4. Capability Registry → Query Neural API                       │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Usage

### Basic Discovery

```rust
use biomeos_core::socket_discovery::SocketDiscovery;

// Create discovery engine
let discovery = SocketDiscovery::new("nat0");

// Discover by primal name
let socket = discovery.discover_primal("beardog").await;
if let Some(discovered) = socket {
    println!("BearDog at: {}", discovered.path.display());
    println!("Discovered via: {:?}", discovered.discovered_via);
}

// Discover by capability
let crypto_socket = discovery.discover_capability("crypto").await;
// Returns BearDog's socket
```

### Convenience Functions

```rust
use biomeos_core::socket_discovery::{discover_socket, build_socket};

// Quick discovery (uses FAMILY_ID from environment)
let socket = discover_socket("beardog").await;

// Build deterministic path (for primals to register their own sockets)
let path = build_socket("beardog", "nat0");
// Returns: /run/user/1000/biomeos/beardog-nat0.sock (or /tmp fallback)
```

### With Neural API Integration

```rust
let discovery = SocketDiscovery::new("nat0")
    .with_neural_api(PathBuf::from("/tmp/neural-api.sock"));

// Now capability queries route through Neural API
let ai_socket = discovery.discover_capability("ai").await;
```

## Discovery Methods

### 1. Environment Hints
Checked first for explicit configuration:
```bash
export BEARDOG_SOCKET=/custom/path/beardog.sock
export SONGBIRD_SOCKET_PATH=/run/songbird.sock
export BIOMEOS_SQUIRREL_SOCKET=/tmp/squirrel.sock
```

### 2. XDG Runtime Directory
XDG-compliant path (Linux/macOS):
```
$XDG_RUNTIME_DIR/biomeos/{primal}-{family}.sock
Example: /run/user/1000/biomeos/beardog-nat0.sock
```

### 3. Family-Scoped /tmp
Fallback with family namespace:
```
/tmp/{primal}-{family}.sock
Example: /tmp/beardog-nat0.sock
```

### 4. Capability Registry
Query Neural API for capability→socket mapping:
```json
// Neural API request
{
  "method": "capability.discover",
  "params": {"capability": "crypto"}
}
// Returns socket path for crypto provider (BearDog)
```

## Capability→Primal Mapping

Built-in fallback mappings (when registry unavailable):

| Capability | Primal |
|------------|--------|
| `crypto`, `security`, `tls`, `genetic` | BearDog |
| `http`, `discovery`, `network`, `mesh` | Songbird |
| `ai`, `inference`, `learning` | Squirrel |
| `compute`, `workload`, `orchestration` | Toadstool |
| `storage`, `data`, `persistence` | NestGate |

## Configuration

### Discovery Strategy

```rust
let strategy = DiscoveryStrategy {
    check_env_hints: true,    // Check env vars first
    use_xdg_runtime: true,    // Use XDG_RUNTIME_DIR
    use_family_tmp: true,     // Use /tmp with family namespace
    query_registry: true,     // Query Neural API
    scan_sockets: false,      // Expensive socket scanning (disabled)
    enable_cache: true,       // Cache discovered sockets
    cache_ttl_secs: 60,       // Cache TTL
};

let discovery = SocketDiscovery::with_strategy("nat0", strategy);
```

### Caching

```rust
// Cache is enabled by default
let discovery = SocketDiscovery::new("nat0");

// Clear cache when needed
discovery.clear_cache().await;

// Cache entries expire after TTL (default: 60s)
```

## Migration Guide

### Before (Hardcoded)

```rust
// ❌ Hardcoded path
let socket = PathBuf::from("/tmp/beardog.sock");

// ❌ Hardcoded with family
let socket = format!("/tmp/beardog-{}.sock", family_id);
```

### After (Discovery)

```rust
// ✅ Discovery by name
let discovery = SocketDiscovery::new(family_id);
let socket = discovery.get_socket_path("beardog").await
    .expect("BearDog not found");

// ✅ Discovery by capability
let crypto_socket = discovery.discover_capability("crypto").await
    .expect("No crypto provider found");

// ✅ Convenience function
let socket = discover_socket("beardog").await
    .expect("BearDog not found");
```

## TRUE PRIMAL Principles

1. **Self-Knowledge Only**: Primals only know their own socket path
2. **Runtime Discovery**: Other primals discovered at runtime
3. **Capability-Based**: Query by capability, not hardcoded name
4. **Graceful Degradation**: Falls through discovery methods
5. **Platform Agnostic**: XDG on Linux, appropriate fallbacks elsewhere

## Files

- `crates/biomeos-core/src/socket_discovery.rs` - Core discovery engine

## Environment Variables

| Variable | Description |
|----------|-------------|
| `FAMILY_ID` | Default family ID for discovery |
| `BIOMEOS_FAMILY_ID` | Alternative family ID |
| `{PRIMAL}_SOCKET` | Explicit socket path hint |
| `{PRIMAL}_SOCKET_PATH` | Alternative socket path hint |
| `NEURAL_API_SOCKET` | Neural API socket for registry queries |
| `XDG_RUNTIME_DIR` | XDG runtime directory |

---

*Created: January 28, 2026*
*Status: Production Ready*

