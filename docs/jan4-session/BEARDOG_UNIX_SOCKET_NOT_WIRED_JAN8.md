# 🔴 CRITICAL: BearDog Unix Socket Not Wired Up

**Date**: January 8, 2026  
**Status**: 🔴 **BLOCKER** - HTTP API is running, Unix socket is NOT  
**For**: BearDog Team (URGENT)

---

## 🎯 The Problem

BearDog v0.15.0 `beardog-server` binary:
- ✅ Has `UnixSocketIpcServer` code
- ❌ **NEVER STARTS IT** in `beardog-server.rs`
- ❌ Always starts HTTP API instead
- ❌ Unix socket remains unused

**Result**: Port 9000 binding when it should be Unix socket ONLY!

---

## 📋 Evidence

### What beardog-server.rs DOES Start:
```rust
// Step 6: Create API Server
let server = BearDogApiServer::new(config, btsp_provider).await?;

// Step 7: Start Server in Background
let server_task = tokio::spawn(async move {
    if let Err(e) = server.serve().await {
        error!("Server error: {}", e);
    }
});
```

**This is the HTTP API server!** ❌

### What beardog-server.rs SHOULD Start:
```rust
// Unix Socket IPC Server (PRIMARY)
let socket_path = format!("/tmp/beardog-{}-{}.sock",
    std::env::var("BEARDOG_FAMILY_ID").unwrap_or("unknown".to_string()),
    std::env::var("BEARDOG_NODE_ID").unwrap_or_else(|| gethostname().to_string_lossy().to_string())
);

let unix_server = UnixSocketIpcServer::new(socket_path, btsp_provider).await?;
let unix_task = tokio::spawn(async move {
    if let Err(e) = unix_server.serve().await {
        error!("Unix socket server error: {}", e);
    }
});
```

---

## ✅ Required Fix

### Update `beardog-server.rs` main function:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ... (existing HSM, genetics, BTSP initialization)

    // Determine socket path from environment
    let family_id = std::env::var("BEARDOG_FAMILY_ID")
        .unwrap_or_else(|_| "unknown".to_string());
    let node_id = std::env::var("BEARDOG_NODE_ID")
        .unwrap_or_else(|| gethostname().to_string_lossy().to_string());
    let socket_path = format!("/tmp/beardog-{}-{}.sock", family_id, node_id);

    info!("🔌 Unix Socket IPC (PRIMARY): {}", socket_path);

    // Create Unix Socket IPC Server (PRIMARY)
    let unix_server = UnixSocketIpcServer::new(socket_path.clone(), btsp_provider.clone()).await?;
    let unix_task = tokio::spawn(async move {
        if let Err(e) = unix_server.serve().await {
            error!("Unix socket server error: {}", e);
        }
    });

    info!("✅ Unix Socket IPC server started");

    // Optional: HTTP API (only if explicitly enabled)
    let http_enabled = std::env::var("BEARDOG_HTTP_ENABLED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(false);

    let http_task = if http_enabled {
        let bind_addr = std::env::var("BEARDOG_BIND_ADDR")
            .or_else(|_| std::env::var("HTTP_PORT").map(|p| format!("0.0.0.0:{}", p)))
            .unwrap_or_else(|_| "0.0.0.0:9000".to_string())
            .parse()?;

        let config = BearDogApiServerConfig {
            bind_addr,
            enable_cors: true,
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        info!("🌐 HTTP API (OPTIONAL): {}", config.bind_addr);

        let http_server = BearDogApiServer::new(config, btsp_provider).await?;
        Some(tokio::spawn(async move {
            if let Err(e) = http_server.serve().await {
                error!("HTTP server error: {}", e);
            }
        }))
    } else {
        info!("🔌 HTTP API: Disabled (port-free mode)");
        None
    };

    info!("");
    info!("╔════════════════════════════════════════════════════════╗");
    info!("║                                                        ║");
    info!("║         ✅ BearDog Service Ready!                      ║");
    info!("║                                                        ║");
    info!("║  Unix Socket: {}                    ", socket_path);
    if http_enabled {
        info!("║  HTTP API: (enabled)                                   ║");
    } else {
        info!("║  HTTP API: (disabled - port-free mode)                ║");
    }
    info!("║                                                        ║");
    info!("║  Press Ctrl+C or send SIGTERM to shutdown gracefully  ║");
    info!("║                                                        ║");
    info!("╚════════════════════════════════════════════════════════╝");
    info!("");

    // Wait for shutdown signal
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("📡 Received SIGINT (Ctrl+C)");
        }
        result = async {
            let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())?;
            sigterm.recv().await;
            Ok::<_, std::io::Error>(())
        } => {
            match result {
                Ok(_) => info!("📡 Received SIGTERM"),
                Err(e) => error!("Error setting up SIGTERM handler: {}", e),
            }
        }
    }

    // Graceful shutdown
    info!("");
    info!("🛑 Shutting down BearDog service...");
    unix_task.abort();
    if let Some(task) = http_task {
        task.abort();
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    info!("✅ BearDog service stopped");
    info!("");

    Ok(())
}
```

---

## 🎯 Key Changes

1. **Unix Socket is PRIMARY** (always starts)
2. **HTTP is OPTIONAL** (only if `BEARDOG_HTTP_ENABLED=true`)
3. **Socket path uses family + node ID** (no conflicts)
4. **Both servers can run simultaneously** (if HTTP enabled)

---

## 📝 Testing Checklist

### Test 1: Port-Free Mode (Default)
```bash
BEARDOG_FAMILY_ID=nat0 \
BEARDOG_NODE_ID=node-alpha \
./beardog-server

# Expected:
# ✅ Unix socket: /tmp/beardog-nat0-node-alpha.sock
# ✅ HTTP API: Disabled
# ✅ Zero HTTP ports bound
```

### Test 2: With HTTP Enabled
```bash
BEARDOG_FAMILY_ID=nat0 \
BEARDOG_NODE_ID=node-beta \
BEARDOG_HTTP_ENABLED=true \
HTTP_PORT=19000 \
./beardog-server

# Expected:
# ✅ Unix socket: /tmp/beardog-nat0-node-beta.sock
# ✅ HTTP API: 0.0.0.0:19000
# ✅ Both servers running
```

### Test 3: Multiple Instances (No Conflicts)
```bash
# Terminal 1:
BEARDOG_NODE_ID=alpha ./beardog-server &

# Terminal 2:
BEARDOG_NODE_ID=beta ./beardog-server &

# Expected:
# ✅ /tmp/beardog-unknown-alpha.sock
# ✅ /tmp/beardog-unknown-beta.sock
# ✅ Zero HTTP ports
# ✅ No conflicts!
```

---

## 📦 Missing Import

You'll also need to add:

```rust
use beardog_tunnel::unix_socket_ipc::UnixSocketIpcServer;
use gethostname::gethostname;
```

And in `Cargo.toml`:

```toml
[dependencies]
gethostname = "0.4"
# ... existing deps
```

---

## 🚦 Priority

**CRITICAL** - This is the ROOT CAUSE of port 9000 binding.

Unix socket code exists but is never used. HTTP API is always started instead.

---

**Next**: Once fixed, biomeOS can deploy multiple nodes on USB spores with zero port conflicts! 🌱
