# biomeOS Deployment Guide

**Version**: 2.0.0 (Tower Atomic)  
**Date**: January 27, 2026  
**Status**: Production Ready

---

## 🚀 Quick Start (Tower Atomic - Recommended)

### Prerequisites

```bash
# Build all binaries
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
cargo build --release -p biomeos-unibin

# Verify binary exists
ls -lh ./target/release/biomeos
```

### Deploy Tower Atomic

```bash
# One-command deployment
./deploy_tower_atomic.sh

# Check status
./deploy_tower_atomic.sh status

# Stop
./deploy_tower_atomic.sh stop
```

### Test HTTPS

```bash
# Via Neural API capability.call (recommended)
echo '{"jsonrpc":"2.0","method":"capability.call","params":{
  "capability":"secure_http",
  "operation":"http.request",
  "args":{"url":"https://api.github.com/zen","method":"GET"}
},"id":1}' | nc -U /tmp/neural-api.sock

# Direct to Songbird
echo '{"jsonrpc":"2.0","method":"http.request","params":{
  "method":"GET","url":"https://httpbin.org/get"
},"id":1}' | nc -U /tmp/songbird-nat0.sock
```

---

## 📊 Verify Deployment

### Check Running Processes

```bash
# List all primal processes
ps aux | grep -E "(beardog|songbird|neural-api)" | grep -v grep

# Expected output:
# beardog-server (or beardog)
# songbird-orchestrator (or songbird)
# neural-api-server (or neural-api)
```

### Check Unix Sockets

```bash
# List all primal sockets
ls -la /tmp/*.sock

# Expected sockets:
# /tmp/beardog-nat0.sock
# /tmp/songbird-nat0.sock
# /tmp/neural-api.sock
```

### Check Logs

```bash
# View all primal logs
ls -lh /tmp/*.log

# Tail specific log
tail -f /tmp/songbird-nat0.log

# View Neural API log
tail -f /tmp/neural-api.log
```

---

## 🛑 Stop Ecosystem

### Via Script (Recommended)

```bash
./deploy_tower_atomic.sh stop
```

### Manual Cleanup

```bash
# Kill specific primal
pkill -f beardog
pkill -f songbird
pkill -f neural-api

# Clean sockets
rm -f /tmp/beardog-*.sock /tmp/songbird-*.sock /tmp/neural-api*.sock
```

---

## 🏗️ Deployment Architecture

### Tower Atomic Stack

```
Neural API (capability.call router)
     ↓
Songbird (HTTP/TLS 1.3)
     ↓
BearDog (crypto: SHA-256, SHA-384, AES-GCM, X25519)
```

### Communication Pattern

```
Consumer → Neural API → capability.call("secure_http", "http.request")
                ↓
           Translation via graph
                ↓
           Songbird.http_request()
                ↓
           BearDog (TLS crypto)
                ↓
           External HTTPS
```

### TRUE PRIMAL Pattern

- Primals don't know each other's APIs
- Communication via semantic capabilities
- Neural API translates via deployment graphs
- Zero coupling between consumers and providers

---

## 📁 Deployment Graphs

Located in `graphs/` directory:

### Tower Atomic Graphs

| Graph | Purpose |
|-------|---------|
| `tower_atomic_bootstrap.toml` | Full Tower Atomic deployment |
| `tower_atomic.toml` | Tower deployment |
| `tower_health_check.toml` | Health verification |
| `tower_shutdown.toml` | Graceful shutdown |

### Additional Graphs

| Graph | Purpose |
|-------|---------|
| `nest_deploy.toml` | NestGate deployment |
| `node_deploy.toml` | ToadStool deployment |
| `ui_deploy.toml` | UI deployment |

---

## 🔧 Configuration

### Environment Variables

```bash
# Family ID (genetic lineage identifier)
export FAMILY_ID=nat0

# Socket directory (default: /tmp)
export SOCKET_DIR=/tmp

# Log level
export RUST_LOG=info
```

### Graph Configuration

Edit TOML files in `graphs/` to customize:
- Binary paths
- Socket paths
- Capability mappings
- Startup timeouts

---

## 🐛 Troubleshooting

### Primal Won't Start

**Check logs**:
```bash
tail -50 /tmp/<primal-name>*.log
```

**Common issues**:
- Socket already in use → cleanup and redeploy
- Binary not found → rebuild with cargo
- Permissions → check socket directory

### Socket Not Created

**Symptoms**: Deployment times out

**Solutions**:
1. Check primal log for errors
2. Verify binary has execute permissions
3. Check socket directory exists
4. Ensure no other process using socket

### TLS Handshake Fails

**Symptoms**: Connection errors to HTTPS sites

**Check**:
```bash
# Verify site supports TLS 1.3
echo | openssl s_client -connect example.com:443 2>&1 | grep Protocol
```

**Note**: 7% of sites require TLS 1.2 (not yet supported)

---

## 🌟 Production Checklist

Before deploying to production:

- [ ] All binaries built with `--release` flag
- [ ] `cargo test --workspace` passes
- [ ] Sockets are in secure location
- [ ] Log rotation configured
- [ ] Monitoring in place
- [ ] Backup/rollback procedure documented

---

## 📚 Documentation

- **[QUICK_START.md](./QUICK_START.md)** - Quick deployment
- **[START_HERE.md](./START_HERE.md)** - Orientation
- **[BIOMEOS_ATOMICS_ARCHITECTURE.md](./BIOMEOS_ATOMICS_ARCHITECTURE.md)** - Architecture
- **[SONGBIRD_EVOLUTION_HANDOFF.md](./SONGBIRD_EVOLUTION_HANDOFF.md)** - TLS roadmap
- **[specs/README.md](./specs/README.md)** - Specifications

---

**Status**: ✅ Production Ready | **TLS**: 93% | **Pure Rust**: 100%

*Deploy with confidence using Tower Atomic!* 🚀
