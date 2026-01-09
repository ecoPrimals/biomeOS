# 🎯 BearDog Standalone Service Requirements

**Date**: January 8, 2026 (Late Evening)  
**Status**: Architecture Clarification  
**For**: BearDog Team

---

## 🔍 Architecture Clarification

### ✅ CORRECT: BearDog is a Standalone Primal

```
Tower (Orchestrator)
  ├─ Spawns → BearDog (standalone primal service)
  ├─ Spawns → Songbird (standalone primal service)
  └─ Spawns → Other primals...
  
Communication: IPC via Unix sockets + JSON-RPC/tarpc
```

### 🔮 FUTURE: Chimera Pattern (Optional)

```
Songbird-BearDog Chimera
  └─ Embeds BearDog for tighter coupling
  └─ Deployed as single unit for specific niches
```

**Note**: The embeddable pattern is great for future chimeras, but NOT the current need!

---

## 🎯 What BearDog Needs as a Standalone Service

### 1. Proper Binary Target ✅/❌

**Current State**: `beardog-server` binary exists but source unclear
**Needed**: Clear binary definition in Cargo.toml

```toml
[[bin]]
name = "beardog-server"
path = "src/bin/beardog-server.rs"  # or crates/beardog-tunnel/src/bin/server.rs
```

### 2. Configurable Port Binding ❌

**Current**: Binds to `0.0.0.0:0` (auto-select port)
**Needed**: Respect environment variable

```rust
// Read from environment
let bind_addr = std::env::var("BEARDOG_BIND_ADDR")
    .unwrap_or_else(|_| "0.0.0.0:9000".to_string())
    .parse()?;

// Or from HTTP_PORT
let port = std::env::var("HTTP_PORT")
    .unwrap_or_else(|_| "9000".to_string())
    .parse()?;
let bind_addr = SocketAddr::from(([0, 0, 0, 0], port));
```

### 3. Service Lifecycle ❌

**Current**: Example exits after initialization
**Needed**: Stay running until SIGTERM/SIGINT

```rust
// Start server
let server_handle = tokio::spawn(async move {
    server.serve().await
});

// Wait for signal
tokio::signal::ctrl_c().await?;

// Graceful shutdown
info!("Shutting down BearDog server...");
server_handle.abort();
```

### 4. Unix Socket IPC ✅

**Current**: Has Unix socket support
**Path**: `/tmp/primals/beardog-{node}.sock`
**Status**: Appears to be working

### 5. Environment Configuration ✅/❌

**Working**:
- ✅ `BEARDOG_HSM_MODE=software` (HSM initialization)

**Needed**:
- ❌ `BEARDOG_BIND_ADDR` or `HTTP_PORT` (port configuration)
- ❌ `BEARDOG_FAMILY_SEED` (read from tower.toml)
- ❌ `NODE_ID` (for socket path)

### 6. Health Endpoint ✅

**Current**: Has `/health` endpoint
**Status**: Working when server stays running

### 7. Process Management ❌

**Needed**:
- Proper signal handling (SIGTERM, SIGINT)
- Graceful shutdown
- Log to stdout/stderr (for tower to capture)
- Don't daemonize (tower manages the process)

---

## 📋 Specific Gaps to Address

### Gap 1: Port Configuration

```rust
// Current (from embeddable example):
let config = BearDogApiServerConfig::default();  // Binds to 0.0.0.0:0

// Needed:
let config = BearDogApiServerConfig {
    bind_addr: std::env::var("BEARDOG_BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:9000".to_string())
        .parse()?,
    enable_cors: std::env::var("BEARDOG_ENABLE_CORS")
        .map(|v| v.parse().unwrap_or(true))
        .unwrap_or(true),
    version: env!("CARGO_PKG_VERSION").to_string(),
};
```

### Gap 2: Service Lifecycle

```rust
// Current (from embeddable example):
server.serve().await?;  // Exits immediately after binding

// Needed:
let server_task = tokio::spawn(async move {
    if let Err(e) = server.serve().await {
        error!("Server error: {}", e);
    }
});

// Wait for shutdown signal
tokio::select! {
    _ = tokio::signal::ctrl_c() => {
        info!("Received SIGINT, shutting down...");
    }
    _ = signal::unix::signal(signal::unix::SignalKind::terminate())? => {
        info!("Received SIGTERM, shutting down...");
    }
}

// Graceful shutdown
server_task.abort();
info!("BearDog server stopped");
```

### Gap 3: Family Seed Loading

```rust
// Needed: Read family seed from environment (set by tower from tower.toml)
if let Ok(family_seed_b64) = std::env::var("BEARDOG_FAMILY_SEED") {
    info!("Loading family seed from environment");
    let family_seed = base64::decode(family_seed_b64)?;
    // Use for genetic lineage initialization
}
```

---

## 🎯 Recommended Solution

### Option 1: Fix Existing beardog-server Binary

1. Find the source for `target/release/beardog-server`
2. Add port configuration
3. Add proper lifecycle management
4. Test with tower

### Option 2: Create New Standalone Server Binary

```rust
// crates/beardog-tunnel/src/bin/beardog-server.rs

use beardog_tunnel::{BeardogBtspProvider, HsmManager};
use beardog_genetics::EcosystemGeneticEngine;
use beardog_tunnel::api::{BearDogApiServer, BearDogApiServerConfig};
use std::sync::Arc;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    info!("🐻 BearDog Standalone Service Starting");

    // 1. Auto-initialize HSM
    std::env::set_var("BEARDOG_HSM_MODE", 
        std::env::var("BEARDOG_HSM_MODE").unwrap_or_else(|_| "software".to_string())
    );
    let hsm = Arc::new(HsmManager::auto_initialize().await?);

    // 2. Initialize genetic engine
    let genetics = Arc::new(EcosystemGeneticEngine::new()?);

    // 3. Create BTSP provider
    let btsp_provider = Arc::new(
        BeardogBtspProvider::new(hsm, genetics).await?
    );

    // 4. Create API server with configurable port
    let bind_addr = std::env::var("BEARDOG_BIND_ADDR")
        .or_else(|_| std::env::var("HTTP_PORT").map(|p| format!("0.0.0.0:{}", p)))
        .unwrap_or_else(|_| "0.0.0.0:9000".to_string())
        .parse()?;

    let config = BearDogApiServerConfig {
        bind_addr,
        enable_cors: true,
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    info!("📡 Binding to: {}", config.bind_addr);

    let server = BearDogApiServer::new(config, btsp_provider).await?;

    // 5. Start server in background
    let server_task = tokio::spawn(async move {
        if let Err(e) = server.serve().await {
            error!("Server error: {}", e);
        }
    });

    info!("✅ BearDog server ready!");

    // 6. Wait for shutdown signal
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received SIGINT");
        }
        _ = async {
            let mut sigterm = tokio::signal::unix::signal(
                tokio::signal::unix::SignalKind::terminate()
            )?;
            sigterm.recv().await;
            Ok::<_, std::io::Error>(())
        } => {
            info!("Received SIGTERM");
        }
    }

    // 7. Graceful shutdown
    info!("Shutting down BearDog server...");
    server_task.abort();
    info!("✅ BearDog server stopped");

    Ok(())
}
```

Then add to `Cargo.toml`:

```toml
[[bin]]
name = "beardog-server"
path = "crates/beardog-tunnel/src/bin/beardog-server.rs"
```

---

## 🧪 Testing Checklist

For BearDog team to validate:

- [ ] Binary builds: `cargo build --release --bin beardog-server`
- [ ] Respects port: `BEARDOG_BIND_ADDR=127.0.0.1:19000 ./beardog-server`
- [ ] Health endpoint works: `curl http://127.0.0.1:19000/health`
- [ ] Stays running (doesn't exit)
- [ ] Handles SIGTERM gracefully: `kill -TERM <pid>`
- [ ] Handles SIGINT gracefully: `Ctrl+C`
- [ ] Works with tower orchestration
- [ ] Unix socket created at `/tmp/primals/beardog-{node}.sock`

---

## 📊 Summary

### What's Working ✅
- HSM initialization (`auto_initialize()`)
- Software HSM provider
- Genetic engine
- BTSP provider
- API endpoints
- Unix socket support

### What's Needed ❌
- Configurable port binding (not port 0)
- Service lifecycle (stay running)
- Signal handling (SIGTERM/SIGINT)
- Environment variable support (BEARDOG_BIND_ADDR)

### Priority
**HIGH** - Blocks biomeOS deployment and federation testing

---

**Handoff To**: BearDog Team  
**From**: biomeOS Team  
**Date**: January 8, 2026

