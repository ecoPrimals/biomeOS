# 🚀 biomeOS Quick Start Guide

**Version**: 0.1.0 (Production Ready)  
**Last Updated**: January 14, 2026

---

## 📋 Prerequisites

- Rust 1.70+ (latest stable)
- Linux or macOS
- 4GB+ RAM
- 10GB+ disk space

---

## 🎯 Quick Deploy (3 Options)

### Option 1: Local NUCLEUS Deployment (Recommended)

**Deploy complete ecosystem in one command:**

```bash
# From biomeOS workspace
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS

# Build (first time only)
cargo build --release

# Deploy NUCLEUS (BearDog + Songbird + Toadstool + NestGate)
./target/release/nucleus deploy --family nat0
```

**What this does:**
- ✅ Deploys 4 primals in correct dependency order
- ✅ Sets up Unix socket communication
- ✅ Configures inter-primal discovery
- ✅ Validates deployment health

**Time**: ~11 seconds

---

### Option 2: LiveSpore USB Deployment

**Deploy from portable USB drive:**

```bash
# Insert USB and mount (typically auto-mounts)
# USB should contain biomeOS/ directory

cd /media/YOUR_USB_NAME/biomeOS

# Deploy using USB-local binaries
./primals/nucleus deploy --family usb0 --graph graphs/nucleus_usb.toml
```

**What this does:**
- ✅ Runs from USB without installation
- ✅ Uses pre-built binaries (no compilation needed)
- ✅ 19 deployment graphs available
- ✅ Completely portable

**Requirements**: 
- LiveSpore USB (created with `livespore-deploy` tool)
- Any Linux system with USB support

---

### Option 3: Individual Primal Launch (Manual)

**For development/testing:**

```bash
# Terminal 1: BearDog (Security)
./target/release/beardog-server --family nat0

# Terminal 2: Songbird (Discovery)
./target/release/songbird-orchestrator --family nat0

# Terminal 3: Toadstool (Compute)
./target/release/toadstool --family nat0

# Terminal 4: NestGate (Storage)
./target/release/nestgate service start --family nat0
```

**Note**: This is manual and not recommended. Use `nucleus deploy` instead.

---

## 🔍 Verify Deployment

### Check Running Primals
```bash
ps aux | grep -E "beardog|songbird|toadstool|nestgate"
```

### Check Unix Sockets
```bash
ls -lh /tmp/*nat0*.sock
# or
ls -lh /run/user/$(id -u)/*nat0*.sock
```

### Health Check
```bash
./target/release/nucleus status --family nat0
```

---

## 🧪 Test the Ecosystem

### Query BearDog (Security)
```bash
# Example: Get identity
curl --unix-socket /tmp/beardog-nat0-default.sock \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"identity.get","id":1}' \
  http://localhost/
```

### Query Songbird (Discovery)
```bash
# Example: Discover primals
curl --unix-socket /tmp/songbird-nat0.sock \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"discovery.list","id":1}' \
  http://localhost/
```

---

## 📊 Available Graphs

**Location**: `graphs/`

| Graph | Purpose | Primals |
|-------|---------|---------|
| `nucleus_simple.toml` | Basic NUCLEUS | 4 core primals |
| `nucleus_ecosystem.toml` | Full ecosystem | 6 primals + UI |
| `nucleus_usb.toml` | USB deployment | Optimized for LiveSpore |
| `tower_deploy.toml` | Security atomic | BearDog + Songbird |
| `node_deploy.toml` | Compute atomic | Tower + Toadstool |
| `nest_deploy.toml` | Storage atomic | Tower + NestGate |

**Custom graphs**: Create your own in `graphs/` using TOML format

---

## 🛠️ Troubleshooting

### Problem: "Socket already in use"

**Solution**: Kill existing primals
```bash
pkill -f "beardog-server|songbird|toadstool|nestgate"
rm -f /tmp/*nat0*.sock
```

### Problem: "Binary not found"

**Solution**: Build first
```bash
cargo build --release
```

### Problem: "Permission denied on socket"

**Solution**: Check socket directory permissions
```bash
ls -ld /tmp
# Should be: drwxrwxrwt (1777)
```

### Problem: NestGate won't start

**Solution**: Set JWT secret (temporary, BearDog preferred)
```bash
export NESTGATE_JWT_SECRET=$(openssl rand -base64 48)
```

---

## 📚 Next Steps

### Learn More
- Read: `STATUS.md` - Current deployment status
- Read: `ROOT_DOCS_INDEX.md` - Complete documentation index
- Read: `archive/sessions-jan14-2026-final/README.md` - Session history

### Advanced Usage
- Multi-family deployments
- Cross-machine federation
- Custom graph creation
- Metrics collection
- Chaos testing

### Development
- Add new primals
- Create custom graphs
- Implement capabilities
- Contribute back

---

## 🎯 Success Criteria

**You know it's working when:**
- ✅ `nucleus deploy` completes in ~11 seconds
- ✅ 3-4 primal processes running
- ✅ Unix sockets created in `/tmp/` or `/run/user/*/`
- ✅ `nucleus status` shows healthy primals
- ✅ No errors in deployment output

---

## 🆘 Get Help

- **Documentation**: `ROOT_DOCS_INDEX.md`
- **Examples**: `examples/` directory
- **Graphs**: `graphs/` with comments
- **Architecture**: `specs/` directory
- **Issues**: GitHub issues (when published)

---

**Status**: ✅ Production Ready  
**Quality**: Zero unsafe code, zero mocks, 99% pure Rust  
**Support**: TRUE PRIMAL compliant, fully documented

*Happy deploying! 🧬🚀✨*

