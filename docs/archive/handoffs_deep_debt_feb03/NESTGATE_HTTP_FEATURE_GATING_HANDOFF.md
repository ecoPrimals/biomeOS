# NestGate HTTP Feature-Gating Handoff

**Date**: February 3, 2026  
**Priority**: 🟡 MEDIUM  
**Location**: `/home/eastgate/Development/ecoPrimals/phase1/nestgate`  
**Estimated Work**: 2-3 hours

---

## Summary

NestGate currently includes HTTP server dependencies (axum, tower-http, warp) as unconditional dependencies. Per the PRIMAL_DEPLOYMENT_STANDARD, HTTP should be optional and socket-only should be the default mode.

**Good News**: The CLI already defaults to `--socket-only true` and has `--enable-http` flag.

**Issue**: HTTP dependencies are still compiled even in socket-only builds, increasing binary size and violating the "minimal dependency" principle.

---

## Current State

### CLI (Already Correct)

```rust
// code/crates/nestgate-bin/src/cli.rs - Lines 103-111
/// Run in Unix socket-only mode (no HTTP server, no external dependencies)
/// Perfect for NUCLEUS atomic patterns and inter-primal communication
/// NOTE: Socket-only is now the DEFAULT per PRIMAL_DEPLOYMENT_STANDARD
#[arg(long, default_value_t = true)]
socket_only: bool,
/// Enable HTTP server mode (legacy/standalone mode)
/// Only use when HTTP API is explicitly required
#[arg(long, conflicts_with = "socket_only")]
enable_http: bool,
```

### Dependencies (Need Feature-Gating)

```toml
# code/crates/nestgate-api/Cargo.toml - Currently unconditional
[dependencies.axum]
workspace = true

[dependencies.tower]
workspace = true

[dependencies.tower-http]
workspace = true

[dependencies.warp]
workspace = true

[dependencies.hyper]
version = "0.14"
```

---

## Implementation Plan

### Phase 1: Feature Definition (30 min)

#### 1.1 Update `nestgate-api/Cargo.toml`

```toml
[features]
default = ["sse", "streaming-rpc"]
sse = []
streaming-rpc = ["tarpc", "bincode", "tokio-util", "tokio-serde"]
dev-stubs = []

# NEW: HTTP feature-gate
http = ["dep:axum", "dep:tower-http", "dep:warp"]

[dependencies.axum]
workspace = true
optional = true  # NEW

[dependencies.tower-http]
workspace = true
optional = true  # NEW

[dependencies.warp]
workspace = true
optional = true  # NEW
```

#### 1.2 Update `nestgate-bin/Cargo.toml`

```toml
[features]
default = []
http = ["nestgate-api/http"]  # Propagate feature
```

### Phase 2: Code Conditional Compilation (1-2 hours)

#### 2.1 HTTP Server Module

Wrap HTTP server code with `#[cfg(feature = "http")]`:

```rust
// code/crates/nestgate-api/src/server.rs (example)
#[cfg(feature = "http")]
pub mod http_server {
    use axum::{Router, routing::get};
    use tower_http::trace::TraceLayer;
    
    pub async fn start_http_server(port: u16, bind: &str) -> Result<(), Error> {
        // HTTP server implementation
    }
}

#[cfg(not(feature = "http"))]
pub mod http_server {
    pub async fn start_http_server(_port: u16, _bind: &str) -> Result<(), Error> {
        Err(Error::msg("HTTP feature not enabled. Use --socket-only mode or rebuild with --features http"))
    }
}
```

#### 2.2 Main Daemon Logic

```rust
// code/crates/nestgate-bin/src/commands/service.rs
pub async fn run_daemon(port: u16, bind: &str, dev: bool, socket_only: bool) -> Result<()> {
    if socket_only {
        // Socket-only mode (always available)
        start_socket_server().await
    } else {
        // HTTP mode (requires feature)
        #[cfg(feature = "http")]
        {
            start_http_server(port, bind).await
        }
        
        #[cfg(not(feature = "http"))]
        {
            anyhow::bail!("HTTP mode requires --features http. Use --socket-only instead.")
        }
    }
}
```

### Phase 3: Build Verification (30 min)

#### 3.1 Test Socket-Only Build

```bash
cd /home/eastgate/Development/ecoPrimals/phase1/nestgate

# Build without HTTP
cargo build --release --no-default-features

# Verify binary size reduction
ls -la target/release/nestgate

# Test socket-only mode
./target/release/nestgate daemon --socket-only
```

#### 3.2 Test HTTP Build

```bash
# Build with HTTP
cargo build --release --features http

# Verify HTTP works
./target/release/nestgate daemon --enable-http --port 8085
```

### Phase 4: Documentation Update (15 min)

Update README and QUICK_START to document:

```markdown
## Build Options

### Socket-Only Mode (Default, Recommended)
```bash
# Minimal build - no HTTP dependencies
cargo build --release

# Run
nestgate daemon  # Socket-only by default
```

### HTTP Mode (Legacy/Standalone)
```bash
# Build with HTTP server
cargo build --release --features http

# Run
nestgate daemon --enable-http --port 8085
```
```

---

## Files to Modify

1. `code/crates/nestgate-api/Cargo.toml` - Add http feature
2. `code/crates/nestgate-bin/Cargo.toml` - Propagate http feature
3. `code/crates/nestgate-api/src/server.rs` - Conditional HTTP code
4. `code/crates/nestgate-bin/src/commands/service.rs` - Runtime check
5. `README.md` - Document build options
6. `QUICK_START.md` - Update examples

---

## Expected Benefits

| Metric | Before | After |
|--------|--------|-------|
| Binary Size (socket-only) | ~4.9 MB | ~3.5 MB (est.) |
| Dependencies | Always HTTP | HTTP optional |
| Compile Time | Slower | Faster without HTTP |
| Attack Surface | HTTP always | Minimal by default |

---

## Verification Checklist

- [ ] Socket-only build compiles without axum/tower-http/warp
- [ ] Socket-only binary is smaller
- [ ] `nestgate daemon` works (socket-only default)
- [ ] `nestgate daemon --enable-http` errors without feature
- [ ] `nestgate daemon --enable-http` works with `--features http`
- [ ] All existing tests pass
- [ ] Documentation updated

---

## Notes

- This work is in the **phase1/nestgate** repository, not biomeOS
- The CLI is already correct (`socket_only = true` default)
- This is a build-time optimization, not a runtime change
- Existing deployments continue to work

---

**Status**: 📝 HANDOFF READY  
**Next**: Execute in phase1/nestgate repository
