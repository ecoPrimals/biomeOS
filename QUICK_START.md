# 🚀 biomeOS Quick Start Guide

**Version**: 0.2.0 (Tower Atomic)  
**Last Updated**: January 27, 2026  
**Status**: ✅ Production Ready

---

## 📋 Prerequisites

- Rust 1.70+ (latest stable)
- Linux (kernel 5.4+)
- Unix socket support
- 4GB+ RAM

---

## 🎯 Quick Deploy

### Option 1: Tower Atomic (Recommended)

**Deploy Pure Rust TLS 1.3 stack in one command:**

```bash
# From biomeOS workspace
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS

# Build
cargo build --release -p biomeos-unibin

# Deploy Tower Atomic (BearDog + Songbird + Neural API)
./deploy_tower_atomic.sh

# Verify
./deploy_tower_atomic.sh status
```

**What this deploys:**
- ✅ BearDog (Pure Rust crypto: SHA-256, SHA-384, AES-GCM)
- ✅ Songbird (Pure Rust HTTP/TLS 1.3)
- ✅ Neural API (capability.call routing)

**Time**: ~10 seconds

---

### Option 2: Test HTTPS Immediately

```bash
# Via Neural API (recommended - uses capability.call)
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

### Option 3: Build Everything

```bash
# Full workspace build
cargo build --release --workspace

# Run tests
cargo test --workspace

# Check specific package
cargo test --package biomeos-atomic-deploy
```

---

## 🔍 Verify Deployment

### Check Running Processes
```bash
ps aux | grep -E "beardog|songbird|neural-api" | grep -v grep
```

### Check Unix Sockets
```bash
ls -la /tmp/*.sock
# Expected:
# /tmp/beardog-nat0.sock
# /tmp/songbird-nat0.sock  
# /tmp/neural-api.sock
```

### Health Check
```bash
./deploy_tower_atomic.sh status
```

### View Logs
```bash
tail -f /tmp/neural-api*.log
tail -f /tmp/beardog*.log
tail -f /tmp/songbird*.log
```

---

## 🛠️ Troubleshooting

### Socket Not Found

```bash
# Check if deployment is running
./deploy_tower_atomic.sh status

# If not running, deploy
./deploy_tower_atomic.sh

# If stale sockets, cleanup and redeploy
./deploy_tower_atomic.sh stop
./deploy_tower_atomic.sh
```

### TLS Handshake Fails

```bash
# Check if site supports TLS 1.3
echo | openssl s_client -connect example.com:443 2>&1 | grep Protocol

# If TLS 1.2 only - not yet supported (7% of sites)
# See SONGBIRD_EVOLUTION_HANDOFF.md for roadmap
```

### Permission Denied

```bash
# Check socket permissions
ls -la /tmp/*.sock

# Sockets should be owned by your user
# If not, stop and redeploy
./deploy_tower_atomic.sh stop
./deploy_tower_atomic.sh
```

---

## 📊 Validation Results

Tower Atomic has been validated against 87 sites:

| Category | Sites | TLS 1.3 Success |
|----------|-------|-----------------|
| AI/ML | 10 | 100% ✅ |
| Cloud | 10 | 90% ✅ |
| Code Hosting | 6 | 83% ✅ |
| Containers | 6 | 100% ✅ |
| Databases | 7 | 100% ✅ |
| Serverless | 7 | 100% ✅ |
| Security | 6 | 100% ✅ |

**Total**: 93% TLS 1.3 success (Pure Rust)

---

## 🧬 LiveSpore USB Deployment

**Deploy genetically-linked spores for federation:**

```bash
# Create sibling from existing parent spore
./scripts/create_sibling_spore.sh /media/parent/biomeOS /media/newusb node-beta

# Verify genetic lineage (offline)
./scripts/verify_sibling_lineage.sh /media/usb1/biomeOS /media/usb2/biomeOS

# Test federation (runs both spores)
./scripts/test_federation.sh
```

**LiveSpore features:**
- ✅ Portable USB deployment
- ✅ Genetic lineage verification
- ✅ Automatic federation trust
- ✅ Tower Atomic stack

---

## 📚 Next Steps

### Learn More
- **[START_HERE.md](./START_HERE.md)** - Quick orientation
- **[DOCUMENTATION_HUB.md](./DOCUMENTATION_HUB.md)** - Full navigation
- **[specs/README.md](./specs/README.md)** - Technical specifications

### Architecture
- **[BIOMEOS_ATOMICS_ARCHITECTURE.md](./BIOMEOS_ATOMICS_ARCHITECTURE.md)** - System design
- **[TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md](./TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md)** - Zero coupling

### Evolution
- **[SONGBIRD_EVOLUTION_HANDOFF.md](./SONGBIRD_EVOLUTION_HANDOFF.md)** - TLS roadmap
- **[INFRASTRUCTURE_EVOLUTION.md](./INFRASTRUCTURE_EVOLUTION.md)** - Future plans

---

## 🎯 Success Criteria

**You know it's working when:**
- ✅ `./deploy_tower_atomic.sh` completes without errors
- ✅ 3 processes running (beardog, songbird, neural-api)
- ✅ Unix sockets created in `/tmp/`
- ✅ `./deploy_tower_atomic.sh status` shows healthy
- ✅ HTTPS requests succeed via `nc -U /tmp/neural-api.sock`

---

**Status**: ✅ Production Ready  
**TLS**: 93% validation | **Pure Rust**: 100%

*Happy deploying! 🧬🚀✨*
