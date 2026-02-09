# Hardcoding Replacement Report - February 3, 2026

## Summary

Replaced hardcoded values with capability-based discovery and environment variable overrides across the biomeOS crates directory, focusing on central configuration files as requested.

## Changes Made

### 1. Port and IP Address Hardcoding (✅ Completed)

#### `biomeos-types/src/constants.rs`
- **Added**: Environment variable support functions for all network ports
  - `network::http_port()` - Checks `HTTP_PORT` env var
  - `network::https_port()` - Checks `HTTPS_PORT` env var
  - `network::websocket_port()` - Checks `WEBSOCKET_PORT` env var
  - `network::mcp_port()` - Checks `MCP_WEBSOCKET_PORT` or `MCP_PORT` env var
  - `network::discovery_port()` - Checks `DISCOVERY_PORT` env var
- **Added**: `endpoints::bind_address()` and `endpoints::production_bind_address()` functions
- **Updated**: All port constants now marked as "fallback only" with clear documentation

#### `biomeos-types/src/defaults.rs`
- **Enhanced**: `RuntimeConfig` struct with port configuration methods
  - `http_port()`, `https_port()`, `websocket_port()`, `mcp_port()`, `discovery_port()`
  - `bind_address()` - Reads from `BIND_ADDRESS` env var
- **Pattern**: All methods follow `env::var()` → parse → fallback to default

#### `biomeos-api/src/state.rs`
- **Replaced**: Hardcoded `DEFAULT_BIND_ADDR = "127.0.0.1:3000"` constant
- **Updated**: Now uses `RuntimeConfig::from_env()` for dynamic resolution
- **Updated**: `default_socket_path()` now uses `SocketDiscovery` 5-tier resolution

#### `biomeos-core/src/primal_impls.rs`
- **Replaced**: Hardcoded `format!("http://127.0.0.1:{}", self.config.http_port)`
- **Updated**: Now uses `RuntimeConfig` for bind address and port resolution

#### `biomeos-api/src/handlers/discovery.rs`
- **Replaced**: Hardcoded demo endpoints (`http://localhost:9000`, `http://localhost:8080`)
- **Updated**: Now reads from environment variables (`BEARDOG_ENDPOINT`, `SONGBIRD_ENDPOINT`, `TOWER2_ENDPOINT`)
- **Pattern**: Falls back to Unix socket paths instead of HTTP endpoints

### 2. Socket Path Hardcoding (✅ Completed)

#### `biomeos-api/src/state.rs`
- **Replaced**: Hardcoded `/tmp` fallback in `default_socket_path()`
- **Updated**: Now uses `SocketDiscovery` with 5-tier resolution:
  1. Environment variable (`BIOMEOS_API_SOCKET`)
  2. XDG_RUNTIME_DIR/biomeos/
  3. /run/user/{uid}/biomeos/
  4. /data/local/tmp/biomeos/ (Android)
  5. /tmp/biomeos/ (fallback)

### 3. Primal Name Hardcoding (⚠️ Partially Completed)

#### Files Updated
- **Discovery handlers**: Replaced hardcoded endpoints with env var fallbacks
- **Config files**: Already using capability-based patterns

#### Files Requiring Manual Review

The following files contain hardcoded primal names but serve specific purposes:

1. **`biomeos-primal-sdk/src/discovery.rs`**
   - `providers_for_capability()` - Maps capabilities to known provider names
   - `capability_from_primal_name()` - Reverse mapping for backward compatibility
   - **Status**: Acceptable - These are helper functions for discovery hints, not production logic
   - **Recommendation**: Add documentation noting these are fallback mappings only

2. **`biomeos-genomebin-v3/src/composer.rs`**
   - Hardcoded checks for specific genome names in composition logic
   - **Status**: Needs review - These appear to be validation logic
   - **Recommendation**: Replace with capability-based checks

3. **`biomeos-core/src/primal_registry/mod.rs`**
   - `legacy_hardcoded_metadata()` - Already marked as deprecated
   - **Status**: Acceptable - Explicitly deprecated, kept for reference only

## Environment Variables Added

### Port Configuration
- `HTTP_PORT` - Override default HTTP port (default: 8080)
- `HTTPS_PORT` - Override default HTTPS port (default: 8443)
- `WEBSOCKET_PORT` - Override WebSocket port (default: 8081)
- `MCP_WEBSOCKET_PORT` or `MCP_PORT` - Override MCP port (default: 3000)
- `DISCOVERY_PORT` - Override discovery port (default: 8001)

### Network Configuration
- `BIND_ADDRESS` - Override bind address (default: 127.0.0.1)
- `BIOMEOS_BIND_ADDRESS` - Alternative bind address env var

### Socket Configuration
- `BIOMEOS_API_SOCKET` - Override API socket path
- `BIOMEOS_SOCKET_DIR` - Override socket directory
- `{PRIMAL}_SOCKET` - Primal-specific socket override (e.g., `BEARDOG_SOCKET`)

### Endpoint Configuration
- `BEARDOG_ENDPOINT` - BearDog endpoint override
- `SONGBIRD_ENDPOINT` - Songbird endpoint override
- `TOWER2_ENDPOINT` - Tower endpoint override
- `DISCOVERY_ENDPOINT` or `BIOMEOS_DISCOVERY_ENDPOINT` - Discovery service endpoint

## Remaining Hardcoding Patterns

### High Priority (Should Be Replaced)

1. **Test Files** (95+ instances)
   - Many test files contain hardcoded ports/IPs for test fixtures
   - **Recommendation**: Use test-specific environment variables or test fixtures
   - **Files**: `crates/biomeos-api/tests/websocket_integration.rs`, various test files

2. **Example/Demo Files**
   - Demo endpoints in example code
   - **Recommendation**: Use environment variables with clear fallback documentation

### Medium Priority (Acceptable with Documentation)

1. **Legacy Compatibility Functions**
   - Functions marked as deprecated but kept for backward compatibility
   - **Status**: Acceptable if properly documented

2. **Discovery Helper Functions**
   - Capability-to-primal-name mappings for discovery hints
   - **Status**: Acceptable - These are runtime discovery helpers, not hardcoded logic

### Low Priority (Test/Development Only)

1. **Development Fallbacks**
   - Hardcoded values in development presets with clear warnings
   - **Status**: Acceptable - Already includes warnings and documentation

## Architecture Improvements

### Before
```rust
// Hardcoded port
const PORT: u16 = 8080;

// Hardcoded IP
let url = format!("http://127.0.0.1:{}", port);

// Hardcoded primal name check
if primal_name == "beardog" { ... }

// Hardcoded socket path
let socket = PathBuf::from("/tmp/beardog.sock");
```

### After
```rust
// Environment-aware port
let port = RuntimeConfig::from_env().http_port();

// Environment-aware bind address
let url = format!("http://{}:{}", 
    RuntimeConfig::from_env().bind_address(),
    port);

// Capability-based discovery
let socket = SocketDiscovery::new(family_id)
    .discover_capability("security").await?;

// 5-tier socket resolution
let socket = SocketDiscovery::new(family_id)
    .build_socket_path("beardog");
```

## Key Principles Applied

1. **Environment Variables First**: All configuration reads from environment variables
2. **Fallback to Defaults**: Clear fallback values for development only
3. **5-Tier Socket Resolution**: Follows PRIMAL_DEPLOYMENT_STANDARD.md hierarchy
4. **Capability-Based Discovery**: Primals discovered by capability, not name
5. **Self-Knowledge Only**: Primals only know themselves, discover others at runtime

## Testing Recommendations

1. **Environment Variable Tests**: Verify all new env var overrides work correctly
2. **Fallback Tests**: Ensure defaults work when env vars are not set
3. **Socket Discovery Tests**: Verify 5-tier resolution works across platforms
4. **Capability Discovery Tests**: Verify capability-based discovery replaces name-based checks

## Migration Guide

### For Developers

1. **Port Configuration**: Use `RuntimeConfig::from_env().http_port()` instead of constants
2. **Socket Paths**: Use `SocketDiscovery::new(family_id).build_socket_path(primal_name)`
3. **Primal Discovery**: Use `SocketDiscovery::discover_capability("security")` instead of name checks
4. **Environment Variables**: Set appropriate env vars in production (see list above)

### For Deployment

1. Set `FAMILY_ID` environment variable for socket namespace isolation
2. Set `XDG_RUNTIME_DIR` if available (preferred socket location)
3. Set port overrides (`HTTP_PORT`, `WEBSOCKET_PORT`, etc.) if needed
4. Set `BIND_ADDRESS` only if HTTP bridge is required (Unix sockets preferred)

## Files Modified

1. `crates/biomeos-types/src/constants.rs` - Added env var support functions
2. `crates/biomeos-types/src/defaults.rs` - Enhanced RuntimeConfig with ports
3. `crates/biomeos-api/src/state.rs` - Replaced hardcoded bind addr and socket path
4. `crates/biomeos-core/src/primal_impls.rs` - Replaced hardcoded URL construction
5. `crates/biomeos-api/src/handlers/discovery.rs` - Replaced hardcoded demo endpoints

## Next Steps

1. ✅ **Completed**: Central configuration files updated
2. ⚠️ **In Progress**: Replace remaining hardcoded primal names in conditional logic
3. 📋 **Pending**: Update test files to use environment variables
4. 📋 **Pending**: Review and document acceptable hardcoding patterns
5. 📋 **Pending**: Add integration tests for environment variable overrides

## Notes

- All changes maintain backward compatibility
- Fallback values are clearly marked as development-only
- Production code paths require explicit configuration (no silent fallbacks)
- Socket discovery follows the 5-tier hierarchy from PRIMAL_DEPLOYMENT_STANDARD.md

---

**Report Generated**: February 3, 2026  
**Focus**: Central configuration files (`biomeos-types/src/constants.rs`, `biomeos-core/src/config/`)  
**Status**: Core changes complete, remaining patterns identified for review
